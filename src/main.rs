use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

// Placeholder PNG data - replace this with your actual thermometer image
// For now, we'll serve a simple 1x1 transparent PNG
const THERMOMETER_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // PNG signature
    0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, // IDAT chunk
    0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00,
    0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, // IEND chunk
    0x42, 0x60, 0x82,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Team {
    name: String,
    image_url: Option<String>,
    total_raised: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThermometerConfig {
    title: String,
    goal: f64,
    teams: Vec<Team>,
    last_updated: String,
}

impl Default for ThermometerConfig {
    fn default() -> Self {
        Self {
            title: "Animal Shelter Donation Drive".to_string(),
            goal: 10000.0,
            teams: vec![],
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Clone)]
struct AppState {
    config: Arc<RwLock<ThermometerConfig>>,
    edit_key: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    message: String,
    config: ThermometerConfig,
}

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

    let state = AppState {
        config: Arc::new(RwLock::new(ThermometerConfig::default())),
        edit_key,
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/thermometer.png", get(thermometer_image))
        .route("/health", get(health_check))
        .route("/config", get(get_config))
        .route("/admin/upload", post(upload_csv))
        .route("/admin/config", post(update_config))
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

async fn root() -> &'static str {
    "Animal Shelter Donation Thermometer API\n\nPublic Endpoints:\n- GET /thermometer.png - Donation progress thermometer image\n- GET /config - Current thermometer configuration\n- GET /health - Health check\n\nAdmin Endpoints (require Authorization header with THERMOMETER_EDIT_KEY):\n- POST /admin/upload - Upload CSV with donation data\n- POST /admin/config - Update thermometer configuration (JSON)\n"
}

async fn thermometer_image(State(_state): State<AppState>) -> Response {
    // TODO: Generate actual thermometer image based on _state.config
    // For now, return placeholder PNG
    (
        [
            ("Content-Type", "image/png"),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
            ("Pragma", "no-cache"),
            ("Expires", "0"),
        ],
        THERMOMETER_PNG,
    )
        .into_response()
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_config(State(state): State<AppState>) -> Json<ThermometerConfig> {
    let config = state.config.read().unwrap();
    Json(config.clone())
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

            // Update config with new team data
            let mut config = state.config.write().unwrap();
            config.teams = teams;
            config.last_updated = chrono::Utc::now().to_rfc3339();

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
    let mut config = state.config.write().unwrap();
    *config = new_config;
    config.last_updated = chrono::Utc::now().to_rfc3339();

    tracing::info!("Updated thermometer config via JSON");

    Ok(Json(SuccessResponse {
        message: "Configuration updated successfully".to_string(),
        config: config.clone(),
    }))
}
