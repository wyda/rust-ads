use crate::ads_services::interfaces::*;

///Index offset allways 0
pub const GET_SYMHANDLE_BY_NAME: AdsServiceInterface = AdsServiceInterface {
    index_group: 0x0000F003,
    index_offset_start: 0x00000000,
    index_offset_end: 0x00000000,
};

///Index offset allways 0
pub const READ_SYMVAL_BY_NAME: AdsServiceInterface = AdsServiceInterface {
    index_group: 0x0000F004,
    index_offset_start: 0x00000000,
    index_offset_end: 0x00000000,
};

///Index offset is symhandle
pub const READ_WRITE_SYMVAL_BY_HANDLE: AdsServiceInterface = AdsServiceInterface {
    index_group: 0x0000F005,
    index_offset_start: 0x00000000,
    index_offset_end: 0xFFFFFFFF,
};
