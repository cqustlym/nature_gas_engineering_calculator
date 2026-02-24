use once_cell::sync::OnceCell;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::env;

pub static POOL: OnceCell<MySqlPool> = OnceCell::new();

pub async fn init_pool() -> Result<(), sqlx::Error> {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .max_connections(500)
        .connect(&url)
        .await?;
    POOL.set(pool).expect("Pool already initialized");
    Ok(())
}

pub fn get_pool() -> &'static MySqlPool {
    POOL.get().expect("Pool not initialized")
}
