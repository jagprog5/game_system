use std::{os::unix::ffi::OsStrExt, path::Path};

use crate::core::color::Color;

/// contains some encoding of the resource. used as lru key.
///
/// contains variants, identified by the last byte.
///
/// for texture from file
///
/// "/path/to/img" + 0x00
///
/// for rendered text:
///
/// u16(16pt) + RGBA + "some text" + 0x01
///
/// for rendered wrapping text:
///
/// u16(16pt) + u32(123pix) + RGBA + "some text" + 0x02
///
/// for user defined key:
///
/// <user defined bytes> + 0x03
///
/// special value for cache_marker_key:
///
/// 0xFF
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureKey {
    data: Vec<u8>,
}

impl TextureKey {
    pub fn cache_marker_key() -> Self {
        Self { data: vec![0xff] }
    }

    pub fn from_user_defined_key(mut data: Vec<u8>) -> Self {
        data.push(0x03);

        Self { data }
    }

    pub fn from_path(texture_path: &Path) -> Self {
        let mut data: Vec<u8> = Default::default();
        let data_len = 1 + texture_path.as_os_str().as_bytes().len();
        data.reserve_exact(data_len);
        unsafe {
            // safety, debug assert below
            data.set_len(data_len);
        }
        let mut index = 0;
        texture_path
            .as_os_str()
            .as_bytes()
            .iter()
            .for_each(|&byte| {
                data[index] = byte;
                index += 1;
            });
        data[index] = b'\x00';
        debug_assert_eq!(data.len(), data_len);
        Self { data }
    }

    pub fn from_rendered_text(text: &str, color: Color, point_size: u16) -> Self {
        let text = text.as_bytes();
        let point_size_bytes = point_size.to_le_bytes();
        let data_len = 1 + size_of::<u16>() + 4 + text.len();
        let mut data: Vec<u8> = Default::default();
        data.reserve_exact(data_len);
        unsafe {
            // safety, debug assert below
            data.set_len(data_len);
        }
        let mut index = 0;
        data[index] = point_size_bytes[0];
        index += 1;
        data[index] = point_size_bytes[1];
        index += 1;
        data[index] = color.r;
        index += 1;
        data[index] = color.g;
        index += 1;
        data[index] = color.b;
        index += 1;
        data[index] = color.a;
        index += 1;
        text.iter().for_each(|&byte| {
            data[index] = byte;
            index += 1;
        });
        data[index] = b'\x01';
        debug_assert_eq!(data.len(), data_len);
        Self { data }
    }

    pub fn from_rendered_wrapped_text(
        text: &str,
        color: Color,
        point_size: u16,
        wrap_width: u32,
    ) -> Self {
        let text = text.as_bytes();
        let point_size_bytes = point_size.to_le_bytes();
        let wrap_width_bytes = wrap_width.to_le_bytes();
        let data_len = 1 + size_of::<u16>() + size_of::<u32>() + 4 + text.len();
        let mut data: Vec<u8> = Default::default();
        data.reserve_exact(data_len);
        unsafe {
            // safety, debug assert below
            data.set_len(data_len);
        }
        let mut index = 0;
        data[index] = point_size_bytes[0];
        index += 1;
        data[index] = point_size_bytes[1];
        index += 1;
        data[index] = wrap_width_bytes[0];
        index += 1;
        data[index] = wrap_width_bytes[1];
        index += 1;
        data[index] = wrap_width_bytes[2];
        index += 1;
        data[index] = wrap_width_bytes[3];
        index += 1;
        data[index] = color.r;
        index += 1;
        data[index] = color.g;
        index += 1;
        data[index] = color.b;
        index += 1;
        data[index] = color.a;
        index += 1;
        text.iter().for_each(|&byte| {
            data[index] = byte;
            index += 1;
        });
        data[index] = b'\x02';
        debug_assert_eq!(data.len(), data_len);
        Self { data }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::{PathBuf, MAIN_SEPARATOR},
        u32,
    };

    use super::*;

    #[test]
    fn test_path() {
        let mut path = PathBuf::default();
        path.push("tester");
        path.push("abc");
        let s = TextureKey::from_path(&path);

        let mut rhs: Vec<u8> = Default::default();
        rhs.extend_from_slice(b"tester");
        rhs.extend_from_slice(&[MAIN_SEPARATOR as u8]);
        rhs.extend_from_slice(b"abc");
        rhs.push(b'\x00');
        assert_eq!(s.data, rhs);
    }

    #[test]
    fn test_text() {
        let s = TextureKey::from_rendered_text(
            "text",
            Color {
                r: 0,
                g: 1,
                b: 2,
                a: 3,
            },
            16,
        );
        let mut rhs: Vec<u8> = Default::default();
        rhs.extend_from_slice(b"\x10\x00");
        rhs.extend_from_slice(b"\x00\x01\x02\x03");
        rhs.extend_from_slice(b"text");
        rhs.push(b'\x01');
        assert_eq!(s.data, rhs);
    }

    #[test]
    fn test_text_wrapped() {
        let s = TextureKey::from_rendered_wrapped_text(
            "text",
            Color {
                r: 0,
                g: 1,
                b: 2,
                a: 3,
            },
            16,
            u32::MAX - 1,
        );
        let mut rhs: Vec<u8> = Default::default();
        rhs.extend_from_slice(b"\x10\x00");
        rhs.extend_from_slice(b"\xFE\xFF\xFF\xFF");
        rhs.extend_from_slice(b"\x00\x01\x02\x03");
        rhs.extend_from_slice(b"text");
        rhs.push(b'\x02');
        assert_eq!(s.data, rhs);
    }
}
