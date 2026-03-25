use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use bevy::prelude::*;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::sync::Client;

use crate::state::{BossDefeated, GameState, RunTimer, format_run_time};

macro_rules! lb_log {
    ($($arg:tt)*) => { if cfg!(debug_assertions) { eprintln!($($arg)*); } };
}

#[derive(Clone, Debug)]
pub struct LeaderboardEntry {
    pub name: String,
    pub time_secs: f64,
}

#[derive(Resource, Default)]
pub struct LeaderboardResource {
    pub entries: Vec<LeaderboardEntry>,
    pub loading: bool,
    pub rank: Option<u32>,
    pub rank_rx: Option<Mutex<Receiver<u32>>>,
    fetch_rx: Option<Mutex<Receiver<Vec<LeaderboardEntry>>>>,
}

impl LeaderboardResource {
    fn start_fetch(&mut self) {
        if self.fetch_rx.is_some() {
            // submit_and_fetch_rank already has a fetch in-flight
            self.loading = true;
            return;
        }
        let Some(uri) = mongo_uri() else { return; };
        self.loading = true;
        let (tx, rx) = mpsc::channel();
        self.fetch_rx = Some(Mutex::new(rx));
        thread::spawn(move || { let _ = tx.send(db_fetch_top10(&uri)); });
    }

    pub fn submit_and_fetch_rank(&mut self, name: String, time_secs: f64) {
        let Some(uri) = mongo_uri() else { return; };
        self.rank = None;
        let (rank_tx, rank_rx) = mpsc::channel();
        let (fetch_tx, fetch_rx) = mpsc::channel();
        self.rank_rx = Some(Mutex::new(rank_rx));
        self.fetch_rx = Some(Mutex::new(fetch_rx));
        thread::spawn(move || {
            db_insert(&uri, &name, time_secs);
            // fetch happens after insert — start_fetch on menu enter won't race
            let _ = rank_tx.send(db_count_faster(&uri, time_secs) + 1);
            let _ = fetch_tx.send(db_fetch_top10(&uri));
        });
    }

    fn poll(&mut self) {
        let fetch_done = if let Some(ref rx) = self.fetch_rx {
            match rx.lock().unwrap().try_recv() {
                Ok(entries) => { self.entries = entries; self.loading = false; true }
                _ => false,
            }
        } else { false };
        if fetch_done { self.fetch_rx = None; }

        let rank_done = if let Some(ref rx) = self.rank_rx {
            match rx.lock().unwrap().try_recv() {
                Ok(rank) => { self.rank = Some(rank); true }
                _ => false,
            }
        } else { false };
        if rank_done { self.rank_rx = None; }
    }
}

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LeaderboardResource>()
            .add_systems(Startup, check_db_connection)
            .add_systems(OnEnter(GameState::MainMenu), fetch_on_menu_enter)
            .add_systems(OnEnter(GameState::InGame), start_run_timer)
            .add_systems(
                Update,
                (poll_db, tick_run_timer, watch_boss_defeated).run_if(in_state(GameState::InGame)),
            )
            .add_systems(Update, poll_db.run_if(in_state(GameState::MainMenu)));
    }
}

fn check_db_connection() {
    let Some(uri) = mongo_uri() else { return; };
    thread::spawn(move || {
        match Client::with_uri_str(&uri) {
            Err(e) => lb_log!("[lb] connection failed: {e}"),
            Ok(client) => match client.database("swordborne").run_command(doc! { "ping": 1 }, None) {
                Ok(_) => lb_log!("[lb] connected to MongoDB"),
                Err(e) => lb_log!("[lb] ping failed: {e}"),
            },
        }
    });
}

fn fetch_on_menu_enter(mut lb: ResMut<LeaderboardResource>) {
    lb.start_fetch();
}

fn start_run_timer(mut timer: ResMut<RunTimer>) {
    timer.start();
}

fn poll_db(mut lb: ResMut<LeaderboardResource>) {
    lb.poll();
}

fn tick_run_timer(time: Res<Time>, mut timer: ResMut<RunTimer>) {
    if timer.running {
        timer.elapsed_secs += time.delta_secs_f64();
    }
}

fn watch_boss_defeated(
    defeated: Res<BossDefeated>,
    mut lb: ResMut<LeaderboardResource>,
    profile: Res<crate::state::PlayerProfile>,
) {
    if !defeated.triggered || lb.rank_rx.is_some() || lb.rank.is_some() {
        return;
    }
    lb.submit_and_fetch_rank(profile.name.clone(), defeated.time_secs);
}

fn mongo_uri() -> Option<String> {
    match std::env::var("SWORDBORNE_MONGO_URI") {
        Ok(uri) => {
            let uri = uri.trim().to_string();
            if uri.is_empty() { lb_log!("[lb] SWORDBORNE_MONGO_URI is empty"); None } else { Some(uri) }
        }
        Err(_) => { lb_log!("[lb] SWORDBORNE_MONGO_URI not set"); None }
    }
}

fn db_client(uri: &str) -> Option<Client> {
    match Client::with_uri_str(uri) {
        Ok(c) => Some(c),
        Err(e) => { lb_log!("[lb] failed to create mongo client: {e}"); None }
    }
}

fn db_fetch_top10(uri: &str) -> Vec<LeaderboardEntry> {
    let Some(client) = db_client(uri) else { return vec![]; };
    let coll = client.database("swordborne").collection::<mongodb::bson::Document>("runs");
    let opts = FindOptions::builder().sort(doc! { "time_secs": 1 }).limit(10).build();
    match coll.find(None, opts) {
        Err(e) => { lb_log!("[lb] fetch failed: {e}"); vec![] }
        Ok(cursor) => cursor.filter_map(|r| {
            let doc = r.ok()?;
            Some(LeaderboardEntry {
                name: doc.get_str("name").ok()?.to_string(),
                time_secs: doc.get_f64("time_secs").ok()?,
            })
        }).collect(),
    }
}

fn db_insert(uri: &str, name: &str, time_secs: f64) {
    let Some(client) = db_client(uri) else { return; };
    let coll = client.database("swordborne").collection::<mongodb::bson::Document>("runs");
    if let Err(e) = coll.insert_one(doc! { "name": name, "time_secs": time_secs }, None) {
        lb_log!("[lb] insert failed: {e}");
    }
    db_trim_to_top10(&client);
}

fn db_trim_to_top10(client: &Client) {
    let coll = client.database("swordborne").collection::<mongodb::bson::Document>("runs");
    let opts = FindOptions::builder().sort(doc! { "time_secs": 1 }).skip(9).limit(1).build();
    let cutoff = match coll.find(None, opts) {
        Err(e) => { lb_log!("[lb] trim query failed: {e}"); return; }
        Ok(mut c) => c.next().and_then(|r| r.ok()).and_then(|doc| doc.get_f64("time_secs").ok()),
    };
    if let Some(cutoff_time) = cutoff {
        if let Err(e) = coll.delete_many(doc! { "time_secs": { "$gt": cutoff_time } }, None) {
            lb_log!("[lb] trim delete failed: {e}");
        }
    }
}

fn db_count_faster(uri: &str, time_secs: f64) -> u32 {
    let Some(client) = db_client(uri) else { return 0; };
    match client
        .database("swordborne")
        .collection::<mongodb::bson::Document>("runs")
        .count_documents(doc! { "time_secs": { "$lt": time_secs } }, None)
    {
        Ok(n) => n as u32,
        Err(e) => { lb_log!("[lb] count failed: {e}"); 0 }
    }
}

pub fn leaderboard_text(lb: &LeaderboardResource) -> String {
    if lb.loading {
        return "  Loading...".into();
    }
    if lb.entries.is_empty() {
        return "  No runs yet. Be the first!".into();
    }
    lb.entries
        .iter()
        .enumerate()
        .map(|(i, e)| format!("  {:>2}.  {:<16}  {}", i + 1, e.name, format_run_time(e.time_secs)))
        .collect::<Vec<_>>()
        .join("\n")
}
