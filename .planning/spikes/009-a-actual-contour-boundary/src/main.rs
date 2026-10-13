use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Deserialize)]
struct Report {
    images: Vec<ImageReport>,
}

#[derive(Deserialize)]
struct ImageReport {
    label: String,
    width: u32,
    height: u32,
    primary: Component,
    probes: Vec<Probe>,
}

#[derive(Deserialize)]
struct Probe {
    threshold: u32,
    component: Component,
}

#[derive(Deserialize)]
struct Component {
    corners: [(u32, u32); 4],
    quad_area_ratio: f32,
    bbox: [u32; 4],
    contour_points: usize,
}

#[derive(Serialize)]
struct Evaluation {
    label: String,
    primary_corners: [(u32, u32); 4],
    first_flood_threshold: Option<u32>,
    usable_outer_contour_after_primary: bool,
    ignore_flood: &'static str,
    abstain_on_any_flood: &'static str,
    conclusion: &'static str,
}

fn main() -> Result<(), String> {
    let report_path = env::args()
        .nth(1)
        .ok_or("usage: actual-contour-boundary-spike /tmp/.../report.json")?;
    let report: Report = serde_json::from_slice(
        &fs::read(&report_path).map_err(|error| format!("read report: {error}"))?,
    )
    .map_err(|error| format!("parse report: {error}"))?;

    let evaluations = report
        .images
        .into_iter()
        .map(|image| {
            let flood = image.probes.iter().find(|probe| {
                let [left, top, right, bottom] = probe.component.bbox;
                left == 0
                    && top == 0
                    && right + 1 == image.width
                    && bottom + 1 == image.height
                    && probe.component.quad_area_ratio > 0.9
            });
            let usable_outer_contour_after_primary = image.probes.iter().any(|probe| {
                probe.component.quad_area_ratio > image.primary.quad_area_ratio + 0.05
                    && probe.component.contour_points > 0
                    && Some(probe.threshold) != flood.map(|candidate| candidate.threshold)
            });
            Evaluation {
                label: image.label,
                primary_corners: image.primary.corners,
                first_flood_threshold: flood.map(|probe| probe.threshold),
                usable_outer_contour_after_primary,
                ignore_flood: "confident-primary",
                abstain_on_any_flood: if flood.is_some() {
                    "unavailable"
                } else {
                    "confident-primary"
                },
                conclusion: if usable_outer_contour_after_primary {
                    "contour-candidate-exists"
                } else {
                    "no-contour-only-rule-can-recover-an-outer-quad"
                },
            }
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string_pretty(&evaluations)
            .map_err(|error| format!("serialize evaluation: {error}"))?
    );
    Ok(())
}
