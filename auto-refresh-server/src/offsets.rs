use once_cell::sync::Lazy;
use skyline::hooks::{getRegionAddress, Region};

static OFFSETS: Lazy<Offsets> = Lazy::new(|| Offsets::new());

static FILESYSTEM_INFO_ADRP_SEARCH_CODE: &[u8] = &[
    0xf3, 0x03, 0x00, 0xaa, 0x1f, 0x01, 0x09, 0x6b, 0xe0, 0x04, 0x00, 0x54,
];

static RES_SERVICE_ADRP_SEARCH_CODE: &[u8] = &[
    0x04, 0x01, 0x49, 0xfa, 0x21, 0x05, 0x00, 0x54, 0x5f, 0x00, 0x00, 0xf9, 0x7f, 0x00, 0x00, 0xf9,
];

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[allow(clippy::inconsistent_digit_grouping)]
fn offset_from_adrp(adrp_offset: usize) -> usize {
    unsafe {
        let adrp = *(offset_to_addr(adrp_offset) as *const u32);
        let immhi = (adrp & 0b0000_0000_1111_1111_1111_1111_1110_0000) >> 3;
        let immlo = (adrp & 0b0110_0000_0000_0000_0000_0000_0000_0000) >> 29;
        let imm = ((immhi | immlo) << 12) as i32 as usize;
        let base = adrp_offset & 0xFFFF_FFFF_FFFF_F000;
        base + imm
    }
}

#[allow(clippy::inconsistent_digit_grouping)]
fn offset_from_ldr(ldr_offset: usize) -> usize {
    unsafe {
        let ldr = *(offset_to_addr(ldr_offset) as *const u32);
        let size = (ldr & 0b1100_0000_0000_0000_0000_0000_0000_0000) >> 30;
        let imm = (ldr & 0b0000_0000_0011_1111_1111_1100_0000_0000) >> 10;
        (imm as usize) << size
    }
}

pub fn offset_to_addr(offset: usize) -> *const () {
    unsafe { (getRegionAddress(Region::Text) as *const u8).add(offset) as _ }
}

fn get_text() -> &'static [u8] {
    unsafe {
        let ptr = getRegionAddress(Region::Text) as *const u8;
        let size = (getRegionAddress(Region::Rodata) as usize) - (ptr as usize);
        std::slice::from_raw_parts(ptr, size)
    }
}

struct Offsets {
    pub filesystem_info: usize,
    pub res_service: usize,
}

impl Offsets {
    pub fn new() -> Self {
        let text = get_text();

        let filesystem_info = {
            let adrp = find_subsequence(text, FILESYSTEM_INFO_ADRP_SEARCH_CODE)
                .expect("Unable to find subsequence")
                + 12;
            let adrp_offset = offset_from_adrp(adrp);
            let ldr_offset = offset_from_ldr(adrp + 4);
            adrp_offset + ldr_offset
        };
        let res_service = {
            let adrp = find_subsequence(text, RES_SERVICE_ADRP_SEARCH_CODE)
                .expect("Unable to find subsequence")
                + 16;
            let adrp_offset = offset_from_adrp(adrp);
            let ldr_offset = offset_from_ldr(adrp + 4);
            adrp_offset + ldr_offset
        };

        Self {
            filesystem_info,
            res_service,
        }
    }
}

pub fn filesystem_info() -> usize {
    OFFSETS.filesystem_info
}

pub fn res_service() -> usize {
    OFFSETS.res_service
}
