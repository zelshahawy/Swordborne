use bevy::prelude::*;

use crate::player::{ActionTimer, HasSword, Player, PlayerActionState};

const SLASH_DURATION: f32 = 0.22;

type SlashQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static HasSword,
        &'static mut PlayerActionState,
        &'static mut ActionTimer,
    ),
    With<Player>,
>;

pub fn start_slash(keyboard: Res<ButtonInput<KeyCode>>, mut query: SlashQuery) {
    if !keyboard.just_pressed(KeyCode::KeyH) {
        return;
    }

    let Ok((has_sword, mut action_state, mut action_timer)) = query.single_mut() else {
        return;
    };

    if !has_sword.0 {
        return;
    }

    if *action_state != PlayerActionState::None {
        return;
    }

    *action_state = PlayerActionState::Slash;
    action_timer.0 = Timer::from_seconds(SLASH_DURATION, TimerMode::Once);
}

pub fn tick_player_action(
    time: Res<Time>,
    mut query: Query<(&mut PlayerActionState, &mut ActionTimer), With<Player>>,
) {
    let Ok((mut action_state, mut action_timer)) = query.single_mut() else {
        return;
    };

    if *action_state == PlayerActionState::None {
        return;
    }

    action_timer.0.tick(time.delta());

    if action_timer.0.is_finished() {
        *action_state = PlayerActionState::None;
    }
}
