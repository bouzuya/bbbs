mod app_state;
mod handler;
mod model;
mod port;
mod store;
mod utils;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app_state::AppState;

#[derive(clap::Parser)]
struct Cli {
    #[clap(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
    let cli = <Cli as clap::Parser>::parse();
    let port = cli.port.unwrap_or(3000);

    let router = handler::router().with_state(AppState::new()).layer(
        tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::http::Request<axum::body::Body>| {
                let matched_path = request
                    .extensions()
                    .get::<axum::extract::MatchedPath>()
                    .map(axum::extract::MatchedPath::as_str);
                tracing::info_span!(
                    "http_request",
                    matched_path,
                    method = ?request.method(),
                    uri = ?request.uri(),
                )
            },
        ),
    );
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap()
}
