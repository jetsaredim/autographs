use image::{ExtendedColorType, ImageReader, codecs::webp::WebPEncoder, imageops::FilterType};
use std::io::Cursor;

pub const MAX_SOURCE_BYTES: usize = 20 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DerivativeVariant {
    Thumbnail,
    Detail,
}

impl DerivativeVariant {
    pub const fn path_segment(self) -> &'static str {
        match self {
            Self::Thumbnail => "thumbnail",
            Self::Detail => "detail",
        }
    }

    const fn max_dimensions(self) -> (u32, u32) {
        match self {
            Self::Thumbnail => (480, 640),
            Self::Detail => (1200, 1600),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedDerivative {
    pub variant: DerivativeVariant,
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub content_type: &'static str,
}

pub fn generate_derivative(
    source: &[u8],
    variant: DerivativeVariant,
) -> Result<GeneratedDerivative, String> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err("private original exceeds the derivative source limit".to_owned());
    }
    let decoded = ImageReader::new(Cursor::new(source))
        .with_guessed_format()
        .map_err(|error| format!("detect private original format: {error}"))?
        .decode()
        .map_err(|error| format!("decode private original: {error}"))?;
    let (max_width, max_height) = variant.max_dimensions();
    let resized = decoded.resize(max_width, max_height, FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let mut bytes = Vec::new();
    WebPEncoder::new_lossless(&mut bytes)
        .encode(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|error| format!("encode sanitized WebP derivative: {error}"))?;
    Ok(GeneratedDerivative {
        variant,
        bytes,
        width: rgba.width(),
        height: rgba.height(),
        content_type: "image/webp",
    })
}
