use bevy::prelude::*;

use crate::dialogue::{DialogueCinematicState, DialogueState, queue_dialogue};
use crate::player::{Facing, HasSword, Player, PlayerActionState, Velocity};
use crate::state::{CampaignState, LevelId, PlayerProfile};
use crate::sword::{Sword, SwordState};

use super::LEVEL_ONE_DOOR_X;
use super::assets::LevelArtHandles;
use super::components::{
    LevelBounds, PendingLevelTransition, TrainingCrate, TrainingDoor, TutorialMarker,
    WizardAnimationFrame, WizardAnimationTimer, WizardNpc,
};
use super::spawn::spawn_level_scene;

const WIZARD_SCALE: f32 = 4.0;
const WIZARD_FOLLOWUP_TRIGGER_X: f32 = -40.0;

pub(super) fn animate_wizard_idle(
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

pub(super) fn constrain_player_to_level(
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

    if campaign.current_level == LevelId::LevelOne
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

pub(super) fn trigger_wizard_intro(
    mut campaign: ResMut<CampaignState>,
    dialogue: Res<DialogueState>,
    mut cinematic: ResMut<DialogueCinematicState>,
    player_profile: Res<PlayerProfile>,
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
    );
}

pub(super) fn trigger_wizard_followup(
    mut campaign: ResMut<CampaignState>,
    dialogue: Res<DialogueState>,
    mut cinematic: ResMut<DialogueCinematicState>,
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
    );
}

pub(super) fn trigger_tutorial_hint(
    mut campaign: ResMut<CampaignState>,
    dialogue: Res<DialogueState>,
    mut cinematic: ResMut<DialogueCinematicState>,
    player_query: Query<&Transform, With<Player>>,
    tutorial_query: Query<&Transform, With<TutorialMarker>>,
) {
    if campaign.current_level != LevelId::LevelOne
        || !campaign.wizard_intro_seen
        || !campaign.wizard_followup_seen
        || campaign.tutorial_hint_seen
        || dialogue.active
        || cinematic.is_active()
    {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(marker_transform) = tutorial_query.single() else {
        return;
    };

    if player_transform
        .translation
        .distance(marker_transform.translation)
        > 145.0
    {
        return;
    }

    campaign.tutorial_hint_seen = true;

    queue_dialogue(
        &mut cinematic,
        "Training Sign",
        vec![
            "Press E near the sword to pick it up.".to_string(),
            "Press H to slash. Press J to throw.".to_string(),
            "If you throw it, you still have to walk over and grab it again.".to_string(),
        ],
        marker_transform.translation + Vec3::new(0.0, 90.0, 0.0),
    );
}

pub(super) fn break_training_crate(
    mut commands: Commands,
    mut campaign: ResMut<CampaignState>,
    crate_query: Query<(Entity, &Transform), With<TrainingCrate>>,
    mut door_query: Query<&mut TrainingDoor>,
    player_query: Query<(&Transform, &Facing, &HasSword, &PlayerActionState), With<Player>>,
    sword_query: Query<(&Transform, &SwordState), With<Sword>>,
) {
    if campaign.current_level != LevelId::LevelOne || campaign.crate_broken {
        return;
    }

    let Ok((crate_entity, crate_transform)) = crate_query.single() else {
        return;
    };

    let mut broken = false;

    if let Ok((player_transform, facing, has_sword, action_state)) = player_query.single()
        && has_sword.0
        && *action_state == PlayerActionState::Slash
    {
        let delta = crate_transform.translation - player_transform.translation;
        broken = delta.x.abs() <= 120.0 && delta.y.abs() <= 80.0 && delta.x * facing.0 >= 0.0;
    }

    if !broken {
        for (sword_transform, sword_state) in &sword_query {
            if *sword_state == SwordState::Equipped {
                continue;
            }

            if sword_transform
                .translation
                .distance(crate_transform.translation)
                <= 86.0
            {
                broken = true;
                break;
            }
        }
    }

    if !broken {
        return;
    }

    commands.entity(crate_entity).despawn();
    campaign.crate_broken = true;

    if let Ok(mut door) = door_query.single_mut() {
        door.open = true;
    }
}

pub(super) fn update_training_door_visual(
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

pub(super) fn try_advance_level(
    campaign: Res<CampaignState>,
    mut pending_transition: ResMut<PendingLevelTransition>,
    player_query: Query<&Transform, With<Player>>,
    door_query: Query<(&Transform, &TrainingDoor), Without<Player>>,
) {
    if campaign.current_level != LevelId::LevelOne || pending_transition.next_level.is_some() {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok((door_transform, door)) = door_query.single() else {
        return;
    };

    if door.open
        && player_transform.translation.x >= door_transform.translation.x + 36.0
        && player_transform.translation.x >= LEVEL_ONE_DOOR_X + 36.0
    {
        pending_transition.next_level = Some(LevelId::LevelTwo);
    }
}

pub(super) fn apply_level_transition(
    mut commands: Commands,
    mut pending_transition: ResMut<PendingLevelTransition>,
    mut campaign: ResMut<CampaignState>,
    art: Res<LevelArtHandles>,
    player_anims: Res<crate::player::PlayerAnimationHandles>,
    sword_visuals: Res<crate::sword::SwordVisualHandles>,
    profile: Res<PlayerProfile>,
    level_entities: Query<Entity, With<super::components::LevelEntity>>,
) {
    let Some(next_level) = pending_transition.next_level.take() else {
        return;
    };

    for entity in &level_entities {
        commands.entity(entity).despawn();
    }

    campaign.current_level = next_level;
    campaign.wizard_intro_seen = false;
    campaign.tutorial_hint_seen = false;
    campaign.crate_broken = false;

    spawn_level_scene(
        &mut commands,
        &art,
        &player_anims,
        &sword_visuals,
        &campaign,
        &profile,
    );
}

pub(super) fn wizard_scale() -> f32 {
    WIZARD_SCALE
}
