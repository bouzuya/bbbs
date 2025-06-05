use axum::extract::State;

trait Name {
    fn name(&self) -> &str;
}

async fn root<S: Name>(State(state): State<S>) -> String {
    format!("Hello, {}!", state.name())
}

fn router<S: Clone + Name + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new().route("/", axum::routing::get(root::<S>))
}

#[derive(Clone)]
struct AppState {
    name: String,
}

impl Name for AppState {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(clap::Parser)]
struct Cli {
    #[clap(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let cli = <Cli as clap::Parser>::parse();
    let port = cli.port.unwrap_or(3000);

    let router = router().with_state(AppState {
        name: "bouzuya".to_owned(),
    });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_root() -> anyhow::Result<()> {
        let router = router().with_state(AppState {
            name: "bouzuya".to_owned(),
        });

        let response = tower::ServiceExt::oneshot(
            router,
            axum::http::Request::builder()
                .method(axum::http::Method::GET)
                .uri("/")
                .body(axum::body::Body::empty())?,
        )
        .await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await?
                .to_vec(),
        )?;
        assert_eq!(body, "Hello, bouzuya!");
        Ok(())
    }
}
