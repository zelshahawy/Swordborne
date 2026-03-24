use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum LevelId {
    #[default]
    LevelOne,
    LevelTwo,
    LevelThree,
}

#[derive(Resource, Default)]
pub struct PlayerProfile {
    pub name: String,
}

#[derive(Resource)]
pub struct CampaignState {
    pub current_level: LevelId,
    pub wizard_intro_seen: bool,
    pub wizard_followup_seen: bool,
    pub tutorial_hint_seen: bool,
    pub crate_broken: bool,
    pub level_two_goal_complete: bool,
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
            puzzle_progress: 0,
        }
    }
}
