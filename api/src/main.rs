use std::{env, fs::{self, Permissions}, net::SocketAddr, os::{linux::fs::MetadataExt, unix::fs::PermissionsExt}, sync::Arc};

use axum::{
    extract,
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, patch, post},
    Router,
};
use routes::*;
use tokio::{net::UnixListener, sync::Mutex};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{adaptors::create_adaptor, governor::DynamicKeyExtractor};
use crate::docs::ApiDoc;

mod adaptors;
mod docs;
mod errors;
mod governor;
mod payloads;
mod routes;

pub struct ApiState<A> {
    adaptor: A,
}

pub type State<A> = extract::State<Arc<Mutex<ApiState<A>>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Load env
    dotenvy::dotenv().ok();

    let shared_state = Arc::new(Mutex::new(ApiState {
        adaptor: create_adaptor().await,
    }));

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST, Method::PATCH])
        .allow_origin(
            env::var("FRONTEND_URL").expect("Missing FRONTEND_URL environment variable")
                .parse::<HeaderValue>()
                .unwrap(),
        );

    // Rate limiting configuration (using tower_governor)
    // From the docs: Allows bursts with up to 20 requests and replenishes
    // one element after 500ms, based on peer IP.
    let governor_config = GovernorConfigBuilder::default()
        .burst_size(20)
        .key_extractor(DynamicKeyExtractor::from_env())
        .finish()
        .unwrap();


    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(get_root))
        .route("/stats", get(stats::get_stats))
        .route("/event", post(event::create_event))
        .route("/event/{event_id}", get(event::get_event))
        .route("/event/{event_id}/people", get(person::get_people))
        .route(
            "/event/{event_id}/people/{person_name}",
            get(person::get_person),
        )
        .route(
            "/event/{event_id}/people/{person_name}",
            patch(person::update_person),
        )
        .route("/tasks/cleanup", get(tasks::cleanup))
        .with_state(shared_state)
        .layer(cors)
        .layer(GovernorLayer::new(governor_config))
        .layer(TraceLayer::new_for_http());

    let address = env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:3000".to_owned());

    if let Some(path) = address.strip_prefix("unix:") {
        if let Ok(stat) = fs::metadata(path) {
            // if it exists, check if it's a socket
            if stat.st_mode() & 0o140000 != 0 {
                // yeet
                println!("Socket at {path} already exists, trying to remove");
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("Minor issue: failed to remove existing socket at {path}, we might fail to bind to this location. {e:?}");
                }
            }
        }
        let listener = UnixListener::bind(path).unwrap_or_else(|e| panic!("Failed to bind to unix socket at {path}: {e:?}"));

        // maybe chown
        if let Ok(mode) = env::var("UNIX_SOCK_MODE") {
            let perms = Permissions::from_mode(u32::from_str_radix(&mode, 8).expect("expected UNIX_SOCK_MODE to be a valid base8 int"));
            fs::set_permissions(path, perms).expect("failed to chmod the new unix socket");
        }

        println!(
            "🦀 Crab Fit API listening at {address} in {} mode",
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
        );

        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler")
            })
            .await
            .unwrap();
    } else {
        let listener = tokio::net::TcpListener::bind(&address).await.unwrap_or_else(|e| panic!("Failed to pind to TCP socket at {address}: {e:?}"));

        println!(
            "🦀 Crab Fit API listening at http://{address} in {} mode",
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
        );

        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler")
            })
            .await
            .unwrap();
    }

}

async fn get_root() -> String {
    format!("Crab Fit API v{}", env!("CARGO_PKG_VERSION"))
}
