use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, TryRecvError};

use bevy::prelude::*;
use serde_json::Value;

use crate::state::{BossDefeated, GameState, PlayerProfile, RunTimer, format_run_time};

macro_rules! lb_log {
    ($($arg:tt)*) => { if cfg!(debug_assertions) { eprintln!($($arg)*); } };
}

#[cfg(target_arch = "wasm32")]
fn sb_url() -> Option<String> {
    option_env!("SUPABASE_URL").map(str::to_string)
}

#[cfg(not(target_arch = "wasm32"))]
fn sb_url() -> Option<String> {
    std::env::var("SUPABASE_URL").ok()
}

#[cfg(target_arch = "wasm32")]
fn sb_key() -> Option<String> {
    option_env!("SUPABASE_ANON_KEY").map(str::to_string)
}

#[cfg(not(target_arch = "wasm32"))]
fn sb_key() -> Option<String> {
    std::env::var("SUPABASE_ANON_KEY").ok()
}

const MAX_TIME_SECS: f64 = 7200.0;

#[derive(Clone, Debug)]
pub struct LeaderboardEntry {
    pub name: String,
    pub time_secs: f64,
}

type FetchResult = Result<Vec<LeaderboardEntry>, String>;

#[derive(Resource, Default)]
pub struct LeaderboardResource {
    pub entries: Vec<LeaderboardEntry>,
    pub loading: bool,
    pub error: Option<String>,
    pub rank: Option<u32>,
    fetch_rx: Option<Mutex<Receiver<FetchResult>>>,
    rank_rx: Option<Mutex<Receiver<u32>>>,
}

impl LeaderboardResource {
    fn start_fetch(&mut self) {
        if self.fetch_rx.is_some() {
            return;
        }
        self.loading = true;
        self.error = None;
        let (tx, rx) = mpsc::channel();
        self.fetch_rx = Some(Mutex::new(rx));

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let _ = tx.send(wasm_fetch_top10().await);
        });

        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let _ = tx.send(native_fetch_top10());
        });
    }

    pub fn submit_and_fetch_rank(&mut self, name: String, time_secs: f64) {
        if time_secs <= 0.0 || time_secs > MAX_TIME_SECS {
            lb_log!("[lb] score out of bounds ({time_secs}), skipping");
            return;
        }
        self.rank = None;
        self.loading = true;
        self.error = None;

        let (fetch_tx, fetch_rx) = mpsc::channel();
        let (rank_tx, rank_rx) = mpsc::channel();
        self.fetch_rx = Some(Mutex::new(fetch_rx));
        self.rank_rx = Some(Mutex::new(rank_rx));

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = wasm_insert(&name, time_secs).await {
                lb_log!("[lb] insert error: {e}");
            }
            let rank = wasm_count_faster(time_secs).await + 1;
            let _ = rank_tx.send(rank);
            let _ = fetch_tx.send(wasm_fetch_top10().await);
        });

        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            if let Err(e) = native_insert(&name, time_secs) {
                lb_log!("[lb] insert error: {e}");
            }
            let rank = native_count_faster(time_secs) + 1;
            let _ = rank_tx.send(rank);
            let _ = fetch_tx.send(native_fetch_top10());
        });
    }

    fn poll(&mut self) {
        let fetch_status = self
            .fetch_rx
            .as_ref()
            .map(|fetch_rx| fetch_rx.lock().unwrap().try_recv());
        match fetch_status {
            Some(Ok(result)) => {
                self.loading = false;
                self.fetch_rx = None;
                match result {
                    Ok(entries) => self.entries = entries,
                    Err(e) => {
                        lb_log!("[lb] fetch error: {e}");
                        self.error = Some(e);
                    }
                }
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.loading = false;
                self.fetch_rx = None;
                self.error = Some("leaderboard fetch worker disconnected".into());
                lb_log!("[lb] fetch worker disconnected");
            }
            _ => {}
        }

        let rank_status = self
            .rank_rx
            .as_ref()
            .map(|rank_rx| rank_rx.lock().unwrap().try_recv());
        match rank_status {
            Some(Ok(rank)) => {
                self.rank = Some(rank);
                self.rank_rx = None;
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.rank_rx = None;
                lb_log!("[lb] rank worker disconnected");
            }
            _ => {}
        }
    }
}

// ─── WASM async HTTP ─────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
async fn wasm_fetch_top10() -> FetchResult {
    let (url, key) = creds().ok_or_else(|| "credentials not set".to_string())?;
    let endpoint = format!(
        "{url}/rest/v1/leaderboard?select=name,time_secs&order=time_secs.asc&limit=10"
    );
    let resp = reqwest::Client::new()
        .get(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let rows = resp.json::<Vec<Value>>().await.map_err(|e| format!("parse error: {e}"))?;
    Ok(parse_entries(rows))
}

#[cfg(target_arch = "wasm32")]
async fn wasm_insert(name: &str, time_secs: f64) -> Result<(), String> {
    let (url, key) = creds().ok_or_else(|| "credentials not set".to_string())?;
    let resp = reqwest::Client::new()
        .post(format!("{url}/rest/v1/leaderboard"))
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {key}"))
        .header("Prefer", "return=minimal")
        .json(&serde_json::json!({ "name": name, "time_secs": time_secs }))
        .send()
        .await
        .map_err(|e| format!("insert request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("insert HTTP {}", resp.status()));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn wasm_count_faster(time_secs: f64) -> u32 {
    let Ok((url, key)) = creds().ok_or(()) else { return 0; };
    let Ok(resp) = reqwest::Client::new()
        .get(format!("{url}/rest/v1/leaderboard?select=id&time_secs=lt.{time_secs}"))
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
    else { return 0; };
    resp.json::<Vec<Value>>().await.unwrap_or_default().len() as u32
}

// ─── Native blocking HTTP ─────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn native_fetch_top10() -> FetchResult {
    let (url, key) = creds().ok_or_else(|| "credentials not set".to_string())?;
    let endpoint = format!(
        "{url}/rest/v1/leaderboard?select=name,time_secs&order=time_secs.asc&limit=10"
    );
    let resp = reqwest::blocking::Client::new()
        .get(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .map_err(|e| format!("request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let rows = resp.json::<Vec<Value>>().map_err(|e| format!("parse error: {e}"))?;
    Ok(parse_entries(rows))
}

#[cfg(not(target_arch = "wasm32"))]
fn native_insert(name: &str, time_secs: f64) -> Result<(), String> {
    let (url, key) = creds().ok_or_else(|| "credentials not set".to_string())?;
    let resp = reqwest::blocking::Client::new()
        .post(format!("{url}/rest/v1/leaderboard"))
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {key}"))
        .header("Prefer", "return=minimal")
        .json(&serde_json::json!({ "name": name, "time_secs": time_secs }))
        .send()
        .map_err(|e| format!("insert request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("insert HTTP {}", resp.status()));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn native_count_faster(time_secs: f64) -> u32 {
    let Ok((url, key)) = creds().ok_or(()) else { return 0; };
    let Ok(resp) = reqwest::blocking::Client::new()
        .get(format!("{url}/rest/v1/leaderboard?select=id&time_secs=lt.{time_secs}"))
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {key}"))
        .send()
    else { return 0; };
    resp.json::<Vec<Value>>().unwrap_or_default().len() as u32
}


fn creds() -> Option<(String, String)> {
    Some((sb_url()?, sb_key()?))
}

fn parse_entries(rows: Vec<Value>) -> Vec<LeaderboardEntry> {
    rows.iter()
        .filter_map(|row| {
            Some(LeaderboardEntry {
                name: row["name"].as_str()?.to_string(),
                time_secs: row["time_secs"].as_f64()?,
            })
        })
        .collect()
}

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LeaderboardResource>()
            .add_systems(OnEnter(GameState::MainMenu), fetch_on_menu_enter)
            .add_systems(OnEnter(GameState::InGame), start_run_timer)
            .add_systems(
                Update,
                (poll_lb, tick_run_timer, watch_boss_defeated)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(Update, poll_lb.run_if(in_state(GameState::MainMenu)));
    }
}

fn fetch_on_menu_enter(mut lb: ResMut<LeaderboardResource>) {
    lb.start_fetch();
}

fn start_run_timer(mut timer: ResMut<RunTimer>) {
    timer.start();
}

fn poll_lb(mut lb: ResMut<LeaderboardResource>) {
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
    profile: Res<PlayerProfile>,
) {
    if !defeated.triggered || lb.rank_rx.is_some() || lb.rank.is_some() {
        return;
    }
    lb.submit_and_fetch_rank(profile.name.clone(), defeated.time_secs);
}

pub fn leaderboard_text(lb: &LeaderboardResource) -> String {
    if sb_url().is_none() || sb_key().is_none() {
        return "  ⚠ SUPABASE credentials not embedded — rebuild with env vars set".into();
    }
    if lb.loading {
        return "  Loading...".into();
    }
    if let Some(ref e) = lb.error {
        return format!("  ⚠ {e}");
    }
    if lb.entries.is_empty() {
        return "  No runs yet. Be the first!".into();
    }
    lb.entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            format!("  {:>2}.  {:<16}  {}", i + 1, e.name, format_run_time(e.time_secs))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
