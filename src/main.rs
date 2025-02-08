use axum::{routing::get, Json, Router};
use std::env;
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
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        // Retrieve environment variables
        let user = env::var("POSTGRES_USER").expect("POSTGRES_USER not set");
        let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD not set");
        let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
        let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST not set");
        let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT not set");

        // Construct the database URL
        format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, db_name
        )

        // Default to the Docker database instance URL for local development
        // String::from("postgres://myuser:mypassword@localhost:5432/mydatabase")
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rest_app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    let rest_listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("REST API is listening at localhost:3000");

    let grpc_listener = TcpListener::bind("0.0.0.0:50051").await?;
    println!("gRPC API is listening at localhost:50051");

    // Spawn REST server in a separate task
    let rest_server = tokio::spawn(async move {
        if let Err(e) = axum::serve(rest_listener, rest_app.into_make_service()).await {
            eprintln!("REST server error: {}", e);
        }
    });

    let db = sqlx::PgPool::connect(&*get_database_url()).await?;
    let counter = counter_service(db.clone()).await;

    let greeter = greeter_service().await;

    // gRPC server with shutdown signal
    let grpc_server = Server::builder()
        .add_service(counter)
        .add_service(greeter)
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(grpc_listener),
            shutdown_signal(), // Listen for SIGINT
        );

    // Wait for both servers to finish
    tokio::select! {
        _ = rest_server => {},
        result = grpc_server => {
            if let Err(e) = result {
                eprintln!("gRPC server error: {}", e);
            }
        }
    }

    println!("Closing database connection");
    db.close().await;

    println!("Sevice shutting down");
    Ok(())
}

// Graceful shutdown handler
async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");
    println!("Received shutdown signal");
}
