use std::vec;

use axum::{
    extract::{Extension, Query},
    Json,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use tracing::log::info;

use crate::context::{self, HistoryParams};

pub async fn get_raw_data(db: PgPool, col_name: &str) -> f64 {
    // :)))))))))))))))))))))))))
    let row = sqlx::query(&format!(
        "SELECT {col_name} from hedge_data_raw ORDER BY unix_time DESC LIMIT 1"
    ))
    .bind(col_name)
    .fetch_one(&db)
    .await
    .unwrap();

    let raw_data: f64 = row.try_get(col_name).unwrap();

    raw_data
}

async fn get_raw_data_json(db: PgPool, col_name: &str, json_key: &str) -> Json<Value> {
    let raw_data: f64 = get_raw_data(db, col_name).await;

    info!("fetched derived {json_key}: {raw_data}");

    Json(json!({ json_key: raw_data }))
}

async fn get_raw_history(db: PgPool, col_name: &str) -> (Vec<i64>, Vec<f64>) {
    let rows = sqlx::query(&format!(
        "SELECT unix_time, {col_name} from hedge_data_raw ORDER BY unix_time"
    ))
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

    (time_history, data_history)
}

async fn get_raw_history_json(
    db: PgPool,
    col_name: &str,
    json_key: &str,
    min_diff: Option<f64>,
    min_offset: Option<f64>,
) -> Json<Value> {
    let (time_history, data_history) = get_raw_history(db, col_name).await;

    info!("fetched derived {json_key} history");

    let mut time_history_new: Vec<i64> = vec![];
    let mut data_history_new: Vec<f64> = vec![];

    for (i, _) in time_history.iter().enumerate() {
        match min_diff {
            Some(min_diff) => {
                if i > 0 && (data_history[i] - data_history[i - 1]).abs() < min_diff {
                    continue;
                }
            }
            None => (),
        }

        match min_offset {
            Some(min_offset) => {
                if data_history[i].abs() < min_offset {
                    continue;
                }
            }
            None => (),
        }

        time_history_new.push(time_history[i]);
        data_history_new.push(data_history[i]);
    }

    Json(json!({
        "unix_time": &time_history_new,
        json_key: &data_history_new
    }))
}

pub async fn get_dot_balance_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_data_json(ctx.db.clone(), "dot_total_balance", "dot_total").await
}

pub async fn get_dot_balance_history(
    ctx: Extension<context::APIState>,
    history_params: Query<HistoryParams>,
) -> Json<Value> {
    get_raw_history_json(
        ctx.db.clone(),
        "dot_total_balance",
        "dot_balance_total",
        history_params.min_diff,
        history_params.min_offset,
    )
    .await
}

pub async fn get_dot_staked_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_data_json(ctx.db.clone(), "dot_staked_balance", "dot_staked_total").await
}

pub async fn get_pps_realized_pnl(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_data_json(ctx.db.clone(), "pps_realized_pnl", "pps_realized_pnl").await
}

pub async fn get_dot_staked_history(
    ctx: Extension<context::APIState>,
    history_params: Query<HistoryParams>,
) -> Json<Value> {
    get_raw_history_json(
        ctx.db.clone(),
        "dot_staked_balance",
        "dot_staked_total",
        history_params.min_diff,
        history_params.min_offset,
    )
    .await
}

pub async fn get_dot_reward_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_raw_data_json(ctx.db.clone(), "dot_total_rewards", "dot_reward_total").await
}

pub async fn get_dot_reward_history(
    ctx: Extension<context::APIState>,
    history_params: Query<HistoryParams>,
) -> Json<Value> {
    get_raw_history_json(
        ctx.db.clone(),
        "dot_total_rewards",
        "dot_reward_total",
        history_params.min_diff,
        history_params.min_offset,
    )
    .await
}
