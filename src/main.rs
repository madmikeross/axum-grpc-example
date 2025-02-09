use axum::{routing::get, Json, Router};
use tokio::net::TcpListener;
use tokio::signal;
use tonic::transport::Server;
use tower_http::cors::CorsLayer;

mod grpc;
mod postgres;

async fn health_check() -> Json<&'static str> {
    Json("API is up and running!")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting server");

    println!("Connecting to database");
    let pool = postgres::connection_pool().await?;

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
    let counter = grpc::counter_service(pool.clone()).await;
    let grpc_server = Server::builder()
        .add_service(counter)
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(grpc_listener),
            shutdown_signal(),
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
