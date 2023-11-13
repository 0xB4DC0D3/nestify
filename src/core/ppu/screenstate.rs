pub struct ScreenState {
    pub bg_next_tile_id: u8,
    pub bg_next_tile_attribute: u8,
    pub bg_next_tile_lsb: u8,
    pub bg_next_tile_msb: u8,
    pub bg_shift_pattern_lo: u16,
    pub bg_shift_pattern_hi: u16,
    pub bg_shift_attribute_lo: u16,
    pub bg_shift_attribute_hi: u16,
    pub sprite_shift_pattern_lo: [u8; 8],
    pub sprite_shift_pattern_hi: [u8; 8],
    pub sprite_count: u8,
    pub sprite_zero_occured: bool,
    pub sprite_zero_rendering: bool,
}

impl ScreenState {
    pub const fn new() -> Self {
        Self {
            bg_next_tile_id: 0,
            bg_next_tile_attribute: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,
            bg_shift_pattern_lo: 0,
            bg_shift_pattern_hi: 0,
            bg_shift_attribute_lo: 0,
            bg_shift_attribute_hi: 0,
            sprite_shift_pattern_lo: [0; 8],
            sprite_shift_pattern_hi: [0; 8],
            sprite_count: 0,
            sprite_zero_occured: false,
            sprite_zero_rendering: false,
        }
    }
}
