use axum::{extract::Extension, Json};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use tracing::log::info;

use crate::context;

async fn get_derived_data(db: PgPool, col_name: &str, json_key: &str) -> Json<Value> {
    // :)))))))))))))))))))))))))
    let row = sqlx::query(&format!(
        "SELECT {col_name} from hedge_data_derived ORDER BY unix_time DESC LIMIT 1"
    ))
    .bind(col_name)
    .fetch_one(&db)
    .await
    .unwrap();

    let raw_data: f64 = row.try_get(col_name).unwrap();

    info!("fetched derived {json_key}: {raw_data}");

    Json(json!({ json_key: raw_data }))
}

async fn get_derived_history(db: PgPool, col_name: &str, json_key: &str) -> Json<Value> {
    let rows = sqlx::query(&format!(
        "SELECT unix_time, {col_name} from hedge_data_derived"
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

    info!("fetched derived {json_key} history");

    Json(json!({
        "unix_time": &time_history,
        json_key: &data_history
    }))
}

// get aggregated PnL
pub async fn get_pnl_aggregated(ctx: Extension<context::APIState>) -> Json<Value> {
    let rows = sqlx::query("SELECT * from hedge_data_derived")
        .fetch_all(&ctx.db)
        .await
        .unwrap();

    let mut aggregated_pnl: f64 = 0.0;

    for row in rows.iter() {
        let total_pnl: f64 = match row.try_get("total_pnl") {
            Ok(num) => num,
            Err(_) => 0.0,
        };
        aggregated_pnl += total_pnl;
    }

    info!("aggregated pnl (USD): {aggregated_pnl}");
    Json(json!({ "pnl_aggregated": aggregated_pnl }))
}

// get current total liquidation value
pub async fn get_liquid_total(ctx: Extension<context::APIState>) -> Json<Value> {
    get_derived_data(ctx.db.clone(), "total_liq_value", "liquid_total").await
}

pub async fn get_liquid_history(ctx: Extension<context::APIState>) -> Json<Value> {
    get_derived_history(ctx.db.clone(), "total_liq_value", "liquid_total").await
}
