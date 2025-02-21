use app::{AppError, AppState};
use axum::{
    Router,
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use config::Config;
use http_body_util::BodyExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod config;
mod handlers;
mod models;
mod technitium;

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

async fn check_zone_existence(
    app_state: &Arc<AppState>,
) -> Result<bool, technitium::TechnitiumError> {
    let client = app_state.client.read().await;
    let mut page_number = 1;

    loop {
        let zones = client
            .list_zones(technitium::ListZonesPayload {
                zone: app_state.config.zone.clone(),
                page_number: Some(page_number),
                zones_per_page: Some(100),
            })
            .await?;
        if zones.zones.iter().any(|z| z.name == app_state.config.zone) {
            return Ok(true);
        }
        if page_number >= zones.total_pages {
            break;
        }
        page_number += 1;
    }
    Ok(false)
}

async fn create_default_zone(app_state: &Arc<AppState>) -> Result<(), technitium::TechnitiumError> {
    let client = app_state.client.read().await;
    let payload = technitium::CreateZonePayload {
        zone: app_state.config.zone.clone(),
        zone_type: technitium::ZoneType::Forwarder,
        protocol: Some(technitium::Protocol::Udp),
        forwarder: Some("this-server".to_string()),
        dnssec_validation: Some(true),
        ..Default::default()
    };

    client.create_zone(payload).await?;
    info!(
        "Zone {} created successfully in Technitium DNS server.",
        &app_state.config.zone
    );
    Ok(())
}

async fn setup_technitium_connection(app_state: Arc<AppState>) {
    // Construct the login payload using the credentials from the configuration
    let login_payload = technitium::LoginPayload {
        username: app_state.config.technitium_username.clone(),
        password: app_state.config.technitium_password.clone(),
    };

    // Attempt to log in to Technitium using the login payload
    let token = match app_state.client.read().await.login(login_payload).await {
        Ok(resp) => resp.token,
        Err(e) => {
            error!("Failed to log in to Technitium DNS server: {}", e);
            std::process::exit(1);
        }
    };

    info!("Successfully logged into Technitium DNS server with received token.");
    *app_state.client.write().await = technitium::TechnitiumClient::new(
        app_state.config.technitium_url.clone(),
        token,
        HTTP_TIMEOUT,
    );
    tokio::spawn(auto_renew_technitium_token(Arc::clone(&app_state)));

    debug!("Verifying and preparing the DNS Zone...");

    let zone_exists = match check_zone_existence(&app_state).await {
        Ok(ret) => ret,
        Err(e) => {
            error!("Failed to list zones: {}", e);
            std::process::exit(1);
        }
    };

    if zone_exists {
        info!(
            "Zone {} exists in Technitium DNS server.",
            &app_state.config.zone
        );
    } else {
        if let Err(e) = create_default_zone(&app_state).await {
            error!(
                "Failed to create the zone {} in Technitium DNS server: {}",
                &app_state.config.zone, e
            );
            std::process::exit(1);
        }
    }

    *app_state.is_ready.write().await = true;
}

async fn auto_renew_technitium_token(app_state: Arc<AppState>) {
    const DURATION_SUCCESS: Duration = Duration::from_secs(20 * 60);
    const DURATION_FAILURE: Duration = Duration::from_secs(60);

    let mut sleep_for = DURATION_SUCCESS;
    loop {
        sleep(sleep_for).await;

        // Construct the login payload using the credentials from the configuration
        let login_payload = technitium::LoginPayload {
            username: app_state.config.technitium_username.clone(),
            password: app_state.config.technitium_password.clone(),
        };

        // Attempt to log in to Technitium using the login payload
        let token = match app_state.client.read().await.login(login_payload).await {
            Ok(resp) => resp.token,
            Err(e) => {
                error!("Failed to renew Technitium DNS server access token: {}", e);
                sleep_for = DURATION_FAILURE;
                continue;
            }
        };

        info!("Successfully renewed Technitium DNS server access token.");
        *app_state.client.write().await = technitium::TechnitiumClient::new(
            app_state.config.technitium_url.clone(),
            token,
            HTTP_TIMEOUT,
        );
        sleep_for = DURATION_SUCCESS;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let client = technitium::TechnitiumClient::new(
        config.technitium_url.clone(),
        Default::default(), // no token initially
        HTTP_TIMEOUT,
    );
    let app_state = Arc::new(AppState {
        config,
        is_ready: RwLock::new(false),
        client: RwLock::new(client),
    });

    // Check and create zone if necessary
    tokio::spawn(setup_technitium_connection(Arc::clone(&app_state)));

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/", get(handlers::negotiate_domain_filter))
        .route("/records", get(handlers::get_records))
        .route("/adjustendpoints", post(handlers::adjust_endpoints))
        .route("/records", post(handlers::apply_record))
        .layer(middleware::from_fn(print_request_response))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::clone(&app_state));

    let listener = tokio::net::TcpListener::bind(&app_state.config.address())
        .await
        .map_err(|e| {
            error!("Failed to bind to address: {}", e);
            e
        })?;

    info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            error!("Server error: {}", e);
            e.into()
        })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
