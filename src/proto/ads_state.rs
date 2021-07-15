#[derive(Debug, PartialEq, Clone)]
pub enum AdsState {
    AdsStateInvalid,
    AdsStateIdle,
    AdsStateReset,
    AdsStateInit,
    AdsStateStart,
    AdsStateRun,
    AdsStateStop,
    AdsStateSaveCFG,
    AdsStateLoadCFG,
    AdsStatePowerFailure,
    AdsStatePowerGood,
    AdsStateError,
    AdsStateShutDown,
    AdsStateSuspend,
    AdsStateResume,
    AdsStateConfig,
    AdsStateReconfig,
}

impl AdsState {
    pub fn get_value(state: AdsState) -> u16 {
        match state {
            AdsState::AdsStateInvalid => 0,
            AdsState::AdsStateIdle => 1,
            AdsState::AdsStateReset => 2,
            AdsState::AdsStateInit => 3,
            AdsState::AdsStateStart => 4,
            AdsState::AdsStateRun => 5,
            AdsState::AdsStateStop => 6,
            AdsState::AdsStateSaveCFG => 7,
            AdsState::AdsStateLoadCFG => 8,
            AdsState::AdsStatePowerFailure => 9,
            AdsState::AdsStatePowerGood => 10,
            AdsState::AdsStateError => 11,
            AdsState::AdsStateShutDown => 12,
            AdsState::AdsStateSuspend => 13,
            AdsState::AdsStateResume => 14,
            AdsState::AdsStateConfig => 15,
            AdsState::AdsStateReconfig => 16,
        }
    }

    pub fn from_u16(state_value: u16) -> Option<Self> {
        match state_value {
            0 => Some(AdsState::AdsStateInvalid),
            1 => Some(AdsState::AdsStateIdle),
            2 => Some(AdsState::AdsStateReset),
            3 => Some(AdsState::AdsStateInit),
            4 => Some(AdsState::AdsStateStart),
            5 => Some(AdsState::AdsStateRun),
            6 => Some(AdsState::AdsStateStop),
            7 => Some(AdsState::AdsStateSaveCFG),
            8 => Some(AdsState::AdsStateLoadCFG),
            9 => Some(AdsState::AdsStatePowerFailure),
            10 => Some(AdsState::AdsStatePowerGood),
            11 => Some(AdsState::AdsStateError),
            12 => Some(AdsState::AdsStateShutDown),
            13 => Some(AdsState::AdsStateSuspend),
            14 => Some(AdsState::AdsStateResume),
            15 => Some(AdsState::AdsStateConfig),
            16 => Some(AdsState::AdsStateReconfig),
            _ => None,
        }
    }
}
