use image::{DynamicImage, GrayImage, Rgb, RgbImage};
use imageproc::{
    drawing::draw_line_segment_mut,
    edges::canny,
    hough::{LineDetectionOptions, PolarLine, detect_lines, intersection_points},
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, env, fs, path::Path};

const EDGE_LOW: f32 = 20.0;
const EDGE_HIGH: f32 = 60.0;
const EDGE_CORRIDOR: i32 = 2;
const MAX_LINES_PER_AXIS: usize = 12;
const MIN_LINE_SUPPORT: f32 = 0.25;
const MAX_LINE_GAP: f32 = 0.55;
const MIN_PAIR_SPAN: f32 = 0.50;
const MIN_MATERIAL_AREA_GROWTH: f32 = 0.05;
const MIN_SIDE_SUPPORT: f32 = 0.35;
const MAX_SIDE_GAP: f32 = 0.40;
const MIN_SIDE_CONTRAST: f32 = 5.0;
const MIN_SIGN_COHERENCE: f32 = 0.60;

#[derive(Deserialize)]
struct Report {
    images: Vec<ImageReport>,
}

#[derive(Deserialize)]
struct ImageReport {
    label: String,
    width: u32,
    height: u32,
    primary: Primary,
}

#[derive(Deserialize)]
struct Primary {
    corners: [(u32, u32); 4],
    quad_area_ratio: f32,
}

#[derive(Clone, Copy, Serialize)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct AxisLine {
    line: PolarLine,
    position: f32,
    support: f32,
    longest_gap: f32,
}

#[derive(Clone, Serialize)]
struct SideEvidence {
    support: f32,
    longest_gap: f32,
    mean_contrast: f32,
    sign_coherence: f32,
    usable_contrast_samples: usize,
}

#[derive(Clone, Serialize)]
struct Candidate {
    corners: [Point; 4],
    area_ratio: f32,
    sides: [SideEvidence; 4],
    valid_sides: usize,
    score: f32,
}

#[derive(Serialize)]
struct Evaluation {
    label: String,
    primary_area_ratio: f32,
    primary_corners: [(u32, u32); 4],
    vertical_lines: usize,
    horizontal_lines: usize,
    larger_candidate: Option<Candidate>,
    decision: &'static str,
}

fn main() -> Result<(), String> {
    let report_path = env::args()
        .nth(1)
        .ok_or("usage: local-gradient-rectangle-spike REPORT_JSON OUTPUT_DIRECTORY")?;
    let output = env::args()
        .nth(2)
        .ok_or("usage: local-gradient-rectangle-spike REPORT_JSON OUTPUT_DIRECTORY")?;
    fs::create_dir_all(&output).map_err(|error| format!("create output: {error}"))?;
    let report: Report = serde_json::from_slice(
        &fs::read(&report_path).map_err(|error| format!("read report: {error}"))?,
    )
    .map_err(|error| format!("parse report: {error}"))?;
    let base = Path::new(&report_path)
        .parent()
        .ok_or("report path has no parent")?;

    let mut evaluations = Vec::new();
    for image_report in report.images {
        let source = image::open(base.join(&image_report.label).join("source.png"))
            .map_err(|error| format!("open {} source: {error}", image_report.label))?
            .to_rgb8();
        if source.dimensions() != (image_report.width, image_report.height) {
            return Err(format!(
                "{} dimensions differ from report",
                image_report.label
            ));
        }
        let evaluation = evaluate(&source, &image_report);
        render_overlay(&source, &evaluation)
            .save(Path::new(&output).join(format!("{}-best.png", image_report.label)))
            .map_err(|error| format!("write overlay: {error}"))?;
        evaluations.push(evaluation);
    }
    let encoded = serde_json::to_vec_pretty(&evaluations)
        .map_err(|error| format!("serialize evaluations: {error}"))?;
    fs::write(Path::new(&output).join("evaluation.json"), &encoded)
        .map_err(|error| format!("write evaluation: {error}"))?;
    println!(
        "{}",
        String::from_utf8(encoded).map_err(|error| format!("render evaluation: {error}"))?
    );
    Ok(())
}

fn evaluate(source: &RgbImage, report: &ImageReport) -> Evaluation {
    let gray = DynamicImage::ImageRgb8(source.clone()).to_luma8();
    let edges = canny(&gray, EDGE_LOW, EDGE_HIGH);
    let lines = detect_lines(
        &edges,
        LineDetectionOptions {
            vote_threshold: source.width().min(source.height()).max(1) / 3,
            suppression_radius: 8,
        },
    );
    let vertical = select_axis_lines(&edges, &lines, true);
    let horizontal = select_axis_lines(&edges, &lines, false);
    let mut candidates = Vec::new();
    for left_index in 0..vertical.len() {
        for right_index in left_index + 1..vertical.len() {
            let left = &vertical[left_index];
            let right = &vertical[right_index];
            if (right.position - left.position) / (source.width() as f32) < MIN_PAIR_SPAN {
                continue;
            }
            for top_index in 0..horizontal.len() {
                for bottom_index in top_index + 1..horizontal.len() {
                    let top = &horizontal[top_index];
                    let bottom = &horizontal[bottom_index];
                    if (bottom.position - top.position) / (source.height() as f32) < MIN_PAIR_SPAN {
                        continue;
                    }
                    let Some(corners) = quad_intersections(left, right, top, bottom) else {
                        continue;
                    };
                    if !corners_in_bounds(corners, source.width(), source.height()) {
                        continue;
                    }
                    let area_ratio = polygon_area(corners)
                        / (source.width().max(1) * source.height().max(1)) as f32;
                    if area_ratio
                        <= (report.primary.quad_area_ratio + MIN_MATERIAL_AREA_GROWTH).max(0.5)
                    {
                        continue;
                    }
                    let center = Point {
                        x: corners.iter().map(|point| point.x).sum::<f32>() / 4.0,
                        y: corners.iter().map(|point| point.y).sum::<f32>() / 4.0,
                    };
                    let sides = [
                        side_evidence(&gray, &edges, corners[0], corners[1], center),
                        side_evidence(&gray, &edges, corners[1], corners[2], center),
                        side_evidence(&gray, &edges, corners[2], corners[3], center),
                        side_evidence(&gray, &edges, corners[3], corners[0], center),
                    ];
                    let valid_sides = sides
                        .iter()
                        .filter(|side| {
                            side.support >= MIN_SIDE_SUPPORT
                                && side.longest_gap <= MAX_SIDE_GAP
                                && side.mean_contrast >= MIN_SIDE_CONTRAST
                                && side.sign_coherence >= MIN_SIGN_COHERENCE
                        })
                        .count();
                    let score = area_ratio * sides.iter().map(|side| side.support).sum::<f32>()
                        / 4.0
                        * (valid_sides as f32 / 4.0);
                    candidates.push(Candidate {
                        corners,
                        area_ratio,
                        sides,
                        valid_sides,
                        score,
                    });
                }
            }
        }
    }
    candidates.sort_by(|left, right| {
        right.valid_sides.cmp(&left.valid_sides).then_with(|| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
        })
    });
    let larger_candidate = candidates
        .into_iter()
        .find(|candidate| candidate.valid_sides >= 3);
    Evaluation {
        label: report.label.clone(),
        primary_area_ratio: report.primary.quad_area_ratio,
        primary_corners: report.primary.corners,
        vertical_lines: vertical.len(),
        horizontal_lines: horizontal.len(),
        decision: if larger_candidate.is_some() {
            "unavailable-larger-gradient-boundary"
        } else {
            "keep-primary"
        },
        larger_candidate,
    }
}

fn select_axis_lines(edges: &GrayImage, lines: &[PolarLine], vertical: bool) -> Vec<AxisLine> {
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
                line_position_at_y(*line, height as f32 / 2.0)
            } else {
                line_position_at_x(*line, width as f32 / 2.0)
            }?;
            let (support, longest_gap) = full_line_support(edges, *line)?;
            (support >= MIN_LINE_SUPPORT && longest_gap <= MAX_LINE_GAP).then_some(AxisLine {
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
    selected.truncate(MAX_LINES_PER_AXIS);
    selected.sort_by(|left, right| {
        left.position
            .partial_cmp(&right.position)
            .unwrap_or(Ordering::Equal)
    });
    selected
}

fn full_line_support(edges: &GrayImage, line: PolarLine) -> Option<(f32, f32)> {
    let (start, end) = intersection_points(line, edges.width(), edges.height())?;
    sample_edge_support(
        edges,
        Point {
            x: start.0,
            y: start.1,
        },
        Point { x: end.0, y: end.1 },
    )
}

fn quad_intersections(
    left: &AxisLine,
    right: &AxisLine,
    top: &AxisLine,
    bottom: &AxisLine,
) -> Option<[Point; 4]> {
    Some([
        intersect(left.line, top.line)?,
        intersect(right.line, top.line)?,
        intersect(right.line, bottom.line)?,
        intersect(left.line, bottom.line)?,
    ])
}

fn intersect(left: PolarLine, right: PolarLine) -> Option<Point> {
    let left_angle = (left.angle_in_degrees as f32).to_radians();
    let right_angle = (right.angle_in_degrees as f32).to_radians();
    let (left_sin, left_cos) = left_angle.sin_cos();
    let (right_sin, right_cos) = right_angle.sin_cos();
    let determinant = left_cos * right_sin - right_cos * left_sin;
    if determinant.abs() < 0.0001 {
        return None;
    }
    Some(Point {
        x: (left.r * right_sin - right.r * left_sin) / determinant,
        y: (left_cos * right.r - right_cos * left.r) / determinant,
    })
}

fn line_position_at_y(line: PolarLine, y: f32) -> Option<f32> {
    let angle = (line.angle_in_degrees as f32).to_radians();
    let (sin, cos) = angle.sin_cos();
    (cos.abs() > 0.0001).then_some((line.r - y * sin) / cos)
}

fn line_position_at_x(line: PolarLine, x: f32) -> Option<f32> {
    let angle = (line.angle_in_degrees as f32).to_radians();
    let (sin, cos) = angle.sin_cos();
    (sin.abs() > 0.0001).then_some((line.r - x * cos) / sin)
}

fn corners_in_bounds(corners: [Point; 4], width: u32, height: u32) -> bool {
    corners.iter().all(|point| {
        point.x >= -4.0
            && point.y >= -4.0
            && point.x <= width as f32 + 4.0
            && point.y <= height as f32 + 4.0
    })
}

fn polygon_area(corners: [Point; 4]) -> f32 {
    corners
        .iter()
        .zip(corners.iter().cycle().skip(1))
        .take(4)
        .map(|(left, right)| left.x * right.y - right.x * left.y)
        .sum::<f32>()
        .abs()
        / 2.0
}

fn sample_edge_support(edges: &GrayImage, start: Point, end: Point) -> Option<(f32, f32)> {
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
        if edge_near(edges, x.round() as i32, y.round() as i32) {
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

fn edge_near(edges: &GrayImage, x: i32, y: i32) -> bool {
    (-EDGE_CORRIDOR..=EDGE_CORRIDOR).any(|offset_y| {
        (-EDGE_CORRIDOR..=EDGE_CORRIDOR).any(|offset_x| {
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

fn side_evidence(
    gray: &GrayImage,
    edges: &GrayImage,
    start: Point,
    end: Point,
    center: Point,
) -> SideEvidence {
    let (support, longest_gap) = sample_edge_support(edges, start, end).unwrap_or((0.0, 1.0));
    let length = (end.x - start.x).hypot(end.y - start.y);
    let samples = length.ceil() as usize + 1;
    let middle = Point {
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
        let inside = sample_gray(gray, x + normal_x * 4.0, y + normal_y * 4.0);
        let outside = sample_gray(gray, x - normal_x * 4.0, y - normal_y * 4.0);
        if let (Some(inside), Some(outside)) = (inside, outside) {
            differences.push(inside - outside);
        }
    }
    let mean_contrast =
        differences.iter().map(|value| value.abs()).sum::<f32>() / differences.len().max(1) as f32;
    let positive = differences.iter().filter(|value| **value >= 0.0).count();
    let negative = differences.len().saturating_sub(positive);
    let sign_coherence = positive.max(negative) as f32 / differences.len().max(1) as f32;
    SideEvidence {
        support,
        longest_gap,
        mean_contrast,
        sign_coherence,
        usable_contrast_samples: differences.len(),
    }
}

fn sample_gray(gray: &GrayImage, x: f32, y: f32) -> Option<f32> {
    let x = x.round() as i32;
    let y = y.round() as i32;
    (x >= 0 && y >= 0 && x < gray.width() as i32 && y < gray.height() as i32)
        .then(|| gray.get_pixel(x as u32, y as u32).0[0] as f32)
}

fn render_overlay(source: &RgbImage, evaluation: &Evaluation) -> RgbImage {
    let mut overlay = source.clone();
    for (left, right) in evaluation
        .primary_corners
        .iter()
        .zip(evaluation.primary_corners.iter().cycle().skip(1))
        .take(4)
    {
        draw_line_segment_mut(
            &mut overlay,
            (left.0 as f32, left.1 as f32),
            (right.0 as f32, right.1 as f32),
            Rgb([255, 0, 255]),
        );
    }
    if let Some(candidate) = &evaluation.larger_candidate {
        for (left, right) in candidate
            .corners
            .iter()
            .zip(candidate.corners.iter().cycle().skip(1))
            .take(4)
        {
            draw_line_segment_mut(
                &mut overlay,
                (left.x, left.y),
                (right.x, right.y),
                Rgb([0, 255, 255]),
            );
        }
    }
    overlay
}
