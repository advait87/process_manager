#![allow(unused)]

use axum::{
    extract:: { Json, State },
    handler::Handler,
    http::StatusCode,
    Router,
    routing:: { get, post },
    response::IntoResponse
};

use serde::{ Deserialize, Serialize };
use sqlx::{ PgPool, postgres::PgPoolOptions, query };
use tower_http::cors::CorsLayer;

pub mod process;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::permissive();
    println!("Hello, world!");
    let app = axum::Router::new()
        .route("/", get("Hello, world!"))
        .route("/select-node", post(select_node))
        .layer(cors)
        ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn select_node(Json(select_node_request): Json<SelectNodeRequest>) -> Result<SelectNodeResponse, SelectNodeError> {
    Err(SelectNodeError::NoNodesOnline)
}

enum SelectNodeError {
    NoNodesOnline
}

#[derive(Debug, Deserialize, Serialize)]
struct SelectNodeRequest {
    client_hash: String,
    process_requirements: process::ProcessRequirements
}

async fn setup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Idk if I should store the hash of the client device whose process is currently running on the
    // node, but for now I'll do it
    query(
        "CREATE TABLE IF NOT EXISTS nodes(
            hash VARCHAR(255) PRIMARY KEY,
            is_online BOOLEAN NOT NULL DEFAULT FALSE,
            is_running BOOLEAN NOT NULL DEFAULT FALSE,
            running_process_hash VARCHAR(255),
            running_client_hash VARCHAR(255)
        )"
    )
        .execute(pool)
        .await?;

    query(
        "CREATE TABLE IF NOT EXISTS clients(
            hash VARCHAR(255) PRIMARY KEY,
            is_running BOOLEAN NOT NULL DEFAULT FALSE,
            running_node_hash VARCHAR(255)
        )"
    )
        .execute(pool)
        .await?;
    Ok(())
}



