use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, services::ServeDir};

mod handlers;
mod models;
mod sui_client;

#[tokio::main]
async fn main() {
    println!("Starting Sui Readable server...");

    // Build our application router with routes
    let app = Router::new()
        // API routes
        .route("/api/explain", post(handlers::explain_transaction)) // POST endpoint for explaining
        .route("/api/health", get(handlers::health_check)) // GET endpoint for health
        .nest_service("/", ServeDir::new("static"))
        // Enable CORS so frontend can call our API
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on http://localhost:3000");
    println!("API endpoint: http://localhost:3000/api/explain");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
