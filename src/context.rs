use sqlx::PgPool;

#[derive(Clone)]
pub struct APIState {
    pub db: PgPool,
}
