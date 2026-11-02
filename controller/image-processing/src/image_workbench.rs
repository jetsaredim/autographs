use std::collections::HashSet;
use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbaImage};
use serde::Serialize;

use crate::{
    derivatives::MAX_SOURCE_BYTES,
    image_adjustments::{
        ImageAdjustment, ImageBoundaryDetection, ImageBoundaryDetectionStatus, ImageCrop,
        ImagePerspective, ImagePoint, apply_image_adjustment, detect_image_boundary,
        validate_ordered_convex_quad,
    },
};

const MAX_ALIGNMENT_PASSES: usize = 8;
const MIN_SCORE_IMPROVEMENT: f32 = 0.005;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAdjustmentAnalysis {
    pub boundary: ImageBoundaryDetection,
    pub next_suggestion: Option<AlignmentSuggestion>,
    pub framing_support: Option<FramingSupportAnalysis>,
    pub framing_boundary_source: Option<&'static str>,
    #[serde(skip)]
    pub preview_aspect_ratio: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentSuggestion {
    pub kind: &'static str,
    pub measurement: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corners: Option<[ImagePoint; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_degrees: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoAlignmentResult {
    pub adjustment: ImageAdjustment,
    pub accepted_passes: usize,
    pub stop_reason: &'static str,
    pub boundary: ImageBoundaryDetection,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FramingMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FramingSupportAnalysis {
    pub item_short_edge_pixels: f32,
    pub margins_pixels: FramingMargins,
    pub max_symmetric_margin_pixels: f32,
    pub item_intersects_invalid_pixels: bool,
    pub limited_by_existing_crop: bool,
}

pub fn analyze_adjustment(
    source: &[u8],
    adjustment: &ImageAdjustment,
    manual_boundary: Option<&[ImagePoint]>,
) -> Result<ImageAdjustmentAnalysis, String> {
    let reduced = decode_analysis_source(source)?;
    analyze_reduced_adjustment(&reduced, adjustment, manual_boundary)
}

fn analyze_reduced_adjustment(
    reduced: &DynamicImage,
    adjustment: &ImageAdjustment,
    manual_boundary: Option<&[ImagePoint]>,
) -> Result<ImageAdjustmentAnalysis, String> {
    adjustment.validate()?;
    if let Some(corners) = manual_boundary {
        validate_ordered_convex_quad(corners)?;
    }
    let rendered = apply_image_adjustment(reduced.clone(), adjustment)?.to_rgba8();
    let preview_aspect_ratio = rendered.width() as f32 / rendered.height() as f32;
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(rendered.clone())
        .write_to(&mut output, ImageFormat::WebP)
        .map_err(|error| format!("encode private analysis preview: {error}"))?;
    let boundary = detect_image_boundary(&output.into_inner())?;
    let automatic =
        boundary.status == ImageBoundaryDetectionStatus::Reliable && boundary.corners.len() == 4;
    let (framing_corners, framing_boundary_source) = if let Some(manual) = manual_boundary {
        (Some(manual), Some("manual"))
    } else if automatic {
        (Some(boundary.corners.as_slice()), Some("automatic"))
    } else {
        (None, None)
    };
    let limited_by_existing_crop = adjustment.crop.is_some()
        || adjustment.zoom > 1.0
        || adjustment.pan_x != 0.0
        || adjustment.pan_y != 0.0;
    let framing_support = framing_corners
        .and_then(|corners| analyze_framing_support(&rendered, corners, limited_by_existing_crop));
    let next_suggestion = if automatic {
        next_alignment_suggestion(&boundary, adjustment, preview_aspect_ratio)
    } else {
        None
    };
    let framing_boundary_source = framing_support.as_ref().and(framing_boundary_source);
    Ok(ImageAdjustmentAnalysis {
        boundary,
        next_suggestion,
        framing_support,
        framing_boundary_source,
        preview_aspect_ratio,
    })
}

fn decode_analysis_source(source: &[u8]) -> Result<DynamicImage, String> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err("private original exceeds the derivative source limit".to_owned());
    }
    let decoded = image::load_from_memory(source)
        .map_err(|error| format!("decode private original for analysis: {error}"))?;
    Ok(decoded.thumbnail(640, 640))
}

pub fn auto_align_adjustment(
    source: &[u8],
    initial: &ImageAdjustment,
) -> Result<AutoAlignmentResult, String> {
    let reduced = decode_analysis_source(source)?;
    auto_align_with(initial, |adjustment| {
        analyze_reduced_adjustment(&reduced, adjustment, None)
    })
}

fn auto_align_with(
    initial: &ImageAdjustment,
    mut evaluate: impl FnMut(&ImageAdjustment) -> Result<ImageAdjustmentAnalysis, String>,
) -> Result<AutoAlignmentResult, String> {
    initial.validate()?;
    let mut current = initial.clone();
    let mut analysis = evaluate(&current)?;
    let mut accepted_passes = 0;
    let mut seen = HashSet::from([current.to_json()?]);
    let mut stop_reason = "iteration limit reached";

    for _ in 0..MAX_ALIGNMENT_PASSES {
        let Some(suggestion) = analysis.next_suggestion.clone() else {
            stop_reason = if analysis.boundary.status == ImageBoundaryDetectionStatus::Reliable
                && !alignment_needs_correction(&analysis.boundary, analysis.preview_aspect_ratio)
            {
                "alignment is within tolerance"
            } else if analysis.boundary.status == ImageBoundaryDetectionStatus::Reliable {
                "no valid improving adjustment can be proposed; retained the last reliable result"
            } else {
                "boundary detection is unreliable; retained the last reliable result"
            };
            break;
        };
        let aspect_ratio = analysis.preview_aspect_ratio;
        let previous_score = alignment_score(&analysis.boundary, aspect_ratio, &suggestion);
        let mut accepted = None;
        let mut saw_repeated = false;
        let mut saw_unreliable = false;
        let fractions: &[f32] = if suggestion.kind == "rotation" {
            &[1.0, 0.5, 1.0 / 3.0, 0.25]
        } else {
            &[1.0, 0.75, 0.5, 1.0 / 3.0, 0.25, 0.125]
        };
        for fraction in fractions {
            let candidate = apply_suggestion_fraction(&current, &suggestion, *fraction);
            if candidate.validate().is_err() {
                continue;
            }
            let signature = candidate.to_json()?;
            if !seen.insert(signature) {
                saw_repeated = true;
                continue;
            }
            let candidate_analysis = evaluate(&candidate)?;
            if candidate_analysis.boundary.status != ImageBoundaryDetectionStatus::Reliable {
                saw_unreliable = true;
                continue;
            }
            let candidate_aspect = candidate_analysis.preview_aspect_ratio;
            let candidate_score =
                alignment_score(&candidate_analysis.boundary, candidate_aspect, &suggestion);
            if candidate_score < previous_score - MIN_SCORE_IMPROVEMENT {
                accepted = Some((
                    candidate,
                    candidate_analysis,
                    previous_score - candidate_score,
                ));
                break;
            }
        }
        let Some((candidate, candidate_analysis, improvement)) = accepted else {
            stop_reason = if saw_repeated {
                "a repeated adjustment was avoided"
            } else if saw_unreliable {
                "boundary detection became unreliable; retained the last reliable result"
            } else {
                "the next pass did not improve alignment; retained the better result"
            };
            break;
        };
        current = candidate;
        analysis = candidate_analysis;
        accepted_passes += 1;
        if analysis.next_suggestion.is_none() {
            stop_reason = if alignment_needs_correction(
                &analysis.boundary,
                analysis.preview_aspect_ratio,
            ) {
                "no valid improving adjustment can be proposed; retained the last reliable result"
            } else {
                "alignment is within tolerance"
            };
            break;
        }
        if analysis
            .next_suggestion
            .as_ref()
            .is_some_and(|next| next.kind == suggestion.kind)
            && improvement < 0.01
        {
            stop_reason = "further improvement is below measurement precision";
            break;
        }
    }
    Ok(AutoAlignmentResult {
        adjustment: current,
        accepted_passes,
        stop_reason,
        boundary: analysis.boundary,
    })
}

fn next_alignment_suggestion(
    boundary: &ImageBoundaryDetection,
    adjustment: &ImageAdjustment,
    aspect_ratio: f32,
) -> Option<AlignmentSuggestion> {
    if boundary.status != ImageBoundaryDetectionStatus::Reliable || boundary.corners.len() != 4 {
        return None;
    }
    let fine = fitted_edges_are_stable(boundary);
    let offsets = edge_offsets(boundary, aspect_ratio, fine)?;
    let common_tilt = offsets.iter().sum::<f32>() / 4.0;
    let parallelism = (offsets[0] - offsets[2])
        .abs()
        .max((offsets[1] - offsets[3]).abs());
    let worst_axis = offsets.iter().map(|value| value.abs()).fold(0.0, f32::max);
    let measurement = if fine { "fine" } else { "coarse" };
    if parallelism >= if fine { 0.15 } else { 0.5 } {
        return Some(AlignmentSuggestion {
            kind: "perspective",
            measurement,
            corners: Some(suggested_perspective_corners(
                &boundary.corners,
                adjustment,
                aspect_ratio,
                common_tilt,
            )?),
            rotation_degrees: None,
        })
        .filter(|proposal| proposal.corners != Some(current_perspective_corners(adjustment)));
    }
    if common_tilt.abs() >= if fine { 0.15 } else { 0.25 } {
        let rotation_degrees =
            ((adjustment.rotation_degrees - common_tilt).clamp(-15.0, 15.0) * 20.0).round() / 20.0;
        return (rotation_degrees != adjustment.rotation_degrees).then_some(AlignmentSuggestion {
            kind: "rotation",
            measurement,
            corners: None,
            rotation_degrees: Some(rotation_degrees),
        });
    }
    if worst_axis >= if fine { 0.25 } else { 0.5 } {
        return Some(AlignmentSuggestion {
            kind: "perspective",
            measurement,
            corners: Some(suggested_perspective_corners(
                &boundary.corners,
                adjustment,
                aspect_ratio,
                common_tilt,
            )?),
            rotation_degrees: None,
        })
        .filter(|proposal| proposal.corners != Some(current_perspective_corners(adjustment)));
    }
    None
}

fn current_perspective_corners(adjustment: &ImageAdjustment) -> [ImagePoint; 4] {
    adjustment
        .perspective
        .as_ref()
        .map(|p| p.corners)
        .unwrap_or([
            ImagePoint { x: 0.0, y: 0.0 },
            ImagePoint { x: 1.0, y: 0.0 },
            ImagePoint { x: 1.0, y: 1.0 },
            ImagePoint { x: 0.0, y: 1.0 },
        ])
}

fn alignment_needs_correction(boundary: &ImageBoundaryDetection, aspect_ratio: f32) -> bool {
    if boundary.status != ImageBoundaryDetectionStatus::Reliable || boundary.corners.len() != 4 {
        return false;
    }
    let fine = fitted_edges_are_stable(boundary);
    let Some(offsets) = edge_offsets(boundary, aspect_ratio, fine) else {
        return false;
    };
    let common_tilt = offsets.iter().sum::<f32>() / 4.0;
    let parallelism = (offsets[0] - offsets[2])
        .abs()
        .max((offsets[1] - offsets[3]).abs());
    let worst_axis = offsets.iter().map(|value| value.abs()).fold(0.0, f32::max);
    parallelism >= if fine { 0.15 } else { 0.5 }
        || common_tilt.abs() >= if fine { 0.15 } else { 0.25 }
        || worst_axis >= if fine { 0.25 } else { 0.5 }
}

fn fitted_edges_are_stable(boundary: &ImageBoundaryDetection) -> bool {
    boundary.edges.len() == 4
        && boundary
            .edges
            .iter()
            .all(|edge| edge.coverage >= 0.7 && edge.straightness_rms <= 0.004)
}

fn edge_offsets(
    boundary: &ImageBoundaryDetection,
    aspect_ratio: f32,
    fine: bool,
) -> Option<[f32; 4]> {
    if fine {
        return Some(std::array::from_fn(|index| {
            boundary.edges[index].axis_deviation_degrees
        }));
    }
    if boundary.corners.len() != 4 {
        return None;
    }
    Some(std::array::from_fn(|index| {
        let start = boundary.corners[index];
        let end = boundary.corners[(index + 1) % 4];
        let angle = ((end.y - start.y).atan2((end.x - start.x) * aspect_ratio)).to_degrees();
        let target = if index % 2 == 0 {
            (angle / 180.0).round() * 180.0
        } else {
            ((angle - 90.0) / 180.0).round() * 180.0 + 90.0
        };
        angle - target
    }))
}

fn suggested_perspective_corners(
    output: &[ImagePoint],
    adjustment: &ImageAdjustment,
    aspect_ratio: f32,
    common_tilt: f32,
) -> Option<[ImagePoint; 4]> {
    let scaled: Vec<_> = output
        .iter()
        .map(|point| ImagePoint {
            x: point.x * aspect_ratio,
            y: point.y,
        })
        .collect();
    let untilted = rotate_points(&scaled, -common_tilt);
    let left = (untilted[0].x + untilted[3].x) / 2.0;
    let right = (untilted[1].x + untilted[2].x) / 2.0;
    let top = (untilted[0].y + untilted[1].y) / 2.0;
    let bottom = (untilted[2].y + untilted[3].y) / 2.0;
    let desired = rotate_points(
        &[
            ImagePoint { x: left, y: top },
            ImagePoint { x: right, y: top },
            ImagePoint {
                x: right,
                y: bottom,
            },
            ImagePoint { x: left, y: bottom },
        ],
        common_tilt,
    );
    let source = current_perspective_corners(adjustment);
    let source_width =
        ((source[1].x - source[0].x).abs() + (source[2].x - source[3].x).abs()) / 2.0;
    let source_height =
        ((source[3].y - source[0].y).abs() + (source[2].y - source[1].y).abs()) / 2.0;
    let result = std::array::from_fn(|index| ImagePoint {
        x: (source[index].x
            - (desired[index].x - scaled[index].x) / aspect_ratio * source_width * 0.75)
            .clamp(0.0, 1.0),
        y: (source[index].y - (desired[index].y - scaled[index].y) * source_height * 0.75)
            .clamp(0.0, 1.0),
    });
    (validate_ordered_convex_quad(&result).is_ok()).then_some(result)
}

fn rotate_points(points: &[ImagePoint], degrees: f32) -> Vec<ImagePoint> {
    let radians = degrees.to_radians();
    let (sine, cosine) = radians.sin_cos();
    let center = ImagePoint {
        x: points.iter().map(|p| p.x).sum::<f32>() / points.len() as f32,
        y: points.iter().map(|p| p.y).sum::<f32>() / points.len() as f32,
    };
    points
        .iter()
        .map(|point| {
            let x = point.x - center.x;
            let y = point.y - center.y;
            ImagePoint {
                x: center.x + cosine * x - sine * y,
                y: center.y + sine * x + cosine * y,
            }
        })
        .collect()
}

fn apply_suggestion_fraction(
    current: &ImageAdjustment,
    suggestion: &AlignmentSuggestion,
    fraction: f32,
) -> ImageAdjustment {
    let mut candidate = current.clone();
    if let Some(rotation) = suggestion.rotation_degrees {
        candidate.rotation_degrees =
            ((current.rotation_degrees + (rotation - current.rotation_degrees) * fraction) * 20.0)
                .round()
                / 20.0;
    } else if let Some(corners) = suggestion.corners {
        let previous = current.perspective.as_ref().map(|p| p.corners).unwrap_or([
            ImagePoint { x: 0.0, y: 0.0 },
            ImagePoint { x: 1.0, y: 0.0 },
            ImagePoint { x: 1.0, y: 1.0 },
            ImagePoint { x: 0.0, y: 1.0 },
        ]);
        candidate.perspective = Some(ImagePerspective {
            corners: std::array::from_fn(|index| ImagePoint {
                x: previous[index].x + (corners[index].x - previous[index].x) * fraction,
                y: previous[index].y + (corners[index].y - previous[index].y) * fraction,
            }),
        });
    }
    candidate
}

fn alignment_score(
    boundary: &ImageBoundaryDetection,
    aspect_ratio: f32,
    suggestion: &AlignmentSuggestion,
) -> f32 {
    let fine = suggestion.measurement == "fine";
    if fine && !fitted_edges_are_stable(boundary) {
        return f32::INFINITY;
    }
    let Some(offsets) = edge_offsets(boundary, aspect_ratio, fine) else {
        return f32::INFINITY;
    };
    let common_tilt = offsets.iter().sum::<f32>() / 4.0;
    if suggestion.kind == "rotation" {
        return (offsets.iter().map(|value| value * value).sum::<f32>() / 4.0).sqrt();
    }
    let worst_parallelism = (offsets[0] - offsets[2])
        .abs()
        .max((offsets[1] - offsets[3]).abs());
    let worst_residual = offsets
        .iter()
        .map(|value| (value - common_tilt).abs())
        .fold(0.0, f32::max);
    worst_parallelism + worst_residual
}

#[derive(Clone, Copy)]
struct PixelRect {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

fn analyze_framing_support(
    image: &RgbaImage,
    corners: &[ImagePoint],
    limited_by_existing_crop: bool,
) -> Option<FramingSupportAnalysis> {
    if corners.len() != 4 || image.width() < 2 || image.height() < 2 {
        return None;
    }
    let width_scale = image.width().saturating_sub(1) as f32;
    let height_scale = image.height().saturating_sub(1) as f32;
    let points: Vec<_> = corners
        .iter()
        .map(|point| (point.x * width_scale, point.y * height_scale))
        .collect();
    let lengths: Vec<_> = (0..4)
        .map(|index| {
            let next = (index + 1) % 4;
            (points[next].0 - points[index].0).hypot(points[next].1 - points[index].1)
        })
        .collect();
    let item_short_edge_pixels =
        ((lengths[0] + lengths[2]) / 2.0).min((lengths[1] + lengths[3]) / 2.0);
    if !item_short_edge_pixels.is_finite() || item_short_edge_pixels < 1.0 {
        return None;
    }
    let left = points
        .iter()
        .map(|p| p.0)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .clamp(0.0, width_scale) as u32;
    let top = points
        .iter()
        .map(|p| p.1)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .clamp(0.0, height_scale) as u32;
    let right = (points
        .iter()
        .map(|p| p.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as u32)
        .saturating_add(1)
        .clamp(left + 1, image.width());
    let bottom = (points
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as u32)
        .saturating_add(1)
        .clamp(top + 1, image.height());
    let item = PixelRect {
        left,
        top,
        right,
        bottom,
    };
    let integral = invalid_alpha_integral(image);
    let stride = image.width() as usize + 1;
    let margin = |side| {
        maximum_valid_margin(&integral, stride, image.width(), image.height(), item, side) as f32
    };
    Some(FramingSupportAnalysis {
        item_short_edge_pixels,
        margins_pixels: FramingMargins {
            top: margin(0),
            right: margin(1),
            bottom: margin(2),
            left: margin(3),
        },
        max_symmetric_margin_pixels: margin(4),
        item_intersects_invalid_pixels: invalid_count(&integral, stride, item) > 0,
        limited_by_existing_crop,
    })
}

fn invalid_alpha_integral(image: &RgbaImage) -> Vec<u32> {
    let stride = image.width() as usize + 1;
    let mut integral = vec![0_u32; stride * (image.height() as usize + 1)];
    for y in 0..image.height() as usize {
        let mut row_invalid = 0_u32;
        for x in 0..image.width() as usize {
            row_invalid += u32::from(image.get_pixel(x as u32, y as u32)[3] < 250);
            integral[(y + 1) * stride + x + 1] = integral[y * stride + x + 1] + row_invalid;
        }
    }
    integral
}

fn invalid_count(integral: &[u32], stride: usize, rect: PixelRect) -> u32 {
    let (left, top, right, bottom) = (
        rect.left as usize,
        rect.top as usize,
        rect.right as usize,
        rect.bottom as usize,
    );
    integral[bottom * stride + right] + integral[top * stride + left]
        - integral[top * stride + right]
        - integral[bottom * stride + left]
}

fn maximum_valid_margin(
    integral: &[u32],
    stride: usize,
    width: u32,
    height: u32,
    item: PixelRect,
    side: u8,
) -> u32 {
    let limit = match side {
        0 => item.top,
        1 => width - item.right,
        2 => height - item.bottom,
        3 => item.left,
        _ => item
            .top
            .min(width - item.right)
            .min(height - item.bottom)
            .min(item.left),
    };
    let expanded = |margin: u32| match side {
        0 => PixelRect {
            top: item.top - margin,
            ..item
        },
        1 => PixelRect {
            right: item.right + margin,
            ..item
        },
        2 => PixelRect {
            bottom: item.bottom + margin,
            ..item
        },
        3 => PixelRect {
            left: item.left - margin,
            ..item
        },
        _ => PixelRect {
            left: item.left - margin,
            top: item.top - margin,
            right: item.right + margin,
            bottom: item.bottom + margin,
        },
    };
    let mut low = 0;
    let mut high = limit;
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if invalid_count(integral, stride, expanded(middle)) == 0 {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}

pub fn suggested_framing_crop(
    boundary: &[ImagePoint],
    support: &FramingSupportAnalysis,
    width: u32,
    height: u32,
    target_percent: f32,
) -> Option<ImageCrop> {
    if boundary.len() != 4
        || !target_percent.is_finite()
        || !(1.0..=12.0).contains(&target_percent)
        || support.item_intersects_invalid_pixels
        || support.limited_by_existing_crop
    {
        return None;
    }
    let band = support.item_short_edge_pixels * target_percent / 100.0;
    if support.max_symmetric_margin_pixels + 0.5 < band {
        return None;
    }
    let x_scale = width.saturating_sub(1).max(1) as f32;
    let y_scale = height.saturating_sub(1).max(1) as f32;
    let crop = ImageCrop {
        left: (boundary.iter().map(|p| p.x).fold(f32::INFINITY, f32::min) - band / x_scale)
            .clamp(0.0, 1.0),
        right: (boundary
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max)
            + band / x_scale)
            .clamp(0.0, 1.0),
        top: (boundary.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) - band / y_scale)
            .clamp(0.0, 1.0),
        bottom: (boundary
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max)
            + band / y_scale)
            .clamp(0.0, 1.0),
    };
    (crop.right - crop.left >= 0.01 && crop.bottom - crop.top >= 0.01).then_some(crop)
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::*;
    use crate::image_adjustments::ImageBoundaryEdgeMeasurement;

    fn mock_analysis(offset: f32, rotation: f32) -> ImageAdjustmentAnalysis {
        ImageAdjustmentAnalysis {
            boundary: ImageBoundaryDetection {
                status: ImageBoundaryDetectionStatus::Reliable,
                corners: vec![
                    ImagePoint { x: 0.2, y: 0.2 },
                    ImagePoint { x: 0.8, y: 0.2 },
                    ImagePoint { x: 0.8, y: 0.8 },
                    ImagePoint { x: 0.2, y: 0.8 },
                ],
                edges: vec![
                    ImageBoundaryEdgeMeasurement {
                        axis_deviation_degrees: offset,
                        straightness_rms: 0.0,
                        bow_deviation: 0.0,
                        coverage: 1.0,
                    };
                    4
                ],
                message: None,
            },
            next_suggestion: Some(AlignmentSuggestion {
                kind: "rotation",
                measurement: "fine",
                corners: None,
                rotation_degrees: Some(rotation),
            }),
            framing_support: None,
            framing_boundary_source: None,
            preview_aspect_ratio: 1.0,
        }
    }

    #[test]
    fn auto_alignment_backs_off_and_retains_only_improving_draft() {
        let result = auto_align_with(&ImageAdjustment::identity(), |adjustment| {
            let offset = if adjustment.rotation_degrees == 0.0 {
                2.0
            } else if adjustment.rotation_degrees == 0.5 {
                1.0
            } else {
                3.0
            };
            Ok(mock_analysis(offset, 1.0))
        })
        .unwrap();
        assert_eq!(result.adjustment.rotation_degrees, 0.5);
        assert_eq!(result.accepted_passes, 1);
    }

    #[test]
    fn auto_alignment_stops_without_improvement_or_repeated_adjustments() {
        let unchanged = auto_align_with(&ImageAdjustment::identity(), |_| {
            Ok(mock_analysis(2.0, 1.0))
        })
        .unwrap();
        assert_eq!(unchanged.adjustment, ImageAdjustment::identity());
        assert_eq!(unchanged.accepted_passes, 0);
        assert!(unchanged.stop_reason.contains("did not improve"));

        let mut initial = ImageAdjustment::identity();
        initial.rotation_degrees = 1.0;
        let repeated = auto_align_with(&initial, |_| Ok(mock_analysis(2.0, 1.0))).unwrap();
        assert_eq!(repeated.adjustment, initial);
        assert_eq!(repeated.stop_reason, "a repeated adjustment was avoided");
    }

    #[test]
    fn perspective_suggestion_abstains_when_corners_are_missing_or_unchanged() {
        let adjustment = ImageAdjustment::identity();
        let edge = |degrees| ImageBoundaryEdgeMeasurement {
            axis_deviation_degrees: degrees,
            straightness_rms: 0.0,
            bow_deviation: 0.0,
            coverage: 1.0,
        };
        let mut boundary = ImageBoundaryDetection {
            status: ImageBoundaryDetectionStatus::Reliable,
            corners: current_perspective_corners(&adjustment).to_vec(),
            edges: vec![edge(1.0), edge(0.0), edge(-1.0), edge(0.0)],
            message: None,
        };
        assert!(alignment_needs_correction(&boundary, 1.0));
        assert!(next_alignment_suggestion(&boundary, &adjustment, 1.0).is_none());

        boundary.corners = vec![ImagePoint { x: 0.5, y: 0.5 }; 4];
        assert!(next_alignment_suggestion(&boundary, &adjustment, 1.0).is_none());
        let result = auto_align_with(&adjustment, |_| {
            Ok(ImageAdjustmentAnalysis {
                boundary: boundary.clone(),
                next_suggestion: next_alignment_suggestion(&boundary, &adjustment, 1.0),
                framing_support: None,
                framing_boundary_source: None,
                preview_aspect_ratio: 1.0,
            })
        })
        .unwrap();
        assert_eq!(result.adjustment, adjustment);
        assert_eq!(result.accepted_passes, 0);
        assert_eq!(
            result.stop_reason,
            "no valid improving adjustment can be proposed; retained the last reliable result"
        );
    }

    #[test]
    fn framing_support_counts_only_photographed_opaque_pixels() {
        let mut image = RgbaImage::from_pixel(40, 40, Rgba([255, 255, 255, 0]));
        for y in 2..38 {
            for x in 2..38 {
                image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        let boundary = [
            ImagePoint { x: 0.25, y: 0.25 },
            ImagePoint { x: 0.75, y: 0.25 },
            ImagePoint { x: 0.75, y: 0.75 },
            ImagePoint { x: 0.25, y: 0.75 },
        ];
        let support = analyze_framing_support(&image, &boundary, false).unwrap();
        assert!(!support.item_intersects_invalid_pixels);
        assert!(support.max_symmetric_margin_pixels >= 6.0);
        assert!(support.max_symmetric_margin_pixels <= 8.0);
        assert!(suggested_framing_crop(&boundary, &support, 40, 40, 4.0).is_some());
        let limited = analyze_framing_support(&image, &boundary, true).unwrap();
        assert!(suggested_framing_crop(&boundary, &limited, 40, 40, 4.0).is_none());
    }
}
