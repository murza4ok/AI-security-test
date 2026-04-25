mod webapp;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use webapp::{handlers::router, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState::load_from_repo().context("failed to load web target state")?);
    let app = router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("web target listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
