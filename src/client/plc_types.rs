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
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymHandle {
    pub handle: u32,
    pub plc_type: PlcTypes,
}

impl SymHandle {
    pub fn new(handle: u32, plc_type: PlcTypes) -> Self {
        SymHandle { handle, plc_type }
    }
}

#[derive(Debug, Clone)]
pub struct Var<'a> {
    pub name: &'a str,
    pub plc_type: PlcTypes,
}

impl<'a> Var<'a> {
    pub fn new(name: &'a str, plc_type: PlcTypes) -> Self {
        Var { name, plc_type }
    }
}
