use axum::{
    extract::Extension,
    routing::{get, post, put, delete},
    Router
};

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::fs;
use anyhow::Context;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod errors;
mod models;
mod views;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = fs::read_to_string(".env").unwrap();
    let (_, database_url) = env.split_once("=").unwrap();

    tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(
        std::env::var("tower_http=trace")
        .unwrap_or_else(|_| "example_tracing_aka_logging=debug, tower_http=debug".into()),
    ))
    .with(tracing_subscriber::fmt::layer())
    .init();

    let pool = PgPoolOptions::new()
    .max_connections(50)
    .connect(&database_url)
    .await
    .context("Could not connect to the database_url")?;

    let app = Router::new()
                .route("/profiles", get(views::all_profiles))
                .route("/profile/:id", get(views::profile))
                .route("/profile", post(views::post_profile))
                .route("/profile/:id", put(views::update_profile))
                .route("/profile/:id", delete(views::delete_profile))
                .layer(Extension(pool))
                .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Listening to {addr:?}");

    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();

    Ok(())

}
