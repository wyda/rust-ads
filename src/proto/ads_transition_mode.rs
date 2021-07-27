#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AdsTransMode {
    None,
    ClientCylcle,
    ClientOnChange,
    Cyclic,
    OnChange,
    CyclicInContext,
    OnChangeInContext,
}

impl From<u32> for AdsTransMode {
    fn from(state_value: u32) -> Self {
        match state_value {
            0 => AdsTransMode::None,
            2 => AdsTransMode::ClientCylcle,
            3 => AdsTransMode::ClientOnChange,
            4 => AdsTransMode::Cyclic,
            5 => AdsTransMode::OnChange,
            6 => AdsTransMode::CyclicInContext,
            7 => AdsTransMode::OnChangeInContext,
            _ => AdsTransMode::None,
        }
    }
}

impl AdsTransMode {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}
