use postgresql_with_axum::main::AppState;
use postgresql_with_axum::route::create_router;

use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;


#[tokio::test]
async fn test_create_note_success() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("--> {:12} - Connected to database", "INFO");
            pool
        }
        Err(err) => {
            println!("--> {:12} - Failed to connect to database: {}", "ERROR", err);
            std::process::exit(1);
        }
    };
    
    let app_state = Arc::new(AppState { db: pool.clone()});
    let app = create_router(app_state.clone());

}