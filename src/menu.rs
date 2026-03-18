use bevy::{
    app::AppExit,
    ecs::hierarchy::ChildSpawnerCommands,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};

use crate::state::{CampaignState, GameState, PlayerProfile};

pub struct MenuPlugin;

#[derive(Resource, Default)]
struct PendingPlayerName {
    value: String,
}

#[derive(Resource, Default)]
struct MenuViewState {
    frame: MenuFrame,
    footer_message: String,
}

#[derive(Resource)]
struct MenuArtHandles {
    background: Handle<Image>,
    fantasy_font: Handle<Font>,
}

#[derive(Component)]
struct MainMenuUi;

#[derive(Component)]
struct NameValueText;

#[derive(Component)]
struct FooterMessageText;

#[derive(Component)]
struct LandingFrame;

#[derive(Component)]
struct NameEntryFrame;

#[derive(Component)]
struct MenuButton {
    action: MenuAction,
}

#[derive(Component)]
struct MenuButtonLabel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MenuAction {
    Play,
    Settings,
    Quit,
    ConfirmName,
    BackToMenu,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum MenuFrame {
    #[default]
    Landing,
    NameEntry,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingPlayerName>()
            .init_resource::<MenuViewState>()
            .init_resource::<MenuArtHandles>()
            .init_resource::<PlayerProfile>()
            .init_resource::<CampaignState>()
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
            .add_systems(
                Update,
                (
                    capture_name_input,
                    sync_name_text,
                    sync_footer_text,
                    sync_menu_frame_visibility,
                    handle_menu_buttons,
                )
                    .run_if(in_state(GameState::MainMenu)),
            );
    }
}

impl FromWorld for MenuArtHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();

        Self {
            background: asset_server.load("main_menu.png"),
            fantasy_font: asset_server.load("apple_chancery.ttf"),
        }
    }
}

fn spawn_main_menu(
    mut commands: Commands,
    art: Res<MenuArtHandles>,
    mut pending_name: ResMut<PendingPlayerName>,
    mut menu_view: ResMut<MenuViewState>,
) {
    pending_name.value.clear();
    menu_view.frame = MenuFrame::Landing;
    menu_view.footer_message = "Press Play to enter your knight name.".to_string();

    commands
        .spawn((
            MainMenuUi,
            Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::new(px(28.0), px(28.0), px(88.0), px(52.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                ImageNode::new(art.background.clone()).with_mode(NodeImageMode::Stretch),
            ));

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.06, 0.34)),
            ));

            root.spawn((Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::new(px(28.0), px(28.0), px(88.0), px(52.0)),
                ..default()
            },))
                .with_children(|layout| {
                    layout
                        .spawn((
                            Node {
                                width: px(620.0),
                                max_width: percent(92.0),
                                min_height: px(620.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(2.0)),
                                border_radius: BorderRadius::px(42.0, 42.0, 34.0, 34.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.03, 0.04, 0.08, 0.58)),
                            BorderColor::all(Color::srgba(0.92, 0.9, 0.82, 0.18)),
                        ))
                        .with_children(|panel| spawn_menu_content(panel, &art));
                });
        });
}

fn spawn_menu_content(panel: &mut ChildSpawnerCommands, art: &MenuArtHandles) {
    panel
        .spawn((Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },))
        .with_children(|frames| {
            spawn_landing_frame(frames, art);
            spawn_name_entry_frame(frames, art);
        });

    panel.spawn((
        FooterMessageText,
        Node {
            position_type: PositionType::Absolute,
            left: px(36.0),
            right: px(36.0),
            bottom: px(20.0),
            ..default()
        },
        Text::new(""),
        TextFont::from_font_size(18.0),
        TextColor(Color::srgb(0.72, 0.77, 0.85)),
    ));
}

fn spawn_landing_frame(panel: &mut ChildSpawnerCommands, art: &MenuArtHandles) {
    panel
        .spawn((
            LandingFrame,
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|content| {
            content.spawn((
                Text::new("THE CASTLE INTERNSHIP TRIALS"),
                fantasy_text(&art.fantasy_font, 22.0),
                TextColor(Color::srgb(0.93, 0.87, 0.74)),
            ));

            content
                .spawn((Node {
                    width: percent(100.0),
                    height: px(152.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(px(18.0)),
                    ..default()
                },))
                .with_children(|title| {
                    title.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(8.0),
                            top: px(10.0),
                            ..default()
                        },
                        Text::new("SwordBorne"),
                        fantasy_text(&art.fantasy_font, 104.0),
                        TextColor(Color::srgba(0.0, 0.0, 0.0, 0.42)),
                    ));

                    title.spawn((
                        Text::new("SwordBorne"),
                        fantasy_text(&art.fantasy_font, 98.0),
                        TextColor(Color::srgb(0.99, 0.98, 0.96)),
                    ));
                });

            content.spawn((
                Text::new("A sword you must carry, throw, and reclaim."),
                fantasy_text(&art.fantasy_font, 30.0),
                TextColor(Color::srgb(0.88, 0.92, 0.97)),
            ));

            content
                .spawn((Node {
                    width: px(340.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(18.0),
                    margin: UiRect::top(px(48.0)),
                    ..default()
                },))
                .with_children(|buttons| {
                    spawn_menu_button(buttons, &art.fantasy_font, "Play", MenuAction::Play, 0.0);
                    spawn_menu_button(
                        buttons,
                        &art.fantasy_font,
                        "Settings",
                        MenuAction::Settings,
                        0.0,
                    );
                    spawn_menu_button(buttons, &art.fantasy_font, "Quit", MenuAction::Quit, 0.0);
                });
        });
}

fn spawn_name_entry_frame(panel: &mut ChildSpawnerCommands, art: &MenuArtHandles) {
    panel
        .spawn((
            NameEntryFrame,
            Visibility::Hidden,
            Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .with_children(|frame| {
            frame
                .spawn((
                    Node {
                        width: px(430.0),
                        max_width: percent(88.0),
                        padding: UiRect::all(px(28.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(18.0),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::px(30.0, 30.0, 24.0, 24.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.06, 0.1, 0.9)),
                    BorderColor::all(Color::srgb(0.88, 0.75, 0.46)),
                ))
                .with_children(|prompt| {
                    prompt.spawn((
                        Text::new("Enter Your Name"),
                        fantasy_text(&art.fantasy_font, 36.0),
                        TextColor(Color::srgb(0.96, 0.9, 0.74)),
                    ));

                    prompt.spawn((
                        Text::new("Big Castle insists the offer letter be addressed properly."),
                        TextFont::from_font_size(16.0),
                        TextColor(Color::srgb(0.74, 0.8, 0.88)),
                    ));

                    prompt
                        .spawn((
                            Node {
                                width: percent(100.0),
                                min_height: px(68.0),
                                padding: UiRect::axes(px(16.0), px(14.0)),
                                border: UiRect::all(px(2.0)),
                                border_radius: BorderRadius::all(px(18.0)),
                                margin: UiRect::top(px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.11, 0.14, 0.2, 0.96)),
                            BorderColor::all(Color::srgb(0.31, 0.4, 0.55)),
                        ))
                        .with_children(|name_box| {
                            name_box.spawn((
                                NameValueText,
                                Text::new("> _"),
                                fantasy_text(&art.fantasy_font, 30.0),
                                TextColor(Color::srgb(0.97, 0.97, 0.99)),
                            ));
                        });

                    prompt
                        .spawn((Node {
                            width: percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            column_gap: px(14.0),
                            margin: UiRect::top(px(6.0)),
                            ..default()
                        },))
                        .with_children(|row| {
                            spawn_menu_button(
                                row,
                                &art.fantasy_font,
                                "Back",
                                MenuAction::BackToMenu,
                                140.0,
                            );

                            spawn_menu_button(
                                row,
                                &art.fantasy_font,
                                "Start Game",
                                MenuAction::ConfirmName,
                                220.0,
                            );
                        });

                    prompt.spawn((
                        Text::new("Press Enter to start or Esc to return."),
                        TextFont::from_font_size(14.0),
                        TextColor(Color::srgb(0.62, 0.68, 0.76)),
                    ));
                });
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    fantasy_font: &Handle<Font>,
    label: &str,
    action: MenuAction,
    width_override: f32,
) {
    let width = if width_override > 0.0 {
        px(width_override)
    } else {
        percent(100.0)
    };
    let height = if action == MenuAction::ConfirmName {
        px(58.0)
    } else if action == MenuAction::BackToMenu {
        px(58.0)
    } else {
        px(74.0)
    };
    let font_size = if action == MenuAction::ConfirmName {
        24.0
    } else if action == MenuAction::BackToMenu {
        24.0
    } else {
        50.0
    };

    parent
        .spawn((
            MenuButton { action },
            Button,
            Node {
                width,
                height,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(2.0)),
                padding: UiRect::axes(px(16.0), px(8.0)),
                border_radius: BorderRadius::all(px(22.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.0)),
        ))
        .with_children(|button| {
            button.spawn((
                MenuButtonLabel,
                Text::new(label),
                fantasy_text(fantasy_font, font_size),
                TextColor(Color::srgb(0.98, 0.97, 0.95)),
            ));
        });
}

fn fantasy_text(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    }
}

fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn capture_name_input(
    mut keyboard_input_reader: MessageReader<KeyboardInput>,
    mut pending_name: ResMut<PendingPlayerName>,
    mut menu_view: ResMut<MenuViewState>,
    mut player_profile: ResMut<PlayerProfile>,
    mut campaign: ResMut<CampaignState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for keyboard_input in keyboard_input_reader.read() {
        if keyboard_input.state != ButtonState::Pressed {
            continue;
        }

        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Enter, _) => {
                if menu_view.frame == MenuFrame::NameEntry {
                    start_new_game(
                        &pending_name.value,
                        &mut player_profile,
                        &mut campaign,
                        &mut next_state,
                    );
                } else {
                    menu_view.frame = MenuFrame::NameEntry;
                    menu_view.footer_message =
                        "Enter your knight name, then press Start Game.".to_string();
                }
            }
            (Key::Escape, _) => {
                if menu_view.frame == MenuFrame::NameEntry {
                    menu_view.frame = MenuFrame::Landing;
                    menu_view.footer_message = "Press Play to enter your knight name.".to_string();
                }
            }
            (Key::Backspace, _) => {
                if menu_view.frame == MenuFrame::NameEntry {
                    pending_name.value.pop();
                }
            }
            (_, Some(inserted_text)) => {
                if menu_view.frame == MenuFrame::NameEntry
                    && inserted_text.chars().all(is_printable_char)
                    && pending_name.value.len() < 18
                {
                    pending_name.value.push_str(inserted_text);
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

    let value = if pending_name.value.is_empty() {
        "_".to_string()
    } else {
        pending_name.value.clone()
    };

    **text = format!("> {value}");
}

fn sync_footer_text(
    menu_view: Res<MenuViewState>,
    mut query: Query<&mut Text, With<FooterMessageText>>,
) {
    if !menu_view.is_changed() {
        return;
    }

    let Ok(mut text) = query.single_mut() else {
        return;
    };

    **text = menu_view.footer_message.clone();
}

fn sync_menu_frame_visibility(
    menu_view: Res<MenuViewState>,
    mut landing_query: Query<&mut Visibility, (With<LandingFrame>, Without<NameEntryFrame>)>,
    mut name_query: Query<&mut Visibility, (With<NameEntryFrame>, Without<LandingFrame>)>,
) {
    if !menu_view.is_changed() {
        return;
    }

    let Ok(mut landing_visibility) = landing_query.single_mut() else {
        return;
    };
    let Ok(mut name_visibility) = name_query.single_mut() else {
        return;
    };

    match menu_view.frame {
        MenuFrame::Landing => {
            *landing_visibility = Visibility::Visible;
            *name_visibility = Visibility::Hidden;
        }
        MenuFrame::NameEntry => {
            *landing_visibility = Visibility::Hidden;
            *name_visibility = Visibility::Visible;
        }
    }
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
    mut menu_view: ResMut<MenuViewState>,
    mut player_profile: ResMut<PlayerProfile>,
    mut campaign: ResMut<CampaignState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for (interaction, button, children, mut background, mut border) in &mut interaction_query {
        let (bg_color, border_color, label_color) = match (*interaction, button.action) {
            (Interaction::Pressed, MenuAction::Play) => (
                Color::srgba(0.95, 0.82, 0.5, 0.18),
                Color::srgba(0.98, 0.84, 0.52, 0.88),
                Color::srgb(1.0, 0.97, 0.88),
            ),
            (Interaction::Hovered, MenuAction::Play) => (
                Color::srgba(0.95, 0.82, 0.5, 0.1),
                Color::srgba(0.98, 0.84, 0.52, 0.56),
                Color::srgb(0.99, 0.95, 0.83),
            ),
            (Interaction::Pressed, MenuAction::Settings) => (
                Color::srgba(0.65, 0.75, 1.0, 0.15),
                Color::srgba(0.72, 0.79, 0.95, 0.84),
                Color::srgb(0.94, 0.97, 1.0),
            ),
            (Interaction::Hovered, MenuAction::Settings) => (
                Color::srgba(0.65, 0.75, 1.0, 0.09),
                Color::srgba(0.72, 0.79, 0.95, 0.52),
                Color::srgb(0.9, 0.95, 1.0),
            ),
            (Interaction::Pressed, MenuAction::Quit) => (
                Color::srgba(0.92, 0.36, 0.32, 0.16),
                Color::srgba(0.97, 0.46, 0.42, 0.88),
                Color::srgb(1.0, 0.92, 0.92),
            ),
            (Interaction::Hovered, MenuAction::Quit) => (
                Color::srgba(0.92, 0.36, 0.32, 0.1),
                Color::srgba(0.97, 0.46, 0.42, 0.54),
                Color::srgb(1.0, 0.9, 0.9),
            ),
            (Interaction::Pressed, MenuAction::ConfirmName) => (
                Color::srgba(0.96, 0.8, 0.42, 0.22),
                Color::srgba(0.98, 0.85, 0.54, 0.9),
                Color::srgb(0.1, 0.08, 0.07),
            ),
            (Interaction::Hovered, MenuAction::ConfirmName) => (
                Color::srgba(0.96, 0.8, 0.42, 0.14),
                Color::srgba(0.98, 0.85, 0.54, 0.64),
                Color::srgb(0.99, 0.96, 0.9),
            ),
            _ => (
                Color::srgba(0.0, 0.0, 0.0, 0.0),
                Color::srgba(1.0, 1.0, 1.0, 0.0),
                Color::srgb(0.98, 0.97, 0.95),
            ),
        };

        *background = BackgroundColor(bg_color);
        *border = BorderColor::all(border_color);

        for child in children.iter() {
            if let Ok(mut text_color) = label_query.get_mut(child) {
                text_color.0 = label_color;
            }
        }

        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.action {
            MenuAction::Play => {
                menu_view.frame = MenuFrame::NameEntry;
                menu_view.footer_message =
                    "Enter your knight name, then press Start Game.".to_string();
            }
            MenuAction::Settings => {
                menu_view.footer_message =
                    "Settings are not wired yet. Level 1 is the current focus.".to_string();
            }
            MenuAction::Quit => {
                app_exit.write(AppExit::Success);
            }
            MenuAction::ConfirmName => {
                start_new_game(
                    &pending_name.value,
                    &mut player_profile,
                    &mut campaign,
                    &mut next_state,
                );
            }
            MenuAction::BackToMenu => {
                menu_view.frame = MenuFrame::Landing;
                menu_view.footer_message = "Press Play to enter your knight name.".to_string();
            }
        }
    }
}

fn start_new_game(
    pending_name: &str,
    player_profile: &mut PlayerProfile,
    campaign: &mut CampaignState,
    next_state: &mut NextState<GameState>,
) {
    player_profile.name = if pending_name.trim().is_empty() {
        "Knight".to_string()
    } else {
        pending_name.trim().to_string()
    };
    *campaign = CampaignState::default();
    next_state.set(GameState::InGame);
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}
