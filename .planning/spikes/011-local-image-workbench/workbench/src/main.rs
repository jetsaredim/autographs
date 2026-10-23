use std::{
    env,
    io::Cursor,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use autographs_image_processing::image_adjustments::{
    ImageAdjustment, ImageBoundaryDetectionStatus, ImagePoint, apply_image_adjustment,
    detect_image_boundary, propose_image_adjustment, validate_ordered_convex_quad,
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path as AxumPath, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use image::{ImageFormat, ImageReader, RgbaImage};
use serde::{Deserialize, Serialize};
use tokio::fs;

const INDEX_HTML: &str = include_str!("../static/index.html");
const APP_CSS: &str = include_str!("../static/app.css");
const APP_JS: &str = include_str!("../static/app.js");

#[derive(Clone)]
struct AppState {
    images: Arc<Vec<ImageEntry>>,
    workspace: Arc<PathBuf>,
    authority: Arc<str>,
    capability: Arc<str>,
}

#[derive(Clone)]
struct ImageEntry {
    name: String,
    path: PathBuf,
    fixture_name: String,
    content_type: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageSummary {
    id: usize,
    name: String,
    has_fixture: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FramingSupportAnalysis {
    item_short_edge_pixels: f32,
    margins_pixels: FramingMargins,
    max_symmetric_margin_pixels: f32,
    item_intersects_invalid_pixels: bool,
    limited_by_existing_crop: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FramingMargins {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManualFramingRequest {
    adjustment: ImageAdjustment,
    corners: Vec<ImagePoint>,
}

type ApiError = (StatusCode, String);
type ApiResult<T> = Result<T, ApiError>;

#[tokio::main]
async fn main() -> Result<(), String> {
    let config = Config::parse(env::args().skip(1))?;
    let images = discover_images(&config.source)?;
    if images.is_empty() {
        return Err("source contains no JPEG, PNG, or WebP images".to_owned());
    }
    fs::create_dir_all(&config.workspace)
        .await
        .map_err(|error| format!("create workspace {}: {error}", config.workspace.display()))?;

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.port);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| format!("bind {address}: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("inspect bound address: {error}"))?;
    let capability = format!("{:032x}", rand::random::<u128>());
    let state = AppState {
        images: Arc::new(images),
        workspace: Arc::new(config.workspace),
        authority: Arc::from(address.to_string()),
        capability: Arc::from(capability.as_str()),
    };
    let image_count = state.images.len();
    let app = app(state);
    println!("Autographs image workbench: http://{address}/?cap={capability}");
    println!("Loaded {image_count} immutable source image(s).");
    axum::serve(listener, app)
        .await
        .map_err(|error| format!("serve workbench: {error}"))
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/app.css", get(css))
        .route("/app.js", get(js))
        .route("/api/images", get(list_images))
        .route("/api/images/{id}/source", get(source_image))
        .route("/api/images/{id}/render", post(render_image))
        .route("/api/images/{id}/framing", post(manual_framing))
        .route("/api/images/{id}/detect", post(detect_edges))
        .route(
            "/api/images/{id}/fixture",
            get(load_fixture).put(save_fixture),
        )
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state, authorize_request))
}

async fn authorize_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let expected_origin = format!("http://{}", state.authority);
    let host_matches = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| host == state.authority.as_ref());
    if !host_matches {
        return StatusCode::FORBIDDEN.into_response();
    }

    let is_api = request.uri().path().starts_with("/api/");
    if is_api {
        let header_capability = request
            .headers()
            .get("x-workbench-capability")
            .and_then(|value| value.to_str().ok());
        let query_capability = request
            .uri()
            .query()
            .and_then(|query| query.split('&').find_map(|pair| pair.strip_prefix("cap=")));
        if header_capability.or(query_capability) != Some(state.capability.as_ref()) {
            return StatusCode::FORBIDDEN.into_response();
        }
        if !matches!(
            *request.method(),
            Method::GET | Method::HEAD | Method::OPTIONS
        ) {
            let origin_matches = request
                .headers()
                .get(header::ORIGIN)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|origin| origin == expected_origin);
            if !origin_matches {
                return StatusCode::FORBIDDEN.into_response();
            }
        }
    }

    let mut response = next.run(request).await;
    if is_api {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

async fn index() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        INDEX_HTML,
    )
}

async fn css() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        APP_CSS,
    )
}

async fn js() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        APP_JS,
    )
}

async fn list_images(State(state): State<AppState>) -> ApiResult<Json<Vec<ImageSummary>>> {
    let mut summaries = Vec::with_capacity(state.images.len());
    for (id, image) in state.images.iter().enumerate() {
        summaries.push(ImageSummary {
            id,
            name: image.name.clone(),
            has_fixture: fs::try_exists(fixture_path(&state, image))
                .await
                .map_err(internal("inspect fixture"))?,
        });
    }
    Ok(Json(summaries))
}

async fn source_image(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<usize>,
) -> ApiResult<Response> {
    let image = image_entry(&state, id)?;
    let bytes = fs::read(&image.path)
        .await
        .map_err(internal("read source image"))?;
    Ok(([(header::CONTENT_TYPE, image.content_type)], bytes).into_response())
}

async fn render_image(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<usize>,
    body: Bytes,
) -> ApiResult<Response> {
    let image = image_entry(&state, id)?;
    let adjustment = parse_adjustment(&body)?;
    let source = fs::read(&image.path)
        .await
        .map_err(internal("read source image"))?;
    let decoded = image::load_from_memory(&source).map_err(bad_request("decode source image"))?;
    let adjusted =
        apply_image_adjustment(decoded, &adjustment).map_err(bad_request("apply adjustment"))?;
    let mut output = Cursor::new(Vec::new());
    adjusted
        .write_to(&mut output, ImageFormat::WebP)
        .map_err(internal("encode preview"))?;
    let preview = output.into_inner();
    let analysis = detect_image_boundary(&preview).ok();
    let analysis_status = analysis
        .as_ref()
        .map(|boundary| match boundary.status {
            ImageBoundaryDetectionStatus::Reliable => "reliable",
            ImageBoundaryDetectionStatus::Tentative => "tentative",
            ImageBoundaryDetectionStatus::Unavailable => "unavailable",
        })
        .unwrap_or("error");
    let analysis_corners = serde_json::to_string(
        &analysis
            .as_ref()
            .map(|proposal| proposal.corners.as_slice())
            .unwrap_or_default(),
    )
    .map_err(internal("serialize output analysis"))?;
    let analysis_edges = serde_json::to_string(
        &analysis
            .as_ref()
            .map(|boundary| boundary.edges.as_slice())
            .unwrap_or_default(),
    )
    .map_err(internal("serialize output edge measurements"))?;
    let framing = analysis
        .as_ref()
        .filter(|boundary| boundary.corners.len() == 4)
        .and_then(|boundary| {
            analyze_framing_support(
                &adjusted.to_rgba8(),
                boundary.corners.as_slice(),
                adjustment.crop.is_some()
                    || adjustment.zoom > 1.0
                    || adjustment.pan_x != 0.0
                    || adjustment.pan_y != 0.0,
            )
        });
    let framing_json =
        serde_json::to_string(&framing).map_err(internal("serialize framing support analysis"))?;
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/webp"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        "x-workbench-boundary-status",
        HeaderValue::from_static(analysis_status),
    );
    headers.insert(
        "x-workbench-boundary-corners",
        HeaderValue::from_str(&analysis_corners).map_err(internal("encode output analysis"))?,
    );
    headers.insert(
        "x-workbench-boundary-edges",
        HeaderValue::from_str(&analysis_edges)
            .map_err(internal("encode output edge measurements"))?,
    );
    headers.insert(
        "x-workbench-framing-support",
        HeaderValue::from_str(&framing_json)
            .map_err(internal("encode framing support analysis"))?,
    );
    Ok((headers, preview).into_response())
}

async fn manual_framing(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<usize>,
    Json(request): Json<ManualFramingRequest>,
) -> ApiResult<Json<FramingSupportAnalysis>> {
    validate_ordered_convex_quad(&request.corners)
        .map_err(bad_request("validate manual framing boundary"))?;
    let image = image_entry(&state, id)?;
    let source = fs::read(&image.path)
        .await
        .map_err(internal("read source image"))?;
    let decoded = image::load_from_memory(&source).map_err(bad_request("decode source image"))?;
    let adjusted = apply_image_adjustment(decoded, &request.adjustment)
        .map_err(bad_request("apply adjustment"))?;
    let limited_by_existing_crop = request.adjustment.crop.is_some()
        || request.adjustment.zoom > 1.0
        || request.adjustment.pan_x != 0.0
        || request.adjustment.pan_y != 0.0;
    let analysis = analyze_framing_support(
        &adjusted.to_rgba8(),
        request.corners.as_slice(),
        limited_by_existing_crop,
    )
    .ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "manual boundary cannot be measured for framing".to_owned(),
        )
    })?;
    Ok(Json(analysis))
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
    corners: &[autographs_image_processing::image_adjustments::ImagePoint],
    limited_by_existing_crop: bool,
) -> Option<FramingSupportAnalysis> {
    if corners.len() != 4 || image.width() < 2 || image.height() < 2 {
        return None;
    }
    let width_scale = image.width().saturating_sub(1) as f32;
    let height_scale = image.height().saturating_sub(1) as f32;
    let points = corners
        .iter()
        .map(|point| (point.x * width_scale, point.y * height_scale))
        .collect::<Vec<_>>();
    let edge_lengths = (0..4)
        .map(|index| {
            let next = (index + 1) % 4;
            (points[next].0 - points[index].0).hypot(points[next].1 - points[index].1)
        })
        .collect::<Vec<_>>();
    let item_width = (edge_lengths[0] + edge_lengths[2]) / 2.0;
    let item_height = (edge_lengths[1] + edge_lengths[3]) / 2.0;
    let item_short_edge_pixels = item_width.min(item_height);
    if !item_short_edge_pixels.is_finite() || item_short_edge_pixels < 1.0 {
        return None;
    }

    let left = points
        .iter()
        .map(|point| point.0)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .clamp(0.0, image.width().saturating_sub(1) as f32) as u32;
    let top = points
        .iter()
        .map(|point| point.1)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .clamp(0.0, image.height().saturating_sub(1) as f32) as u32;
    let right = (points
        .iter()
        .map(|point| point.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as u32)
        .saturating_add(1)
        .clamp(left.saturating_add(1), image.width());
    let bottom = (points
        .iter()
        .map(|point| point.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as u32)
        .saturating_add(1)
        .clamp(top.saturating_add(1), image.height());
    let item = PixelRect {
        left,
        top,
        right,
        bottom,
    };
    let invalid_integral = invalid_alpha_integral(image);
    let stride = image.width() as usize + 1;
    let item_intersects_invalid_pixels = invalid_count(&invalid_integral, stride, item) > 0;
    let margin = |side| {
        maximum_valid_margin(
            &invalid_integral,
            stride,
            image.width(),
            image.height(),
            item,
            side,
        ) as f32
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
        item_intersects_invalid_pixels,
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
    let left = rect.left as usize;
    let top = rect.top as usize;
    let right = rect.right as usize;
    let bottom = rect.bottom as usize;
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

async fn detect_edges(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<usize>,
) -> ApiResult<Response> {
    let image = image_entry(&state, id)?;
    let source = fs::read(&image.path)
        .await
        .map_err(internal("read source image"))?;
    let proposal =
        propose_image_adjustment(&source).map_err(bad_request("propose image adjustment"))?;
    Ok(Json(proposal).into_response())
}

async fn load_fixture(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<usize>,
) -> ApiResult<Response> {
    let image = image_entry(&state, id)?;
    let path = fixture_path(&state, image);
    if !fs::try_exists(&path)
        .await
        .map_err(internal("inspect fixture"))?
    {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }
    let json = fs::read_to_string(path)
        .await
        .map_err(internal("read fixture"))?;
    let adjustment = ImageAdjustment::from_json(json.trim()).map_err(internal("parse fixture"))?;
    Ok(Json(adjustment).into_response())
}

async fn save_fixture(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<usize>,
    body: Bytes,
) -> ApiResult<Response> {
    let image = image_entry(&state, id)?;
    let adjustment = parse_adjustment(&body)?;
    let json = format!(
        "{}\n",
        adjustment
            .to_json()
            .map_err(bad_request("serialize fixture"))?
    );
    let destination = fixture_path(&state, image);
    let temporary = destination.with_extension("json.tmp");
    fs::write(&temporary, json)
        .await
        .map_err(internal("write fixture"))?;
    fs::rename(&temporary, &destination)
        .await
        .map_err(internal("replace fixture"))?;
    Ok(Json(adjustment).into_response())
}

fn parse_adjustment(body: &[u8]) -> ApiResult<ImageAdjustment> {
    let json = std::str::from_utf8(body).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "adjustment must be UTF-8 JSON".to_owned(),
        )
    })?;
    ImageAdjustment::from_json(json).map_err(bad_request("parse adjustment"))
}

fn image_entry(state: &AppState, id: usize) -> ApiResult<&ImageEntry> {
    state
        .images
        .get(id)
        .ok_or((StatusCode::NOT_FOUND, "unknown image id".to_owned()))
}

fn fixture_path(state: &AppState, image: &ImageEntry) -> PathBuf {
    state.workspace.join(&image.fixture_name)
}

fn discover_images(source: &Path) -> Result<Vec<ImageEntry>, String> {
    let metadata = std::fs::metadata(source)
        .map_err(|error| format!("inspect source {}: {error}", source.display()))?;
    let mut paths = if metadata.is_file() {
        vec![source.to_path_buf()]
    } else if metadata.is_dir() {
        std::fs::read_dir(source)
            .map_err(|error| format!("read source directory {}: {error}", source.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && image_content_type(path).is_some())
            .collect()
    } else {
        return Err("source must be an image file or directory".to_owned());
    };
    paths.sort();
    paths
        .into_iter()
        .enumerate()
        .map(|(id, path)| {
            let content_type = image_content_type(&path)
                .ok_or_else(|| format!("unsupported source image: {}", path.display()))?;
            ImageReader::open(&path)
                .map_err(|error| format!("open source image {}: {error}", path.display()))?
                .with_guessed_format()
                .map_err(|error| format!("inspect source image {}: {error}", path.display()))?;
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("image")
                .to_owned();
            Ok(ImageEntry {
                name,
                path,
                fixture_name: format!("{id:04}-adjustment.json"),
                content_type,
            })
        })
        .collect()
}

fn image_content_type(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

fn internal<E: std::fmt::Display>(context: &'static str) -> impl FnOnce(E) -> ApiError {
    move |error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{context}: {error}"),
        )
    }
}

fn bad_request<E: std::fmt::Display>(context: &'static str) -> impl FnOnce(E) -> ApiError {
    move |error| (StatusCode::BAD_REQUEST, format!("{context}: {error}"))
}

struct Config {
    source: PathBuf,
    workspace: PathBuf,
    port: u16,
}

impl Config {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut source = None;
        let mut workspace = None;
        let mut port = 8788;
        let mut args = args.peekable();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--source" => source = args.next().map(PathBuf::from),
                "--workspace" => workspace = args.next().map(PathBuf::from),
                "--port" => {
                    port = args
                        .next()
                        .ok_or("--port requires a value")?
                        .parse()
                        .map_err(|_| "--port must be an integer from 1 to 65535")?;
                }
                "--help" | "-h" => {
                    return Err(
                        "usage: autographs-image-workbench --source PATH --workspace PATH [--port 8788]"
                            .to_owned(),
                    );
                }
                _ => return Err(format!("unknown argument: {argument}")),
            }
        }
        let source = source.ok_or("--source is required")?;
        let workspace = workspace.ok_or("--workspace is required")?;
        if source == workspace || workspace.starts_with(&source) {
            return Err("workspace must be separate from the source path".to_owned());
        }
        Ok(Self {
            source,
            workspace,
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use image::{DynamicImage, Rgb, RgbImage};
    use tempfile::tempdir;
    use tower::ServiceExt;

    fn test_state(images: Vec<ImageEntry>, workspace: &Path) -> AppState {
        AppState {
            images: Arc::new(images),
            workspace: Arc::new(workspace.to_path_buf()),
            authority: Arc::from("127.0.0.1:8788"),
            capability: Arc::from("test-capability"),
        }
    }

    #[test]
    fn discovers_only_allowlisted_images_in_stable_order() {
        let directory = tempdir().expect("tempdir");
        std::fs::write(directory.path().join("z.webp"), b"placeholder").expect("webp");
        std::fs::write(directory.path().join("a.jpg"), b"placeholder").expect("jpg");
        std::fs::write(directory.path().join("ignored.gif"), b"placeholder").expect("gif");
        let images = discover_images(directory.path()).expect("discover");
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].name, "a.jpg");
        assert_eq!(images[1].name, "z.webp");
        assert_eq!(images[0].fixture_name, "0000-adjustment.json");
    }

    #[tokio::test]
    async fn render_uses_shared_adjustment_without_mutating_source() {
        let source_dir = tempdir().expect("source tempdir");
        let workspace = tempdir().expect("workspace tempdir");
        let source_path = source_dir.path().join("card.png");
        let image = RgbImage::from_pixel(24, 16, Rgb([90, 100, 110]));
        DynamicImage::ImageRgb8(image)
            .save(&source_path)
            .expect("save source");
        let before = std::fs::read(&source_path).expect("read before");
        let state = test_state(
            discover_images(&source_path).expect("discover"),
            workspace.path(),
        );
        let response = render_image(
            State(state),
            AxumPath(0),
            Bytes::from(ImageAdjustment::identity().to_json().expect("json")),
        )
        .await
        .expect("render");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .contains_key("x-workbench-boundary-status")
        );
        assert!(
            response
                .headers()
                .contains_key("x-workbench-boundary-edges")
        );
        assert!(
            response
                .headers()
                .contains_key("x-workbench-framing-support")
        );
        assert_eq!(std::fs::read(&source_path).expect("read after"), before);
    }

    #[tokio::test]
    async fn render_reports_detected_boundary_for_output_measurement() {
        let source_dir = tempdir().expect("source tempdir");
        let workspace = tempdir().expect("workspace tempdir");
        let source_path = source_dir.path().join("card.png");
        let mut image = RgbImage::from_pixel(160, 120, Rgb([190, 190, 190]));
        for y in 15..105 {
            for x in 20..140 {
                image.put_pixel(x, y, Rgb([24, 28, 32]));
            }
        }
        DynamicImage::ImageRgb8(image)
            .save(&source_path)
            .expect("save source");
        let state = test_state(
            discover_images(&source_path).expect("discover"),
            workspace.path(),
        );

        let response = render_image(
            State(state),
            AxumPath(0),
            Bytes::from(ImageAdjustment::identity().to_json().expect("json")),
        )
        .await
        .expect("render");

        assert_eq!(
            response.headers()["x-workbench-boundary-status"],
            "reliable"
        );
        let corners = response.headers()["x-workbench-boundary-corners"]
            .to_str()
            .expect("corners header");
        assert_eq!(
            serde_json::from_str::<Vec<serde_json::Value>>(corners)
                .expect("corners json")
                .len(),
            4
        );
        let edges = response.headers()["x-workbench-boundary-edges"]
            .to_str()
            .expect("edges header");
        assert_eq!(
            serde_json::from_str::<Vec<serde_json::Value>>(edges)
                .expect("edges json")
                .len(),
            4
        );
    }

    #[tokio::test]
    async fn manual_boundary_can_drive_real_pixel_framing_support() {
        let source_dir = tempdir().expect("source tempdir");
        let workspace = tempdir().expect("workspace tempdir");
        let source_path = source_dir.path().join("slab.png");
        let image = RgbImage::from_pixel(120, 100, Rgb([180, 180, 180]));
        DynamicImage::ImageRgb8(image)
            .save(&source_path)
            .expect("save source");
        let state = test_state(
            discover_images(&source_path).expect("discover"),
            workspace.path(),
        );
        let request = ManualFramingRequest {
            adjustment: ImageAdjustment::identity(),
            corners: vec![
                ImagePoint { x: 0.10, y: 0.15 },
                ImagePoint { x: 0.90, y: 0.15 },
                ImagePoint { x: 0.90, y: 0.85 },
                ImagePoint { x: 0.10, y: 0.85 },
            ],
        };

        let Json(analysis) = manual_framing(State(state), AxumPath(0), Json(request))
            .await
            .expect("manual framing");

        assert!(analysis.max_symmetric_margin_pixels > 9.0);
        assert!(!analysis.item_intersects_invalid_pixels);
    }

    #[tokio::test]
    async fn manual_boundary_rejects_invalid_quadrilateral_topology() {
        let source_dir = tempdir().expect("source tempdir");
        let workspace = tempdir().expect("workspace tempdir");
        let source_path = source_dir.path().join("slab.png");
        DynamicImage::ImageRgb8(RgbImage::from_pixel(120, 100, Rgb([180, 180, 180])))
            .save(&source_path)
            .expect("save source");
        let state = test_state(
            discover_images(&source_path).expect("discover"),
            workspace.path(),
        );
        let invalid_boundaries = [
            vec![
                ImagePoint { x: 0.1, y: 0.1 },
                ImagePoint { x: 0.9, y: 0.9 },
                ImagePoint { x: 0.9, y: 0.1 },
                ImagePoint { x: 0.1, y: 0.9 },
            ],
            vec![
                ImagePoint { x: 0.1, y: 0.1 },
                ImagePoint { x: 0.9, y: 0.1 },
                ImagePoint { x: 0.4, y: 0.4 },
                ImagePoint { x: 0.1, y: 0.9 },
            ],
            vec![
                ImagePoint { x: 0.1, y: 0.1 },
                ImagePoint { x: 0.1, y: 0.9 },
                ImagePoint { x: 0.9, y: 0.9 },
                ImagePoint { x: 0.9, y: 0.1 },
            ],
        ];

        for corners in invalid_boundaries {
            let result = manual_framing(
                State(state.clone()),
                AxumPath(0),
                Json(ManualFramingRequest {
                    adjustment: ImageAdjustment::identity(),
                    corners,
                }),
            )
            .await;
            assert!(matches!(result, Err((StatusCode::BAD_REQUEST, _))));
        }
    }

    #[test]
    fn rejects_workspace_nested_under_source() {
        let result = Config::parse(
            [
                "--source",
                "/private/images",
                "--workspace",
                "/private/images/output",
            ]
            .into_iter()
            .map(str::to_owned),
        );
        assert!(result.is_err());
    }

    #[test]
    fn uses_the_single_documented_default_port() {
        let config = Config::parse(
            [
                "--source",
                "/private/images",
                "--workspace",
                "/tmp/image-workbench",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("config");
        assert_eq!(config.port, 8788);
    }

    #[test]
    fn workbench_exposes_operator_controlled_alignment_suggestions() {
        assert!(INDEX_HTML.contains("id=\"alignment-advice\""));
        assert!(APP_JS.contains("Suggested next experiment"));
        assert!(APP_JS.contains("Auto-align to tolerance"));
        assert!(APP_JS.contains("for (let pass = 1; pass <= 8; pass += 1)"));
        assert!(APP_JS.contains("a repeated adjustment was avoided"));
        assert!(APP_JS.contains("the next pass worsened alignment"));
        assert!(APP_JS.contains("the rejected proposal is no longer actionable"));
        assert!(APP_JS.contains("alignmentSuppression?.adjustment"));
        assert!(APP_JS.contains("renderPreview({ publish: false })"));
        assert!(APP_JS.contains("publishPreviewResult(finalResult)"));
        assert!(APP_JS.contains("the reliable coarse boundary"));
        assert!(APP_JS.contains("kind === \"rotation\""));
        assert!(APP_JS.contains("coarse alignment complete"));
        assert!(INDEX_HTML.contains("Background framing"));
        assert!(APP_JS.contains("Top ${marginLabel}"));
        assert!(APP_JS.contains("pp off center"));
        assert!(APP_JS.contains("Worst corner from square (${cornerNames[worstCornerIndex]})"));
        assert!(INDEX_HTML.contains("normalized inward distance"));
        assert!(APP_JS.contains("Suggestions use the reliable coarse boundary instead"));
        assert!(APP_JS.contains("fittedEdgesAreStable"));
        assert!(INDEX_HTML.contains("Reset measurement to automatic"));
        assert!(INDEX_HTML.contains("Background target"));
        assert!(APP_JS.contains("Maximum symmetric real-pixel band"));
        assert!(APP_JS.contains("Retake recommended"));
        assert!(APP_JS.contains("does not change crop, zoom, or pan"));
        assert!(INDEX_HTML.contains("id=\"framing-overlay\""));
        assert!(APP_JS.contains("Preview crop"));
        assert!(APP_JS.contains("Apply to draft"));
        assert!(APP_JS.contains("Undo framing"));
        assert!(APP_JS.contains("Assess using manual boundary"));
        assert!(APP_JS.contains("/framing`"));
        assert!(APP_JS.contains("Confirmed manual boundary."));
        assert!(APP_JS.contains("Apply recentering"));
        assert!(APP_JS.contains("The existing thin border can still be recentered"));
        assert!(APP_JS.contains("reassessManualFraming: preserveManualBoundary"));
        assert!(APP_CSS.contains(
            ".measurement-overlay[hidden], .framing-overlay[hidden] { display: none !important; }"
        ));
        assert!(APP_JS.contains("overlay.toggleAttribute(\"hidden\", !visible)"));
        assert!(APP_JS.contains("$(\"#measurement-overlay\").toggleAttribute("));
        assert!(
            APP_JS.contains("Applied ${appliedBand.toFixed(1)}% framing crop to the unsaved draft")
        );
    }

    #[test]
    fn framing_support_uses_real_alpha_clearance() {
        let mut image = RgbaImage::from_pixel(100, 80, image::Rgba([20, 30, 40, 255]));
        for y in 0..80 {
            for x in 0..8 {
                image.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            }
        }
        let corners = [
            autographs_image_processing::image_adjustments::ImagePoint { x: 0.20, y: 0.25 },
            autographs_image_processing::image_adjustments::ImagePoint { x: 0.80, y: 0.25 },
            autographs_image_processing::image_adjustments::ImagePoint { x: 0.80, y: 0.75 },
            autographs_image_processing::image_adjustments::ImagePoint { x: 0.20, y: 0.75 },
        ];

        let analysis = analyze_framing_support(&image, &corners, false).expect("analysis");

        assert!(analysis.margins_pixels.left >= 11.0);
        assert!(analysis.margins_pixels.left <= 12.0);
        assert!(analysis.max_symmetric_margin_pixels >= 11.0);
        assert!(analysis.max_symmetric_margin_pixels <= 12.0);
        assert!(!analysis.item_intersects_invalid_pixels);
    }

    #[tokio::test]
    async fn router_rejects_attacker_hosts_and_cross_origin_fixture_writes() {
        let source_dir = tempdir().expect("source tempdir");
        let workspace = tempdir().expect("workspace tempdir");
        let source_path = source_dir.path().join("card.png");
        DynamicImage::ImageRgb8(RgbImage::from_pixel(24, 16, Rgb([90, 100, 110])))
            .save(&source_path)
            .expect("save source");
        let router = app(test_state(
            discover_images(&source_path).expect("discover"),
            workspace.path(),
        ));

        for uri in ["/api/images", "/api/images/0/source"] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(uri)
                        .header(header::HOST, "attacker.example:8788")
                        .header("x-workbench-capability", "test-capability")
                        .body(Body::empty())
                        .expect("request"),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/images/0/fixture")
                    .header(header::HOST, "127.0.0.1:8788")
                    .header(header::ORIGIN, "http://attacker.example:8788")
                    .header("x-workbench-capability", "test-capability")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        ImageAdjustment::identity().to_json().expect("json"),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(!workspace.path().join("0000-adjustment.json").exists());

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/images/0/source")
                    .header(header::HOST, "127.0.0.1:8788")
                    .header("x-workbench-capability", "test-capability")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
}
