use serde::Deserialize;
use sqlx::PgPool;

#[derive(Clone)]
pub struct APIState {
    pub db: PgPool,
}

#[derive(Deserialize)]
pub struct HistoryParams {
    pub min_diff: Option<f64>,
    pub min_offset: Option<f64>,
}
