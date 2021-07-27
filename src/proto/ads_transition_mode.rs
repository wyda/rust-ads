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
            1 => AdsTransMode::ClientCylcle,
            2 => AdsTransMode::ClientOnChange,
            3 => AdsTransMode::Cyclic,
            4 => AdsTransMode::OnChange,
            5 => AdsTransMode::CyclicInContext,
            6 => AdsTransMode::OnChangeInContext,
            _ => AdsTransMode::None,
        }
    }
}

impl AdsTransMode {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}
