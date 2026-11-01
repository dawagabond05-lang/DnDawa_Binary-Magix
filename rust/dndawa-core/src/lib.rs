#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameState {
    pub turn: u64,
    pub player_health: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wait,
}

pub fn apply_action(state: &mut GameState, action: Action) {
    match action {
        Action::Wait => state.turn += 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, GameState, apply_action};

    #[test]
    fn wait_advances_turn_once_and_preserves_player_health() {
        let mut state = GameState {
            turn: 7,
            player_health: 24,
        };

        apply_action(&mut state, Action::Wait);

        assert_eq!(state.turn, 8);
        assert_eq!(state.player_health, 24);
    }
}
