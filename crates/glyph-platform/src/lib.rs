//! lib.rs - Cross-platform filesystem and OS abstraction for Glyph.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::{fs, io, string::String};

pub struct PlatformFs;

impl PlatformFs {
    #[cfg(feature = "std")]
    pub fn read_to_string(path: &str) -> io::Result<String> {
        fs::read_to_string(path)
    }

    #[cfg(feature = "std")]
    pub fn write(path: &str, content: &str) -> io::Result<()> {
        fs::write(path, content)
    }

    #[cfg(feature = "std")]
    pub fn exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    #[cfg(not(feature = "std"))]
    pub fn read_to_string(_path: &str) -> Result<String, ()> {
        Err(())
    }

    #[cfg(not(feature = "std"))]
    pub fn write(_path: &str, _content: &str) -> Result<(), ()> {
        Err(())
    }

    #[cfg(not(feature = "std"))]
    pub fn exists(_path: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_fs() {
        let test_path = "target/glyph_platform_test.txt";
        let _ = PlatformFs::write(test_path, "Glyph Platform Test");
        if PlatformFs::exists(test_path) {
            let content = PlatformFs::read_to_string(test_path).unwrap();
            assert_eq!(content, "Glyph Platform Test");
            let _ = std::fs::remove_file(test_path);
        }
    }
}
