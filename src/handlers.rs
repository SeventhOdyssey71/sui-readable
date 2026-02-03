use crate::models::{ExplainRequest, ExplainResponse};
use crate::sui_client::SuiClient;
use axum::{Json, http::StatusCode};

// Handle POST /api/explain requests

//This function receives a transaction digest from the user, uses SuiClient to fetch and explain it and returns the explanation as JSON.

pub async fn explain_transaction(
    Json(payload): Json<ExplainRequest>, // Automatically parse JSON body
) -> (StatusCode, Json<ExplainResponse>) {
    println!("Explaining transaction: {}", payload.digest);

    // Create a new Sui client
    let client = match SuiClient::new().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create Sui client: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExplainResponse {
                    success: false,
                    explanation: None,
                    error: Some(format!("Failed to connect to Sui: {}", e)),
                }),
            );
        }
    };

    // Fetch and explain the transaction
    match client.explain_transaction(&payload.digest).await {
        Ok(explanation) => {
            println!("Successfully explained transaction");
            (
                StatusCode::OK,
                Json(ExplainResponse {
                    success: true,
                    explanation: Some(explanation),
                    error: None,
                }),
            )
        }
        Err(e) => {
            eprintln!("Failed to explain transaction: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ExplainResponse {
                    success: false,
                    explanation: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

/// Health check endpoint - just returns OK
pub async fn health_check() -> &'static str {
    "OK"
}
