use axum::{extract::Extension, Json};
use serde_json::{json, Value};
use sqlx::Row;
use tracing::log::info;

use crate::context;

// get aggregated PnL
pub async fn get_pnl(ctx: Extension<context::APIState>) -> Json<Value> {
    let rows = sqlx::query("SELECT * from hedge_data_derived")
        .fetch_all(&ctx.db)
        .await
        .unwrap();

    let mut actual_pnl: f64 = 0.0;

    for row in rows.iter() {
        let total_pnl: f64 = match row.try_get("total_pnl") {
            Ok(num) => num,
            Err(_) => 0.0,
        };
        actual_pnl += total_pnl;
    }

    info!("total pnl (USD): {actual_pnl}");
    Json(json!({ "pnl": actual_pnl }))
}

// get current total liquidation value
pub async fn get_total_liq(ctx: Extension<context::APIState>) -> Json<Value> {
    let row = sqlx::query(
        "SELECT total_liq_value from hedge_data_derived ORDER BY unix_time DESC LIMIT 1",
    )
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    let total_liq: f64 = row.try_get("total_liq_value").unwrap();

    info!("total liquidation value (USD): {total_liq}");
    Json(json!({ "total_liq_value": total_liq }))
}
