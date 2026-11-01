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

/// Why an action was refused. The state is never changed when an error is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionError {
    /// The turn counter is already at `u64::MAX`.
    TurnOverflow,
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TurnOverflow => f.write_str("turn counter is already at its maximum"),
        }
    }
}

impl std::error::Error for ActionError {}

/// Apply `action` to `state`. On `Err`, `state` is left exactly as it was.
pub fn apply_action(state: &mut GameState, action: Action) -> Result<(), ActionError> {
    match action {
        Action::Wait => {
            state.turn = state.turn.checked_add(1).ok_or(ActionError::TurnOverflow)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Action, ActionError, GameState, apply_action};

    #[test]
    fn wait_advances_turn_once_and_preserves_player_health() {
        let mut state = GameState {
            turn: 7,
            player_health: 24,
        };

        assert_eq!(apply_action(&mut state, Action::Wait), Ok(()));

        assert_eq!(state.turn, 8);
        assert_eq!(state.player_health, 24);
    }

    #[test]
    fn wait_at_max_turn_returns_overflow_and_keeps_state() {
        let mut state = GameState {
            turn: u64::MAX,
            player_health: 24,
        };
        let before = state;

        assert_eq!(
            apply_action(&mut state, Action::Wait),
            Err(ActionError::TurnOverflow)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn wait_one_below_max_reaches_max() {
        let mut state = GameState {
            turn: u64::MAX - 1,
            player_health: 24,
        };

        assert_eq!(apply_action(&mut state, Action::Wait), Ok(()));
        assert_eq!(state.turn, u64::MAX);
    }

    #[test]
    fn turn_overflow_is_a_displayable_std_error() {
        let error: Box<dyn std::error::Error> = Box::new(ActionError::TurnOverflow);

        assert_eq!(error.to_string(), "turn counter is already at its maximum");
        assert!(error.source().is_none());
    }
}
