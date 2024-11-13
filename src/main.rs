mod route;
mod handler;
mod model;
mod schema;
mod error;

use std::sync::Arc;
use dotenv::dotenv;

use axum::{response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use route::create_router;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
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

    //let app_state = Arc::new(AppState { db: pool.clone() });
    // let app = Router::new()
    //     .route("/api/healthchecker", get(health_checker_handler))
    //     .with_state(app_state);
    let app_state: Arc<AppState> = Arc::new(AppState { db: pool.clone() });
    let app = create_router(app_state.clone());


    println!("--> {:12} - Started running server on port 8000", "INFO");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
