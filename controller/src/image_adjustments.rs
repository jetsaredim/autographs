use image::{DynamicImage, Rgba, RgbaImage, imageops};
use imageproc::geometric_transformations::{
    Border, Interpolation, Projection, rotate_about_center, warp,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::derivatives::{DerivativeVariant, GeneratedDerivative};

pub const IMAGE_ADJUSTMENT_TRANSFORM_VERSION: &str = "image-adjustment-transform-01";
pub const AUTO_CORRECTION_UNAVAILABLE_MESSAGE: &str =
    "Auto correction could not find reliable edges. Adjust the corners manually.";
pub const AUTO_CORRECTION_NOT_NEEDED_MESSAGE: &str =
    "No meaningful perspective correction was detected.";

const AUTO_CORRECTION_MAX_DIMENSION: u32 = 640;
const AUTO_CORRECTION_MIN_COLOR_DISTANCE: u32 = 32;
const AUTO_CORRECTION_MIN_COMPONENT_RATIO: f32 = 0.01;
const AUTO_CORRECTION_MIN_QUAD_AREA_RATIO: f32 = 0.15;
const AUTO_CORRECTION_MAX_QUAD_AREA_RATIO: f32 = 0.985;
const AUTO_CORRECTION_IDENTITY_TOLERANCE: f32 = 0.025;
const AUTO_CORRECTION_MIN_STABILITY_THRESHOLD: u32 = 8;
const AUTO_CORRECTION_MAX_STABLE_COMPONENT_GROWTH: f32 = 1.25;
const AUTO_CORRECTION_MAX_STABLE_AREA_GROWTH: f32 = 0.05;

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
#[serde(rename_all = "camelCase")]
pub enum ImageAdjustmentProposalStatus {
    Confident,
    NotNeeded,
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

    pub fn into_canonical(mut self) -> Option<Self> {
        if self
            .perspective
            .as_ref()
            .is_some_and(ImagePerspective::is_full_frame)
        {
            self.perspective = None;
        }
        (!self.is_identity()).then_some(self)
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
        perspective_projection(self.normalized_source_points(), unit_target_points())?;
        Ok(())
    }

    fn normalized_source_points(&self) -> [(f32, f32); 4] {
        self.corners.map(|corner| (corner.x, corner.y))
    }

    fn is_full_frame(&self) -> bool {
        self.corners
            == [
                ImagePoint { x: 0.0, y: 0.0 },
                ImagePoint { x: 1.0, y: 0.0 },
                ImagePoint { x: 1.0, y: 1.0 },
                ImagePoint { x: 0.0, y: 1.0 },
            ]
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
    let image = decoded
        .thumbnail(AUTO_CORRECTION_MAX_DIMENSION, AUTO_CORRECTION_MAX_DIMENSION)
        .to_rgb8();
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Ok(unavailable_proposal());
    }

    let background = border_median_rgb(&image);
    let threshold = adaptive_color_distance_threshold(&image, background);
    let Some(component) = largest_contrast_component(&image, background, threshold) else {
        return Ok(unavailable_proposal());
    };
    let corners = component.corners;
    let normalized_corners = corners.map(|(x, y)| normalized_point(x, y, width, height));
    let perspective = ImagePerspective {
        corners: normalized_corners,
    };
    if perspective.validate().is_err() {
        return Ok(unavailable_proposal());
    }
    let area_ratio = quadrilateral_area_ratio(normalized_corners);
    if !(AUTO_CORRECTION_MIN_QUAD_AREA_RATIO..=AUTO_CORRECTION_MAX_QUAD_AREA_RATIO)
        .contains(&area_ratio)
    {
        return Ok(unavailable_proposal());
    }
    if !component_boundary_is_stable(&image, background, threshold, &component, area_ratio) {
        return Ok(unavailable_proposal());
    }
    if corners_are_near_full_frame(normalized_corners) {
        return Ok(not_needed_proposal());
    }

    Ok(ImageAdjustmentProposal {
        status: ImageAdjustmentProposalStatus::Confident,
        corners: normalized_corners.to_vec(),
        message: None,
    })
}

fn border_median_rgb(image: &image::RgbImage) -> [u8; 3] {
    let (width, height) = image.dimensions();
    let mut channels = [Vec::new(), Vec::new(), Vec::new()];
    for x in 0..width {
        for y in [0, height.saturating_sub(1)] {
            let pixel = image.get_pixel(x, y);
            for channel in 0..3 {
                channels[channel].push(pixel[channel]);
            }
        }
    }
    for y in 1..height.saturating_sub(1) {
        for x in [0, width.saturating_sub(1)] {
            let pixel = image.get_pixel(x, y);
            for channel in 0..3 {
                channels[channel].push(pixel[channel]);
            }
        }
    }
    for values in &mut channels {
        values.sort_unstable();
    }
    channels.map(|values| values[values.len() / 2])
}

fn adaptive_color_distance_threshold(image: &image::RgbImage, background: [u8; 3]) -> u32 {
    let (width, height) = image.dimensions();
    let mut distances = Vec::with_capacity((width + height).saturating_mul(2) as usize);
    for x in 0..width {
        distances.push(color_distance(image.get_pixel(x, 0).0, background));
        distances.push(color_distance(
            image.get_pixel(x, height.saturating_sub(1)).0,
            background,
        ));
    }
    for y in 1..height.saturating_sub(1) {
        distances.push(color_distance(image.get_pixel(0, y).0, background));
        distances.push(color_distance(
            image.get_pixel(width.saturating_sub(1), y).0,
            background,
        ));
    }
    distances.sort_unstable();
    let background_noise = distances[distances.len().saturating_mul(9) / 10];
    AUTO_CORRECTION_MIN_COLOR_DISTANCE.max(background_noise.saturating_mul(2).saturating_add(8))
}

fn color_distance(color: [u8; 3], background: [u8; 3]) -> u32 {
    let squared = color
        .into_iter()
        .zip(background)
        .map(|(value, reference)| {
            let delta = i32::from(value) - i32::from(reference);
            (delta * delta) as u32
        })
        .sum::<u32>();
    (squared as f32).sqrt().round() as u32
}

fn largest_contrast_component(
    image: &image::RgbImage,
    background: [u8; 3],
    threshold: u32,
) -> Option<ContrastComponent> {
    let (width, height) = image.dimensions();
    let total = width as usize * height as usize;
    let foreground = image
        .pixels()
        .map(|pixel| color_distance(pixel.0, background) >= threshold)
        .collect::<Vec<_>>();
    let mut visited = vec![false; total];
    let mut largest = None;

    for index in 0..total {
        if visited[index] || !foreground[index] {
            continue;
        }
        let component = trace_contrast_component(index, width, height, &foreground, &mut visited);
        if largest
            .as_ref()
            .is_none_or(|current: &ContrastComponent| component.count > current.count)
        {
            largest = Some(component);
        }
    }

    largest.and_then(|component| {
        let ratio = component.count as f32 / total.max(1) as f32;
        (ratio >= AUTO_CORRECTION_MIN_COMPONENT_RATIO).then_some(component)
    })
}

struct ContrastComponent {
    anchor: usize,
    count: usize,
    corners: [(u32, u32); 4],
}

fn trace_contrast_component(
    start: usize,
    width: u32,
    height: u32,
    foreground: &[bool],
    visited: &mut [bool],
) -> ContrastComponent {
    let mut queue = VecDeque::from([start]);
    visited[start] = true;
    let mut count = 0usize;
    let mut corners = [(0, 0); 4];
    let mut scores = [i64::MAX, i64::MIN, i64::MIN, i64::MAX];

    while let Some(index) = queue.pop_front() {
        count += 1;
        let x = index as u32 % width;
        let y = index as u32 / width;
        let sum = i64::from(x) + i64::from(y);
        let difference = i64::from(x) - i64::from(y);
        for (corner, score, candidate, prefer_lower) in [
            (0, sum, sum, true),
            (1, difference, difference, false),
            (2, sum, sum, false),
            (3, difference, difference, true),
        ] {
            if (prefer_lower && candidate < scores[corner])
                || (!prefer_lower && candidate > scores[corner])
            {
                scores[corner] = score;
                corners[corner] = (x, y);
            }
        }

        let min_x = x.saturating_sub(1);
        let max_x = (x + 1).min(width.saturating_sub(1));
        let min_y = y.saturating_sub(1);
        let max_y = (y + 1).min(height.saturating_sub(1));
        for neighbor_y in min_y..=max_y {
            for neighbor_x in min_x..=max_x {
                let neighbor = (neighbor_y * width + neighbor_x) as usize;
                if !visited[neighbor] && foreground[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
    }

    ContrastComponent {
        anchor: start,
        count,
        corners,
    }
}

fn component_boundary_is_stable(
    image: &image::RgbImage,
    background: [u8; 3],
    threshold: u32,
    component: &ContrastComponent,
    area_ratio: f32,
) -> bool {
    let (width, height) = image.dimensions();
    let mut previous_threshold = threshold;
    for weak_threshold in [
        threshold.saturating_mul(3) / 4,
        threshold / 2,
        threshold / 3,
        AUTO_CORRECTION_MIN_COLOR_DISTANCE,
        AUTO_CORRECTION_MIN_COLOR_DISTANCE / 2,
        AUTO_CORRECTION_MIN_STABILITY_THRESHOLD,
    ]
    .map(|value| value.max(AUTO_CORRECTION_MIN_STABILITY_THRESHOLD))
    {
        if weak_threshold >= previous_threshold {
            continue;
        }
        previous_threshold = weak_threshold;
        let foreground = image
            .pixels()
            .map(|pixel| color_distance(pixel.0, background) >= weak_threshold)
            .collect::<Vec<_>>();
        if !foreground[component.anchor] {
            return false;
        }
        let mut visited = vec![false; foreground.len()];
        let weak_component =
            trace_contrast_component(component.anchor, width, height, &foreground, &mut visited);
        let weak_area_ratio = quadrilateral_area_ratio(
            weak_component
                .corners
                .map(|(x, y)| normalized_point(x, y, width, height)),
        );
        let population_growth = weak_component.count as f32 / component.count.max(1) as f32;
        if population_growth > AUTO_CORRECTION_MAX_STABLE_COMPONENT_GROWTH
            && weak_area_ratio - area_ratio > AUTO_CORRECTION_MAX_STABLE_AREA_GROWTH
        {
            return false;
        }
    }
    true
}

fn quadrilateral_area_ratio(corners: [ImagePoint; 4]) -> f32 {
    corners
        .iter()
        .zip(corners.iter().cycle().skip(1))
        .take(4)
        .map(|(left, right)| left.x * right.y - right.x * left.y)
        .sum::<f32>()
        .abs()
        / 2.0
}

fn corners_are_near_full_frame(corners: [ImagePoint; 4]) -> bool {
    corners
        .into_iter()
        .zip([
            ImagePoint { x: 0.0, y: 0.0 },
            ImagePoint { x: 1.0, y: 0.0 },
            ImagePoint { x: 1.0, y: 1.0 },
            ImagePoint { x: 0.0, y: 1.0 },
        ])
        .all(|(corner, full_frame)| {
            (corner.x - full_frame.x).abs() <= AUTO_CORRECTION_IDENTITY_TOLERANCE
                && (corner.y - full_frame.y).abs() <= AUTO_CORRECTION_IDENTITY_TOLERANCE
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
    let projection = perspective_projection(source, target)?;
    Ok(warp(
        image,
        projection,
        Interpolation::Bilinear,
        Border::Constant(Rgba([0, 0, 0, 0])),
    ))
}

fn perspective_projection(
    source: [(f32, f32); 4],
    target: [(f32, f32); 4],
) -> Result<Projection, String> {
    Projection::from_control_points(source, target)
        .ok_or_else(|| "perspective corners do not form a valid projection".to_owned())
}

fn unit_target_points() -> [(f32, f32); 4] {
    [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
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

fn not_needed_proposal() -> ImageAdjustmentProposal {
    ImageAdjustmentProposal {
        status: ImageAdjustmentProposalStatus::NotNeeded,
        corners: Vec::new(),
        message: Some(AUTO_CORRECTION_NOT_NEEDED_MESSAGE.to_owned()),
    }
}
