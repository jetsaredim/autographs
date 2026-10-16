use autographs_controller::{
    derivatives::{DerivativeVariant, generate_derivative},
    image_adjustments::{
        ImageAdjustment, ImageAdjustmentProposalStatus, ImageBoundaryDetectionStatus, ImageCrop,
        ImagePerspective, ImagePoint, detect_image_boundary, generate_adjusted_derivative,
        propose_image_adjustment,
    },
};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

#[test]
fn identity_adjustment_validates_and_uses_stable_cache_key() {
    let adjustment = ImageAdjustment::identity();

    adjustment.validate().unwrap();
    assert!(adjustment.is_identity());
    assert_eq!(adjustment.canonical_cache_key().unwrap(), "adjustment:none");

    let round_trip = ImageAdjustment::from_json(&adjustment.to_json().unwrap()).unwrap();
    assert_eq!(round_trip, adjustment);
}

#[test]
fn adjustment_validation_rejects_unbounded_values() {
    let mut rotation = ImageAdjustment::identity();
    rotation.rotation_degrees = 15.1;
    assert!(rotation.validate().unwrap_err().contains("rotation"));

    let mut zoom = ImageAdjustment::identity();
    zoom.zoom = 3.01;
    assert!(zoom.validate().unwrap_err().contains("zoom"));

    let mut pan = ImageAdjustment::identity();
    pan.pan_x = -1.01;
    assert!(pan.validate().unwrap_err().contains("pan"));

    let mut crop = ImageAdjustment::identity();
    crop.crop = Some(ImageCrop {
        left: 0.2,
        top: 0.2,
        right: 0.1,
        bottom: 0.8,
    });
    assert!(crop.validate().unwrap_err().contains("crop"));

    let mut perspective = ImageAdjustment::identity();
    perspective.perspective = Some(ImagePerspective {
        corners: [
            ImagePoint { x: 0.0, y: 0.0 },
            ImagePoint { x: 1.2, y: 0.0 },
            ImagePoint { x: 1.0, y: 1.0 },
            ImagePoint { x: 0.0, y: 1.0 },
        ],
    });
    assert!(perspective.validate().unwrap_err().contains("perspective"));
}

#[test]
fn adjustment_validation_rejects_degenerate_perspective_corners() {
    let mut duplicate = ImageAdjustment::identity();
    duplicate.perspective = Some(ImagePerspective {
        corners: [
            ImagePoint { x: 0.1, y: 0.1 },
            ImagePoint { x: 0.1, y: 0.1 },
            ImagePoint { x: 0.9, y: 0.9 },
            ImagePoint { x: 0.1, y: 0.9 },
        ],
    });
    assert!(duplicate.validate().unwrap_err().contains("projection"));

    let mut collinear = ImageAdjustment::identity();
    collinear.perspective = Some(ImagePerspective {
        corners: [
            ImagePoint { x: 0.1, y: 0.1 },
            ImagePoint { x: 0.5, y: 0.1 },
            ImagePoint { x: 0.9, y: 0.1 },
            ImagePoint { x: 0.1, y: 0.9 },
        ],
    });
    assert!(collinear.validate().unwrap_err().contains("projection"));
}

#[test]
fn adjusted_derivative_generates_webp_without_changing_plain_derivative() {
    let source = fixture_png([220, 20, 20], 96, 72);
    let plain = generate_derivative(&source, DerivativeVariant::Thumbnail).unwrap();
    let adjustment = ImageAdjustment {
        rotation_degrees: 4.0,
        zoom: 1.15,
        pan_x: 0.08,
        pan_y: -0.05,
        crop: Some(ImageCrop {
            left: 0.05,
            top: 0.05,
            right: 0.95,
            bottom: 0.95,
        }),
        perspective: None,
    };

    let adjusted =
        generate_adjusted_derivative(&source, DerivativeVariant::Thumbnail, Some(&adjustment))
            .unwrap();
    let plain_after = generate_derivative(&source, DerivativeVariant::Thumbnail).unwrap();

    assert_eq!(adjusted.variant, DerivativeVariant::Thumbnail);
    assert_eq!(adjusted.content_type, "image/webp");
    assert!(!adjusted.bytes.is_empty());
    assert_eq!(plain.bytes, plain_after.bytes);
    assert_ne!(plain.bytes, adjusted.bytes);
}

#[test]
fn perspective_adjustment_maps_skewed_source_corners_to_output_rectangle() {
    let width = 64;
    let height = 64;
    let source_corners = [(16, 8), (49, 14), (43, 54), (10, 47)];
    let corner_colors = [
        Rgba([240, 10, 10, 255]),
        Rgba([10, 220, 10, 255]),
        Rgba([10, 10, 230, 255]),
        Rgba([230, 220, 10, 255]),
    ];
    let mut image = RgbaImage::new(width, height);
    for (corner, color) in source_corners.into_iter().zip(corner_colors) {
        image.put_pixel(corner.0, corner.1, color);
    }

    let adjustment = ImageAdjustment {
        perspective: Some(ImagePerspective {
            corners: source_corners.map(|(x, y)| normalized_test_point(x, y, width, height)),
        }),
        ..ImageAdjustment::identity()
    };
    let adjusted = autographs_controller::image_adjustments::apply_image_adjustment(
        DynamicImage::ImageRgba8(image),
        &adjustment,
    )
    .unwrap()
    .to_rgba8();

    assert_pixel_close(*adjusted.get_pixel(0, 0), corner_colors[0]);
    assert_pixel_close(*adjusted.get_pixel(width - 1, 0), corner_colors[1]);
    assert_pixel_close(*adjusted.get_pixel(width - 1, height - 1), corner_colors[2]);
    assert_pixel_close(*adjusted.get_pixel(0, height - 1), corner_colors[3]);
}

#[test]
fn auto_assist_returns_confident_corners_for_high_contrast_skew_fixture() {
    let source = skewed_card_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Confident);
    assert_eq!(proposal.corners.len(), 4);
    for corner in &proposal.corners {
        assert!((0.0..=1.0).contains(&corner.x));
        assert!((0.0..=1.0).contains(&corner.y));
    }
    assert_detected_corners_near(
        &proposal.corners,
        &[(26, 18), (91, 18), (96, 73), (20, 73)],
        120,
        90,
        0.025,
    );
}

#[test]
fn auto_assist_finds_dark_card_against_light_background() {
    let source = dark_card_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Confident);
    assert_detected_corners_near(
        &proposal.corners,
        &[(34, 20), (125, 20), (128, 104), (30, 104)],
        160,
        120,
        0.025,
    );
}

#[test]
fn auto_assist_abstains_when_low_contrast_card_encloses_dark_content() {
    let source = low_contrast_physical_slab_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Unavailable);
    assert!(proposal.corners.is_empty());
}

#[test]
fn auto_assist_abstains_when_low_contrast_skewed_slab_encloses_dark_content() {
    let source = low_contrast_skewed_slab_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Unavailable);
    assert!(proposal.corners.is_empty());
}

#[test]
fn auto_assist_abstains_when_gradient_rectangle_encloses_stable_internal_primary() {
    let source = low_contrast_physical_slab_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Unavailable);
    assert!(proposal.corners.is_empty());
}

#[test]
fn auto_assist_abstains_when_border_noise_hides_low_contrast_enclosure() {
    let source = noisy_border_low_contrast_card_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Unavailable);
    assert!(proposal.corners.is_empty());
}

#[test]
fn auto_assist_abstains_when_filled_slab_encloses_compact_dark_content() {
    let source = filled_slab_with_compact_dark_content_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Unavailable);
    assert!(proposal.corners.is_empty());
}

#[test]
fn auto_assist_keeps_outer_card_when_its_border_has_reliable_contrast() {
    let source = bordered_card_with_internal_content_fixture([190, 190, 190]);
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Confident);
    assert_detected_corners_near(
        &proposal.corners,
        &[(18, 10), (161, 10), (161, 129), (18, 129)],
        180,
        140,
        0.025,
    );
}

#[test]
fn auto_assist_keeps_accurate_primary_when_hough_lines_lack_localized_coherent_contrast() {
    let source = hough_rich_accurate_card_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Confident);
    assert_detected_corners_near(
        &proposal.corners,
        &[(35, 30), (144, 30), (144, 109), (35, 109)],
        180,
        140,
        0.025,
    );
}

#[test]
fn auto_assist_returns_unavailable_copy_for_low_confidence_fixture() {
    let source = fixture_png([128, 128, 128], 96, 72);
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Unavailable);
    assert_eq!(
        proposal.message.as_deref(),
        Some("Auto correction could not find reliable edges. Adjust the corners manually.")
    );
    assert!(proposal.corners.is_empty());
}

#[test]
fn auto_assist_reports_when_no_meaningful_correction_is_needed() {
    let source = nearly_full_frame_card_fixture(2);
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::NotNeeded);
    assert_eq!(
        proposal.message.as_deref(),
        Some("No meaningful perspective correction was detected.")
    );
    assert!(proposal.corners.is_empty());
    assert_eq!(
        serde_json::to_value(&proposal).unwrap()["status"],
        "notNeeded"
    );
}

#[test]
fn boundary_detection_returns_near_frame_corners_even_when_correction_is_not_needed() {
    let source = nearly_full_frame_card_fixture(2);
    let boundary = detect_image_boundary(&source).unwrap();

    assert_eq!(boundary.status, ImageBoundaryDetectionStatus::Reliable);
    assert_detected_corners_near(
        &boundary.corners,
        &[(2, 2), (157, 2), (157, 117), (2, 117)],
        160,
        120,
        0.025,
    );
}

#[test]
fn boundary_detection_exposes_outer_gradient_boundary_without_weakening_auto_assist() {
    let source = low_contrast_physical_slab_fixture();
    let boundary = detect_image_boundary(&source).unwrap();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(boundary.status, ImageBoundaryDetectionStatus::Reliable);
    assert_eq!(boundary.corners.len(), 4);
    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Unavailable);
    assert!(proposal.corners.is_empty());
}

#[test]
fn boundary_detection_remains_unavailable_without_a_contrast_component() {
    let source = fixture_png([128, 128, 128], 96, 72);
    let boundary = detect_image_boundary(&source).unwrap();

    assert_eq!(boundary.status, ImageBoundaryDetectionStatus::Unavailable);
    assert!(boundary.corners.is_empty());
}

#[test]
fn boundary_detection_marks_a_small_contrast_candidate_tentative() {
    let mut image = RgbaImage::from_pixel(100, 80, Rgba([190, 190, 190, 255]));
    for y in 28..52 {
        for x in 35..65 {
            image.put_pixel(x, y, Rgba([24, 28, 32, 255]));
        }
    }
    let boundary = detect_image_boundary(&encode_png(image)).unwrap();

    assert_eq!(boundary.status, ImageBoundaryDetectionStatus::Tentative);
    assert_eq!(boundary.corners.len(), 4);
}

#[test]
fn auto_assist_keeps_correction_just_outside_identity_tolerance() {
    let source = nearly_full_frame_card_fixture(5);
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Confident);
    assert_eq!(proposal.corners.len(), 4);
}

fn fixture_png(color: [u8; 3], width: u32, height: u32) -> Vec<u8> {
    let mut image = RgbaImage::new(width, height);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([color[0], color[1], color[2], 255]);
    }
    encode_png(image)
}

fn skewed_card_fixture() -> Vec<u8> {
    let mut image = RgbaImage::new(120, 90);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([12, 12, 12, 255]);
    }
    for y in 18..74 {
        let left = 26i32 - ((y as i32 - 18) / 9);
        let right = 92i32 + ((y as i32 - 18) / 11);
        for x in left..right {
            if x >= 0 && x < image.width() as i32 {
                image.put_pixel(x as u32, y, Rgba([245, 245, 245, 255]));
            }
        }
    }
    encode_png(image)
}

fn dark_card_fixture() -> Vec<u8> {
    let mut image = RgbaImage::new(160, 120);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([238, 238, 238, 255]);
    }
    for y in 20..105 {
        let left = 34i32 - ((y as i32 - 20) / 21);
        let right = 126i32 + ((y as i32 - 20) / 28);
        for x in left..right {
            image.put_pixel(x as u32, y, Rgba([24, 28, 32, 255]));
        }
    }
    encode_png(image)
}

fn bordered_card_with_internal_content_fixture(card_color: [u8; 3]) -> Vec<u8> {
    let mut image = RgbaImage::new(180, 140);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([238, 238, 238, 255]);
    }
    for y in 10..130 {
        for x in 18..162 {
            image.put_pixel(
                x,
                y,
                Rgba([card_color[0], card_color[1], card_color[2], 255]),
            );
        }
    }
    for y in 30..110 {
        for x in 44..136 {
            image.put_pixel(x, y, Rgba([28, 32, 36, 255]));
        }
    }
    encode_png(image)
}

fn low_contrast_physical_slab_fixture() -> Vec<u8> {
    let mut image = RgbaImage::new(180, 140);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([242, 242, 242, 255]);
    }
    for y in 10..130 {
        for x in 18..162 {
            image.put_pixel(x, y, Rgba([210, 210, 210, 255]));
        }
    }
    for y in 30..110 {
        for x in 44..136 {
            image.put_pixel(x, y, Rgba([28, 32, 36, 255]));
        }
    }
    add_adaptive_threshold_border_noise(&mut image);
    encode_png(image)
}

fn low_contrast_skewed_slab_fixture() -> Vec<u8> {
    let mut image = RgbaImage::new(180, 140);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([242, 242, 242, 255]);
    }
    for y in 10..130 {
        let shift = (((y - 10) as f32) * 0.08).round() as u32;
        for x in 12 + shift..156 + shift {
            image.put_pixel(x, y, Rgba([210, 210, 210, 255]));
        }
    }
    for y in 35..105 {
        let shift = (((y - 35) as f32) * 0.08).round() as u32;
        for x in 45 + shift..135 + shift {
            image.put_pixel(x, y, Rgba([28, 32, 36, 255]));
        }
    }
    add_adaptive_threshold_border_noise(&mut image);
    encode_png(image)
}

fn noisy_border_low_contrast_card_fixture() -> Vec<u8> {
    let mut image = RgbaImage::new(180, 140);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([242, 242, 242, 255]);
    }
    for y in 10..130 {
        for x in 18..162 {
            image.put_pixel(x, y, Rgba([210, 210, 210, 255]));
        }
    }
    for y in 30..110 {
        for x in 44..136 {
            image.put_pixel(x, y, Rgba([28, 32, 36, 255]));
        }
    }
    for x in (0..image.width()).step_by(4) {
        image.put_pixel(x, 0, Rgba([176, 176, 176, 255]));
        image.put_pixel(x, image.height() - 1, Rgba([176, 176, 176, 255]));
    }
    for y in (1..image.height() - 1).step_by(4) {
        image.put_pixel(0, y, Rgba([176, 176, 176, 255]));
        image.put_pixel(image.width() - 1, y, Rgba([176, 176, 176, 255]));
    }
    encode_png(image)
}

fn filled_slab_with_compact_dark_content_fixture() -> Vec<u8> {
    let mut image = RgbaImage::new(180, 140);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([242, 242, 242, 255]);
    }

    for y in 10..130 {
        for x in 18..162 {
            image.put_pixel(x, y, Rgba([210, 210, 210, 255]));
        }
    }

    for y in 40..100 {
        for x in 50..130 {
            image.put_pixel(x, y, Rgba([28, 32, 36, 255]));
        }
    }
    add_adaptive_threshold_border_noise(&mut image);
    encode_png(image)
}

fn add_adaptive_threshold_border_noise(image: &mut RgbaImage) {
    for x in (0..image.width()).step_by(4) {
        image.put_pixel(x, 0, Rgba([176, 176, 176, 255]));
        image.put_pixel(x, image.height() - 1, Rgba([176, 176, 176, 255]));
    }
    for y in (1..image.height() - 1).step_by(4) {
        image.put_pixel(0, y, Rgba([176, 176, 176, 255]));
        image.put_pixel(image.width() - 1, y, Rgba([176, 176, 176, 255]));
    }
}

fn hough_rich_accurate_card_fixture() -> Vec<u8> {
    let mut image = RgbaImage::new(180, 140);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([238, 238, 238, 255]);
    }
    for y in 30..110 {
        for x in 35..145 {
            image.put_pixel(x, y, Rgba([40, 44, 48, 255]));
        }
    }
    for x in 10..170 {
        let color = if (x / 8) % 2 == 0 { 80 } else { 250 };
        image.put_pixel(x, 8, Rgba([color, color, color, 255]));
        image.put_pixel(x, 131, Rgba([color, color, color, 255]));
    }
    for y in 8..132 {
        let color = if (y / 8) % 2 == 0 { 80 } else { 250 };
        image.put_pixel(10, y, Rgba([color, color, color, 255]));
        image.put_pixel(169, y, Rgba([color, color, color, 255]));
    }
    encode_png(image)
}

fn nearly_full_frame_card_fixture(inset: u32) -> Vec<u8> {
    let mut image = RgbaImage::new(160, 120);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([238, 238, 238, 255]);
    }
    for y in inset..image.height() - inset {
        for x in inset..image.width() - inset {
            image.put_pixel(x, y, Rgba([24, 28, 32, 255]));
        }
    }
    encode_png(image)
}

fn encode_png(image: RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    DynamicImage::ImageRgba8(image)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}

fn normalized_test_point(x: u32, y: u32, width: u32, height: u32) -> ImagePoint {
    ImagePoint {
        x: x as f32 / width.saturating_sub(1).max(1) as f32,
        y: y as f32 / height.saturating_sub(1).max(1) as f32,
    }
}

fn assert_detected_corners_near(
    actual: &[ImagePoint],
    expected_pixels: &[(u32, u32); 4],
    width: u32,
    height: u32,
    tolerance: f32,
) {
    assert_eq!(actual.len(), expected_pixels.len());
    for (actual, &(x, y)) in actual.iter().zip(expected_pixels) {
        let expected = normalized_test_point(x, y, width, height);
        assert!(
            (actual.x - expected.x).abs() <= tolerance
                && (actual.y - expected.y).abs() <= tolerance,
            "expected corner near ({:.3}, {:.3}), got ({:.3}, {:.3})",
            expected.x,
            expected.y,
            actual.x,
            actual.y,
        );
    }
}

fn assert_pixel_close(actual: Rgba<u8>, expected: Rgba<u8>) {
    for (actual, expected) in actual.0.into_iter().zip(expected.0) {
        assert!(
            actual.abs_diff(expected) <= 2,
            "expected pixel channel near {expected}, got {actual}"
        );
    }
}
