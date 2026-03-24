use bevy::prelude::*;

use crate::level::TrainingDoor;
use crate::player::{Facing, HasSword, Player, PlayerActionState};
use crate::state::{BlockColor, CampaignState, GameState, LevelId};
use crate::sword::{Sword, SwordState};

pub struct PuzzlePlugin;

const BLOCK_HIT_RADIUS: f32 = 80.0;
const BLOCK_SLASH_RANGE_X: f32 = 120.0;
const BLOCK_SLASH_RANGE_Y: f32 = 96.0;
const HIT_COOLDOWN_SECS: f32 = 0.45;

#[derive(Component)]
pub struct PuzzleBlock {
    pub color: BlockColor,
    pub activated: bool,
    /// Seconds remaining before this block can register another hit.
    pub hit_cooldown: f32,
}

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (check_block_hit, update_block_visuals, sync_puzzle_door)
                .run_if(in_state(GameState::InGame))
                .run_if(|c: Res<CampaignState>| {
                    c.current_level == LevelId::LevelThree
                        || c.current_level == LevelId::LevelFour
                }),
        );
    }
}

/// Detects and processes one block hit per frame.
///
/// Snapshot-based double-borrow strategy: we snapshot block positions into a
/// `Vec` during the cooldown-tick pass, run hit-detection on the snapshot (no
/// live query borrow), then mutate blocks in a second pass.
fn check_block_hit(
    time: Res<Time>,
    mut campaign: ResMut<CampaignState>,
    mut block_query: Query<(&Transform, &mut PuzzleBlock)>,
    player_query: Query<
        (&Transform, &Facing, &HasSword, &PlayerActionState),
        (With<Player>, Changed<PlayerActionState>),
    >,
    sword_query: Query<(&Transform, &SwordState), With<Sword>>,
) {
    if campaign.puzzle_progress >= campaign.puzzle_sequence.len() {
        return;
    }

    // Pass 1 – tick cooldowns and snapshot the state we need for detection.
    let mut snapshot: Vec<(Vec3, BlockColor, bool, f32)> = Vec::new();
    for (tf, mut block) in &mut block_query {
        block.hit_cooldown = (block.hit_cooldown - time.delta_secs()).max(0.0);
        snapshot.push((tf.translation, block.color, block.activated, block.hit_cooldown));
    }

    // Detection – run against the snapshot (no live query borrow needed).
    let mut hit_color: Option<BlockColor> = None;

    // Slash hit — `Changed<PlayerActionState>` ensures this only fires on the
    // first frame the action transitions to Slash, preventing double-counting.
    if let Ok((player_tf, facing, has_sword, action_state)) = player_query.single() {
        if has_sword.0 && *action_state == PlayerActionState::Slash {
            for &(block_pos, color, activated, cooldown) in &snapshot {
                if activated || cooldown > 0.0 {
                    continue;
                }
                let delta = block_pos - player_tf.translation;
                if delta.x.abs() <= BLOCK_SLASH_RANGE_X
                    && delta.y.abs() <= BLOCK_SLASH_RANGE_Y
                    && delta.x * facing.0 >= 0.0
                {
                    hit_color = Some(color);
                    break;
                }
            }
        }
    }

    // Thrown-sword proximity hit.
    if hit_color.is_none() {
        'sword_loop: for (sword_tf, sword_state) in &sword_query {
            if *sword_state == SwordState::Equipped {
                continue;
            }
            for &(block_pos, color, activated, cooldown) in &snapshot {
                if activated || cooldown > 0.0 {
                    continue;
                }
                if sword_tf.translation.distance(block_pos) <= BLOCK_HIT_RADIUS {
                    hit_color = Some(color);
                    break 'sword_loop;
                }
            }
        }
    }

    let Some(color) = hit_color else {
        return;
    };

    let expected = campaign.puzzle_sequence[campaign.puzzle_progress];

    // Pass 2 – apply state changes.
    if color == expected {
        for (_, mut block) in &mut block_query {
            if block.color == color && !block.activated {
                block.activated = true;
                block.hit_cooldown = HIT_COOLDOWN_SECS;
                break;
            }
        }
        campaign.puzzle_progress += 1;
    } else {
        for (_, mut block) in &mut block_query {
            block.activated = false;
            block.hit_cooldown = 0.0;
        }
        campaign.puzzle_progress = 0;
    }
}

/// Syncs sprite color to reflect activated/inactive state.
fn update_block_visuals(
    mut block_query: Query<(&PuzzleBlock, &mut Sprite), Changed<PuzzleBlock>>,
) {
    for (block, mut sprite) in &mut block_query {
        sprite.color = if block.activated {
            block.color.bright_color()
        } else {
            block.color.dim_color()
        };
    }
}

/// Opens the training door as soon as the full sequence is entered correctly.
fn sync_puzzle_door(
    campaign: Res<CampaignState>,
    mut door_query: Query<&mut TrainingDoor>,
) {
    if !campaign.is_changed() {
        return;
    }
    if campaign.puzzle_progress < campaign.puzzle_sequence.len() {
        return;
    }
    for mut door in &mut door_query {
        if !door.open {
            door.open = true;
        }
    }
}
