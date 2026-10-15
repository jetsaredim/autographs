use image::{DynamicImage, GrayImage, Rgb, RgbImage};
use imageproc::{
    contours::{BorderType, find_contours},
    drawing::{draw_line_segment_mut, draw_polygon_mut},
    edges::canny,
    hough::{LineDetectionOptions, detect_lines, draw_polar_lines},
    point::Point,
};
use serde::Serialize;
use std::{
    collections::VecDeque,
    env, fs,
    path::{Path, PathBuf},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const MAX_DIMENSION: u32 = 640;
const MIN_COLOR_DISTANCE: u32 = 32;
const MIN_STABILITY_THRESHOLD: u32 = 8;

#[derive(Serialize)]
struct Report {
    generated_at: String,
    images: Vec<ImageReport>,
    events: Vec<Event>,
}

#[derive(Serialize)]
struct Event {
    timestamp: String,
    category: &'static str,
    detail: String,
}

#[derive(Serialize)]
struct ImageReport {
    label: String,
    width: u32,
    height: u32,
    background: [u8; 3],
    primary_threshold: u32,
    primary: ComponentMetrics,
    probes: Vec<ProbeMetrics>,
    edge_sets: Vec<EdgeMetrics>,
}

#[derive(Serialize)]
struct ProbeMetrics {
    threshold: u32,
    retains_primary: bool,
    old_quad_contains_primary: bool,
    area_growth: f32,
    component: ComponentMetrics,
}

#[derive(Serialize)]
struct ComponentMetrics {
    count: usize,
    corners: [(u32, u32); 4],
    quad_area_ratio: f32,
    bbox: [u32; 4],
    bbox_density: f32,
    contour_points: usize,
    contour_perimeter_ratio: f32,
}

#[derive(Serialize)]
struct EdgeMetrics {
    low: f32,
    high: f32,
    edge_pixels: usize,
    hough_lines: usize,
}

struct Component {
    anchor: usize,
    members: Vec<usize>,
    corners: [(u32, u32); 4],
}

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut output = None;
    let mut compact = false;
    let mut inputs = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "--output" {
            output = args.next().map(PathBuf::from);
        } else if arg == "--compact" {
            compact = true;
        } else if let Some((label, path)) = arg.split_once('=') {
            if label.is_empty()
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            {
                return Err(format!("invalid label: {label}"));
            }
            inputs.push((label.to_owned(), PathBuf::from(path)));
        } else {
            return Err(format!("expected label=/path/to/image, got {arg}"));
        }
    }
    let output = output.ok_or("missing --output DIRECTORY")?;
    if inputs.is_empty() {
        return Err("provide at least one label=/path/to/image input".to_owned());
    }
    fs::create_dir_all(&output).map_err(|error| format!("create output directory: {error}"))?;

    let mut events = Vec::new();
    let mut images = Vec::new();
    for (label, path) in inputs {
        events.push(event("input", format!("started {label}")));
        images.push(analyze(&label, &path, &output, compact)?);
        events.push(event("input", format!("completed {label}")));
    }
    let report = Report {
        generated_at: now(),
        images,
        events,
    };
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).map_err(|error| format!("serialize report: {error}"))?,
    )
    .map_err(|error| format!("write report: {error}"))?;
    fs::write(output.join("index.html"), render_html(&report))
        .map_err(|error| format!("write gallery: {error}"))?;
    println!("{}", output.join("index.html").display());
    Ok(())
}

fn analyze(label: &str, path: &Path, output: &Path, compact: bool) -> Result<ImageReport, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {label}: {error}"))?;
    let decoded =
        image::load_from_memory(&bytes).map_err(|error| format!("decode {label}: {error}"))?;
    let source = decoded.thumbnail(MAX_DIMENSION, MAX_DIMENSION).to_rgb8();
    let (width, height) = source.dimensions();
    let label_dir = output.join(label);
    fs::create_dir_all(&label_dir).map_err(|error| format!("create {label} output: {error}"))?;
    source
        .save(label_dir.join("source.png"))
        .map_err(|error| format!("write {label} source: {error}"))?;

    let background = border_median_rgb(&source);
    let primary_threshold = adaptive_threshold(&source, background);
    let primary_foreground = foreground(&source, background, primary_threshold);
    let primary = largest_component(width, height, &primary_foreground)
        .ok_or_else(|| format!("no primary component for {label}"))?;
    let primary_mask = component_mask(width, height, &primary.members);
    if !compact {
        primary_mask
            .save(label_dir.join("primary-mask.png"))
            .map_err(|error| format!("write primary mask: {error}"))?;
    }
    let primary_contour = largest_outer_contour(&primary_mask);
    if !compact {
        render_component_overlay(&source, &primary, &primary_contour)
            .save(label_dir.join("primary-overlay.png"))
            .map_err(|error| format!("write primary overlay: {error}"))?;
    }
    let primary_metrics = component_metrics(width, height, &primary, &primary_contour);

    let mut probes = Vec::new();
    let mut previous = primary_threshold;
    for threshold in [
        primary_threshold.saturating_mul(3) / 4,
        primary_threshold / 2,
        primary_threshold / 3,
        MIN_COLOR_DISTANCE,
        MIN_COLOR_DISTANCE / 2,
        MIN_STABILITY_THRESHOLD,
    ]
    .map(|value| value.max(MIN_STABILITY_THRESHOLD))
    {
        if threshold >= previous {
            continue;
        }
        previous = threshold;
        let weak_foreground = foreground(&source, background, threshold);
        if !weak_foreground[primary.anchor] {
            continue;
        }
        let weak = trace_component(
            primary.anchor,
            width,
            height,
            &weak_foreground,
            &mut vec![false; weak_foreground.len()],
        );
        let weak_mask = component_mask(width, height, &weak.members);
        let contour = largest_outer_contour(&weak_mask);
        if !compact {
            weak_mask
                .save(label_dir.join(format!("threshold-{threshold}-mask.png")))
                .map_err(|error| format!("write threshold mask: {error}"))?;
            render_component_overlay(&source, &weak, &contour)
                .save(label_dir.join(format!("threshold-{threshold}-overlay.png")))
                .map_err(|error| format!("write threshold overlay: {error}"))?;
        }
        let metrics = component_metrics(width, height, &weak, &contour);
        probes.push(ProbeMetrics {
            threshold,
            retains_primary: primary
                .members
                .iter()
                .all(|index| weak_mask.as_raw()[*index] != 0),
            old_quad_contains_primary: primary
                .corners
                .into_iter()
                .all(|point| point_in_quad(point, weak.corners)),
            area_growth: metrics.quad_area_ratio - primary_metrics.quad_area_ratio,
            component: metrics,
        });
    }

    let mut edge_sets = Vec::new();
    if !compact {
        let gray = DynamicImage::ImageRgb8(source.clone()).to_luma8();
        for (low, high) in [(10.0, 30.0), (20.0, 60.0), (40.0, 120.0)] {
            let edges = canny(&gray, low, high);
            let lines = detect_lines(
                &edges,
                LineDetectionOptions {
                    vote_threshold: width.min(height).max(1) / 3,
                    suppression_radius: 8,
                },
            );
            edges
                .save(label_dir.join(format!("canny-{low:.0}-{high:.0}.png")))
                .map_err(|error| format!("write canny image: {error}"))?;
            draw_polar_lines(&source, &lines, Rgb([255, 0, 255]))
                .save(label_dir.join(format!("hough-{low:.0}-{high:.0}.png")))
                .map_err(|error| format!("write hough image: {error}"))?;
            edge_sets.push(EdgeMetrics {
                low,
                high,
                edge_pixels: edges.pixels().filter(|pixel| pixel.0[0] != 0).count(),
                hough_lines: lines.len(),
            });
        }
    }

    Ok(ImageReport {
        label: label.to_owned(),
        width,
        height,
        background,
        primary_threshold,
        primary: primary_metrics,
        probes,
        edge_sets,
    })
}

fn event(category: &'static str, detail: String) -> Event {
    Event {
        timestamp: now(),
        category,
        detail,
    }
}

fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_owned())
}

fn border_median_rgb(image: &RgbImage) -> [u8; 3] {
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

fn adaptive_threshold(image: &RgbImage, background: [u8; 3]) -> u32 {
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
    let noise = distances[distances.len().saturating_mul(9) / 10];
    MIN_COLOR_DISTANCE.max(noise.saturating_mul(2).saturating_add(8))
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

fn foreground(image: &RgbImage, background: [u8; 3], threshold: u32) -> Vec<bool> {
    image
        .pixels()
        .map(|pixel| color_distance(pixel.0, background) >= threshold)
        .collect()
}

fn largest_component(width: u32, height: u32, foreground: &[bool]) -> Option<Component> {
    let mut visited = vec![false; foreground.len()];
    let mut largest = None;
    for index in 0..foreground.len() {
        if visited[index] || !foreground[index] {
            continue;
        }
        let candidate = trace_component(index, width, height, foreground, &mut visited);
        if largest
            .as_ref()
            .is_none_or(|current: &Component| candidate.members.len() > current.members.len())
        {
            largest = Some(candidate);
        }
    }
    largest
}

fn trace_component(
    start: usize,
    width: u32,
    height: u32,
    foreground: &[bool],
    visited: &mut [bool],
) -> Component {
    let mut queue = VecDeque::from([start]);
    let mut members = Vec::new();
    let mut corners = [(0, 0); 4];
    let mut scores = [i64::MAX, i64::MIN, i64::MIN, i64::MAX];
    visited[start] = true;
    while let Some(index) = queue.pop_front() {
        members.push(index);
        let x = index as u32 % width;
        let y = index as u32 / width;
        let sum = i64::from(x) + i64::from(y);
        let difference = i64::from(x) - i64::from(y);
        for (corner, candidate, lower) in [
            (0, sum, true),
            (1, difference, false),
            (2, sum, false),
            (3, difference, true),
        ] {
            if (lower && candidate < scores[corner]) || (!lower && candidate > scores[corner]) {
                scores[corner] = candidate;
                corners[corner] = (x, y);
            }
        }
        for neighbor_y in y.saturating_sub(1)..=(y + 1).min(height.saturating_sub(1)) {
            for neighbor_x in x.saturating_sub(1)..=(x + 1).min(width.saturating_sub(1)) {
                let neighbor = (neighbor_y * width + neighbor_x) as usize;
                if !visited[neighbor] && foreground[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
    }
    Component {
        anchor: start,
        members,
        corners,
    }
}

fn component_mask(width: u32, height: u32, members: &[usize]) -> GrayImage {
    let mut mask = GrayImage::new(width, height);
    for index in members {
        mask.as_mut()[*index] = 255;
    }
    mask
}

fn largest_outer_contour(mask: &GrayImage) -> Vec<Point<u32>> {
    find_contours::<u32>(mask)
        .into_iter()
        .filter(|contour| contour.border_type == BorderType::Outer)
        .max_by_key(|contour| contour.points.len())
        .map(|contour| contour.points)
        .unwrap_or_default()
}

fn component_metrics(
    width: u32,
    height: u32,
    component: &Component,
    contour: &[Point<u32>],
) -> ComponentMetrics {
    let min_x = component
        .members
        .iter()
        .map(|index| *index as u32 % width)
        .min()
        .unwrap_or(0);
    let max_x = component
        .members
        .iter()
        .map(|index| *index as u32 % width)
        .max()
        .unwrap_or(0);
    let min_y = component
        .members
        .iter()
        .map(|index| *index as u32 / width)
        .min()
        .unwrap_or(0);
    let max_y = component
        .members
        .iter()
        .map(|index| *index as u32 / width)
        .max()
        .unwrap_or(0);
    let bbox_area = (max_x - min_x + 1) as usize * (max_y - min_y + 1) as usize;
    ComponentMetrics {
        count: component.members.len(),
        corners: component.corners,
        quad_area_ratio: quad_area(component.corners, width, height),
        bbox: [min_x, min_y, max_x, max_y],
        bbox_density: component.members.len() as f32 / bbox_area.max(1) as f32,
        contour_points: contour.len(),
        contour_perimeter_ratio: contour.len() as f32 / (2 * (width + height)).max(1) as f32,
    }
}

fn quad_area(corners: [(u32, u32); 4], width: u32, height: u32) -> f32 {
    corners
        .iter()
        .zip(corners.iter().cycle().skip(1))
        .take(4)
        .map(|(left, right)| {
            let lx = left.0 as f32 / width.saturating_sub(1).max(1) as f32;
            let ly = left.1 as f32 / height.saturating_sub(1).max(1) as f32;
            let rx = right.0 as f32 / width.saturating_sub(1).max(1) as f32;
            let ry = right.1 as f32 / height.saturating_sub(1).max(1) as f32;
            lx * ry - rx * ly
        })
        .sum::<f32>()
        .abs()
        / 2.0
}

fn point_in_quad(point: (u32, u32), quad: [(u32, u32); 4]) -> bool {
    let (px, py) = (point.0 as f32, point.1 as f32);
    let mut sign = 0i8;
    for (left, right) in quad.iter().zip(quad.iter().cycle().skip(1)).take(4) {
        let cross = (right.0 as f32 - left.0 as f32) * (py - left.1 as f32)
            - (right.1 as f32 - left.1 as f32) * (px - left.0 as f32);
        if cross.abs() < 0.001 {
            continue;
        }
        let next = if cross > 0.0 { 1 } else { -1 };
        if sign != 0 && sign != next {
            return false;
        }
        sign = next;
    }
    true
}

fn render_component_overlay(
    source: &RgbImage,
    component: &Component,
    contour: &[Point<u32>],
) -> RgbImage {
    let mut overlay = source.clone();
    if contour.len() >= 3 {
        let polygon = contour
            .iter()
            .map(|point| Point::new(point.x as i32, point.y as i32))
            .collect::<Vec<_>>();
        draw_polygon_mut(&mut overlay, &polygon, Rgb([0, 255, 255]));
    }
    for (left, right) in component
        .corners
        .iter()
        .zip(component.corners.iter().cycle().skip(1))
        .take(4)
    {
        draw_line_segment_mut(
            &mut overlay,
            (left.0 as f32, left.1 as f32),
            (right.0 as f32, right.1 as f32),
            Rgb([255, 0, 255]),
        );
    }
    overlay
}

fn render_html(report: &Report) -> String {
    let mut sections = String::new();
    for image in &report.images {
        let mut probes = String::new();
        for probe in &image.probes {
            probes.push_str(&format!(
                "<figure><img src=\"{0}/threshold-{1}-overlay.png\"><figcaption>threshold {1}: area {2:.3}, density {3:.3}, contour {4}</figcaption></figure>",
                image.label,
                probe.threshold,
                probe.component.quad_area_ratio,
                probe.component.bbox_density,
                probe.component.contour_points
            ));
        }
        sections.push_str(&format!(
            "<section><h2>{0}</h2><p>{1}x{2}, primary threshold {3}</p><div class=\"grid\"><figure><img src=\"{0}/source.png\"><figcaption>source thumbnail</figcaption></figure><figure><img src=\"{0}/primary-overlay.png\"><figcaption>primary: cyan contour, magenta extrema chord</figcaption></figure>{4}<figure><img src=\"{0}/canny-20-60.png\"><figcaption>Canny 20/60</figcaption></figure><figure><img src=\"{0}/hough-20-60.png\"><figcaption>Hough 20/60</figcaption></figure></div></section>",
            image.label, image.width, image.height, image.primary_threshold, probes
        ));
    }
    format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Detector diagnostics</title><style>body{{font:16px system-ui;margin:2rem;background:#16181d;color:#eee}}.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem}}figure{{margin:0;padding:1rem;background:#22252c}}img{{max-width:100%;height:auto}}figcaption{{margin-top:.5rem;color:#ccd}}code{{color:#9ee}}</style><h1>Detector mask and contour diagnostics</h1><p>Generated {}</p>{}<p>Raw redacted metrics: <a href=\"report.json\">report.json</a></p>",
        report.generated_at, sections
    )
}
