use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
}

// ── Block colors ─────────────────────────────────────────────────────────────

pub const RED_DIM: Color = Color::srgb(0.45, 0.08, 0.08);
pub const GREEN_DIM: Color = Color::srgb(0.08, 0.38, 0.10);
pub const BLUE_DIM: Color = Color::srgb(0.08, 0.14, 0.52);
pub const RED_BRIGHT: Color = Color::srgb(0.95, 0.22, 0.22);
pub const GREEN_BRIGHT: Color = Color::srgb(0.18, 0.92, 0.28);
pub const BLUE_BRIGHT: Color = Color::srgb(0.22, 0.44, 0.98);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[allow(dead_code)]
pub enum BlockColor {
    #[default]
    Green,
    Red,
    Blue,
}

impl BlockColor {
    pub fn dim_color(self) -> Color {
        match self {
            BlockColor::Red => RED_DIM,
            BlockColor::Green => GREEN_DIM,
            BlockColor::Blue => BLUE_DIM,
        }
    }

    pub fn bright_color(self) -> Color {
        match self {
            BlockColor::Red => RED_BRIGHT,
            BlockColor::Green => GREEN_BRIGHT,
            BlockColor::Blue => BLUE_BRIGHT,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            BlockColor::Red => "RED",
            BlockColor::Green => "GRN",
            BlockColor::Blue => "BLU",
        }
    }
}

pub(crate) fn random_puzzle_sequence() -> Vec<BlockColor> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let mut arr = [BlockColor::Green, BlockColor::Red, BlockColor::Blue];
    let mut seed = nanos as u64 | 1;

    for i in (1..3usize).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j = (seed as usize) % (i + 1);
        arr.swap(i, j);
    }
    arr.to_vec()
}

/// Fixed 5-step sequence for Level 4: Blue → Red → Green → Red → Blue.
/// The deliberate repeat forces the player to hit distinct block instances
/// in strict order across the room.
pub(crate) fn level_four_sequence() -> Vec<BlockColor> {
    vec![
        BlockColor::Blue,
        BlockColor::Red,
        BlockColor::Green,
        BlockColor::Red,
        BlockColor::Blue,
    ]
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum LevelId {
    #[default]
    LevelOne,
    LevelTwo,
    LevelThree,
    LevelFour,
    LevelFive,
}

#[derive(Resource, Default)]
pub struct PlayerProfile {
    pub name: String,
}

#[derive(Resource)]
pub struct PlayerHealth {
    pub current: i32,
    pub invincibility_timer: f32,
}

impl Default for PlayerHealth {
    fn default() -> Self {
        Self { current: 3, invincibility_timer: 0.0 }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum FadePhase {
    #[default]
    Idle,
    FadeOut(f32),
    FadeIn(f32),
}

#[derive(Resource, Default)]
pub struct FadeState {
    pub phase: FadePhase,
    pub trigger_restart: bool,
    pub execute_restart: bool,
}

#[derive(Resource)]
pub struct CampaignState {
    pub current_level: LevelId,
    pub wizard_intro_seen: bool,
    pub wizard_followup_seen: bool,
    pub tutorial_hint_seen: bool,
    pub crate_broken: bool,
    pub level_two_goal_complete: bool,
    pub puzzle_sequence: Vec<BlockColor>,
    pub puzzle_progress: usize,
}

impl Default for CampaignState {
    fn default() -> Self {
        Self {
            current_level: LevelId::LevelOne,
            wizard_intro_seen: false,
            wizard_followup_seen: false,
            tutorial_hint_seen: false,
            crate_broken: false,
            level_two_goal_complete: false,
            puzzle_sequence: random_puzzle_sequence(), // overridden per-level on reset
            puzzle_progress: 0,
        }
    }
}
