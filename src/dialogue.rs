use bevy::prelude::*;

use crate::fonts::GameFonts;
use crate::state::GameState;

pub struct DialoguePlugin;

const DIALOGUE_ZOOM_IN_DURATION: f32 = 0.35;
const DIALOGUE_ZOOM_OUT_DURATION: f32 = 0.28;
const DIALOGUE_TARGET_SCALE: f32 = 0.78;
const PORTRAIT_PANEL_WIDTH: f32 = 220.0;
const BOX_HEIGHT: f32 = 230.0;

#[derive(Resource)]
pub struct DialoguePortraits {
    pub wizard: Handle<Image>,
    pub dark_wizard: Handle<Image>,
    #[allow(dead_code)]
    pub knight: Handle<Image>,
}

impl FromWorld for DialoguePortraits {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();
        Self {
            wizard: asset_server.load("wizard.png"),
            dark_wizard: asset_server.load("dark_wizard.png"),
            knight: asset_server.load("dungeon/frames/knight_m_idle_anim_f0.png"),
        }
    }
}

#[derive(Resource, Default)]
pub struct DialogueState {
    pub active: bool,
    pub speaker: String,
    pub lines: Vec<String>,
    pub index: usize,
    pub portrait: Option<Handle<Image>>,
}

#[derive(Resource, Default)]
pub struct DialogueCinematicState {
    phase: DialogueCinematicPhase,
    timer: Timer,
    speaker: String,
    lines: Vec<String>,
    portrait: Option<Handle<Image>>,
    focus: Vec3,
    base_scale: f32,
    target_scale: f32,
    base_translation: Vec3,
    target_translation: Vec3,
    camera_captured: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum DialogueCinematicPhase {
    #[default]
    Idle,
    ZoomIn,
    Dialogue,
    ZoomOut,
}

#[derive(Component)]
struct DialogueUiRoot;

#[derive(Component)]
struct DialogueSpeakerText;

#[derive(Component)]
struct DialogueBodyText;

#[derive(Component)]
struct DialogueHintText;

#[derive(Component)]
struct DialoguePortraitImage;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogueState>()
            .init_resource::<DialogueCinematicState>()
            .init_resource::<DialoguePortraits>()
            .add_systems(OnEnter(GameState::InGame), spawn_dialogue_ui)
            .add_systems(OnExit(GameState::InGame), despawn_dialogue_ui)
            .add_systems(
                Update,
                (run_dialogue_cinematic, sync_dialogue_ui, advance_dialogue)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

impl DialogueCinematicState {
    pub fn is_active(&self) -> bool {
        self.phase != DialogueCinematicPhase::Idle
    }
}

pub fn gameplay_unlocked(
    dialogue: Option<Res<DialogueState>>,
    cinematic: Option<Res<DialogueCinematicState>>,
) -> bool {
    let dialogue_active = dialogue.is_some_and(|dialogue| dialogue.active);
    let cinematic_active = cinematic.is_some_and(|cinematic| cinematic.is_active());

    !dialogue_active && !cinematic_active
}

pub fn start_dialogue<I, S>(
    dialogue: &mut DialogueState,
    speaker: impl Into<String>,
    lines: I,
    portrait: Option<Handle<Image>>,
) where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    dialogue.active = true;
    dialogue.speaker = speaker.into();
    dialogue.lines = lines.into_iter().map(Into::into).collect();
    dialogue.portrait = portrait;
    dialogue.index = 0;
}

pub fn queue_dialogue<I, S>(
    cinematic: &mut DialogueCinematicState,
    speaker: impl Into<String>,
    lines: I,
    focus: Vec3,
    portrait: Option<Handle<Image>>,
) where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    if cinematic.phase != DialogueCinematicPhase::Idle {
        return;
    }

    cinematic.phase = DialogueCinematicPhase::ZoomIn;
    cinematic.timer = Timer::from_seconds(DIALOGUE_ZOOM_IN_DURATION, TimerMode::Once);
    cinematic.speaker = speaker.into();
    cinematic.lines = lines.into_iter().map(Into::into).collect();
    cinematic.portrait = portrait;
    cinematic.focus = focus;
    cinematic.target_scale = DIALOGUE_TARGET_SCALE;
    cinematic.camera_captured = false;
}

fn spawn_dialogue_ui(mut commands: Commands, fonts: Res<GameFonts>) {
    commands
        .spawn((
            DialogueUiRoot,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                height: Val::Px(BOX_HEIGHT),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                border: UiRect::top(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.10, 0.95)),
            BorderColor::all(Color::srgb(0.82, 0.72, 0.44)),
        ))
        .with_children(|root| {
            // Portrait panel on the left
            root.spawn((
                Node {
                    width: Val::Px(PORTRAIT_PANEL_WIDTH),
                    flex_shrink: 0.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::right(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.08, 0.98)),
                BorderColor::all(Color::srgb(0.82, 0.72, 0.44)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    DialoguePortraitImage,
                    Visibility::Hidden,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode::default().with_mode(NodeImageMode::Stretch),
                ));
            });

            // Text panel on the right
            root.spawn((Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(36.0), Val::Px(22.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },))
            .with_children(|text| {
                text.spawn((
                    DialogueSpeakerText,
                    Text::new(""),
                    TextFont {
                        font: fonts.pixel_bold.clone(),
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.97, 0.86, 0.58)),
                ));

                text.spawn((
                    DialogueBodyText,
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                    Text::new(""),
                    TextFont {
                        font: fonts.pixel_regular.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.94, 0.95, 0.98)),
                ));

                text.spawn((
                    DialogueHintText,
                    Text::new("[ Space / Enter / E ] to continue"),
                    TextFont {
                        font: fonts.pixel_regular.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.55, 0.64, 0.80)),
                ));
            });
        });
}

fn despawn_dialogue_ui(mut commands: Commands, query: Query<Entity, With<DialogueUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn sync_dialogue_ui(
    dialogue: Res<DialogueState>,
    mut root_query: Query<
        &mut Visibility,
        (With<DialogueUiRoot>, Without<DialoguePortraitImage>),
    >,
    mut speaker_query: Query<&mut Text, (With<DialogueSpeakerText>, Without<DialogueBodyText>)>,
    mut body_query: Query<&mut Text, (With<DialogueBodyText>, Without<DialogueSpeakerText>)>,
    mut portrait_query: Query<
        (&mut ImageNode, &mut Visibility),
        (With<DialoguePortraitImage>, Without<DialogueUiRoot>),
    >,
) {
    let Ok(mut visibility) = root_query.single_mut() else {
        return;
    };
    let Ok(mut speaker_text) = speaker_query.single_mut() else {
        return;
    };
    let Ok(mut body_text) = body_query.single_mut() else {
        return;
    };

    if !dialogue.active || dialogue.lines.is_empty() {
        *visibility = Visibility::Hidden;
        **speaker_text = String::new();
        **body_text = String::new();
        if let Ok((_, mut portrait_vis)) = portrait_query.single_mut() {
            *portrait_vis = Visibility::Hidden;
        }
        return;
    }

    *visibility = Visibility::Visible;
    **speaker_text = dialogue.speaker.clone();
    **body_text = dialogue.lines[dialogue.index].clone();

    if let Ok((mut image_node, mut portrait_vis)) = portrait_query.single_mut() {
        if let Some(handle) = &dialogue.portrait {
            image_node.image = handle.clone();
            *portrait_vis = Visibility::Visible;
        } else {
            *portrait_vis = Visibility::Hidden;
        }
    }
}

fn advance_dialogue(keyboard: Res<ButtonInput<KeyCode>>, mut dialogue: ResMut<DialogueState>) {
    if !dialogue.active {
        return;
    }

    if !keyboard.just_pressed(KeyCode::Space)
        && !keyboard.just_pressed(KeyCode::Enter)
        && !keyboard.just_pressed(KeyCode::KeyE)
    {
        return;
    }

    if dialogue.index + 1 < dialogue.lines.len() {
        dialogue.index += 1;
    } else {
        dialogue.active = false;
        dialogue.speaker.clear();
        dialogue.lines.clear();
        dialogue.index = 0;
    }
}

fn run_dialogue_cinematic(
    time: Res<Time>,
    mut dialogue: ResMut<DialogueState>,
    mut cinematic: ResMut<DialogueCinematicState>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    if cinematic.phase == DialogueCinematicPhase::Idle {
        return;
    }

    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let Projection::Orthographic(orthographic) = &mut *projection else {
        return;
    };

    if !cinematic.camera_captured {
        cinematic.base_scale = orthographic.scale;
        cinematic.base_translation = camera_transform.translation;
        cinematic.target_scale = cinematic.base_scale * DIALOGUE_TARGET_SCALE;
        cinematic.target_translation = Vec3::new(
            cinematic.focus.x,
            cinematic.base_translation.y,
            cinematic.base_translation.z,
        );
        cinematic.camera_captured = true;
    }

    match cinematic.phase {
        DialogueCinematicPhase::Idle => {}
        DialogueCinematicPhase::ZoomIn => {
            cinematic.timer.tick(time.delta());
            let t = smoothstep(cinematic.timer.fraction());
            orthographic.scale = lerp_f32(cinematic.base_scale, cinematic.target_scale, t);
            camera_transform.translation = cinematic
                .base_translation
                .lerp(cinematic.target_translation, t);

            if cinematic.timer.is_finished() {
                start_dialogue(
                    &mut dialogue,
                    cinematic.speaker.clone(),
                    cinematic.lines.clone(),
                    cinematic.portrait.clone(),
                );
                cinematic.phase = DialogueCinematicPhase::Dialogue;
            }
        }
        DialogueCinematicPhase::Dialogue => {
            orthographic.scale = cinematic.target_scale;
            camera_transform.translation = cinematic.target_translation;

            if !dialogue.active {
                cinematic.phase = DialogueCinematicPhase::ZoomOut;
                cinematic.timer = Timer::from_seconds(DIALOGUE_ZOOM_OUT_DURATION, TimerMode::Once);
            }
        }
        DialogueCinematicPhase::ZoomOut => {
            cinematic.timer.tick(time.delta());
            let t = smoothstep(cinematic.timer.fraction());
            orthographic.scale = lerp_f32(cinematic.target_scale, cinematic.base_scale, t);
            camera_transform.translation = cinematic
                .target_translation
                .lerp(cinematic.base_translation, t);

            if cinematic.timer.is_finished() {
                orthographic.scale = cinematic.base_scale;
                camera_transform.translation = cinematic.base_translation;
                cinematic.phase = DialogueCinematicPhase::Idle;
                cinematic.speaker.clear();
                cinematic.lines.clear();
                cinematic.portrait = None;
                cinematic.focus = Vec3::ZERO;
                cinematic.camera_captured = false;
            }
        }
    }
}

fn lerp_f32(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
