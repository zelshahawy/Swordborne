use bevy::{
    app::AppExit,
    ecs::hierarchy::ChildSpawnerCommands,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};

use crate::fonts::GameFonts;
use crate::state::{CampaignState, GameState, PlayerProfile};

use crate::leaderboard::{LeaderboardResource, leaderboard_text};

pub struct MenuPlugin;

const MENU_FADE_SPEED: f32 = 3.0;
const MAX_PLAYER_NAME_CHARS: usize = 25;

#[derive(Resource, Default)]
struct PendingPlayerName {
    value: String,
}

#[derive(Resource, Default)]
struct MenuFadeOut {
    active: bool,
    alpha: f32,
}

#[derive(Resource)]
struct MenuArtHandles {
    background: Handle<Image>,
}

impl FromWorld for MenuArtHandles {
    fn from_world(world: &mut World) -> Self {
        Self {
            background: world.resource::<AssetServer>().load("main_menu.png"),
        }
    }
}

#[derive(Component)]
struct MainMenuUi;

#[derive(Component)]
struct MenuFadeOverlay;

#[derive(Component)]
struct NameValueText;

#[derive(Component)]
struct LeaderboardDisplayText;

#[derive(Component)]
struct MenuButton {
    action: MenuAction,
}

#[derive(Component)]
struct MenuButtonLabel;

#[derive(Component)]
struct MenuControlsOverlay;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MenuAction {
    Play,
    Controls,
    Quit,
}

impl MenuAction {
    fn colors(self, interaction: Interaction) -> (Color, Color, Color) {
        let white = Color::srgb(1.0, 1.0, 1.0);
        match interaction {
            Interaction::Pressed => (
                Color::srgba(1.0, 1.0, 1.0, 0.18),
                Color::srgba(1.0, 1.0, 1.0, 0.80),
                white,
            ),
            Interaction::Hovered => (
                Color::srgba(1.0, 1.0, 1.0, 0.08),
                Color::srgba(1.0, 1.0, 1.0, 0.45),
                white,
            ),
            Interaction::None => (
                Color::srgba(0.0, 0.0, 0.0, 0.0),
                Color::srgba(1.0, 1.0, 1.0, 0.22),
                white,
            ),
        }
    }
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingPlayerName>()
            .init_resource::<MenuFadeOut>()
            .init_resource::<MenuArtHandles>()
            .init_resource::<PlayerProfile>()
            .init_resource::<CampaignState>()
            .add_systems(OnEnter(GameState::MainMenu), (spawn_main_menu, reset_menu_fade))
            .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
            .add_systems(
                Update,
                (
                    ensure_main_menu_present,
                    capture_name_input,
                    sync_name_text,
                    handle_menu_buttons,
                    tick_menu_fade,
                )
                    .run_if(in_state(GameState::MainMenu)),
            );

        app.add_systems(
            Update,
            sync_leaderboard_text.run_if(in_state(GameState::MainMenu)),
        );
    }
}

fn ensure_main_menu_present(
    menu_query: Query<Entity, With<MainMenuUi>>,
    commands: Commands,
    art: Res<MenuArtHandles>,
    fonts: Res<GameFonts>,
    pending_name: ResMut<PendingPlayerName>,
) {
    if !menu_query.is_empty() {
        return;
    }

    spawn_main_menu(commands, art, fonts, pending_name);
}

fn spawn_main_menu(
    mut commands: Commands,
    art: Res<MenuArtHandles>,
    fonts: Res<GameFonts>,
    mut pending_name: ResMut<PendingPlayerName>,
) {
    pending_name.value.clear();

    commands
        .spawn((
            MainMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|root| {
            // background art
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode::new(art.background.clone()).with_mode(NodeImageMode::Stretch),
            ));

            // dark overlay
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.06, 0.55)),
            ));

            // two-column layout
            root.spawn((Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(40.0),
                padding: UiRect::axes(Val::Px(60.0), Val::Px(60.0)),
                ..default()
            },))
            .with_children(|row| {
                spawn_left_panel(row, &fonts);
                spawn_right_panel(row, &fonts);
            });

            // fade overlay for transition
            root.spawn((
                MainMenuUi,
                MenuFadeOverlay,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                ZIndex(20),
            ));

            // controls overlay (hidden by default)
            root.spawn((
                MainMenuUi,
                MenuControlsOverlay,
                Visibility::Hidden,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                ZIndex(10),
            ))
            .with_children(|overlay| {
                overlay.spawn((
                    MainMenuUi,
                    Node {
                        padding: UiRect::all(Val::Px(36.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.10, 0.96)),
                    BorderColor::all(Color::srgba(0.5, 0.5, 0.8, 0.6)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        MainMenuUi,
                        Text::new("CONTROLS"),
                        TextFont { font: fonts.pixel_bold.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.95, 0.90, 0.60)),
                    ));
                    for (key, action) in [
                        ("A / D", "Move left / right"),
                        ("Space", "Jump"),
                        ("E", "Pick up sword"),
                        ("Left Click", "Slash"),
                        ("Hold Right Click", "Aim throw"),
                        ("Release Right Click", "Throw sword"),
                        ("R", "Restart level"),
                        ("ESC", "Toggle in-game menu"),
                    ] {
                        panel.spawn((
                            MainMenuUi,
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(16.0),
                                width: Val::Px(340.0),
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            row.spawn((
                                MainMenuUi,
                                Text::new(key),
                                TextFont { font: fonts.pixel_bold.clone(), font_size: 13.0, ..default() },
                                TextColor(Color::srgb(0.70, 0.85, 1.0)),
                                Node { width: Val::Px(160.0), ..default() },
                            ));
                            row.spawn((
                                MainMenuUi,
                                Text::new(action),
                                TextFont { font: fonts.pixel_regular.clone(), font_size: 13.0, ..default() },
                                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                            ));
                        });
                    }
                    panel.spawn((
                        MainMenuUi,
                        Text::new("Press ESC to close"),
                        TextFont { font: fonts.pixel_regular.clone(), font_size: 12.0, ..default() },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 0.6)),
                        Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
                    ));
                });
            });
        });
}

fn spawn_left_panel(parent: &mut ChildSpawnerCommands, fonts: &GameFonts) {
    parent
        .spawn((Node {
            width: Val::Px(380.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(20.0),
            padding: UiRect::all(Val::Px(36.0)),
            ..default()
        },))
        .with_children(|col| {
            col.spawn((
                Text::new("Your Big Castle Internship"),
                TextFont { font: fonts.pixel_bold.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.93, 0.88, 0.76)),
            ));

            // title
            col.spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
            ))
            .with_children(|title| {
                title.spawn((
                    Text::new("SwordBorne"),
                    TextFont { font: fonts.pixel_bold.clone(), font_size: 60.0, ..default() },
                    TextColor(Color::srgb(0.99, 0.98, 0.96)),
                ));
            });

            // divider
            col.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.95, 0.93, 0.88, 0.18)),
            ));

            // name label
            col.spawn((
                Text::new("Knight Name"),
                TextFont { font: fonts.pixel_regular.clone(), font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.72, 0.77, 0.85)),
            ));

            // name input box
            col.spawn((
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(52.0),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.11, 0.14, 0.2, 0.9)),
                BorderColor::all(Color::srgb(0.31, 0.4, 0.55)),
            ))
            .with_children(|name_box| {
                name_box.spawn((
                    NameValueText,
                    Text::new("> _"),
                    TextFont { font: fonts.pixel_regular.clone(), font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.97, 0.97, 0.99)),
                ));
            });

            // buttons
            col.spawn((Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },))
            .with_children(|buttons| {
                spawn_menu_button(buttons, fonts, "Play", MenuAction::Play);
                spawn_menu_button(buttons, fonts, "Controls", MenuAction::Controls);
                #[cfg(not(target_arch = "wasm32"))]
                spawn_menu_button(buttons, fonts, "Quit", MenuAction::Quit);
            });

            col.spawn((
                Text::new("Enter name (1-25 chars), then press Enter"),
                TextFont { font: fonts.pixel_bold.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgba(0.72, 0.77, 0.85, 0.7)),
            ));
        });
}

fn spawn_right_panel(parent: &mut ChildSpawnerCommands, fonts: &GameFonts) {
    parent
        .spawn((Node {
            width: Val::Px(420.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(28.0)),
            row_gap: Val::Px(12.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.06, 0.5)),
        BorderColor::all(Color::srgba(0.95, 0.93, 0.88, 0.08)),
        ))
        .with_children(|col| {
            col.spawn((
                Text::new("LEADERBOARD"),
                TextFont { font: fonts.pixel_bold.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.95, 0.90, 0.60)),
            ));

            col.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.95, 0.93, 0.88, 0.12)),
            ));

            col.spawn((
                LeaderboardDisplayText,
                Text::new("  Loading..."),
                TextFont { font: fonts.pixel_regular.clone(), font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.82, 0.86, 0.95)),
            ));
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    label: &str,
    action: MenuAction,
) {
    let (default_bg, default_border, default_label) = action.colors(Interaction::None);
    parent
        .spawn((
            MenuButton { action },
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(62.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(18.0)),
                ..default()
            },
            BackgroundColor(default_bg),
            BorderColor::all(default_border),
        ))
        .with_children(|btn| {
            btn.spawn((
                MenuButtonLabel,
                Text::new(label),
                TextFont { font: fonts.pixel_bold.clone(), font_size: 30.0, ..default() },
                TextColor(default_label),
            ));
        });
}

fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn capture_name_input(
    mut keyboard_input_reader: MessageReader<KeyboardInput>,
    mut pending_name: ResMut<PendingPlayerName>,
    mut player_profile: ResMut<PlayerProfile>,
    mut campaign: ResMut<CampaignState>,
    mut fade: ResMut<MenuFadeOut>,
    mut controls_overlay: Query<&mut Visibility, With<MenuControlsOverlay>>,
) {
    for keyboard_input in keyboard_input_reader.read() {
        if keyboard_input.state != ButtonState::Pressed {
            continue;
        }

        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Escape, _) => {
                if let Ok(mut vis) = controls_overlay.single_mut() {
                    *vis = Visibility::Hidden;
                }
            }
            (Key::Enter, _) => start_new_game(
                &pending_name.value,
                &mut player_profile,
                &mut campaign,
                &mut fade,
            ),
            (Key::Backspace, _) => {
                pending_name.value.pop();
            }
            (_, Some(inserted_text)) => {
                let mut current_len = pending_name.value.chars().count();
                for chr in inserted_text.chars() {
                    if !is_printable_char(chr) || current_len >= MAX_PLAYER_NAME_CHARS {
                        continue;
                    }
                    pending_name.value.push(chr);
                    current_len += 1;
                }
            }
            _ => {}
        }
    }
}

fn sync_name_text(
    pending_name: Res<PendingPlayerName>,
    mut query: Query<&mut Text, With<NameValueText>>,
) {
    if !pending_name.is_changed() {
        return;
    }
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    let display = if pending_name.value.is_empty() { "_".into() } else { pending_name.value.clone() };
    **text = format!("> {display}");
}

fn sync_leaderboard_text(
    lb: Res<LeaderboardResource>,
    mut query: Query<&mut Text, With<LeaderboardDisplayText>>,
) {
    if !lb.is_changed() {
        return;
    }
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    **text = leaderboard_text(&lb);
}

fn handle_menu_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &MenuButton,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut label_query: Query<&mut TextColor, With<MenuButtonLabel>>,
    pending_name: Res<PendingPlayerName>,
    mut player_profile: ResMut<PlayerProfile>,
    mut campaign: ResMut<CampaignState>,
    mut fade: ResMut<MenuFadeOut>,
    mut app_exit: MessageWriter<AppExit>,
    mut controls_overlay: Query<&mut Visibility, With<MenuControlsOverlay>>,
) {
    for (interaction, button, children, mut background, mut border) in &mut interaction_query {
        let (bg, bd, label) = button.action.colors(*interaction);
        *background = BackgroundColor(bg);
        *border = BorderColor::all(bd);

        for child in children.iter() {
            if let Ok(mut text_color) = label_query.get_mut(child) {
                text_color.0 = label;
            }
        }

        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.action {
            MenuAction::Play => start_new_game(
                &pending_name.value,
                &mut player_profile,
                &mut campaign,
                &mut fade,
            ),
            MenuAction::Controls => {
                if let Ok(mut vis) = controls_overlay.single_mut() {
                    *vis = Visibility::Visible;
                }
            }
            MenuAction::Quit => {
                app_exit.write(AppExit::Success);
            }
        }
    }
}

fn start_new_game(
    pending_name: &str,
    player_profile: &mut PlayerProfile,
    campaign: &mut CampaignState,
    fade: &mut MenuFadeOut,
) {
    if fade.active { return; }
    let trimmed_name = pending_name.trim();
    if trimmed_name.is_empty() {
        return;
    }
    player_profile.name = trimmed_name.chars().take(MAX_PLAYER_NAME_CHARS).collect();
    *campaign = CampaignState::default();
    fade.active = true;
}

fn reset_menu_fade(mut fade: ResMut<MenuFadeOut>) {
    *fade = MenuFadeOut::default();
}

fn tick_menu_fade(
    time: Res<Time>,
    mut fade: ResMut<MenuFadeOut>,
    mut bg_query: Query<&mut BackgroundColor, With<MenuFadeOverlay>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !fade.active { return; }
    fade.alpha = (fade.alpha + time.delta_secs() * MENU_FADE_SPEED).min(1.0);
    if let Ok(mut bg) = bg_query.single_mut() {
        bg.0 = Color::srgba(0.0, 0.0, 0.0, fade.alpha);
    }
    if fade.alpha >= 1.0 {
        next_state.set(GameState::InGame);
    }
}

fn is_printable_char(chr: char) -> bool {
    let is_private = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);
    !is_private && !chr.is_ascii_control()
}
