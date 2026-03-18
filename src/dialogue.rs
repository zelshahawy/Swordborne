use bevy::prelude::*;

use crate::state::GameState;

pub struct DialoguePlugin;

const DIALOGUE_ZOOM_IN_DURATION: f32 = 0.35;
const DIALOGUE_ZOOM_OUT_DURATION: f32 = 0.28;
const DIALOGUE_TARGET_SCALE: f32 = 0.78;

#[derive(Resource, Default)]
pub struct DialogueState {
    pub active: bool,
    pub speaker: String,
    pub lines: Vec<String>,
    pub index: usize,
}

#[derive(Resource)]
struct DialogueUiHandles {
    fantasy_font: Handle<Font>,
}

#[derive(Resource, Default)]
pub struct DialogueCinematicState {
    phase: DialogueCinematicPhase,
    timer: Timer,
    speaker: String,
    lines: Vec<String>,
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

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogueState>()
            .init_resource::<DialogueUiHandles>()
            .init_resource::<DialogueCinematicState>()
            .add_systems(OnEnter(GameState::InGame), spawn_dialogue_ui)
            .add_systems(OnExit(GameState::InGame), despawn_dialogue_ui)
            .add_systems(
                Update,
                (run_dialogue_cinematic, sync_dialogue_ui, advance_dialogue)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

impl FromWorld for DialogueUiHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();

        Self {
            fantasy_font: asset_server.load("apple_chancery.ttf"),
        }
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

pub fn start_dialogue<I, S>(dialogue: &mut DialogueState, speaker: impl Into<String>, lines: I)
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    dialogue.active = true;
    dialogue.speaker = speaker.into();
    dialogue.lines = lines.into_iter().map(Into::into).collect();
    dialogue.index = 0;
}

pub fn queue_dialogue<I, S>(
    cinematic: &mut DialogueCinematicState,
    speaker: impl Into<String>,
    lines: I,
    focus: Vec3,
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
    cinematic.focus = focus;
    cinematic.target_scale = DIALOGUE_TARGET_SCALE;
    cinematic.camera_captured = false;
}

fn spawn_dialogue_ui(mut commands: Commands, ui: Res<DialogueUiHandles>) {
    commands
        .spawn((
            DialogueUiRoot,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                left: px(40.0),
                right: px(40.0),
                bottom: px(32.0),
                min_height: px(180.0),
                padding: UiRect::axes(px(28.0), px(18.0)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10.0),
                border: UiRect::all(px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.07, 0.12, 0.92)),
            BorderColor::all(Color::srgb(0.82, 0.72, 0.44)),
        ))
        .with_children(|parent| {
            parent.spawn((
                DialogueSpeakerText,
                Text::new(""),
                TextFont {
                    font: ui.fantasy_font.clone(),
                    font_size: 38.0,
                    ..default()
                },
                TextColor(Color::srgb(0.97, 0.86, 0.58)),
            ));

            parent.spawn((
                DialogueBodyText,
                Text::new(""),
                TextFont {
                    font: ui.fantasy_font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.95, 0.98)),
            ));

            parent.spawn((
                DialogueHintText,
                Text::new("Press Space, Enter, or E to continue"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.67, 0.75, 0.9)),
            ));
        });
}

fn despawn_dialogue_ui(mut commands: Commands, query: Query<Entity, With<DialogueUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn sync_dialogue_ui(
    dialogue: Res<DialogueState>,
    mut root_query: Query<&mut Visibility, With<DialogueUiRoot>>,
    mut speaker_query: Query<&mut Text, (With<DialogueSpeakerText>, Without<DialogueBodyText>)>,
    mut body_query: Query<&mut Text, (With<DialogueBodyText>, Without<DialogueSpeakerText>)>,
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
        return;
    }

    *visibility = Visibility::Visible;
    **speaker_text = dialogue.speaker.clone();
    **body_text = dialogue.lines[dialogue.index].clone();
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
            cinematic.focus.y,
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
