use axum::{routing::get, Json, Router};
use std::env;
use std::fs;
use tokio::net::TcpListener;
use tokio::signal;
use tonic::transport::Server;
use tower_http::cors::CorsLayer;

mod grpc;
use crate::grpc::greeter_service;
use grpc::counter_service;

async fn health_check() -> Json<&'static str> {
    Json("API is up and running!")
}

fn get_database_url() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER not set");
    let password_file = env::var("POSTGRES_PASSWORD_FILE").expect("POSTGRES_PASSWORD_FILE not set");
    let password =
        fs::read_to_string(password_file).unwrap_or_else(|_| panic!("Could not read password"));
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST not set");
    let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT not set");

    format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, db_name
    )
}

async fn connect_to_database() -> Result<sqlx::PgPool, sqlx::Error> {
    let mut retries = 5;

    while retries > 0 {
        match sqlx::PgPool::connect(&get_database_url()).await {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                eprintln!("Database connection failed: {}. Retrying...", e);
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }

    Err(sqlx::Error::Configuration(
        "Failed to connect to database".into(),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting server");

    println!("Connecting to database");
    let pool = connect_to_database().await?;

    println!("Running database migrations");
    sqlx::migrate!().run(&pool).await?;

    println!("Starting REST service");
    let rest_app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());
    let rest_listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("REST service is listening at localhost:3000");
    let rest_server = tokio::spawn(async move {
        if let Err(e) = axum::serve(rest_listener, rest_app.into_make_service()).await {
            eprintln!("REST server error: {}", e);
        }
    });

    println!("Starting gRPc service");
    let grpc_listener = TcpListener::bind("0.0.0.0:50051").await?;
    println!("gRPC service is listening at localhost:50051");
    let counter = counter_service(pool.clone()).await;
    let greeter = greeter_service().await;
    let grpc_server = Server::builder()
        .add_service(counter)
        .add_service(greeter)
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(grpc_listener),
            shutdown_signal(), // Listen for SIGINT
        );

    tokio::select! {
        _ = rest_server => {},
        result = grpc_server => {
            if let Err(e) = result {
                eprintln!("gRPC server error: {}", e);
            }
        }
    }

    println!("Closing database connection");
    pool.close().await;

    println!("Service shutting down");
    Ok(())
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");
    println!("Received shutdown signal");
}
