use std::net::SocketAddr;

use axum::{extract::Extension, routing::get, Router};
use sqlx::postgres::PgPoolOptions;

use tower_http::trace::TraceLayer;

use tracing::log::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::db_query::derived;
use crate::db_query::raw;

mod db_query;

mod context;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let postgres_url = dotenv::var("POSTGRES_URL").unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "hedging_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await
        .expect("error connecting Postgres");

    let app = Router::new()
        .route("/pnl/total", get(derived::get_pnl_total))
        .route("/liquid/total", get(derived::get_liquid_total))
        .route("/liquid/total/history", get(derived::get_liquid_history))
        .route("/dot/balance", get(raw::get_dot_balance_total))
        .route("/dot/balance/history", get(raw::get_dot_balance_history))
        .route("/dot/staked", get(raw::get_dot_staked_total))
        .route("/dot/staked/history", get(raw::get_dot_staked_history))
        .route("/dot/reward", get(raw::get_dot_reward_total))
        .route("/dot/reward/history", get(raw::get_dot_reward_history))
        .layer(Extension(context::APIState { db: pool }))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("error running HTTP server")
}
