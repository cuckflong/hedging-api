use axum::{
    extract::{Extension, Query},
    Json,
};
use chrono::NaiveDateTime;
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use tracing::log::info;

use crate::context::{self, HistoryParams};

use super::raw;

async fn get_derived_data(db: PgPool, col_name: &str) -> f64 {
    // :)))))))))))))))))))))))))
    let row = sqlx::query(&format!(
        "SELECT {col_name} from hedge_data_derived ORDER BY unix_time DESC LIMIT 1"
    ))
    .bind(col_name)
    .fetch_one(&db)
    .await
    .unwrap();

    let raw_data: f64 = row.try_get(col_name).unwrap();

    raw_data
}

async fn get_derived_data_json(db: PgPool, col_name: &str, json_key: &str) -> Json<Value> {
    let raw_data: f64 = get_derived_data(db, col_name).await;

    info!("fetched derived {json_key}: {raw_data}");

    Json(json!({ json_key: raw_data }))
}

async fn get_derived_history(db: PgPool, col_name: &str) -> (Vec<i64>, Vec<f64>) {
    let rows = sqlx::query(&format!(
        "SELECT unix_time, {col_name} from hedge_data_derived ORDER BY unix_time"
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

async fn get_derived_history_json(
    db: PgPool,
    col_name: &str,
    json_key: &str,
    min_diff: Option<f64>,
    min_offset: Option<f64>,
) -> Json<Value> {
    let (time_history, data_history) = get_derived_history(db, col_name).await;

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

        if i != 0 && (data_history[i] - data_history[i - 1]).abs() > 100.0 {
            continue;
        }

        time_history_new.push(time_history[i]);
        data_history_new.push(data_history[i]);
    }

    Json(json!({
        "unix_time": &time_history_new,
        json_key: &data_history_new
    }))
}

async fn calc_timespan(db: PgPool) -> i64 {
    let mut row = sqlx::query(&format!("SELECT MIN(unix_time) from hedge_data_derived"))
        .fetch_one(&db)
        .await
        .unwrap();

    let start: i64 = row.try_get("min").unwrap();
    let start_time = NaiveDateTime::from_timestamp(start, 0);

    row = sqlx::query(&format!("SELECT MAX(unix_time) from hedge_data_derived"))
        .fetch_one(&db)
        .await
        .unwrap();

    let end: i64 = row.try_get("max").unwrap();
    let end_time = NaiveDateTime::from_timestamp(end, 0);

    (end_time - start_time).num_days()
}

async fn calc_pnl_aggregated(db: PgPool) -> f64 {
    let rows = sqlx::query("SELECT * from hedge_data_derived")
        .fetch_all(&db)
        .await
        .unwrap();

    let mut aggregated_pnl: f64 = 0.0;

    for row in rows.iter() {
        let pnl: f64 = match row.try_get("pnl") {
            Ok(num) => num,
            Err(_) => 0.0,
        };
        aggregated_pnl += pnl;
    }

    aggregated_pnl
}

// get aggregated PnL
pub async fn get_pnl_aggregated(ctx: Extension<context::APIState>) -> Json<Value> {
    let aggregated_pnl = calc_pnl_aggregated(ctx.db.clone()).await;

    info!("aggregated pnl (USD): {aggregated_pnl}");
    Json(json!({ "pnl_aggregated": aggregated_pnl }))
}

// get current total liquidation value
pub async fn get_liquid_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_derived_data_json(ctx.db.clone(), "total_liquid_value", "liquid_total").await
}

pub async fn get_liquid_history(
    ctx: Extension<context::APIState>,
    history_params: Query<HistoryParams>,
) -> Json<Value> {
    get_derived_history_json(
        ctx.db.clone(),
        "total_liquid_value",
        "liquid_total",
        history_params.min_diff,
        history_params.min_offset,
    )
    .await
}

pub async fn get_margin_ratio(ctx: Extension<context::APIState>) -> Json<Value> {
    get_derived_data_json(ctx.db.clone(), "margin_ratio", "margin_level").await
}

pub async fn get_staked_ratio(ctx: Extension<context::APIState>) -> Json<Value> {
    get_derived_data_json(ctx.db.clone(), "staked_ratio", "staked_ratio").await
}

pub async fn get_swap_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_derived_data_json(ctx.db.clone(), "pps_total_swap", "total_swap").await
}

pub async fn get_cost_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_derived_data_json(ctx.db.clone(), "total_cost", "total_cost").await
}

pub async fn get_net_exposure(ctx: Extension<context::APIState>) -> Json<Value> {
    get_derived_data_json(ctx.db.clone(), "dot_net_position", "net_exposure").await
}

pub async fn get_net_exposure_history(
    ctx: Extension<context::APIState>,
    history_params: Query<HistoryParams>,
) -> Json<Value> {
    get_derived_history_json(
        ctx.db.clone(),
        "dot_net_position",
        "net_exposure",
        history_params.min_diff,
        history_params.min_offset,
    )
    .await
}

pub async fn get_pnl_total(ctx: Extension<context::APIState>) -> Json<Value> {
    let total_cost: f64 = get_derived_data(ctx.db.clone(), "total_cost").await;
    let total_liquid_value: f64 = get_derived_data(ctx.db.clone(), "total_liquid_value").await;
    let realized_pnl: f64 = raw::get_raw_data(ctx.db.clone(), "pps_realized_pnl").await;
    let closed_swap: f64 = raw::get_raw_data(ctx.db.clone(), "pps_closed_swao").await;

    Json(json!({
        "pnl_total": total_liquid_value - total_cost + realized_pnl + closed_swap
    }))
}

pub async fn get_pnl_apr(ctx: Extension<context::APIState>) -> Json<Value> {
    let total_cost: f64 = get_derived_data(ctx.db.clone(), "total_cost").await;
    let aggregated_pnl = calc_pnl_aggregated(ctx.db.clone()).await;
    let days: i64 = calc_timespan(ctx.db.clone()).await;

    let apr: f64 = ((aggregated_pnl / days as f64) * 365.0) / total_cost * 100.0;

    Json(json!({ "apr": apr }))
}

pub async fn get_pnl_history(
    ctx: Extension<context::APIState>,
    history_params: Query<HistoryParams>,
) -> Json<Value> {
    get_derived_history_json(
        ctx.db.clone(),
        "pnl",
        "pnl",
        history_params.min_diff,
        history_params.min_offset,
    )
    .await
}

pub async fn get_swap_history(
    ctx: Extension<context::APIState>,
    history_params: Query<HistoryParams>,
) -> Json<Value> {
    get_derived_history_json(
        ctx.db.clone(),
        "pps_total_swap",
        "pps_total_swap",
        history_params.min_diff,
        history_params.min_offset,
    )
    .await
}
