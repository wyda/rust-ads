#[derive(Debug, Clone)]
pub enum PlcTypes {
    Bool,
    Byte,
    Word,
    DWord,
    LWord,
    SInt,
    USInt,
    Int,
    UInt,
    DInt,
    UDInt,
    LInt,
    ULInt,
    Real,
    LReal,
    Time,
    TimeOfDay,
    Date,
    DateAndTime,
}

impl PlcTypes {
    pub fn size(&self) -> usize {
        match self {
            PlcTypes::Bool => 1,
            PlcTypes::Byte => 1,
            PlcTypes::Word => 2,
            PlcTypes::DWord => 4,
            PlcTypes::LWord => 8,
            PlcTypes::SInt => 1,
            PlcTypes::USInt => 1,
            PlcTypes::Int => 2,
            PlcTypes::UInt => 2,
            PlcTypes::DInt => 4,
            PlcTypes::UDInt => 4,
            PlcTypes::LInt => 8,
            PlcTypes::ULInt => 8,
            PlcTypes::Real => 4,
            PlcTypes::LReal => 8,
            PlcTypes::Time => 4,
            PlcTypes::TimeOfDay => 4,
            PlcTypes::Date => 4,
            PlcTypes::DateAndTime => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub plc_type: PlcTypes,
    pub data: Vec<u8>,
}

impl Var {
    pub fn new(name: String, plc_type: PlcTypes, data: Option<Vec<u8>>) -> Self {
        if let Some(data) = data {
            Var {
                name,
                plc_type,
                data,
            }
        } else {
            let data = Vec::new();
            Var {
                name,
                plc_type,
                data,
            }
        }
    }
}
