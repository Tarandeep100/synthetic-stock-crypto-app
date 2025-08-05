use actix_web::{web, App, HttpResponse, HttpServer, middleware, Result};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use dotenv;

// mod services;
// use services::solana_service::SolanaService;

// Configuration
#[derive(Clone)]
pub struct Config {
    // Alpaca configuration
    pub alpaca_api_key: String,
    pub alpaca_secret_key: String,
    pub alpaca_base_url: String,
    pub alpaca_data_url: String,
    
    // OKX configuration
    pub okx_api_key: String,
    pub okx_secret_key: String,
    pub okx_passphrase: String,
    pub okx_project_id: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            alpaca_api_key: std::env::var("ALPACA_API_KEY_ID").unwrap_or_else(|_| "test_key".to_string()),
            alpaca_secret_key: std::env::var("ALPACA_API_SECRET_KEY").unwrap_or_else(|_| "test_secret".to_string()),
            alpaca_base_url: std::env::var("ALPACA_API_BASE_URL")
                .unwrap_or_else(|_| "https://paper-api.alpaca.markets".to_string()),
            alpaca_data_url: std::env::var("ALPACA_API_DATA_URL")
                .unwrap_or_else(|_| "https://data.alpaca.markets".to_string()),
            okx_api_key: std::env::var("OKX_API_KEY").unwrap_or_else(|_| "test_key".to_string()),
            okx_secret_key: std::env::var("OKX_SECRET_KEY").unwrap_or_else(|_| "test_secret".to_string()),
            okx_passphrase: std::env::var("OKX_API_PASSPHRASE").unwrap_or_else(|_| "test_passphrase".to_string()),
            okx_project_id: std::env::var("OKX_PROJECT_ID").unwrap_or_else(|_| "test_project".to_string()),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopStock {
    pub symbol: String,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub volume: Option<u64>,
    pub market_cap: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopStocksResponse {
    pub most_active: Vec<TopStock>,
    pub top_gainers: Vec<TopStock>,
    pub top_losers: Vec<TopStock>,
    pub updated_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CachedData {
    pub data: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize)]
pub struct StockOrderRequest {
    pub symbol: String,
    pub notional: String, // USD amount
}

// Cache helper functions
const CACHE_DURATION_HOURS: u64 = 24;
const PRICE_CACHE_DURATION_HOURS: u64 = 1; // Shorter cache for prices
const CACHE_DIR: &str = "cache";

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn is_cache_valid(cache_timestamp: u64) -> bool {
    let current_time = get_current_timestamp();
    let cache_age_hours = (current_time - cache_timestamp) / 3600;
    cache_age_hours < CACHE_DURATION_HOURS
}

fn is_price_cache_valid(cache_timestamp: u64) -> bool {
    let current_time = get_current_timestamp();
    let cache_age_hours = (current_time - cache_timestamp) / 3600;
    cache_age_hours < PRICE_CACHE_DURATION_HOURS
}

fn get_cache_file_path(cache_key: &str) -> String {
    format!("{}/{}.json", CACHE_DIR, cache_key)
}

fn read_cache(cache_key: &str) -> Option<CachedData> {
    let cache_path = get_cache_file_path(cache_key);
    
    if !Path::new(&cache_path).exists() {
        return None;
    }
    
    match fs::read_to_string(&cache_path) {
        Ok(content) => {
            match serde_json::from_str::<CachedData>(&content) {
                Ok(cached_data) => {
                    if is_cache_valid(cached_data.timestamp) {
                        println!("✅ Cache hit for {}: {} hours old", cache_key, (get_current_timestamp() - cached_data.timestamp) / 3600);
                        Some(cached_data)
                    } else {
                        println!("⏰ Cache expired for {}: {} hours old", cache_key, (get_current_timestamp() - cached_data.timestamp) / 3600);
                        None
                    }
                }
                Err(e) => {
                    println!("❌ Failed to parse cache file {}: {}", cache_key, e);
                    None
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to read cache file {}: {}", cache_key, e);
            None
        }
    }
}

fn read_price_cache(cache_key: &str) -> Option<CachedData> {
    let cache_path = get_cache_file_path(cache_key);
    
    if !Path::new(&cache_path).exists() {
        return None;
    }
    
    match fs::read_to_string(&cache_path) {
        Ok(content) => {
            match serde_json::from_str::<CachedData>(&content) {
                Ok(cached_data) => {
                    if is_price_cache_valid(cached_data.timestamp) {
                        println!("✅ Price cache hit for {}: {} minutes old", cache_key, (get_current_timestamp() - cached_data.timestamp) / 60);
                        Some(cached_data)
                    } else {
                        println!("⏰ Price cache expired for {}: {} minutes old", cache_key, (get_current_timestamp() - cached_data.timestamp) / 60);
                        None
                    }
                }
                Err(e) => {
                    println!("❌ Failed to parse price cache file {}: {}", cache_key, e);
                    None
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to read price cache file {}: {}", cache_key, e);
            None
        }
    }
}

fn write_cache(cache_key: &str, data: &serde_json::Value) -> Result<(), std::io::Error> {
    // Ensure cache directory exists
    fs::create_dir_all(CACHE_DIR)?;
    
    let cached_data = CachedData {
        data: data.clone(),
        timestamp: get_current_timestamp(),
    };
    
    let cache_path = get_cache_file_path(cache_key);
    let json_content = serde_json::to_string_pretty(&cached_data)?;
    
    fs::write(&cache_path, json_content)?;
    println!("💾 Cache updated for {}", cache_key);
    
    Ok(())
}

// API handlers
pub async fn get_stock_price(
    config: web::Data<Config>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let symbol = path.into_inner();
    let cache_key = format!("stock_price_{}", symbol);
    
    // Try to read from cache first
    if let Some(cached_data) = read_price_cache(&cache_key) {
        return Ok(HttpResponse::Ok().json(cached_data.data));
    }
    
    println!("🔄 Price cache miss for {}, fetching fresh data...", symbol);
    let client = Client::new();
    
    println!("📡 Making Alpaca Stock Price API request:");
    println!("   Symbol: {}", symbol);
    
    let url = format!("{}/v2/stocks/{}/snapshot", config.alpaca_data_url, symbol);
    println!("   URL: {}", url);
    
    let response = client
        .get(&url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .send()
        .await
        .map_err(|e| {
            println!("❌ Alpaca Stock Price API Request Error: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    let status = response.status();
    println!("📨 Alpaca Stock Price API Response:");
    println!("   Status: {}", status);
    
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        println!("❌ Alpaca Stock Price API Error: {}", error_body);
        
        // Handle different error types appropriately
        match status.as_u16() {
            404 => return Err(actix_web::error::ErrorNotFound(format!("Stock symbol '{}' not found", symbol))),
            401 | 403 => return Err(actix_web::error::ErrorUnauthorized("Invalid API credentials")),
            429 => return Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded")),
            500..=599 => return Err(actix_web::error::ErrorBadGateway("Alpaca API server error")),
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid request to Alpaca API")),
        }
    }
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| {
            println!("❌ Failed to parse Alpaca response as JSON: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    println!("📄 Alpaca API Response Data: {}", serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Failed to serialize".to_string()));
    
    // Helper function to safely extract price as string
    let extract_price = |value: &serde_json::Value| -> Option<String> {
        match value {
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::String(s) => Some(s.clone()),
            _ => {
                println!("⚠️ Price value is not a number or string: {:?}", value);
                None
            }
        }
    };
    
    // Helper function to cache and return price response
    let cache_and_return_price = |price: String, source: &str| -> Result<HttpResponse> {
        println!("✅ Found {} price: {}", source, price);
        let price_response = PriceResponse {
            symbol: symbol.clone(),
            price,
            timestamp: Utc::now().timestamp(),
        };
        
        // Cache the successful response
        let response_json = serde_json::to_value(&price_response)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        
        if let Err(e) = write_cache(&cache_key, &response_json) {
            println!("⚠️ Failed to write price cache for {}: {}", symbol, e);
            // Continue without caching
        } else {
            println!("💾 Price cached for {} (1 hour duration)", symbol);
        }
        
        Ok(HttpResponse::Ok().json(price_response))
    };

    // Try snapshot format
    if let Some(latest_quote) = data.get("latestQuote") {
        if let Some(ask_price) = latest_quote.get("ap").and_then(extract_price) {
            return cache_and_return_price(ask_price, "ask");
        }
        if let Some(bid_price) = latest_quote.get("bp").and_then(extract_price) {
            return cache_and_return_price(bid_price, "bid");
        }
    }
    
    // Try previous close from daily bar
    if let Some(daily_bar) = data.get("dailyBar") {
        if let Some(close_price) = daily_bar.get("c").and_then(extract_price) {
            return cache_and_return_price(close_price, "daily close");
        }
    }
    
    // Try any other price field we can find
    if let Some(latest_trade) = data.get("latestTrade") {
        if let Some(trade_price) = latest_trade.get("p").and_then(extract_price) {
            return cache_and_return_price(trade_price, "trade");
        }
    }
    
    println!("❌ No price data available for symbol: {}", symbol);
    Err(actix_web::error::ErrorNotFound("No price data available for this symbol"))
}

pub async fn get_crypto_price(
    config: web::Data<Config>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let chain_id = query.get("chainId").unwrap_or(&"1".to_string()).clone();
    let token_address = query.get("tokenAddress").unwrap_or(&"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()).clone();
    let cache_key = format!("crypto_price_{}_{}", chain_id, token_address);
    
    // Try to read from cache first
    if let Some(cached_data) = read_price_cache(&cache_key) {
        return Ok(HttpResponse::Ok().json(cached_data.data));
    }
    
    println!("🔄 Crypto price cache miss for {}, fetching fresh data...", token_address);
    let client = Client::new();
    
    let timestamp = Utc::now().timestamp_millis().to_string();
    let endpoint = "/api/v5/dex/aggregator/quote";
    let full_path = format!("{}?chainId={}&fromTokenAddress={}&toTokenAddress=0xdAC17F958D2ee523a2206206994597C13D831ec7&amount=1000000", 
                           endpoint, chain_id, token_address);
    
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
        return Err(actix_web::error::ErrorBadGateway("Failed to fetch crypto price"));
    }
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Convert quote response to price format
    if let Some(quote_data) = data.get("data").and_then(|d| d.get(0)) {
        if let Some(to_token_amount) = quote_data.get("toTokenAmount") {
            let price_response = serde_json::json!({
                "code": "0",
                "msg": "",
                "data": [{
                    "chainId": chain_id,
                    "tokenContractAddress": token_address,
                    "price": to_token_amount,
                    "timestamp": Utc::now().timestamp_millis().to_string()
                }]
            });
            
            // Cache the successful response
            if let Err(e) = write_cache(&cache_key, &price_response) {
                println!("⚠️ Failed to write crypto price cache for {}: {}", token_address, e);
                // Continue without caching
            } else {
                println!("💾 Crypto price cached for {} (1 hour duration)", token_address);
            }
            
            return Ok(HttpResponse::Ok().json(price_response));
        }
    }
    
    // Cache and return raw data if we can't parse it properly
    if let Err(e) = write_cache(&cache_key, &data) {
        println!("⚠️ Failed to write crypto price cache (raw) for {}: {}", token_address, e);
    } else {
        println!("💾 Crypto price (raw) cached for {} (1 hour duration)", token_address);
    }
    
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
    let client = Client::new();
    let chain_id = query.get("chainId").unwrap_or(&"1".to_string()).clone();
    
    let timestamp = Utc::now().timestamp_millis().to_string();
    let endpoint = "/api/v5/dex/aggregator/all-tokens";
    let full_path = format!("{}?chainId={}", endpoint, chain_id);
    
    // Sign the request
    let sign_message = format!("{}GET{}", timestamp, full_path);
    let mut mac = Hmac::<Sha256>::new_from_slice(config.okx_secret_key.as_bytes())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    mac.update(sign_message.as_bytes());
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    
    let url = format!("https://www.okx.com{}", full_path);
    
    // Add timeout and retry logic for better reliability
    let response = client
        .get(&url)
        .header("OK-ACCESS-KEY", &config.okx_api_key)
        .header("OK-ACCESS-SIGN", signature)
        .header("OK-ACCESS-TIMESTAMP", timestamp)
        .header("OK-ACCESS-PASSPHRASE", &config.okx_passphrase)
        .header("OK-ACCESS-PROJECT-ID", &config.okx_project_id)
        .timeout(std::time::Duration::from_secs(10)) // 10 second timeout
        .send()
        .await
        .map_err(|e| {
            println!("⚠️ OKX API request failed: {}", e);
            actix_web::error::ErrorBadGateway("Failed to connect to OKX API")
        })?;
    
    if !response.status().is_success() {
        let status = response.status();
        println!("⚠️ OKX API returned error status: {}", status);
        return Err(actix_web::error::ErrorBadGateway(format!("OKX API error: {}", status)));
    }
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| {
            println!("⚠️ Failed to parse OKX response: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to parse OKX API response")
        })?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn get_stock_list(config: web::Data<Config>) -> Result<HttpResponse> {
    let client = Client::new();
    let url = format!("{}/v2/assets", config.alpaca_base_url);
    
    let response = client
        .get(&url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .query(&[("status", "active"), ("asset_class", "us_equity")])
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if !response.status().is_success() {
        return Err(actix_web::error::ErrorBadGateway("Failed to fetch stock list"));
    }
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(data))
}

pub async fn get_top_stocks(config: web::Data<Config>) -> Result<HttpResponse> {
    let cache_key = "top_stocks";
    
    // Try to read from cache first
    if let Some(cached_data) = read_cache(cache_key) {
        return Ok(HttpResponse::Ok().json(cached_data.data));
    }
    
    println!("🔄 Cache miss for top stocks, fetching fresh data...");
    let client = Client::new();
    
    // Fetch most active stocks from Alpaca Screener API
    let most_active_url = format!("{}/v1beta1/screener/stocks/most-actives", config.alpaca_data_url);
    let gainers_url = format!("{}/v1beta1/screener/movers", config.alpaca_data_url);
    
    println!("📡 Fetching most active stocks from: {}", most_active_url);
    
    // Fetch most active stocks
    let most_active_response = client
        .get(&most_active_url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .query(&[("top", "20")]) // Get top 20
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| {
            println!("❌ Failed to fetch most active stocks: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    let most_active_data: serde_json::Value = if most_active_response.status().is_success() {
        most_active_response.json().await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    } else {
        println!("⚠️ Most active stocks API returned: {}", most_active_response.status());
        serde_json::json!([]) // Empty array fallback
    };
    
    println!("📡 Fetching market movers from: {}", gainers_url);
    
    // Fetch market movers (gainers and losers)
    let movers_response = client
        .get(&gainers_url)
        .header("APCA-API-KEY-ID", &config.alpaca_api_key)
        .header("APCA-API-SECRET-KEY", &config.alpaca_secret_key)
        .query(&[("top", "10")]) // Get top 10 gainers and losers
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| {
            println!("❌ Failed to fetch market movers: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    let movers_data: serde_json::Value = if movers_response.status().is_success() {
        movers_response.json().await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    } else {
        println!("⚠️ Market movers API returned: {}", movers_response.status());
        serde_json::json!({"gainers": [], "losers": []}) // Empty fallback
    };
    
    // Combine the results
    let response_data = serde_json::json!({
        "most_active": most_active_data,
        "gainers": movers_data.get("gainers").unwrap_or(&serde_json::json!([])),
        "losers": movers_data.get("losers").unwrap_or(&serde_json::json!([])),
        "updated_at": get_current_timestamp(),
        "cache_duration_hours": CACHE_DURATION_HOURS
    });
    
    // Cache the results
    if let Err(e) = write_cache(cache_key, &response_data) {
        println!("⚠️ Failed to write cache: {}", e);
        // Continue without caching
    }
    
    println!("✅ Top stocks data fetched and cached successfully");
    Ok(HttpResponse::Ok().json(response_data))
}

// Main function
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    
    let config = web::Data::new(Config::from_env());
    
    println!("🚀 Starting StockSwap API server at http://127.0.0.1:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        "Content-Type",
                        "Authorization", 
                        "Accept",
                        "Origin",
                        "X-Requested-With"
                    ])
                    .max_age(3600)
            )
            // Stock endpoints
            .route("/api/stock/price/{symbol}", web::get().to(get_stock_price))
            .route("/api/stock/list", web::get().to(get_stock_list))
            .route("/api/stock/top", web::get().to(get_top_stocks))
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
                    "status": "healthy",
                    "service": "StockSwap API"
                }))
            }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}