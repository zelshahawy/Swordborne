use bevy::prelude::*;

use crate::fonts::GameFonts;
use crate::state::{BossDefeated, GameState, format_run_time};

#[cfg(not(target_arch = "wasm32"))]
use crate::leaderboard::LeaderboardResource;

pub struct VictoryPlugin;

#[derive(Component)]
struct VictoryOverlay;

#[derive(Component)]
struct VictoryRankText;

#[derive(Component)]
struct ReturnToMenuButton;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::InGame), despawn_victory_overlay)
            .add_systems(
                Update,
                (spawn_victory_on_defeat, handle_return_button)
                    .run_if(in_state(GameState::InGame)),
            );

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            sync_rank_text.run_if(in_state(GameState::InGame)),
        );
    }
}

fn despawn_victory_overlay(
    mut commands: Commands,
    query: Query<Entity, With<VictoryOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_victory_on_defeat(
    mut commands: Commands,
    defeated: Res<BossDefeated>,
    fonts: Res<GameFonts>,
    existing: Query<(), With<VictoryOverlay>>,
) {
    if !defeated.triggered || !existing.is_empty() {
        return;
    }
    let time_str = format_run_time(defeated.time_secs);
    commands
        .spawn((
            VictoryOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
            ZIndex(50),
        ))
        .with_children(|root| {
            root.spawn((
                VictoryOverlay,
                Node {
                    width: Val::Px(500.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(44.0)),
                    row_gap: Val::Px(16.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.04, 0.08, 0.96)),
                BorderColor::all(Color::srgb(0.82, 0.65, 0.22)),
            ))
            .with_children(|card| {
                card.spawn((
                    VictoryOverlay,
                    Text::new("VICTORY"),
                    TextFont { font: fonts.pixel_bold.clone(), font_size: 56.0, ..default() },
                    TextColor(Color::srgb(0.98, 0.88, 0.42)),
                ));

                card.spawn((
                    VictoryOverlay,
                    Text::new(format!("Your time:  {time_str}")),
                    TextFont { font: fonts.pixel_bold.clone(), font_size: 22.0, ..default() },
                    TextColor(Color::srgb(0.95, 0.95, 0.90)),
                ));

                card.spawn((
                    VictoryOverlay,
                    VictoryRankText,
                    Text::new(if cfg!(target_arch = "wasm32") { "" } else { "Computing rank..." }),
                    TextFont { font: fonts.pixel_regular.clone(), font_size: 17.0, ..default() },
                    TextColor(Color::srgb(0.72, 0.77, 0.90)),
                ));

                card.spawn((
                    VictoryOverlay,
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Px(1.0),
                        margin: UiRect::vertical(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.95, 0.93, 0.88, 0.14)),
                ));

                card.spawn((
                    VictoryOverlay,
                    ReturnToMenuButton,
                    Button,
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(18.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        VictoryOverlay,
                        Text::new("Return to Menu"),
                        TextFont { font: fonts.pixel_bold.clone(), font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_rank_text(
    lb: Res<LeaderboardResource>,
    mut query: Query<&mut Text, With<VictoryRankText>>,
) {
    if !lb.is_changed() {
        return;
    }
    let Ok(mut text) = query.single_mut() else { return; };
    if let Some(rank) = lb.rank {
        **text = format!("You ranked  #{rank}  on the leaderboard");
    }
}

fn handle_return_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReturnToMenuButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut defeated: ResMut<BossDefeated>,
    #[cfg(not(target_arch = "wasm32"))]
    mut lb: ResMut<LeaderboardResource>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            defeated.triggered = false;
            #[cfg(not(target_arch = "wasm32"))]
            { lb.rank = None; }
            next_state.set(GameState::MainMenu);
        }
    }
}
