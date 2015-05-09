
pub trait Fontmap {
    // The hex value provided is assumed to be between 0x0 and 0xF
    fn get_char(hex: u8) -> u8;
}
