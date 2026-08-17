use image::{DynamicImage, Rgba, RgbaImage, imageops};
use imageproc::geometric_transformations::{
    Border, Interpolation, Projection, rotate_about_center, warp,
};
use serde::{Deserialize, Serialize};

use crate::derivatives::{DerivativeVariant, GeneratedDerivative};

pub const IMAGE_ADJUSTMENT_TRANSFORM_VERSION: &str = "image-adjustment-transform-01";
pub const AUTO_CORRECTION_UNAVAILABLE_MESSAGE: &str =
    "Auto correction could not find reliable edges. Adjust the corners manually.";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAdjustment {
    pub rotation_degrees: f32,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub crop: Option<ImageCrop>,
    pub perspective: Option<ImagePerspective>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageCrop {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePerspective {
    pub corners: [ImagePoint; 4],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAdjustmentProposal {
    pub status: ImageAdjustmentProposalStatus,
    pub corners: Vec<ImagePoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageAdjustmentProposalStatus {
    Confident,
    Unavailable,
}

impl ImageAdjustment {
    pub const fn identity() -> Self {
        Self {
            rotation_degrees: 0.0,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            crop: None,
            perspective: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_f32(
            self.rotation_degrees,
            -15.0,
            15.0,
            "rotationDegrees must be between -15.0 and 15.0",
        )?;
        validate_f32(self.zoom, 1.0, 3.0, "zoom must be between 1.0 and 3.0")?;
        validate_f32(self.pan_x, -1.0, 1.0, "panX must be between -1.0 and 1.0")?;
        validate_f32(self.pan_y, -1.0, 1.0, "panY must be between -1.0 and 1.0")?;
        if let Some(crop) = self.crop {
            crop.validate()?;
        }
        if let Some(perspective) = &self.perspective {
            perspective.validate()?;
        }
        Ok(())
    }

    pub fn canonical_cache_key(&self) -> Result<String, String> {
        self.validate()?;
        if self.is_identity() {
            return Ok("adjustment:none".to_owned());
        }
        Ok(format!(
            "adjustment:{}:{}",
            IMAGE_ADJUSTMENT_TRANSFORM_VERSION,
            self.to_json()?
        ))
    }

    pub fn is_identity(&self) -> bool {
        self.rotation_degrees == 0.0
            && self.zoom == 1.0
            && self.pan_x == 0.0
            && self.pan_y == 0.0
            && self.crop.is_none()
            && self.perspective.is_none()
    }

    pub fn from_json(value: &str) -> Result<Self, String> {
        let adjustment = serde_json::from_str::<Self>(value)
            .map_err(|error| format!("parse image adjustment: {error}"))?;
        adjustment.validate()?;
        Ok(adjustment)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| format!("serialize image adjustment: {error}"))
    }
}

impl ImageCrop {
    fn validate(self) -> Result<(), String> {
        validate_f32(self.left, 0.0, 1.0, "crop left must be normalized")?;
        validate_f32(self.top, 0.0, 1.0, "crop top must be normalized")?;
        validate_f32(self.right, 0.0, 1.0, "crop right must be normalized")?;
        validate_f32(self.bottom, 0.0, 1.0, "crop bottom must be normalized")?;
        if self.left >= self.right || self.top >= self.bottom {
            return Err("crop bounds must have positive width and height".to_owned());
        }
        Ok(())
    }
}

impl ImagePerspective {
    fn validate(&self) -> Result<(), String> {
        for corner in self.corners {
            validate_f32(
                corner.x,
                0.0,
                1.0,
                "perspective corner x must be normalized",
            )?;
            validate_f32(
                corner.y,
                0.0,
                1.0,
                "perspective corner y must be normalized",
            )?;
        }
        Ok(())
    }
}

pub fn apply_image_adjustment(
    decoded: DynamicImage,
    adjustment: &ImageAdjustment,
) -> Result<DynamicImage, String> {
    adjustment.validate()?;
    if adjustment.is_identity() {
        return Ok(decoded);
    }
    let mut rgba = decoded.to_rgba8();
    if let Some(perspective) = &adjustment.perspective {
        rgba = apply_perspective(&rgba, perspective)?;
    }
    if adjustment.rotation_degrees != 0.0 {
        rgba = rotate_about_center(
            &rgba,
            adjustment.rotation_degrees.to_radians(),
            Interpolation::Bilinear,
            Border::Constant(Rgba([0, 0, 0, 0])),
        );
    }
    if let Some(crop) = adjustment.crop {
        rgba = crop_normalized(&rgba, crop);
    }
    if adjustment.zoom > 1.0 || adjustment.pan_x != 0.0 || adjustment.pan_y != 0.0 {
        rgba = zoom_and_pan(&rgba, adjustment.zoom, adjustment.pan_x, adjustment.pan_y);
    }
    Ok(DynamicImage::ImageRgba8(rgba))
}

pub fn propose_image_adjustment(source: &[u8]) -> Result<ImageAdjustmentProposal, String> {
    let decoded = image::load_from_memory(source)
        .map_err(|error| format!("decode private original for auto correction: {error}"))?;
    let gray = decoded.to_luma8();
    let (width, height) = gray.dimensions();
    if width == 0 || height == 0 {
        return Ok(unavailable_proposal());
    }

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut bright_count = 0usize;
    for (x, y, pixel) in gray.enumerate_pixels() {
        if pixel[0] >= 210 {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
            bright_count += 1;
        }
    }

    let total = (width as usize).saturating_mul(height as usize);
    let bright_ratio = bright_count as f32 / total.max(1) as f32;
    let box_width = max_x.saturating_sub(min_x).saturating_add(1);
    let box_height = max_y.saturating_sub(min_y).saturating_add(1);
    if bright_count == 0
        || !(0.18..=0.85).contains(&bright_ratio)
        || box_width < width / 4
        || box_height < height / 4
    {
        return Ok(unavailable_proposal());
    }

    Ok(ImageAdjustmentProposal {
        status: ImageAdjustmentProposalStatus::Confident,
        corners: vec![
            normalized_point(min_x, min_y, width, height),
            normalized_point(max_x, min_y, width, height),
            normalized_point(max_x, max_y, width, height),
            normalized_point(min_x, max_y, width, height),
        ],
        message: None,
    })
}

pub fn generate_adjusted_derivative(
    source: &[u8],
    variant: DerivativeVariant,
    adjustment: Option<&ImageAdjustment>,
) -> Result<GeneratedDerivative, String> {
    crate::derivatives::generate_derivative_from_adjustment(source, variant, adjustment)
}

fn validate_f32(value: f32, min: f32, max: f32, message: &str) -> Result<(), String> {
    if !value.is_finite() || value < min || value > max {
        return Err(message.to_owned());
    }
    Ok(())
}

fn apply_perspective(
    image: &RgbaImage,
    perspective: &ImagePerspective,
) -> Result<RgbaImage, String> {
    let (width, height) = image.dimensions();
    let source = perspective.corners.map(|corner| {
        (
            corner.x * width.saturating_sub(1) as f32,
            corner.y * height.saturating_sub(1) as f32,
        )
    });
    let target = [
        (0.0, 0.0),
        (width.saturating_sub(1) as f32, 0.0),
        (
            width.saturating_sub(1) as f32,
            height.saturating_sub(1) as f32,
        ),
        (0.0, height.saturating_sub(1) as f32),
    ];
    let projection = Projection::from_control_points(target, source)
        .ok_or_else(|| "perspective corners do not form a valid projection".to_owned())?;
    Ok(warp(
        image,
        projection,
        Interpolation::Bilinear,
        Border::Constant(Rgba([0, 0, 0, 0])),
    ))
}

fn crop_normalized(image: &RgbaImage, crop: ImageCrop) -> RgbaImage {
    let (width, height) = image.dimensions();
    let left = (crop.left * width as f32).floor() as u32;
    let top = (crop.top * height as f32).floor() as u32;
    let right = ((crop.right * width as f32).ceil() as u32).clamp(left + 1, width);
    let bottom = ((crop.bottom * height as f32).ceil() as u32).clamp(top + 1, height);
    imageops::crop_imm(image, left, top, right - left, bottom - top).to_image()
}

fn zoom_and_pan(image: &RgbaImage, zoom: f32, pan_x: f32, pan_y: f32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let crop_width = ((width as f32 / zoom).round() as u32).clamp(1, width);
    let crop_height = ((height as f32 / zoom).round() as u32).clamp(1, height);
    let max_x = width - crop_width;
    let max_y = height - crop_height;
    let center_x = max_x as f32 / 2.0;
    let center_y = max_y as f32 / 2.0;
    let x = (center_x + pan_x * center_x)
        .round()
        .clamp(0.0, max_x as f32) as u32;
    let y = (center_y + pan_y * center_y)
        .round()
        .clamp(0.0, max_y as f32) as u32;
    imageops::crop_imm(image, x, y, crop_width, crop_height).to_image()
}

fn normalized_point(x: u32, y: u32, width: u32, height: u32) -> ImagePoint {
    ImagePoint {
        x: x as f32 / width.saturating_sub(1).max(1) as f32,
        y: y as f32 / height.saturating_sub(1).max(1) as f32,
    }
}

fn unavailable_proposal() -> ImageAdjustmentProposal {
    ImageAdjustmentProposal {
        status: ImageAdjustmentProposalStatus::Unavailable,
        corners: Vec::new(),
        message: Some(AUTO_CORRECTION_UNAVAILABLE_MESSAGE.to_owned()),
    }
}
