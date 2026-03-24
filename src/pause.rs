use bevy::prelude::*;

use crate::fonts::GameFonts;
use crate::state::{FadePhase, FadeState, GameState};

const FADE_SPEED: f32 = 4.0;

#[derive(Component)]
struct FadeOverlay;

#[derive(Component)]
struct ControlsOverlay;

#[derive(Component)]
struct PauseUiEntity;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_pause_ui)
            .add_systems(OnExit(GameState::InGame), despawn_pause_ui)
            .add_systems(
                Update,
                (tick_fade, toggle_controls_overlay).run_if(in_state(GameState::InGame)),
            );
    }
}

fn spawn_pause_ui(mut commands: Commands, fonts: Res<GameFonts>) {
    commands.spawn((
        PauseUiEntity,
        FadeOverlay,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        ZIndex(1000),
    ));

    commands
        .spawn((
            PauseUiEntity,
            ControlsOverlay,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            ZIndex(999),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    padding: UiRect::all(Val::Px(32.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.06, 0.10, 0.96)),
                BorderColor::all(Color::srgba(0.5, 0.5, 0.8, 0.6)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("CONTROLS"),
                    TextFont {
                        font: fonts.pixel_bold.clone(),
                        font_size: 18.0,
                        ..default()
                    },
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
                    ("ESC", "Toggle this menu"),
                ] {
                    panel.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(16.0),
                        width: Val::Px(340.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(key),
                            TextFont {
                                font: fonts.pixel_bold.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.70, 0.85, 1.0)),
                            Node { width: Val::Px(160.0), ..default() },
                        ));
                        row.spawn((
                            Text::new(action),
                            TextFont {
                                font: fonts.pixel_regular.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.85, 0.85, 0.85)),
                        ));
                    });
                }
            });
        });
}

fn despawn_pause_ui(mut commands: Commands, query: Query<Entity, With<PauseUiEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn tick_fade(
    time: Res<Time>,
    mut fade_state: ResMut<FadeState>,
    mut bg_query: Query<&mut BackgroundColor, With<FadeOverlay>>,
) {
    let Ok(mut bg) = bg_query.single_mut() else { return; };
    let dt = time.delta_secs();

    match fade_state.phase {
        FadePhase::Idle => {
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
        FadePhase::FadeOut(t) => {
            let new_t = (t + dt * FADE_SPEED).min(1.0);
            bg.0 = Color::srgba(0.0, 0.0, 0.0, new_t);
            if new_t >= 1.0 {
                if fade_state.trigger_restart {
                    fade_state.execute_restart = true;
                    fade_state.trigger_restart = false;
                }
                fade_state.phase = FadePhase::FadeIn(1.0);
            } else {
                fade_state.phase = FadePhase::FadeOut(new_t);
            }
        }
        FadePhase::FadeIn(t) => {
            let new_t = (t - dt * FADE_SPEED).max(0.0);
            bg.0 = Color::srgba(0.0, 0.0, 0.0, new_t);
            fade_state.phase =
                if new_t <= 0.0 { FadePhase::Idle } else { FadePhase::FadeIn(new_t) };
        }
    }
}

fn toggle_controls_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Visibility, With<ControlsOverlay>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    let Ok(mut vis) = query.single_mut() else { return; };
    *vis = match *vis {
        Visibility::Hidden => Visibility::Visible,
        _ => Visibility::Hidden,
    };
}
