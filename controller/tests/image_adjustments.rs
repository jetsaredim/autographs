use autographs_controller::{
    derivatives::{DerivativeVariant, generate_derivative},
    image_adjustments::{
        ImageAdjustment, ImageAdjustmentProposalStatus, ImageCrop, ImagePerspective, ImagePoint,
        generate_adjusted_derivative, propose_image_adjustment,
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
fn auto_assist_returns_confident_corners_for_high_contrast_skew_fixture() {
    let source = skewed_card_fixture();
    let proposal = propose_image_adjustment(&source).unwrap();

    assert_eq!(proposal.status, ImageAdjustmentProposalStatus::Confident);
    assert_eq!(proposal.corners.len(), 4);
    for corner in proposal.corners {
        assert!((0.0..=1.0).contains(&corner.x));
        assert!((0.0..=1.0).contains(&corner.y));
    }
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

fn encode_png(image: RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    DynamicImage::ImageRgba8(image)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}
