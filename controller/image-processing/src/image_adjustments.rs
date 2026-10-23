use image::{DynamicImage, GrayImage, Rgba, RgbaImage, imageops};
use imageproc::geometric_transformations::{
    Border, Interpolation, Projection, rotate_about_center, warp,
};
use imageproc::{
    edges::canny,
    hough::{LineDetectionOptions, PolarLine, detect_lines, intersection_points},
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::VecDeque};

use crate::derivatives::{DerivativeVariant, GeneratedDerivative};

pub const IMAGE_ADJUSTMENT_TRANSFORM_VERSION: &str = "image-adjustment-transform-01";
pub const AUTO_CORRECTION_UNAVAILABLE_MESSAGE: &str =
    "Auto correction could not find reliable edges. Adjust the corners manually.";
pub const AUTO_CORRECTION_NOT_NEEDED_MESSAGE: &str =
    "No meaningful perspective correction was detected.";
pub const IMAGE_BOUNDARY_TENTATIVE_MESSAGE: &str =
    "A possible boundary was detected but did not pass automatic-correction safety checks.";

const AUTO_CORRECTION_MAX_DIMENSION: u32 = 640;
const AUTO_CORRECTION_MIN_COLOR_DISTANCE: u32 = 32;
const AUTO_CORRECTION_MIN_COMPONENT_RATIO: f32 = 0.01;
const AUTO_CORRECTION_MIN_QUAD_AREA_RATIO: f32 = 0.15;
const AUTO_CORRECTION_MAX_QUAD_AREA_RATIO: f32 = 0.985;
const AUTO_CORRECTION_IDENTITY_TOLERANCE: f32 = 0.025;
const GRADIENT_EDGE_LOW: f32 = 20.0;
const GRADIENT_EDGE_HIGH: f32 = 60.0;
const GRADIENT_EDGE_CORRIDOR: i32 = 2;
const GRADIENT_MAX_LINES_PER_AXIS: usize = 12;
const GRADIENT_MIN_LINE_SUPPORT: f32 = 0.25;
const GRADIENT_MAX_LINE_GAP: f32 = 0.55;
const GRADIENT_MIN_PAIR_SPAN: f32 = 0.50;
const GRADIENT_MIN_MATERIAL_AREA_GROWTH: f32 = 0.05;
const GRADIENT_MIN_SIDE_SUPPORT: f32 = 0.35;
const GRADIENT_MAX_SIDE_GAP: f32 = 0.40;
const GRADIENT_MIN_SIDE_CONTRAST: f32 = 5.0;
const GRADIENT_MIN_SIGN_COHERENCE: f32 = 0.60;
const BOUNDARY_REFINEMENT_BINS: usize = 48;
const BOUNDARY_REFINEMENT_CORRIDOR: f32 = 6.0;
const BOUNDARY_REFINEMENT_MIN_COVERAGE: f32 = 0.35;
const MIN_QUAD_EDGE_LENGTH: f32 = 0.001;
const MIN_QUAD_AREA: f32 = 0.0001;
const MIN_QUAD_TURN: f32 = 0.00001;

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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageBoundaryDetection {
    pub status: ImageBoundaryDetectionStatus,
    pub corners: Vec<ImagePoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<ImageBoundaryEdgeMeasurement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageBoundaryEdgeMeasurement {
    pub axis_deviation_degrees: f32,
    pub straightness_rms: f32,
    pub bow_deviation: f32,
    pub coverage: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageBoundaryDetectionStatus {
    Reliable,
    Tentative,
    Unavailable,
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
        validate_ordered_convex_quad(&self.corners)?;
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

pub fn validate_ordered_convex_quad(corners: &[ImagePoint]) -> Result<(), String> {
    if corners.len() != 4 {
        return Err("quadrilateral requires four TL/TR/BR/BL corners".to_owned());
    }
    for corner in corners {
        validate_f32(
            corner.x,
            0.0,
            1.0,
            "quadrilateral corner x must be finite and normalized",
        )?;
        validate_f32(
            corner.y,
            0.0,
            1.0,
            "quadrilateral corner y must be finite and normalized",
        )?;
    }
    for index in 0..4 {
        let next = (index + 1) % 4;
        let edge = (corners[next].x - corners[index].x).hypot(corners[next].y - corners[index].y);
        if edge < MIN_QUAD_EDGE_LENGTH {
            return Err("quadrilateral edges must have meaningful length".to_owned());
        }
    }
    if segments_intersect(corners[0], corners[1], corners[2], corners[3])
        || segments_intersect(corners[1], corners[2], corners[3], corners[0])
    {
        return Err("quadrilateral edges must not intersect".to_owned());
    }
    for index in 0..4 {
        let first = corners[index];
        let second = corners[(index + 1) % 4];
        let third = corners[(index + 2) % 4];
        if cross_product(first, second, third) <= MIN_QUAD_TURN {
            return Err(
                "quadrilateral must be convex and ordered clockwise as TL/TR/BR/BL".to_owned(),
            );
        }
    }
    let signed_area = signed_quadrilateral_area(corners);
    if signed_area < MIN_QUAD_AREA {
        return Err("quadrilateral must have meaningful positive area".to_owned());
    }
    if corners[0].y + corners[1].y >= corners[2].y + corners[3].y
        || corners[0].x + corners[3].x >= corners[1].x + corners[2].x
    {
        return Err("quadrilateral corners must be ordered TL/TR/BR/BL".to_owned());
    }
    Ok(())
}

fn cross_product(first: ImagePoint, second: ImagePoint, third: ImagePoint) -> f32 {
    (second.x - first.x) * (third.y - second.y) - (second.y - first.y) * (third.x - second.x)
}

fn signed_quadrilateral_area(corners: &[ImagePoint]) -> f32 {
    corners
        .iter()
        .zip(corners.iter().cycle().skip(1))
        .take(4)
        .map(|(left, right)| left.x * right.y - right.x * left.y)
        .sum::<f32>()
        / 2.0
}

fn segments_intersect(
    first_start: ImagePoint,
    first_end: ImagePoint,
    second_start: ImagePoint,
    second_end: ImagePoint,
) -> bool {
    let first_side = cross_product(first_start, first_end, second_start);
    let second_side = cross_product(first_start, first_end, second_end);
    let third_side = cross_product(second_start, second_end, first_start);
    let fourth_side = cross_product(second_start, second_end, first_end);
    first_side * second_side < 0.0 && third_side * fourth_side < 0.0
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
    let Some(candidate) = locate_image_boundary_candidate(source)? else {
        return Ok(unavailable_proposal());
    };
    let normalized_corners = candidate.corners;
    if !(AUTO_CORRECTION_MIN_QUAD_AREA_RATIO..=AUTO_CORRECTION_MAX_QUAD_AREA_RATIO)
        .contains(&candidate.area_ratio)
    {
        return Ok(unavailable_proposal());
    }
    if candidate.larger_gradient_corners.is_some() {
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

pub fn detect_image_boundary(source: &[u8]) -> Result<ImageBoundaryDetection, String> {
    let Some(candidate) = locate_image_boundary_candidate(source)? else {
        if let Some(corners) = locate_gradient_image_boundary(source)? {
            return measured_image_boundary(
                source,
                ImageBoundaryDetectionStatus::Reliable,
                corners,
                None,
            );
        }
        return Ok(ImageBoundaryDetection {
            status: ImageBoundaryDetectionStatus::Unavailable,
            corners: Vec::new(),
            edges: Vec::new(),
            message: Some(AUTO_CORRECTION_UNAVAILABLE_MESSAGE.to_owned()),
        });
    };
    if let Some(corners) = candidate.larger_gradient_corners {
        return measured_image_boundary(
            source,
            ImageBoundaryDetectionStatus::Reliable,
            corners,
            None,
        );
    }
    let tentative = candidate.area_ratio < AUTO_CORRECTION_MIN_QUAD_AREA_RATIO;
    measured_image_boundary(
        source,
        if tentative {
            ImageBoundaryDetectionStatus::Tentative
        } else {
            ImageBoundaryDetectionStatus::Reliable
        },
        candidate.corners,
        tentative.then(|| IMAGE_BOUNDARY_TENTATIVE_MESSAGE.to_owned()),
    )
}

fn measured_image_boundary(
    source: &[u8],
    status: ImageBoundaryDetectionStatus,
    corners: [ImagePoint; 4],
    message: Option<String>,
) -> Result<ImageBoundaryDetection, String> {
    let refinement = refine_image_boundary(source, corners)?;
    let (corners, edges) = refinement
        .map(|refinement| (refinement.corners.to_vec(), refinement.edges.to_vec()))
        .unwrap_or_else(|| (corners.to_vec(), Vec::new()));
    Ok(ImageBoundaryDetection {
        status,
        corners,
        edges,
        message,
    })
}

struct ImageBoundaryCandidate {
    corners: [ImagePoint; 4],
    area_ratio: f32,
    larger_gradient_corners: Option<[ImagePoint; 4]>,
}

fn locate_image_boundary_candidate(
    source: &[u8],
) -> Result<Option<ImageBoundaryCandidate>, String> {
    let decoded = image::load_from_memory(source)
        .map_err(|error| format!("decode private original for auto correction: {error}"))?;
    let image = decoded
        .thumbnail(AUTO_CORRECTION_MAX_DIMENSION, AUTO_CORRECTION_MAX_DIMENSION)
        .to_rgb8();
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Ok(None);
    }

    let background = border_median_rgb(&image);
    let threshold = adaptive_color_distance_threshold(&image, background);
    let Some(component) = largest_contrast_component(&image, background, threshold) else {
        return Ok(None);
    };
    let corners = component.corners;
    let normalized_corners = corners.map(|(x, y)| normalized_point(x, y, width, height));
    let perspective = ImagePerspective {
        corners: normalized_corners,
    };
    if perspective.validate().is_err() {
        return Ok(None);
    }
    let area_ratio = quadrilateral_area_ratio(normalized_corners);
    let larger_gradient_corners = strongest_gradient_boundary(
        &image,
        (area_ratio + GRADIENT_MIN_MATERIAL_AREA_GROWTH).max(0.50),
    )
    .map(|corners| normalize_gradient_corners(corners, width, height));
    Ok(Some(ImageBoundaryCandidate {
        corners: normalized_corners,
        area_ratio,
        larger_gradient_corners,
    }))
}

fn locate_gradient_image_boundary(source: &[u8]) -> Result<Option<[ImagePoint; 4]>, String> {
    let decoded = image::load_from_memory(source)
        .map_err(|error| format!("decode private original for boundary detection: {error}"))?;
    let image = decoded
        .thumbnail(AUTO_CORRECTION_MAX_DIMENSION, AUTO_CORRECTION_MAX_DIMENSION)
        .to_rgb8();
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Ok(None);
    }
    Ok(
        strongest_gradient_boundary(&image, AUTO_CORRECTION_MIN_QUAD_AREA_RATIO)
            .map(|corners| normalize_gradient_corners(corners, width, height)),
    )
}

fn normalize_gradient_corners(
    corners: [GradientPoint; 4],
    width: u32,
    height: u32,
) -> [ImagePoint; 4] {
    let x_scale = width.saturating_sub(1).max(1) as f32;
    let y_scale = height.saturating_sub(1).max(1) as f32;
    corners.map(|point| ImagePoint {
        x: (point.x / x_scale).clamp(0.0, 1.0),
        y: (point.y / y_scale).clamp(0.0, 1.0),
    })
}

struct RefinedImageBoundary {
    corners: [ImagePoint; 4],
    edges: [ImageBoundaryEdgeMeasurement; 4],
}

#[derive(Clone, Copy)]
struct FittedBoundaryLine {
    horizontal: bool,
    slope: f32,
    intercept: f32,
    measurement: ImageBoundaryEdgeMeasurement,
}

fn refine_image_boundary(
    source: &[u8],
    corners: [ImagePoint; 4],
) -> Result<Option<RefinedImageBoundary>, String> {
    let decoded = image::load_from_memory(source)
        .map_err(|error| format!("decode private original for boundary refinement: {error}"))?;
    let image = decoded
        .thumbnail(AUTO_CORRECTION_MAX_DIMENSION, AUTO_CORRECTION_MAX_DIMENSION)
        .to_rgb8();
    let (width, height) = image.dimensions();
    if width < 2 || height < 2 {
        return Ok(None);
    }
    let gray = DynamicImage::ImageRgb8(image).to_luma8();
    let edge_map = canny(&gray, GRADIENT_EDGE_LOW, GRADIENT_EDGE_HIGH);
    let pixel_corners = corners.map(|point| GradientPoint {
        x: point.x * width.saturating_sub(1) as f32,
        y: point.y * height.saturating_sub(1) as f32,
    });
    let mut lines = Vec::with_capacity(4);
    for index in 0..4 {
        let Some(line) = fit_boundary_line(
            &edge_map,
            pixel_corners[index],
            pixel_corners[(index + 1) % 4],
            index % 2 == 0,
        ) else {
            return Ok(None);
        };
        lines.push(line);
    }
    let pixel_intersections = [
        fitted_line_intersection(lines[3], lines[0]),
        fitted_line_intersection(lines[0], lines[1]),
        fitted_line_intersection(lines[1], lines[2]),
        fitted_line_intersection(lines[2], lines[3]),
    ];
    let Some(pixel_corners) = pixel_intersections.into_iter().collect::<Option<Vec<_>>>() else {
        return Ok(None);
    };
    let refined_corners = std::array::from_fn(|index| ImagePoint {
        x: (pixel_corners[index].x / width.saturating_sub(1) as f32).clamp(0.0, 1.0),
        y: (pixel_corners[index].y / height.saturating_sub(1) as f32).clamp(0.0, 1.0),
    });
    if (ImagePerspective {
        corners: refined_corners,
    })
    .validate()
    .is_err()
    {
        return Ok(None);
    }
    Ok(Some(RefinedImageBoundary {
        corners: refined_corners,
        edges: std::array::from_fn(|index| lines[index].measurement),
    }))
}

fn fit_boundary_line(
    edges: &GrayImage,
    start: GradientPoint,
    end: GradientPoint,
    horizontal: bool,
) -> Option<FittedBoundaryLine> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_squared = dx * dx + dy * dy;
    let length = length_squared.sqrt();
    if length < 1.0 {
        return None;
    }
    let min_x = (start.x.min(end.x) - BOUNDARY_REFINEMENT_CORRIDOR)
        .floor()
        .max(0.0) as u32;
    let max_x = (start.x.max(end.x) + BOUNDARY_REFINEMENT_CORRIDOR)
        .ceil()
        .min(edges.width().saturating_sub(1) as f32) as u32;
    let min_y = (start.y.min(end.y) - BOUNDARY_REFINEMENT_CORRIDOR)
        .floor()
        .max(0.0) as u32;
    let max_y = (start.y.max(end.y) + BOUNDARY_REFINEMENT_CORRIDOR)
        .ceil()
        .min(edges.height().saturating_sub(1) as f32) as u32;
    let mut bins = [None::<(GradientPoint, f32, f32)>; BOUNDARY_REFINEMENT_BINS];
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if edges.get_pixel(x, y)[0] == 0 {
                continue;
            }
            let point = GradientPoint {
                x: x as f32,
                y: y as f32,
            };
            let relative_x = point.x - start.x;
            let relative_y = point.y - start.y;
            let position = (relative_x * dx + relative_y * dy) / length_squared;
            if !(0.10..=0.90).contains(&position) {
                continue;
            }
            let distance = (dx * relative_y - dy * relative_x).abs() / length;
            if distance > BOUNDARY_REFINEMENT_CORRIDOR {
                continue;
            }
            let bin = (((position - 0.10) / 0.80) * BOUNDARY_REFINEMENT_BINS as f32)
                .floor()
                .clamp(0.0, BOUNDARY_REFINEMENT_BINS.saturating_sub(1) as f32)
                as usize;
            if bins[bin].is_none_or(|(_, _, current_distance)| distance < current_distance) {
                bins[bin] = Some((point, position, distance));
            }
        }
    }
    let points = bins
        .into_iter()
        .flatten()
        .map(|(point, position, _)| (point, position))
        .collect::<Vec<_>>();
    let coverage = points.len() as f32 / BOUNDARY_REFINEMENT_BINS as f32;
    if coverage < BOUNDARY_REFINEMENT_MIN_COVERAGE || points.len() < 8 {
        return None;
    }
    let independent_mean = points
        .iter()
        .map(|(point, _)| if horizontal { point.x } else { point.y })
        .sum::<f32>()
        / points.len() as f32;
    let dependent_mean = points
        .iter()
        .map(|(point, _)| if horizontal { point.y } else { point.x })
        .sum::<f32>()
        / points.len() as f32;
    let covariance = points
        .iter()
        .map(|(point, _)| {
            let independent = if horizontal { point.x } else { point.y };
            let dependent = if horizontal { point.y } else { point.x };
            (independent - independent_mean) * (dependent - dependent_mean)
        })
        .sum::<f32>();
    let variance = points
        .iter()
        .map(|(point, _)| {
            let independent = if horizontal { point.x } else { point.y };
            (independent - independent_mean).powi(2)
        })
        .sum::<f32>();
    if variance <= f32::EPSILON {
        return None;
    }
    let slope = covariance / variance;
    let intercept = dependent_mean - slope * independent_mean;
    let residuals = points
        .iter()
        .map(|(point, position)| {
            let independent = if horizontal { point.x } else { point.y };
            let dependent = if horizontal { point.y } else { point.x };
            (position, dependent - (slope * independent + intercept))
        })
        .collect::<Vec<_>>();
    let rms_pixels = (residuals
        .iter()
        .map(|(_, residual)| residual * residual)
        .sum::<f32>()
        / residuals.len() as f32)
        .sqrt();
    let center_residuals = residuals
        .iter()
        .filter(|(position, _)| (0.40..=0.60).contains(*position))
        .map(|(_, residual)| *residual)
        .collect::<Vec<_>>();
    let outer_residuals = residuals
        .iter()
        .filter(|(position, _)| {
            (0.10..=0.30).contains(*position) || (0.70..=0.90).contains(*position)
        })
        .map(|(_, residual)| *residual)
        .collect::<Vec<_>>();
    let bow_pixels = if center_residuals.is_empty() || outer_residuals.is_empty() {
        0.0
    } else {
        (center_residuals.iter().sum::<f32>() / center_residuals.len() as f32
            - outer_residuals.iter().sum::<f32>() / outer_residuals.len() as f32)
            .abs()
    };
    let perpendicular_scale = if horizontal {
        edges.height().saturating_sub(1).max(1) as f32
    } else {
        edges.width().saturating_sub(1).max(1) as f32
    };
    Some(FittedBoundaryLine {
        horizontal,
        slope,
        intercept,
        measurement: ImageBoundaryEdgeMeasurement {
            axis_deviation_degrees: if horizontal {
                slope.atan().to_degrees()
            } else {
                -slope.atan().to_degrees()
            },
            straightness_rms: rms_pixels / perpendicular_scale,
            bow_deviation: bow_pixels / perpendicular_scale,
            coverage,
        },
    })
}

fn fitted_line_intersection(
    first: FittedBoundaryLine,
    second: FittedBoundaryLine,
) -> Option<GradientPoint> {
    let (first_a, first_b, first_c) = fitted_line_coefficients(first);
    let (second_a, second_b, second_c) = fitted_line_coefficients(second);
    let determinant = first_a * second_b - second_a * first_b;
    if determinant.abs() <= f32::EPSILON {
        return None;
    }
    Some(GradientPoint {
        x: (first_b * second_c - second_b * first_c) / determinant,
        y: (first_c * second_a - second_c * first_a) / determinant,
    })
}

fn fitted_line_coefficients(line: FittedBoundaryLine) -> (f32, f32, f32) {
    if line.horizontal {
        (-line.slope, 1.0, -line.intercept)
    } else {
        (1.0, -line.slope, -line.intercept)
    }
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

    ContrastComponent { count, corners }
}

#[derive(Clone, Copy)]
struct GradientPoint {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct GradientAxisLine {
    line: PolarLine,
    position: f32,
    support: f32,
    longest_gap: f32,
}

struct GradientSideEvidence {
    support: f32,
    longest_gap: f32,
    mean_contrast: f32,
    sign_coherence: f32,
}

fn strongest_gradient_boundary(
    image: &image::RgbImage,
    minimum_area_ratio: f32,
) -> Option<[GradientPoint; 4]> {
    let gray = DynamicImage::ImageRgb8(image.clone()).to_luma8();
    let edges = canny(&gray, GRADIENT_EDGE_LOW, GRADIENT_EDGE_HIGH);
    let lines = detect_lines(
        &edges,
        LineDetectionOptions {
            vote_threshold: image.width().min(image.height()).max(1) / 3,
            suppression_radius: 8,
        },
    );
    let vertical = select_gradient_axis_lines(&edges, &lines, true);
    let horizontal = select_gradient_axis_lines(&edges, &lines, false);
    debug_assert!(vertical.len() <= GRADIENT_MAX_LINES_PER_AXIS);
    debug_assert!(horizontal.len() <= GRADIENT_MAX_LINES_PER_AXIS);
    let mut combinations = 0usize;
    let mut best_candidate = None::<(usize, f32, [GradientPoint; 4])>;
    for left_index in 0..vertical.len() {
        for right_index in left_index + 1..vertical.len() {
            let left = &vertical[left_index];
            let right = &vertical[right_index];
            if (right.position - left.position) / (image.width().max(1) as f32)
                < GRADIENT_MIN_PAIR_SPAN
            {
                continue;
            }
            for top_index in 0..horizontal.len() {
                for bottom_index in top_index + 1..horizontal.len() {
                    combinations += 1;
                    debug_assert!(combinations <= 4_356);
                    let top = &horizontal[top_index];
                    let bottom = &horizontal[bottom_index];
                    if (bottom.position - top.position) / (image.height().max(1) as f32)
                        < GRADIENT_MIN_PAIR_SPAN
                    {
                        continue;
                    }
                    let Some(corners) = gradient_quad_intersections(left, right, top, bottom)
                    else {
                        continue;
                    };
                    if !gradient_candidate_corners_in_bounds(corners, image.width(), image.height())
                    {
                        continue;
                    }
                    let area_ratio = gradient_polygon_area(corners)
                        / (image.width().max(1) * image.height().max(1)) as f32;
                    if area_ratio <= minimum_area_ratio {
                        continue;
                    }
                    let center = GradientPoint {
                        x: corners.iter().map(|point| point.x).sum::<f32>() / 4.0,
                        y: corners.iter().map(|point| point.y).sum::<f32>() / 4.0,
                    };
                    let sides = [
                        gradient_side_evidence(&gray, &edges, corners[0], corners[1], center),
                        gradient_side_evidence(&gray, &edges, corners[1], corners[2], center),
                        gradient_side_evidence(&gray, &edges, corners[2], corners[3], center),
                        gradient_side_evidence(&gray, &edges, corners[3], corners[0], center),
                    ];
                    let valid_sides = sides
                        .iter()
                        .filter(|side| {
                            side.support >= GRADIENT_MIN_SIDE_SUPPORT
                                && side.longest_gap <= GRADIENT_MAX_SIDE_GAP
                                && side.mean_contrast >= GRADIENT_MIN_SIDE_CONTRAST
                                && side.sign_coherence >= GRADIENT_MIN_SIGN_COHERENCE
                        })
                        .count();
                    let score = area_ratio * sides.iter().map(|side| side.support).sum::<f32>()
                        / 4.0
                        * (valid_sides as f32 / 4.0);
                    if best_candidate.is_none_or(|(best_sides, best_score, _)| {
                        valid_sides > best_sides
                            || (valid_sides == best_sides && score > best_score)
                    }) {
                        best_candidate = Some((valid_sides, score, corners));
                    }
                }
            }
        }
    }
    best_candidate.and_then(|(valid_sides, _, corners)| (valid_sides >= 3).then_some(corners))
}

fn select_gradient_axis_lines(
    edges: &GrayImage,
    lines: &[PolarLine],
    vertical: bool,
) -> Vec<GradientAxisLine> {
    let (width, height) = edges.dimensions();
    let mut selected = lines
        .iter()
        .filter(|line| {
            if vertical {
                line.angle_in_degrees <= 15 || line.angle_in_degrees >= 165
            } else {
                (75..=105).contains(&line.angle_in_degrees)
            }
        })
        .filter_map(|line| {
            let position = if vertical {
                gradient_line_position_at_y(*line, height as f32 / 2.0)
            } else {
                gradient_line_position_at_x(*line, width as f32 / 2.0)
            }?;
            let (support, longest_gap) = gradient_full_line_support(edges, *line)?;
            (support >= GRADIENT_MIN_LINE_SUPPORT && longest_gap <= GRADIENT_MAX_LINE_GAP)
                .then_some(GradientAxisLine {
                    line: *line,
                    position,
                    support,
                    longest_gap,
                })
        })
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        let left_score = left.support - left.longest_gap * 0.25;
        let right_score = right.support - right.longest_gap * 0.25;
        right_score
            .partial_cmp(&left_score)
            .unwrap_or(Ordering::Equal)
    });
    selected.truncate(GRADIENT_MAX_LINES_PER_AXIS);
    selected.sort_by(|left, right| {
        left.position
            .partial_cmp(&right.position)
            .unwrap_or(Ordering::Equal)
    });
    selected
}

fn gradient_full_line_support(edges: &GrayImage, line: PolarLine) -> Option<(f32, f32)> {
    let (start, end) = intersection_points(line, edges.width(), edges.height())?;
    gradient_sample_edge_support(
        edges,
        GradientPoint {
            x: start.0,
            y: start.1,
        },
        GradientPoint { x: end.0, y: end.1 },
    )
}

fn gradient_quad_intersections(
    left: &GradientAxisLine,
    right: &GradientAxisLine,
    top: &GradientAxisLine,
    bottom: &GradientAxisLine,
) -> Option<[GradientPoint; 4]> {
    Some([
        gradient_intersection(left.line, top.line)?,
        gradient_intersection(right.line, top.line)?,
        gradient_intersection(right.line, bottom.line)?,
        gradient_intersection(left.line, bottom.line)?,
    ])
}

fn gradient_intersection(left: PolarLine, right: PolarLine) -> Option<GradientPoint> {
    let left_angle = (left.angle_in_degrees as f32).to_radians();
    let right_angle = (right.angle_in_degrees as f32).to_radians();
    let (left_sin, left_cos) = left_angle.sin_cos();
    let (right_sin, right_cos) = right_angle.sin_cos();
    let determinant = left_cos * right_sin - right_cos * left_sin;
    if determinant.abs() < 0.0001 {
        return None;
    }
    Some(GradientPoint {
        x: (left.r * right_sin - right.r * left_sin) / determinant,
        y: (left_cos * right.r - right_cos * left.r) / determinant,
    })
}

fn gradient_line_position_at_y(line: PolarLine, y: f32) -> Option<f32> {
    let angle = (line.angle_in_degrees as f32).to_radians();
    let (sin, cos) = angle.sin_cos();
    (cos.abs() > 0.0001).then_some((line.r - y * sin) / cos)
}

fn gradient_line_position_at_x(line: PolarLine, x: f32) -> Option<f32> {
    let angle = (line.angle_in_degrees as f32).to_radians();
    let (sin, cos) = angle.sin_cos();
    (sin.abs() > 0.0001).then_some((line.r - x * cos) / sin)
}

fn gradient_candidate_corners_in_bounds(
    corners: [GradientPoint; 4],
    width: u32,
    height: u32,
) -> bool {
    corners.iter().all(|point| {
        point.x >= -4.0
            && point.y >= -4.0
            && point.x <= width as f32 + 4.0
            && point.y <= height as f32 + 4.0
    })
}

fn gradient_polygon_area(corners: [GradientPoint; 4]) -> f32 {
    corners
        .iter()
        .zip(corners.iter().cycle().skip(1))
        .take(4)
        .map(|(left, right)| left.x * right.y - right.x * left.y)
        .sum::<f32>()
        .abs()
        / 2.0
}

fn gradient_sample_edge_support(
    edges: &GrayImage,
    start: GradientPoint,
    end: GradientPoint,
) -> Option<(f32, f32)> {
    let length = (end.x - start.x).hypot(end.y - start.y);
    let samples = length.ceil() as usize + 1;
    if samples < 2 {
        return None;
    }
    let mut supported = 0usize;
    let mut current_gap = 0usize;
    let mut longest_gap = 0usize;
    for index in 0..samples {
        let fraction = index as f32 / (samples - 1) as f32;
        let x = start.x + (end.x - start.x) * fraction;
        let y = start.y + (end.y - start.y) * fraction;
        if gradient_edge_near(edges, x.round() as i32, y.round() as i32) {
            supported += 1;
            current_gap = 0;
        } else {
            current_gap += 1;
            longest_gap = longest_gap.max(current_gap);
        }
    }
    Some((
        supported as f32 / samples as f32,
        longest_gap as f32 / samples as f32,
    ))
}

fn gradient_edge_near(edges: &GrayImage, x: i32, y: i32) -> bool {
    (-GRADIENT_EDGE_CORRIDOR..=GRADIENT_EDGE_CORRIDOR).any(|offset_y| {
        (-GRADIENT_EDGE_CORRIDOR..=GRADIENT_EDGE_CORRIDOR).any(|offset_x| {
            let sample_x = x + offset_x;
            let sample_y = y + offset_y;
            sample_x >= 0
                && sample_y >= 0
                && sample_x < edges.width() as i32
                && sample_y < edges.height() as i32
                && edges.get_pixel(sample_x as u32, sample_y as u32).0[0] != 0
        })
    })
}

fn gradient_side_evidence(
    gray: &GrayImage,
    edges: &GrayImage,
    start: GradientPoint,
    end: GradientPoint,
    center: GradientPoint,
) -> GradientSideEvidence {
    let (support, longest_gap) =
        gradient_sample_edge_support(edges, start, end).unwrap_or((0.0, 1.0));
    let length = (end.x - start.x).hypot(end.y - start.y);
    let samples = length.ceil() as usize + 1;
    let middle = GradientPoint {
        x: (start.x + end.x) / 2.0,
        y: (start.y + end.y) / 2.0,
    };
    let toward_x = center.x - middle.x;
    let toward_y = center.y - middle.y;
    let toward_length = toward_x.hypot(toward_y).max(0.001);
    let normal_x = toward_x / toward_length;
    let normal_y = toward_y / toward_length;
    let mut differences = Vec::new();
    for index in 0..samples {
        let fraction = index as f32 / (samples - 1).max(1) as f32;
        let x = start.x + (end.x - start.x) * fraction;
        let y = start.y + (end.y - start.y) * fraction;
        let inside = gradient_sample_gray(gray, x + normal_x * 4.0, y + normal_y * 4.0);
        let outside = gradient_sample_gray(gray, x - normal_x * 4.0, y - normal_y * 4.0);
        if let (Some(inside), Some(outside)) = (inside, outside) {
            differences.push(inside - outside);
        }
    }
    let mean_contrast =
        differences.iter().map(|value| value.abs()).sum::<f32>() / differences.len().max(1) as f32;
    let positive = differences.iter().filter(|value| **value >= 0.0).count();
    let negative = differences.len().saturating_sub(positive);
    let sign_coherence = positive.max(negative) as f32 / differences.len().max(1) as f32;
    GradientSideEvidence {
        support,
        longest_gap,
        mean_contrast,
        sign_coherence,
    }
}

fn gradient_sample_gray(gray: &GrayImage, x: f32, y: f32) -> Option<f32> {
    let x = x.round() as i32;
    let y = y.round() as i32;
    (x >= 0 && y >= 0 && x < gray.width() as i32 && y < gray.height() as i32)
        .then(|| gray.get_pixel(x as u32, y as u32).0[0] as f32)
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

#[cfg(test)]
mod tests {
    use super::{GradientPoint, gradient_candidate_corners_in_bounds};

    #[test]
    fn gradient_candidate_bounds_match_imageproc_geometric_coordinates() {
        let width = 120;
        let height = 90;
        let corners = |x, y| [GradientPoint { x, y }; 4];

        assert!(gradient_candidate_corners_in_bounds(
            corners(-4.0, -4.0),
            width,
            height
        ));
        assert!(gradient_candidate_corners_in_bounds(
            corners(width as f32, height as f32),
            width,
            height
        ));
        assert!(gradient_candidate_corners_in_bounds(
            corners(width as f32 + 4.0, height as f32 + 4.0),
            width,
            height
        ));
        assert!(!gradient_candidate_corners_in_bounds(
            corners(-4.001, 0.0),
            width,
            height
        ));
        assert!(!gradient_candidate_corners_in_bounds(
            corners(width as f32 + 4.001, 0.0),
            width,
            height
        ));
        assert!(!gradient_candidate_corners_in_bounds(
            corners(0.0, height as f32 + 4.001),
            width,
            height
        ));
    }
}
