use std::vec;

use axum::{extract::Extension, Json};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use tracing::log::info;

use crate::context;

async fn get_raw_data(db: PgPool, col_name: &str, json_key: &str) -> Json<Value> {
    // :)))))))))))))))))))))))))
    let row = sqlx::query(&format!(
        "SELECT {col_name} from hedge_data_raw ORDER BY unix_time DESC LIMIT 1"
    ))
    .bind(col_name)
    .fetch_one(&db)
    .await
    .unwrap();

    let raw_data: f64 = row.try_get(col_name).unwrap();

    info!("fetched raw {json_key}: {raw_data}");

    Json(json!({ json_key: raw_data }))
}

async fn get_raw_history(db: PgPool, col_name: &str, json_key: &str) -> Json<Value> {
    let rows = sqlx::query(&format!("SELECT unix_time, {col_name} from hedge_data_raw"))
        .fetch_all(&db)
        .await
        .unwrap();

    let mut time_history: Vec<i64> = vec![];
    let mut data_history: Vec<f64> = vec![];

    for row in rows.iter() {
        let unix_time: i64 = row.try_get("unix_time").unwrap();
        let raw_data: f64 = row.try_get(col_name).unwrap();
        time_history.push(unix_time);
        data_history.push(raw_data);
    }

    info!("fetched raw {json_key} history");

    Json(json!({
        "unix_time": &time_history,
        json_key: &data_history
    }))
}

pub async fn get_dot_balance_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_data(ctx.db.clone(), "dot_total_balance", "dot_total").await
}

pub async fn get_dot_balance_history(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_history(ctx.db.clone(), "dot_total_balance", "dot_balance_total").await
}

pub async fn get_dot_staked_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_data(ctx.db.clone(), "dot_staked_balance", "dot_staked_total").await
}

pub async fn get_dot_staked_history(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_history(ctx.db.clone(), "dot_staked_balance", "dot_staked_total").await
}

pub async fn get_dot_reward_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_data(ctx.db.clone(), "dot_total_rewards", "dot_reward_total").await
}

pub async fn get_dot_reward_history(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_history(ctx.db.clone(), "dot_total_rewards", "dot_reward_total").await
}
