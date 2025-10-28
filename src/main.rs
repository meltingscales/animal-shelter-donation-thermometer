mod storage;
mod thermometer;
mod color_constants;

use askama::Template;
use axum::{
    extract::{Multipart, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use storage::{ConfigStorage, create_storage};
use thermometer::{generate_thermometer_svg, svg_to_png};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

// Empty filters module for askama templates
mod filters {}

// Query parameters for thermometer image
#[derive(Debug, Deserialize)]
struct ThermometerQuery {
    #[serde(default = "default_scale")]
    scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct Team {
    name: String,
    image_url: Option<String>,
    total_raised: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct ThermometerConfig {
    organization_name: String,
    title: String,
    goal: f64,
    teams: Vec<Team>,
    last_updated: String,
}

impl Default for ThermometerConfig {
    fn default() -> Self {
        Self {
            organization_name: "Community Animal Rescue Effort".to_string(),
            title: "Animal Shelter Donation Drive".to_string(),
            goal: 10000.0,
            teams: vec![],
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Clone)]
struct AppState {
    storage: Arc<dyn ConfigStorage>,
    edit_key: String,
}

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize, ToSchema)]
struct SuccessResponse {
    message: String,
    config: ThermometerConfig,
}

// Template structures for Askama
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    organization_name: String,
    title: String,
    last_updated: String,
    total_raised: String,
    goal: String,
    progress_percent: String,
    progress_percent_raw: f64,  // For the progress bar width
    team_count: usize,
    teams: Vec<Team>,
    base_url: String,
}

#[derive(Template)]
#[template(path = "faq.html")]
struct FaqTemplate {}

#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate {}

// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        get_config,
        upload_csv,
        update_config,
    ),
    components(
        schemas(
            Team,
            ThermometerConfig,
            ErrorResponse,
            SuccessResponse,
        )
    ),
    tags(
        (name = "Public", description = "Public endpoints"),
        (name = "Admin", description = "Admin endpoints (authentication required)"),
    ),
    info(
        title = "Animal Shelter Donation Thermometer API",
        version = "1.0.0",
        description = "API for managing donation thermometer data.\n\n**Authentication:** Admin endpoints require an `Authorization` header with the `THERMOMETER_EDIT_KEY`.",
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // Initialize logging (disable in Cloud Run to avoid startup issues)
    // Cloud Run sets K_SERVICE environment variable
    if std::env::var("K_SERVICE").is_err() {
        tracing_subscriber::registry()
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info"))
            )
            .with(tracing_subscriber::fmt::layer().compact())
            .init();
    }

    tracing::info!("Starting Animal Shelter Donation Thermometer server");

    // Get or generate the edit key from environment variable
    let edit_key = std::env::var("THERMOMETER_EDIT_KEY")
        .unwrap_or_else(|_| {
            let key = Uuid::new_v4().to_string();
            tracing::warn!("THERMOMETER_EDIT_KEY not set, generated new key: {}", key);
            key
        });

    // Initialize storage (Firestore if GCP_PROJECT is set, otherwise in-memory)
    let storage = create_storage().await;

    let state = AppState {
        storage,
        edit_key,
    };

    let app = Router::new()
        .route("/", get(home_page))
        .route("/faq", get(faq_page))
        .route("/admin", get(admin_page))
        .route("/admin/sample-csv", get(download_sample_csv))
        .route("/thermometer-light.png", get(thermometer_light_image))
        .route("/thermometer-light.svg", get(thermometer_light_svg))
        .route("/thermometer-dark.png", get(thermometer_dark_image))
        .route("/thermometer-dark.svg", get(thermometer_dark_svg))
        .route("/health", get(health_check))
        .route("/config", get(get_config))
        .route("/admin/upload", post(upload_csv))
        .route("/admin/config", post(update_config))
        .merge(SwaggerUi::new("/openapi").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB limit
        );

    // Cloud Run provides PORT environment variable, default to 8080
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server listening on {}", addr);

    // Graceful shutdown handler
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down gracefully");
        },
    }
}

async fn home_page(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<HomeTemplate, StatusCode> {
    let config = state.storage.load_config().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_raised: f64 = config.teams.iter().map(|t| t.total_raised).sum();
    let progress_percent = if config.goal > 0.0 {
        let raw_percent = (total_raised / config.goal * 100.0).min(100.0);
        (raw_percent * 100.0).round() / 100.0  // Round to 2 decimal places
    } else {
        0.0
    };

    // Build base URL from request headers
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:8080");

    // Check if we're behind a proxy (Cloud Run sets X-Forwarded-Proto)
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    let base_url = format!("{}://{}", proto, host);

    Ok(HomeTemplate {
        organization_name: config.organization_name.clone(),
        title: config.title.clone(),
        last_updated: config.last_updated.clone(),
        total_raised: format!("{:.2}", total_raised),
        goal: format!("{:.2}", config.goal),
        progress_percent: format!("{:.2}", progress_percent),
        progress_percent_raw: progress_percent,
        team_count: config.teams.len(),
        teams: config.teams.clone(),
        base_url,
    })
}

async fn faq_page() -> FaqTemplate {
    FaqTemplate {}
}

async fn admin_page() -> AdminTemplate {
    AdminTemplate {}
}

async fn download_sample_csv() -> Response {
    // Create sample CSV data
    let sample_csv = r#"name,image_url,total_raised
Team Alpha,https://example.com/alpha.jpg,2500.00
Team Beta,https://example.com/beta.jpg,3200.50
Team Gamma,,1800.00
PUP ALL NIGHT: THE PM PACK,,6987.00
UnderDogs,https://example.com/underdogs.png,5010.00
Hairball Wizards,,4101.25"#;

    (
        [
            ("Content-Type", "text/csv"),
            ("Content-Disposition", "attachment; filename=\"sample-teams.csv\""),
        ],
        sample_csv,
    )
        .into_response()
}

async fn thermometer_light_svg(State(state): State<AppState>) -> Response {
    // Load configuration
    let config = match state.storage.load_config().await {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load config for thermometer: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load configuration",
            )
                .into_response();
        }
    };

    // Base width for the thermometer
    let base_width = 800u32;

    // Generate SVG
    let svg = generate_thermometer_svg(&config, base_width, false);

    (
        [
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
            ("Pragma", "no-cache"),
            ("Expires", "0"),
        ],
        svg,
    )
        .into_response()
}

async fn thermometer_dark_svg(State(state): State<AppState>) -> Response {
    // Load configuration
    let config = match state.storage.load_config().await {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load config for thermometer: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load configuration",
            )
                .into_response();
        }
    };

    // Base width for the thermometer
    let base_width = 800u32;

    // Generate SVG
    let svg = generate_thermometer_svg(&config, base_width, true);

    (
        [
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
            ("Pragma", "no-cache"),
            ("Expires", "0"),
        ],
        svg,
    )
        .into_response()
}

async fn thermometer_light_image(
    State(state): State<AppState>,
    Query(params): Query<ThermometerQuery>,
) -> Response {
    // Load configuration
    let config = match state.storage.load_config().await {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load config for thermometer: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load configuration",
            )
                .into_response();
        }
    };

    // Validate scale parameter (between 0.1 and 5.0)
    let scale = params.scale.max(0.1).min(5.0);

    // Base width for the thermometer (will be scaled)
    let base_width = 800u32;

    // Generate SVG
    let svg = generate_thermometer_svg(&config, base_width, false);

    // Convert SVG to PNG
    let png_data = match svg_to_png(&svg, scale) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to render thermometer PNG: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render thermometer image",
            )
                .into_response();
        }
    };

    (
        [
            ("Content-Type", "image/png"),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
            ("Pragma", "no-cache"),
            ("Expires", "0"),
        ],
        png_data,
    )
        .into_response()
}

async fn thermometer_dark_image(
    State(state): State<AppState>,
    Query(params): Query<ThermometerQuery>,
) -> Response {
    // Load configuration
    let config = match state.storage.load_config().await {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load config for thermometer: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load configuration",
            )
                .into_response();
        }
    };

    // Validate scale parameter (between 0.1 and 5.0)
    let scale = params.scale.max(0.1).min(5.0);

    // Base width for the thermometer (will be scaled)
    let base_width = 800u32;

    // Generate SVG
    let svg = generate_thermometer_svg(&config, base_width, true);

    // Convert SVG to PNG
    let png_data = match svg_to_png(&svg, scale) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to render thermometer PNG: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render thermometer image",
            )
                .into_response();
        }
    };

    (
        [
            ("Content-Type", "image/png"),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
            ("Pragma", "no-cache"),
            ("Expires", "0"),
        ],
        png_data,
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Public",
    responses(
        (status = 200, description = "Service is healthy")
    )
)]
async fn health_check() -> &'static str {
    "OK"
}

#[utoipa::path(
    get,
    path = "/config",
    tag = "Public",
    responses(
        (status = 200, description = "Current thermometer configuration", body = ThermometerConfig)
    )
)]
async fn get_config(State(state): State<AppState>) -> Result<Json<ThermometerConfig>, StatusCode> {
    let config = state.storage.load_config().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(config))
}

fn verify_auth(headers: &HeaderMap, expected_key: &str) -> Result<(), StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Support both "Bearer <key>" and just "<key>"
    let provided_key = auth_header.strip_prefix("Bearer ").unwrap_or(auth_header);

    if provided_key != expected_key {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(())
}

#[utoipa::path(
    post,
    path = "/admin/upload",
    tag = "Admin",
    responses(
        (status = 200, description = "CSV uploaded successfully", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Bad request", body = ErrorResponse)
    )
)]
async fn upload_csv(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify authentication
    verify_auth(&headers, &state.edit_key).map_err(|status| {
        (
            status,
            Json(ErrorResponse {
                error: "Invalid or missing Authorization header".to_string(),
            }),
        )
    })?;

    // Process the uploaded CSV file
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Failed to read multipart data: {}", e),
            }),
        )
    })? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("Failed to read file data: {}", e),
                    }),
                )
            })?;

            // Parse CSV
            let mut reader = csv::Reader::from_reader(data.as_ref());
            let mut teams: Vec<Team> = Vec::new();

            for result in reader.deserialize() {
                let team: Team = result.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Failed to parse CSV: {}", e),
                        }),
                    )
                })?;
                teams.push(team);
            }

            // Load current config and update with new team data
            let mut config = state.storage.load_config().await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to load config: {}", e),
                    }),
                )
            })?;

            config.teams = teams;
            config.last_updated = chrono::Utc::now().to_rfc3339();

            // Save updated config
            state.storage.save_config(&config).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to save config: {}", e),
                    }),
                )
            })?;

            tracing::info!("Updated thermometer config with {} teams", config.teams.len());

            return Ok(Json(SuccessResponse {
                message: "CSV uploaded successfully".to_string(),
                config: config.clone(),
            }));
        }
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: "No file uploaded".to_string(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/admin/config",
    tag = "Admin",
    request_body = ThermometerConfig,
    responses(
        (status = 200, description = "Configuration updated successfully", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
async fn update_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(new_config): Json<ThermometerConfig>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify authentication
    verify_auth(&headers, &state.edit_key).map_err(|status| {
        (
            status,
            Json(ErrorResponse {
                error: "Invalid or missing Authorization header".to_string(),
            }),
        )
    })?;

    // Update the configuration
    let mut config = new_config;
    config.last_updated = chrono::Utc::now().to_rfc3339();

    // Save updated config
    state.storage.save_config(&config).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to save config: {}", e),
            }),
        )
    })?;

    tracing::info!("Updated thermometer config via JSON");

    Ok(Json(SuccessResponse {
        message: "Configuration updated successfully".to_string(),
        config: config.clone(),
    }))
}
