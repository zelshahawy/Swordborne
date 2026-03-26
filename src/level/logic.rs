use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::dialogue::{DialogueCinematicState, DialoguePortraits, DialogueState, queue_dialogue};
use crate::fonts::GameFonts;
use crate::level::{
    BreakableCrate, CrateBreakShard, CrateReward, LevelArtHandles, LevelBounds, LevelEntity,
    LevelFourCompletionText, LevelThreeCompletionText, LevelTwoCompletionText,
    PendingLevelTransition, TrainingDoor, WizardAnimationFrame, WizardAnimationTimer, WizardNpc,
    frame_level_camera, level_bounds_for, level_camera_focus_x, spawn_level_scene,
};
use crate::player::{Facing, HasSword, Player, PlayerActionState, Velocity};
use crate::state::{BossDefeated, CampaignState, FadePhase, FadeState, LevelId, PlayerHealth, PlayerProfile};
use crate::sword::{Sword, SwordAimGuide, SwordAimReticle, SwordAimState, SwordState, SwordTrail};

const WIZARD_FOLLOWUP_TRIGGER_X: f32 = -40.0;
const CRATE_BREAK_RADIUS: f32 = 86.0;
const CRATE_SHARD_LIFETIME: f32 = 0.42;
const CRATE_SHARD_GRAVITY: f32 = -980.0;

pub(crate) fn animate_wizard_idle(
    time: Res<Time>,
    art: Res<LevelArtHandles>,
    mut query: Query<
        (
            &mut Sprite,
            &mut WizardAnimationTimer,
            &mut WizardAnimationFrame,
        ),
        With<WizardNpc>,
    >,
) {
    for (mut sprite, mut timer, mut frame) in &mut query {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            frame.0 = (frame.0 + 1) % art.wizard_idle_frames.len();
            sprite.image = art.wizard_idle_frames[frame.0].clone();
            sprite.texture_atlas = None;
        }
    }
}

pub(crate) fn constrain_player_to_level(
    campaign: Res<CampaignState>,
    bounds: Res<LevelBounds>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    door_query: Query<(&Transform, &TrainingDoor), Without<Player>>,
) {
    let Ok((mut transform, mut velocity)) = player_query.single_mut() else {
        return;
    };

    let min_x = bounds.player_left_x;
    let mut max_x = bounds.player_right_x;

    // Block the player at a closed door in levels that have one.
    let level_has_door = matches!(
        campaign.current_level,
        LevelId::LevelOne | LevelId::LevelTwo | LevelId::LevelThree | LevelId::LevelFour
        // LevelFive has no exit door — boss room ends on defeat
    );
    if level_has_door
        && let Ok((door_transform, door)) = door_query.single()
        && !door.open
    {
        max_x = door_transform.translation.x - 88.0;
    }

    if transform.translation.x < min_x {
        transform.translation.x = min_x;
        velocity.x = 0.0;
    }

    if transform.translation.x > max_x {
        transform.translation.x = max_x;
        velocity.x = 0.0;
    }
}

pub(crate) fn trigger_wizard_intro(
    mut campaign: ResMut<CampaignState>,
    dialogue: Res<DialogueState>,
    mut cinematic: ResMut<DialogueCinematicState>,
    player_profile: Res<PlayerProfile>,
    portraits: Res<DialoguePortraits>,
    player_query: Query<&Transform, With<Player>>,
    wizard_query: Query<&Transform, With<WizardNpc>>,
) {
    if campaign.current_level != LevelId::LevelOne
        || campaign.wizard_intro_seen
        || dialogue.active
        || cinematic.is_active()
    {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(wizard_transform) = wizard_query.single() else {
        return;
    };

    if player_transform
        .translation
        .distance(wizard_transform.translation)
        > 180.0
    {
        return;
    }

    campaign.wizard_intro_seen = true;

    let knight_name = if player_profile.name.is_empty() {
        "Knight"
    } else {
        player_profile.name.as_str()
    };

    queue_dialogue(
        &mut cinematic,
        "Wizard",
        vec![
            format!(
                "Ah, {knight_name}. Splendid news: Big Castle has accepted you for an internship. Very prestigious. Alarmingly fond of paperwork."
            ),
            "The hiring ravens said your sword arm showed 'strong leadership potential,' which is castle-speak for 'please survive orientation.'"
                .to_string(),
        ],
        wizard_transform.translation + Vec3::new(0.0, 110.0, 0.0),
        Some(portraits.wizard.clone()),
    );
}

pub(crate) fn trigger_wizard_followup(
    mut campaign: ResMut<CampaignState>,
    dialogue: Res<DialogueState>,
    mut cinematic: ResMut<DialogueCinematicState>,
    portraits: Res<DialoguePortraits>,
    player_query: Query<&Transform, With<Player>>,
) {
    if campaign.current_level != LevelId::LevelOne
        || !campaign.wizard_intro_seen
        || campaign.wizard_followup_seen
        || dialogue.active
        || cinematic.is_active()
    {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    if player_transform.translation.x < WIZARD_FOLLOWUP_TRIGGER_X {
        return;
    }

    campaign.wizard_followup_seen = true;

    queue_dialogue(
        &mut cinematic,
        "Wizard",
        vec![
            "One tiny complication: your offer letter is buried somewhere deep in this dungeon. Big Castle calls that 'the final interview loop.'"
                .to_string(),
            "Take the sword, break the training crate, and press on. Every opened door brings you one step closer to onboarding."
                .to_string(),
        ],
        player_transform.translation + Vec3::new(0.0, 90.0, 0.0),
        Some(portraits.wizard.clone()),
    );
}

pub(crate) fn trigger_dark_wizard_intro(
    mut campaign: ResMut<CampaignState>,
    dialogue: Res<DialogueState>,
    mut cinematic: ResMut<DialogueCinematicState>,
    player_profile: Res<PlayerProfile>,
    portraits: Res<DialoguePortraits>,
    player_query: Query<&Transform, With<Player>>,
) {
    if campaign.current_level != LevelId::LevelFive
        || campaign.wizard_intro_seen
        || dialogue.active
        || cinematic.is_active()
    {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    campaign.wizard_intro_seen = true;

    let knight_name = if player_profile.name.is_empty() {
        "Knight"
    } else {
        player_profile.name.as_str()
    };

    queue_dialogue(
        &mut cinematic,
        "Dark Wizard",
        vec![
            format!(
                "So, {knight_name}... you actually made it this far. Impressive. Foolish, but impressive."
            ),
            "No one has ever left my lair with their offer letter. Today will be no exception."
                .to_string(),
        ],
        player_transform.translation + Vec3::new(200.0, 110.0, 0.0),
        Some(portraits.dark_wizard.clone()),
    );
}

pub(crate) fn break_crates(
    mut commands: Commands,
    mut campaign: ResMut<CampaignState>,
    art: Res<LevelArtHandles>,
    crate_query: Query<(Entity, &Transform, &BreakableCrate)>,
    mut door_query: Query<&mut TrainingDoor>,
    player_query: Query<(&Transform, &Facing, &HasSword, &PlayerActionState), With<Player>>,
    sword_query: Query<(&Transform, &SwordState), With<Sword>>,
) {
    let player_attack = player_query.single().ok();

    for (crate_entity, crate_transform, crate_info) in &crate_query {
        let mut broken = false;

        if let Some((player_transform, facing, has_sword, action_state)) = player_attack
            && has_sword.0
            && *action_state == PlayerActionState::Slash
        {
            let delta = crate_transform.translation - player_transform.translation;
            broken = delta.x.abs() <= 120.0 && delta.y.abs() <= 96.0 && delta.x * facing.0 >= 0.0;
        }

        if !broken {
            for (sword_transform, sword_state) in &sword_query {
                if *sword_state == SwordState::Equipped {
                    continue;
                }

                if sword_transform
                    .translation
                    .distance(crate_transform.translation)
                    <= CRATE_BREAK_RADIUS
                {
                    broken = true;
                    break;
                }
            }
        }

        if !broken {
            continue;
        }

        commands.entity(crate_entity).despawn();
        spawn_crate_break_effect(&mut commands, &art, crate_transform.translation);

        match crate_info.reward {
            CrateReward::OpenTrainingDoor => {
                campaign.crate_broken = true;

                if let Ok(mut door) = door_query.single_mut() {
                    door.open = true;
                }
            }
            CrateReward::CompleteLevelTwo => {
                campaign.level_two_goal_complete = true;
            }
        }
    }
}

pub(crate) fn update_training_door_visual(
    art: Res<LevelArtHandles>,
    mut query: Query<(&TrainingDoor, &mut Sprite), Changed<TrainingDoor>>,
) {
    for (door, mut sprite) in &mut query {
        *sprite = if door.open {
            Sprite::from_image(art.door_open.clone())
        } else {
            Sprite::from_image(art.door_closed.clone())
        };
    }
}

pub(crate) fn sync_level_two_completion_text(
    campaign: Res<CampaignState>,
    mut query: Query<&mut Visibility, With<LevelTwoCompletionText>>,
) {
    if !campaign.is_changed() {
        return;
    }

    for mut visibility in &mut query {
        *visibility =
            if campaign.current_level == LevelId::LevelTwo && campaign.level_two_goal_complete {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
    }
}

pub(crate) fn update_crate_break_shards(
    mut commands: Commands,
    time: Res<Time>,
    mut shard_query: Query<(Entity, &mut Transform, &mut Sprite, &mut CrateBreakShard)>,
) {
    for (entity, mut transform, mut sprite, mut shard) in &mut shard_query {
        shard.timer.tick(time.delta());
        shard.velocity.y += CRATE_SHARD_GRAVITY * time.delta_secs();
        transform.translation.x += shard.velocity.x * time.delta_secs();
        transform.translation.y += shard.velocity.y * time.delta_secs();
        transform.rotate_z(shard.spin_speed * time.delta_secs());

        let remaining = 1.0
            - (shard.timer.elapsed_secs() / shard.timer.duration().as_secs_f32()).clamp(0.0, 1.0);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, remaining);

        if shard.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn try_advance_level(
    campaign: Res<CampaignState>,
    mut pending_transition: ResMut<PendingLevelTransition>,
    player_query: Query<&Transform, With<Player>>,
    door_query: Query<(&Transform, &TrainingDoor), Without<Player>>,
) {
    if pending_transition.next_level.is_some() {
        return;
    }

    let next = match campaign.current_level {
        LevelId::LevelOne => LevelId::LevelTwo,
        LevelId::LevelTwo => LevelId::LevelThree,
        LevelId::LevelThree => LevelId::LevelFour,
        LevelId::LevelFour => LevelId::LevelFive,
        LevelId::LevelFive => return,
    };

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok((door_transform, door)) = door_query.single() else {
        return;
    };

    if door.open && player_transform.translation.x >= door_transform.translation.x + 8.0 {
        pending_transition.next_level = Some(next);
    }
}

pub(crate) fn request_level_restart(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut fade_state: ResMut<FadeState>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) && fade_state.phase == FadePhase::Idle {
        fade_state.phase = FadePhase::FadeOut(0.0);
    }
}

pub(crate) fn execute_level_restart(
    mut fade_state: ResMut<FadeState>,
    mut player_health: ResMut<PlayerHealth>,
    mut defeated: ResMut<BossDefeated>,
    mut commands: Commands,
    mut campaign: ResMut<CampaignState>,
    mut dialogue: ResMut<DialogueState>,
    mut cinematic: ResMut<DialogueCinematicState>,
    mut sword_aim: ResMut<SwordAimState>,
    art: Res<LevelArtHandles>,
    fonts: Res<GameFonts>,
    player_anims: Res<crate::player::PlayerAnimationHandles>,
    sword_visuals: Res<crate::sword::SwordVisualHandles>,
    profile: Res<PlayerProfile>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut params: ParamSet<(
        Query<Entity, With<LevelEntity>>,
        Query<Entity, With<SwordTrail>>,
        Query<&mut Visibility, With<SwordAimGuide>>,
        Query<&mut Visibility, With<SwordAimReticle>>,
    )>,
) {
    if fade_state.phase != FadePhase::Restart {
        return;
    }
    fade_state.phase = FadePhase::FadeIn(1.0);

    for entity in params.p0().iter() {
        commands.entity(entity).despawn();
    }
    for entity in params.p1().iter() {
        commands.entity(entity).despawn();
    }
    if let Ok(mut vis) = params.p2().single_mut() {
        *vis = Visibility::Hidden;
    }
    if let Ok(mut vis) = params.p3().single_mut() {
        *vis = Visibility::Hidden;
    }

    *dialogue = DialogueState::default();
    *cinematic = DialogueCinematicState::default();
    sword_aim.reset();
    reset_level_progress(&mut campaign);
    *player_health = PlayerHealth::default();
    *defeated = BossDefeated::default();

    frame_level_camera(
        &mut camera_query,
        Some(&window_query),
        Some(level_bounds_for(campaign.current_level)),
        Some(level_camera_focus_x(campaign.current_level)),
    );
    spawn_level_scene(&mut commands, &art, &fonts, &player_anims, &sword_visuals, &campaign, &profile);
}

pub(crate) fn apply_level_transition(
    mut commands: Commands,
    mut pending_transition: ResMut<PendingLevelTransition>,
    mut campaign: ResMut<CampaignState>,
    art: Res<LevelArtHandles>,
    fonts: Res<GameFonts>,
    player_anims: Res<crate::player::PlayerAnimationHandles>,
    sword_visuals: Res<crate::sword::SwordVisualHandles>,
    profile: Res<PlayerProfile>,
    level_entities: Query<Entity, With<LevelEntity>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let Some(next_level) = pending_transition.next_level.take() else {
        return;
    };

    for entity in &level_entities {
        commands.entity(entity).despawn();
    }

    campaign.current_level = next_level;
    reset_level_progress(&mut campaign);

    frame_level_camera(
        &mut camera_query,
        Some(&window_query),
        Some(level_bounds_for(campaign.current_level)),
        Some(level_camera_focus_x(campaign.current_level)),
    );

    spawn_level_scene(
        &mut commands,
        &art,
        &fonts,
        &player_anims,
        &sword_visuals,
        &campaign,
        &profile,
    );
}

fn spawn_crate_break_effect(commands: &mut Commands, art: &LevelArtHandles, position: Vec3) {
    let center = position + Vec3::new(0.0, 42.0, 6.0);
    let shards = [
        (
            Vec2::new(-150.0, 320.0),
            -6.0,
            Vec2::new(26.0, 30.0),
            Vec2::new(-18.0, 18.0),
        ),
        (
            Vec2::new(-70.0, 380.0),
            -3.5,
            Vec2::new(22.0, 34.0),
            Vec2::new(16.0, 22.0),
        ),
        (
            Vec2::new(70.0, 380.0),
            3.5,
            Vec2::new(22.0, 34.0),
            Vec2::new(-16.0, 22.0),
        ),
        (
            Vec2::new(150.0, 320.0),
            6.0,
            Vec2::new(26.0, 30.0),
            Vec2::new(18.0, 18.0),
        ),
    ];

    for (velocity, spin_speed, size, offset) in shards {
        commands.spawn((
            LevelEntity,
            Sprite {
                image: art.crate_texture.clone(),
                custom_size: Some(size),
                ..default()
            },
            Transform::from_translation(center + offset.extend(0.0)),
            CrateBreakShard {
                velocity,
                spin_speed,
                timer: Timer::from_seconds(CRATE_SHARD_LIFETIME, TimerMode::Once),
            },
        ));
    }
}

pub(crate) fn sync_level_two_door(
    campaign: Res<CampaignState>,
    mut door_query: Query<&mut TrainingDoor>,
) {
    if !campaign.is_changed() {
        return;
    }
    if campaign.current_level != LevelId::LevelTwo || !campaign.level_two_goal_complete {
        return;
    }
    for mut door in &mut door_query {
        if !door.open {
            door.open = true;
        }
    }
}

pub(crate) fn sync_level_three_completion_text(
    campaign: Res<CampaignState>,
    mut query: Query<&mut Visibility, With<LevelThreeCompletionText>>,
) {
    if !campaign.is_changed() {
        return;
    }
    for mut vis in &mut query {
        *vis = if campaign.current_level == LevelId::LevelThree
            && campaign.puzzle_progress >= campaign.puzzle_sequence.len()
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn reset_level_progress(campaign: &mut CampaignState) {
    campaign.wizard_intro_seen = false;
    campaign.wizard_followup_seen = false;
    campaign.tutorial_hint_seen = false;
    campaign.crate_broken = false;
    campaign.level_two_goal_complete = false;
    campaign.puzzle_sequence = match campaign.current_level {
        LevelId::LevelFour => crate::state::level_four_sequence(),
        _ => crate::state::random_puzzle_sequence(), // LevelFive uses no puzzle
    };
    campaign.puzzle_progress = 0;
}

pub(crate) fn sync_level_four_completion_text(
    campaign: Res<CampaignState>,
    mut query: Query<&mut Visibility, With<LevelFourCompletionText>>,
) {
    if !campaign.is_changed() {
        return;
    }
    for mut vis in &mut query {
        *vis = if campaign.current_level == LevelId::LevelFour
            && campaign.puzzle_progress >= campaign.puzzle_sequence.len()
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
