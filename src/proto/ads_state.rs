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
}
