//! Which game build is running. The bridge only ever touches the one build PR #59 analysed.
//!
//! Method (documented choice): the whole-file SHA-256 and the `.text` file hashes in PR #59 describe bytes on disk,
//! not the relocated image in memory, so neither can be recomputed from the live process. Instead:
//! 1. the live PE header's TimeDateStamp and SizeOfImage must equal the recorded build (`0x6ae1b2c7`, `0x58f8000`);
//! 2. every hooked function's recorded entry bytes (32 bytes from `realm-hooks.json`) must equal the live bytes.
//!    Those prologues hold no absolute addresses (pushes, `sub rsp, imm`, `lea rbp,[rsp+imm]`, `cmp r8,[rcx+imm]`,
//!    `mov [rbp+imm], imm`), so ASLR relocation can't change them, and a different build or another mod's patch can.
//!    (The entry-byte check lives with each hook in `detour::plan`.)
//! Anything else reports `none` before a single byte is written.

pub const TIMESTAMP: u32 = 0x6ae1_b2c7;
pub const SIZE_OF_IMAGE: u32 = 0x058f_8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub timestamp: u32,
    pub size_of_image: u32,
}

/// The TimeDateStamp and SizeOfImage of a mapped PE image's first page.
pub fn parse(page: &[u8]) -> Option<Header> {
    let word = |at: usize| page.get(at..at + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    if page.get(..2)? != b"MZ" {
        return None;
    }
    let nt = word(0x3C)? as usize;
    if page.get(nt..nt + 4)? != b"PE\0\0" {
        return None;
    }
    // COFF header: TimeDateStamp at +8. Optional header starts at +0x18; PE32+ magic 0x20B; SizeOfImage at +0x38.
    if page.get(nt + 0x18..nt + 0x1A)? != [0x0B, 0x02] {
        return None;
    }
    Some(Header { timestamp: word(nt + 8)?, size_of_image: word(nt + 0x18 + 0x38)? })
}

pub fn supported(h: Option<Header>) -> Result<(), String> {
    match h {
        Some(Header { timestamp: TIMESTAMP, size_of_image: SIZE_OF_IMAGE }) => Ok(()),
        Some(h) => Err(format!(
            "unsupported game build (timestamp {:#x}, image size {:#x}; this bridge knows {TIMESTAMP:#x} / {SIZE_OF_IMAGE:#x})",
            h.timestamp, h.size_of_image
        )),
        None => Err("the game's PE header could not be read".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn image(timestamp: u32, size: u32) -> Vec<u8> {
        let mut p = vec![0u8; 0x200];
        p[..2].copy_from_slice(b"MZ");
        p[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        p[0x80..0x84].copy_from_slice(b"PE\0\0");
        p[0x88..0x8C].copy_from_slice(&timestamp.to_le_bytes());
        p[0x98..0x9A].copy_from_slice(&[0x0B, 0x02]);
        p[0xD0..0xD4].copy_from_slice(&size.to_le_bytes());
        p
    }
    #[test]
    fn only_the_analysed_build_is_supported() {
        assert!(supported(parse(&image(TIMESTAMP, SIZE_OF_IMAGE))).is_ok());
        assert!(supported(parse(&image(TIMESTAMP + 1, SIZE_OF_IMAGE))).is_err());
        assert!(supported(parse(&image(TIMESTAMP, SIZE_OF_IMAGE + 0x1000))).is_err());
        assert!(supported(parse(b"not a pe")).is_err());
        let mut pe32 = image(TIMESTAMP, SIZE_OF_IMAGE);
        pe32[0x98] = 0x0B;
        pe32[0x99] = 0x01;
        assert!(supported(parse(&pe32)).is_err(), "32-bit images are refused");
    }
}
