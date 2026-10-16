use std::{
    env,
    io::Cursor,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use autographs_image_processing::image_adjustments::{
    ImageAdjustment, ImageAdjustmentProposalStatus, apply_image_adjustment,
    propose_image_adjustment,
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path as AxumPath, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use image::{ImageFormat, ImageReader};
use serde::Serialize;
use tokio::fs;

const INDEX_HTML: &str = include_str!("../static/index.html");
const APP_CSS: &str = include_str!("../static/app.css");
const APP_JS: &str = include_str!("../static/app.js");

#[derive(Clone)]
struct AppState {
    images: Arc<Vec<ImageEntry>>,
    workspace: Arc<PathBuf>,
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

    let state = AppState {
        images: Arc::new(images),
        workspace: Arc::new(config.workspace),
    };
    let image_count = state.images.len();
    let app = app(state);
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.port);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| format!("bind {address}: {error}"))?;
    println!("Autographs image workbench: http://{address}");
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
        .route("/api/images/{id}/detect", post(detect_edges))
        .route(
            "/api/images/{id}/fixture",
            get(load_fixture).put(save_fixture),
        )
        .with_state(state)
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
    let analysis = propose_image_adjustment(&preview).ok();
    let analysis_status = analysis
        .as_ref()
        .map(|proposal| match proposal.status {
            ImageAdjustmentProposalStatus::Confident => "confident",
            ImageAdjustmentProposalStatus::NotNeeded => "notNeeded",
            ImageAdjustmentProposalStatus::Unavailable => "unavailable",
        })
        .unwrap_or("error");
    let analysis_corners = serde_json::to_string(
        &analysis
            .as_ref()
            .map(|proposal| proposal.corners.as_slice())
            .unwrap_or_default(),
    )
    .map_err(internal("serialize output analysis"))?;
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
    Ok((headers, preview).into_response())
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
    use image::{DynamicImage, Rgb, RgbImage};
    use tempfile::tempdir;

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
        let state = AppState {
            images: Arc::new(discover_images(&source_path).expect("discover")),
            workspace: Arc::new(workspace.path().to_path_buf()),
        };
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
        assert_eq!(std::fs::read(&source_path).expect("read after"), before);
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
}
