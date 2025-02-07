use axum::{routing::get, Router, Json};
use tonic::transport::Server;
use tower_http::cors::CorsLayer;
use tokio::net::TcpListener;
use tokio::signal;


mod grpc;
use grpc::grpc_router;

async fn health_check() -> Json<&'static str> {
    Json("API is up and running!")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rest_app = Router::new().route("/health", get(health_check)).layer(CorsLayer::permissive());

    let grpc_service = grpc_router();

    let rest_listener = TcpListener::bind("0.0.0.0:3000").await?;
    let grpc_listener = TcpListener::bind("0.0.0.0:50051").await?;

    println!("Axum REST API running on localhost:3000");
    println!("gRPC Server running on localhost:50051");

    // Spawn REST server in a separate task
    let rest_server = tokio::spawn(async move {
        if let Err(e) = axum::serve(rest_listener, rest_app.into_make_service()).await {
            eprintln!("REST server error: {}", e);
        }
    });

    // gRPC server with shutdown signal
    let grpc_server = Server::builder()
        .add_service(grpc_service)
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(grpc_listener),
            shutdown_signal(),  // Listen for SIGINT
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

    println!("Servers shutting down.");
    Ok(())
}

// Graceful shutdown handler
async fn shutdown_signal() {
    signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
    println!("Received shutdown signal. Stopping servers...");
}