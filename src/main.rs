use axum::{routing::get, Router, Json};
use tonic::transport::Server;
use tower_http::cors::CorsLayer;
use tokio::net::TcpListener;


mod grpc;
use grpc::grpc_router;

async fn health_check() -> Json<&'static str> {
    Json("API is up and running!")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rest_app = Router::new().route("/health", get(health_check)).layer(CorsLayer::permissive());

    let grpc_service = grpc_router();

    let rest_listener = TcpListener::bind("127.0.0.1:3000").await?;
    let grpc_listener = TcpListener::bind("127.0.0.1:50051").await?;

    println!("Axum REST API running on localhost:3000");
    println!("gRPC Server running on localhost:50051");

    tokio::spawn(async move {
        axum::serve(rest_listener, rest_app.into_make_service()).await.expect("failed to start REST server");
    });

    Server::builder()
        .add_service(grpc_service)
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(grpc_listener))
        .await?;

    Ok(())
}