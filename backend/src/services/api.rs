use actix_web::{web, App, HttpResponse, HttpServer, middleware, Result};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;

// Configuration
#[derive(Clone)]
pub struct Config {
    // Alpaca configuration
    pub alpaca_api_key: String,
    pub alpaca_secret_key: String,
    pub alpaca_base_url: String,
    
    // OKX configuration
    pub okx_api_key: String,
    pub okx_secret_key: String,
    pub okx_passphrase: String,
    pub okx_project_id: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            alpaca_api_key: std::env::var("ALPACA_API_KEY_ID").expect("ALPACA_API_KEY_ID must be set"),
            alpaca_secret_key: std::env::var("ALPACA_API_SECRET_KEY").expect("ALPACA_API_SECRET_KEY must be set"),
            alpaca_base_url: std::env::var("ALPACA_API_BASE_URL")
                .unwrap_or_else(|_| "https://paper-api.alpaca.markets".to_string()),
            
            okx_api_key: std::env::var("OKX_API_KEY").expect("OKX_API_KEY must be set"),
            okx_secret_key: std::env::var("OKX_SECRET_KEY").expect("OKX_SECRET_KEY must be set"),
            okx_passphrase: std::env::var("OKX_API_PASSPHRASE").expect("OKX_API_PASSPHRASE must be set"),
            okx_project_id: std::env::var("OKX_PROJECT_ID").expect("OKX_PROJECT_ID must be set"),
        }
    }
}

// Request/Response structures
#[derive(Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price: String,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct SwapRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub chain_id: String,
}

#[derive(Debug, Deserialize)]
pub struct StockOrderRequest {
    pub symbol: String,
    pub notional: String, // USD amount
}

// API handlers
pub async fn get_stock_price(
    config: web::Data<Config>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let symbol = path.into_inner();
    let client = Client::new();
    
    let url = format!("{}/v2/stocks/{}/trades/latest", config.alpaca_base_url, symbol);
    
    let response = client
        .get(&url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if !response.status().is_success() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Failed to fetch stock price"
        })));
    }
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if let Some(trade) = data.get("trade") {
        if let Some(price) = trade.get("p") {
            return Ok(HttpResponse::Ok().json(PriceResponse {
                symbol,
                price: price.to_string(),
                timestamp: Utc::now().timestamp(),
            }));
        }
    }
    
    Ok(HttpResponse::NotFound().json(serde_json::json!({
        "error": "Price not found"
    })))
}

pub async fn get_crypto_price(
    config: web::Data<Config>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let chain_id = query.get("chainId").unwrap_or(&"1".to_string()).clone();
    let token_address = query.get("tokenAddress").ok_or_else(|| {
        actix_web::error::ErrorBadRequest("tokenAddress is required")
    })?;
    
    let client = Client::new();
    let timestamp = Utc::now().to_rfc3339();
    
    // OKX Market API endpoint for token price
    let endpoint = "/api/v5/dex/market/token-price-multi-chain";
    let params = format!("?chainId={}&tokenAddresses={}", chain_id, token_address);
    let full_path = format!("{}{}", endpoint, params);
    
    // Sign the request
    let sign_message = format!("{}GET{}", timestamp, full_path);
    let mut mac = Hmac::<Sha256>::new_from_slice(config.okx_secret_key.as_bytes())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    mac.update(sign_message.as_bytes());
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    
    let url = format!("https://www.okx.com{}", full_path);
    
    let response = client
        .get(&url)
        .header("OK-ACCESS-KEY", &config.okx_api_key)
        .header("OK-ACCESS-SIGN", signature)
        .header("OK-ACCESS-TIMESTAMP", timestamp)
        .header("OK-ACCESS-PASSPHRASE", &config.okx_passphrase)
        .header("OK-ACCESS-PROJECT-ID", &config.okx_project_id)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if !response.status().is_success() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Failed to fetch crypto price"
        })));
    }
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn get_swap_quote(
    config: web::Data<Config>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let client = Client::new();
    let timestamp = Utc::now().to_rfc3339();
    
    // Build query parameters
    let params = serde_urlencoded::to_string(&query.into_inner())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let endpoint = "/api/v5/dex/aggregator/quote";
    let full_path = format!("{}?{}", endpoint, params);
    
    // Sign the request
    let sign_message = format!("{}GET{}", timestamp, full_path);
    let mut mac = Hmac::<Sha256>::new_from_slice(config.okx_secret_key.as_bytes())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    mac.update(sign_message.as_bytes());
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    
    let url = format!("https://www.okx.com{}", full_path);
    
    let response = client
        .get(&url)
        .header("OK-ACCESS-KEY", &config.okx_api_key)
        .header("OK-ACCESS-SIGN", signature)
        .header("OK-ACCESS-TIMESTAMP", timestamp)
        .header("OK-ACCESS-PASSPHRASE", &config.okx_passphrase)
        .header("OK-ACCESS-PROJECT-ID", &config.okx_project_id)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn swap_crypto_to_usdt(
    config: web::Data<Config>,
    swap_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let client = Client::new();
    let timestamp = Utc::now().to_rfc3339();
    
    let endpoint = "/api/v5/dex/aggregator/swap";
    let body = swap_data.to_string();
    
    // Sign the request
    let sign_message = format!("{}POST{}{}", timestamp, endpoint, body);
    let mut mac = Hmac::<Sha256>::new_from_slice(config.okx_secret_key.as_bytes())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    mac.update(sign_message.as_bytes());
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    
    let url = format!("https://www.okx.com{}", endpoint);
    
    let response = client
        .post(&url)
        .header("OK-ACCESS-KEY", &config.okx_api_key)
        .header("OK-ACCESS-SIGN", signature)
        .header("OK-ACCESS-TIMESTAMP", timestamp)
        .header("OK-ACCESS-PASSPHRASE", &config.okx_passphrase)
        .header("OK-ACCESS-PROJECT-ID", &config.okx_project_id)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn buy_stock_with_usdt(
    config: web::Data<Config>,
    order: web::Json<StockOrderRequest>,
) -> Result<HttpResponse> {
    let client = Client::new();
    
    let order_data = serde_json::json!({
        "symbol": order.symbol,
        "notional": order.notional,
        "side": "buy",
        "type": "market",
        "time_in_force": "day"
    });
    
    let url = format!("{}/v2/orders", config.alpaca_base_url);
    
    let response = client
        .post(&url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .header("Content-Type", "application/json")
        .json(&order_data)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn get_account_info(
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let client = Client::new();
    
    let url = format!("{}/v2/account", config.alpaca_base_url);
    
    let response = client
        .get(&url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn get_positions(
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let client = Client::new();
    
    let url = format!("{}/v2/positions", config.alpaca_base_url);
    
    let response = client
        .get(&url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn get_token_list(
    config: web::Data<Config>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let chain_id = query.get("chainId").unwrap_or(&"1".to_string()).clone();
    
    let client = Client::new();
    let timestamp = Utc::now().to_rfc3339();
    
    let endpoint = "/api/v5/dex/aggregator/all-tokens";
    let params = format!("?chainId={}", chain_id);
    let full_path = format!("{}{}", endpoint, params);
    
    // Sign the request
    let sign_message = format!("{}GET{}", timestamp, full_path);
    let mut mac = Hmac::<Sha256>::new_from_slice(config.okx_secret_key.as_bytes())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    mac.update(sign_message.as_bytes());
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    
    let url = format!("https://www.okx.com{}", full_path);
    
    let response = client
        .get(&url)
        .header("OK-ACCESS-KEY", &config.okx_api_key)
        .header("OK-ACCESS-SIGN", signature)
        .header("OK-ACCESS-TIMESTAMP", timestamp)
        .header("OK-ACCESS-PASSPHRASE", &config.okx_passphrase)
        .header("OK-ACCESS-PROJECT-ID", &config.okx_project_id)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn get_stock_list(
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let client = Client::new();
    
    let url = format!("{}/v2/assets?status=active&asset_class=us_equity", config.alpaca_base_url);
    
    let response = client
        .get(&url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

// Main function
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let config = web::Data::new(Config::from_env());
    
    println!("Starting server at http://127.0.0.1:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", "*"))
                    .add(("Access-Control-Allow-Methods", "GET, POST, OPTIONS"))
                    .add(("Access-Control-Allow-Headers", "Content-Type"))
            )
            // Stock endpoints
            .route("/api/stock/price/{symbol}", web::get().to(get_stock_price))
            .route("/api/stock/list", web::get().to(get_stock_list))
            .route("/api/stock/buy", web::post().to(buy_stock_with_usdt))
            .route("/api/account", web::get().to(get_account_info))
            .route("/api/positions", web::get().to(get_positions))
            
            // Crypto endpoints
            .route("/api/crypto/price", web::get().to(get_crypto_price))
            .route("/api/crypto/quote", web::get().to(get_swap_quote))
            .route("/api/crypto/swap", web::post().to(swap_crypto_to_usdt))
            .route("/api/crypto/tokens", web::get().to(get_token_list))
            
            // Health check
            .route("/health", web::get().to(|| async { 
                HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy"
                }))
            }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}