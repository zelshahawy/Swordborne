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

pub struct MenuPlugin;

// ── footer messages ──────────────────────────────────────────────────────────

const FOOTER_LANDING: &str = "Your Big Castle internship awaits.";
const FOOTER_NAME_ENTRY: &str = "Enter your knight name, then press Start Game.";
const FOOTER_CONTROLS: &str = "Press Esc to return to the menu.";

// ── resources ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct PendingPlayerName {
    value: String,
}

#[derive(Resource, Default)]
struct MenuViewState {
    frame: MenuFrame,
    footer_message: String,
}

impl MenuViewState {
    fn go_to_landing(&mut self) {
        self.frame = MenuFrame::Landing;
        self.footer_message = FOOTER_LANDING.into();
    }

    fn go_to_name_entry(&mut self) {
        self.frame = MenuFrame::NameEntry;
        self.footer_message = FOOTER_NAME_ENTRY.into();
    }

    fn go_to_controls(&mut self) {
        self.frame = MenuFrame::Controls;
        self.footer_message = FOOTER_CONTROLS.into();
    }
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

// ── components ───────────────────────────────────────────────────────────────

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
struct ControlsFrame;

#[derive(Component)]
struct MenuButton {
    action: MenuAction,
}

#[derive(Component)]
struct MenuButtonLabel;

// ── enums ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MenuAction {
    Play,
    Settings,
    Quit,
    ConfirmName,
    BackToMenu,
}

impl MenuAction {
    /// Returns the button's height.
    fn height(self) -> Val {
        match self {
            Self::ConfirmName | Self::BackToMenu => px(58.0),
            _ => px(74.0),
        }
    }

    /// Returns the label font size.
    fn font_size(self) -> f32 {
        match self {
            Self::ConfirmName | Self::BackToMenu => 20.0,
            _ => 36.0,
        }
    }

    /// Visual colors for a given interaction state.
    /// All buttons share a uniform white/neutral palette — no per-button tints.
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum MenuFrame {
    #[default]
    Landing,
    NameEntry,
    Controls,
}

// ── plugin ───────────────────────────────────────────────────────────────────

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

// ── spawn ─────────────────────────────────────────────────────────────────────

fn spawn_main_menu(
    mut commands: Commands,
    art: Res<MenuArtHandles>,
    fonts: Res<GameFonts>,
    mut pending_name: ResMut<PendingPlayerName>,
    mut menu_view: ResMut<MenuViewState>,
) {
    pending_name.value.clear();
    menu_view.go_to_landing();

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
            // background art
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                ImageNode::new(art.background.clone()).with_mode(NodeImageMode::Stretch),
            ));

            // dark overlay
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.06, 0.48)),
            ));

            // centred layout
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
                                width: px(700.0),
                                max_width: percent(92.0),
                                min_height: px(640.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(1.0)),
                                border_radius: BorderRadius::px(44.0, 44.0, 36.0, 36.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.02, 0.03, 0.06, 0.45)),
                            BorderColor::all(Color::srgba(0.95, 0.93, 0.88, 0.08)),
                        ))
                        .with_children(|panel| spawn_menu_content(panel, &fonts));
                });
        });
}

fn spawn_menu_content(panel: &mut ChildSpawnerCommands, fonts: &GameFonts) {
    panel
        .spawn((Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },))
        .with_children(|frames| {
            spawn_landing_frame(frames, fonts);
            spawn_name_entry_frame(frames, fonts);
            spawn_controls_frame(frames, fonts);
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
        make_font(&fonts.pixel_regular, 15.0),
        TextColor(Color::srgb(0.72, 0.77, 0.85)),
    ));
}

fn spawn_landing_frame(panel: &mut ChildSpawnerCommands, fonts: &GameFonts) {
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
                Text::new("BIG CASTLE INTERNSHIP PROGRAM"),
                make_font(&fonts.pixel_regular, 15.0),
                TextColor(Color::srgb(0.93, 0.88, 0.76)),
            ));

            // title with drop-shadow
            content
                .spawn((Node {
                    width: percent(100.0),
                    height: px(168.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(px(20.0)),
                    ..default()
                },))
                .with_children(|title| {
                    title.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(6.0),
                            top: px(8.0),
                            ..default()
                        },
                        Text::new("SwordBorne"),
                        make_font(&fonts.pixel_bold, 84.0),
                        TextColor(Color::srgba(0.01, 0.01, 0.02, 0.26)),
                    ));

                    title.spawn((
                        Text::new("SwordBorne"),
                        make_font(&fonts.pixel_bold, 80.0),
                        TextColor(Color::srgb(0.99, 0.98, 0.96)),
                    ));
                });

            // divider
            content.spawn((
                Node {
                    width: px(240.0),
                    height: px(1.0),
                    margin: UiRect::axes(px(0.0), px(22.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.95, 0.93, 0.88, 0.18)),
            ));

            // buttons
            content
                .spawn((Node {
                    width: px(340.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(18.0),
                    ..default()
                },))
                .with_children(|buttons| {
                    spawn_menu_button(buttons, fonts, "Play", MenuAction::Play, None);
                    spawn_menu_button(buttons, fonts, "Controls", MenuAction::Settings, None);
                    spawn_menu_button(buttons, fonts, "Quit", MenuAction::Quit, None);
                });

            content.spawn((
                Node {
                    margin: UiRect::top(px(16.0)),
                    ..default()
                },
                Text::new("Press [ ENTER ] to begin"),
                make_font(&fonts.pixel_regular, 13.0),
                TextColor(Color::srgba(0.72, 0.77, 0.85, 0.6)),
            ));
        });
}

fn spawn_name_entry_frame(panel: &mut ChildSpawnerCommands, fonts: &GameFonts) {
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
                        make_font(&fonts.pixel_bold, 28.0),
                        TextColor(Color::srgb(0.96, 0.9, 0.74)),
                    ));

                    prompt.spawn((
                        Text::new("Big Castle insists the offer letter be addressed properly."),
                        make_font(&fonts.pixel_regular, 14.0),
                        TextColor(Color::srgb(0.74, 0.8, 0.88)),
                    ));

                    // name input box
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
                                make_font(&fonts.pixel_regular, 20.0),
                                TextColor(Color::srgb(0.97, 0.97, 0.99)),
                            ));
                        });

                    // action row
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
                                fonts,
                                "Back",
                                MenuAction::BackToMenu,
                                Some(140.0),
                            );
                            spawn_menu_button(
                                row,
                                fonts,
                                "Start Game",
                                MenuAction::ConfirmName,
                                Some(220.0),
                            );
                        });

                    prompt.spawn((
                        Text::new("Press Enter to start or Esc to return."),
                        make_font(&fonts.pixel_regular, 12.0),
                        TextColor(Color::srgb(0.62, 0.68, 0.76)),
                    ));
                });
        });
}

fn spawn_controls_frame(panel: &mut ChildSpawnerCommands, fonts: &GameFonts) {
    panel
        .spawn((
            ControlsFrame,
            Visibility::Hidden,
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
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
                        width: px(520.0),
                        max_width: percent(90.0),
                        padding: UiRect::all(px(36.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12.0),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(28.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.06, 0.1, 0.92)),
                    BorderColor::all(Color::srgba(0.95, 0.93, 0.88, 0.14)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new("CONTROLS"),
                        make_font(&fonts.pixel_bold, 28.0),
                        TextColor(Color::srgb(0.99, 0.98, 0.96)),
                    ));

                    // divider
                    card.spawn((
                        Node {
                            width: percent(100.0),
                            height: px(1.0),
                            margin: UiRect::vertical(px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.95, 0.93, 0.88, 0.15)),
                    ));

                    let rows: &[(&str, &str)] = &[
                        ("Move",         "A / D   or   ← →"),
                        ("Jump",         "Space   or   W"),
                        ("Slash",        "Left Click   or   H"),
                        ("Aim Sword",    "Hold Right Click"),
                        ("Throw Sword",  "Release Right Click"),
                        ("Pick Up",      "Walk near the sword"),
                    ];

                    for (action, binding) in rows {
                        spawn_control_row(card, fonts, action, binding);
                    }

                    // back button
                    card.spawn((Node {
                        margin: UiRect::top(px(18.0)),
                        width: percent(100.0),
                        ..default()
                    },))
                    .with_children(|btn_row| {
                        spawn_menu_button(btn_row, fonts, "Back", MenuAction::BackToMenu, None);
                    });
                });
        });
}

fn spawn_control_row(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    action: &str,
    binding: &str,
) {
    parent
        .spawn((Node {
            width: percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new(action.to_owned()),
                make_font(&fonts.pixel_bold, 18.0),
                TextColor(Color::srgb(0.93, 0.88, 0.76)),
            ));
            row.spawn((
                Text::new(binding.to_owned()),
                make_font(&fonts.pixel_regular, 17.0),
                TextColor(Color::srgb(0.82, 0.88, 0.96)),
            ));
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    label: &str,
    action: MenuAction,
    fixed_width: Option<f32>,
) {
    let width = fixed_width.map(px).unwrap_or_else(|| percent(100.0));
    let (default_bg, default_border, default_label) = action.colors(Interaction::None);

    parent
        .spawn((
            MenuButton { action },
            Button,
            Node {
                width,
                height: action.height(),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(2.0)),
                padding: UiRect::axes(px(16.0), px(8.0)),
                border_radius: BorderRadius::all(px(22.0)),
                ..default()
            },
            BackgroundColor(default_bg),
            BorderColor::all(default_border),
        ))
        .with_children(|button| {
            button.spawn((
                MenuButtonLabel,
                Text::new(label),
                make_font(&fonts.pixel_bold, action.font_size()),
                TextColor(default_label),
            ));
        });
}

fn make_font(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    }
}

// ── systems ───────────────────────────────────────────────────────────────────

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
            (Key::Enter, _) => match menu_view.frame {
                MenuFrame::NameEntry => start_new_game(
                    &pending_name.value,
                    &mut player_profile,
                    &mut campaign,
                    &mut next_state,
                ),
                MenuFrame::Landing => menu_view.go_to_name_entry(),
                MenuFrame::Controls => {}
            },
            (Key::Escape, _) => match menu_view.frame {
                MenuFrame::NameEntry | MenuFrame::Controls => menu_view.go_to_landing(),
                MenuFrame::Landing => {}
            },
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
    let display = if pending_name.value.is_empty() {
        "_".into()
    } else {
        pending_name.value.clone()
    };
    **text = format!("> {display}");
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
    mut landing_q: Query<
        &mut Visibility,
        (With<LandingFrame>, Without<NameEntryFrame>, Without<ControlsFrame>),
    >,
    mut name_q: Query<
        &mut Visibility,
        (With<NameEntryFrame>, Without<LandingFrame>, Without<ControlsFrame>),
    >,
    mut controls_q: Query<
        &mut Visibility,
        (With<ControlsFrame>, Without<LandingFrame>, Without<NameEntryFrame>),
    >,
) {
    if !menu_view.is_changed() {
        return;
    }
    let Ok(mut landing_vis) = landing_q.single_mut() else {
        return;
    };
    let Ok(mut name_vis) = name_q.single_mut() else {
        return;
    };
    let Ok(mut controls_vis) = controls_q.single_mut() else {
        return;
    };

    (*landing_vis, *name_vis, *controls_vis) = match menu_view.frame {
        MenuFrame::Landing => (Visibility::Visible, Visibility::Hidden, Visibility::Hidden),
        MenuFrame::NameEntry => (Visibility::Hidden, Visibility::Visible, Visibility::Hidden),
        MenuFrame::Controls => (Visibility::Hidden, Visibility::Hidden, Visibility::Visible),
    };
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
            MenuAction::Play => menu_view.go_to_name_entry(),
            MenuAction::Settings => menu_view.go_to_controls(),
            MenuAction::Quit => {
                app_exit.write(AppExit::Success);
            }
            MenuAction::ConfirmName => start_new_game(
                &pending_name.value,
                &mut player_profile,
                &mut campaign,
                &mut next_state,
            ),
            MenuAction::BackToMenu => menu_view.go_to_landing(),
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
        "Knight".into()
    } else {
        pending_name.trim().into()
    };
    *campaign = CampaignState::default();
    next_state.set(GameState::InGame);
}

fn is_printable_char(chr: char) -> bool {
    let is_private = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);
    !is_private && !chr.is_ascii_control()
}
