use std::collections::HashMap;

use axum::{
    extract::{Extension, Query},
    Json,
};
use serde_json::{json, Value};
use sqlx::Row;
use tracing::log::info;

use crate::context;

pub async fn get_dot_total(ctx: Extension<context::APIState>) -> Json<Value> {
    let row =
        sqlx::query("SELECT dot_total_balance from hedge_data_raw ORDER BY unix_time DESC LIMIT 1")
            .fetch_one(&ctx.db)
            .await
            .unwrap();

    let dot_total: f64 = row.try_get("dot_total_balance").unwrap();
    info!("dot total balance (DOT): {dot_total}");
    Json(json!({ "dot_total": dot_total }))
}

pub async fn get_dot_staked(ctx: Extension<context::APIState>) -> Json<Value> {
    let row = sqlx::query(
        "SELECT dot_staked_balance from hedge_data_raw ORDER BY unix_time DESC LIMIT 1",
    )
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    let dot_staked: f64 = row.try_get("dot_staked_balance").unwrap();
    info!("dot staked balance (DOT): {dot_staked}");
    Json(json!({ "dot_staked": dot_staked }))
}

pub async fn get_dot_reward(ctx: Extension<context::APIState>) -> Json<Value> {
    let row =
        sqlx::query("SELECT dot_total_rewards from hedge_data_raw ORDER BY unix_time DESC LIMIT 1")
            .fetch_one(&ctx.db)
            .await
            .unwrap();

    let dot_reward: f64 = row.try_get("dot_total_rewards").unwrap();
    info!("dot total reward (DOT): {dot_reward}");
    Json(json!({ "dot_reward": dot_reward }))
}

pub async fn get_dot_staked_history(ctx: Extension<context::APIState>) -> Json<Value> {
    struct StakedHistory {
        unix_time: i64,
        staked_balance: f64,
    }

    let mut history: Vec<StakedHistory>;

    let rows = sqlx::query("SELECT unix_time, dot_staked_balance from hedge_data_raw")
        .fetch_all(&ctx.db)
        .await
        .unwrap();
    for row in rows.iter() {}
}
