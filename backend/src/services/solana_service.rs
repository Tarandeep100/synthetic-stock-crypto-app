// use anchor_client::{Client, Cluster, Program};
// use anchor_lang::prelude::*;
// use solana_client::pubsub_client::PubsubClient;
// use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
// use solana_sdk::{
//     commitment_config::CommitmentConfig,
//     pubkey::Pubkey,
//     signature::Keypair,
//     signer::Signer,
// };
// use tokio_tungstenite::{connect_async, tungstenite::Message};
// use serde_json;
// use std::str::FromStr;
// use super::api;

// #[derive(Debug, Clone)]
// pub struct BuyOrderPlaced {
//     pub order_id: u64,
//     pub user: Pubkey,
//     pub stock_symbol: String,
//     pub sol_amount: u64,
//     pub max_price_per_share: u64,
//     pub timestamp: i64,
// }

// pub struct SolanaService {
//     pub program_id: Pubkey,
//     pub backend_authority: Keypair,
//     pub rpc_url: String,
//     pub ws_url: String,
// }

// impl SolanaService {
//     pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
//         let program_id = Pubkey::from_str(&std::env::var("STOCK_CONTRACTS_PROGRAM_ID")?)?;
//         let backend_authority_path = std::env::var("BACKEND_AUTHORITY_KEYPAIR")?;
//         let backend_authority = Keypair::from_bytes(
//             &std::fs::read(&backend_authority_path)?
//         )?;
        
//         let rpc_url = std::env::var("SOLANA_RPC_URL")
//             .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        
//         let ws_url = rpc_url.replace("https://", "wss://").replace("http://", "ws://");
        
//         Ok(Self {
//             program_id,
//             backend_authority,
//             rpc_url,
//             ws_url,
//         })
//     }

//     pub async fn start_event_listener(&self) -> Result<(), Box<dyn std::error::Error>> {
//         println!("🎧 Starting Solana event listener for program: {}", self.program_id);
        
//         let (ws_stream, _) = connect_async(&self.ws_url).await?;
        
//         // Subscribe to program logs
//         let subscribe_request = serde_json::json!({
//             "jsonrpc": "2.0",
//             "id": 1,
//             "method": "logsSubscribe",
//             "params": [
//                 {
//                     "mentions": [self.program_id.to_string()]
//                 },
//                 {
//                     "commitment": "confirmed"
//                 }
//             ]
//         });

//         // Send subscription request
//         ws_stream.send(Message::Text(subscribe_request.to_string())).await?;

//         // Listen for events
//         while let Some(msg) = ws_stream.next().await {
//             match msg {
//                 Ok(Message::Text(text)) => {
//                     if let Ok(response) = serde_json::from_str::<serde_json::Value>(&text) {
//                         if let Some(params) = response.get("params") {
//                             if let Some(result) = params.get("result") {
//                                 if let Some(logs) = result.get("logs") {
//                                     self.process_logs(logs.as_array().unwrap()).await;
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 Ok(Message::Close(_)) => {
//                     println!("❌ WebSocket connection closed");
//                     break;
//                 }
//                 Err(e) => {
//                     println!("❌ WebSocket error: {}", e);
//                 }
//             }
//         }

//         Ok(())
//     }

//     async fn process_logs(&self, logs: &[serde_json::Value]) {
//         for log in logs {
//             if let Some(log_str) = log.as_str() {
//                 // Look for BuyOrderPlaced event
//                 if log_str.contains("BuyOrderPlaced") {
//                     println!("🔥 BuyOrderPlaced event detected: {}", log_str);
                    
//                     // Parse the event data
//                     if let Some(event) = self.parse_buy_order_event(log_str) {
//                         // Process the buy order
//                         self.process_buy_order(event).await;
//                     }
//                 }
//             }
//         }
//     }

//     fn parse_buy_order_event(&self, log: &str) -> Option<BuyOrderPlaced> {
//         // Parse the event from logs
//         // This is a simplified parser - you might need to implement proper event parsing
//         // based on your event structure
        
//         // For now, return a mock event - implement proper parsing
//         println!("🎯 Parsing buy order event from: {}", log);
//         None // TODO: Implement proper event parsing
//     }

//     async fn process_buy_order(&self, event: BuyOrderPlaced) {
//         println!("🎯 Processing buy order: {:?}", event);
        
//         // 1. Get current stock price using your existing API
//         match crate::get_stock_price_internal(&event.stock_symbol).await {
//             Ok(price_data) => {
//                 let stock_price = price_data.price.parse::<f64>().unwrap_or(0.0);
//                 let sol_to_usd = 100.0; // You'll need to get real SOL price
//                 let sol_value_usd = (event.sol_amount as f64 / 1_000_000_000.0) * sol_to_usd;
                
//                 // Calculate shares to purchase
//                 let shares_to_buy = (sol_value_usd / stock_price).floor() as u64;
//                 let total_cost = shares_to_buy as f64 * stock_price;
//                 let cost_in_sol = ((total_cost / sol_to_usd) * 1_000_000_000.0) as u64;
//                 let refund_amount = event.sol_amount.saturating_sub(cost_in_sol);
                
//                 // 2. Execute stock purchase via your APIs
//                 match self.execute_stock_purchase(&event.stock_symbol, total_cost).await {
//                     Ok(_) => {
//                         // 3. Call fulfill_buy_order on smart contract
//                         if let Err(e) = self.fulfill_buy_order_on_chain(
//                             event.order_id,
//                             event.user,
//                             &event.stock_symbol,
//                             shares_to_buy,
//                             (stock_price * 1_000_000_000.0) as u64, // Convert to lamports
//                             cost_in_sol,
//                             refund_amount,
//                         ).await {
//                             println!("❌ Failed to fulfill order on-chain: {}", e);
//                         }
//                     }
//                     Err(e) => {
//                         println!("❌ Failed to execute stock purchase: {}", e);
//                         // TODO: Cancel the order or handle failure
//                     }
//                 }
//             }
//             Err(e) => {
//                 println!("❌ Failed to get stock price: {}", e);
//             }
//         }
//     }

//     async fn execute_stock_purchase(&self, symbol: &str, amount_usd: f64) -> Result<(), Box<dyn std::error::Error>> {
//         // Use your existing stock API to purchase
//         println!("📈 Purchasing ${:.2} worth of {}", amount_usd, symbol);
        
//         // TODO: Integrate with your existing buy_stock_with_usdt function
//         // This would call your Alpaca API
        
//         Ok(())
//     }

//     async fn fulfill_buy_order_on_chain(
//         &self,
//         order_id: u64,
//         user: Pubkey,
//         stock_symbol: &str,
//         shares_purchased: u64,
//         price_per_share: u64,
//         total_cost: u64,
//         refund_amount: u64,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         println!("⛓️ Fulfilling buy order on-chain: order_id={}, shares={}", order_id, shares_purchased);
        
//         // TODO: Implement the actual smart contract call
//         // You'll need to use anchor_client to call fulfill_buy_order
        
//         let cluster = Cluster::Custom(self.rpc_url.clone(), self.ws_url.clone());
//         let client = Client::new(cluster, std::rc::Rc::new(self.backend_authority.insecure_clone()));
        
//         // Get the program
//         let program = client.program(self.program_id)?;
        
//         // Call fulfill_buy_order
//         // let tx = program
//         //     .request()
//         //     .instruction(fulfill_buy_order_ix)
//         //     .send()?;
        
//         println!("✅ Order fulfilled on-chain");
//         Ok(())
//     }
// }

// // Helper function to get stock price (integrate with existing code)
// async fn get_stock_price_internal(symbol: &str) -> Result<crate::PriceResponse, Box<dyn std::error::Error>> {
//     // Use your existing get_stock_price logic
//     Ok(crate::PriceResponse {
//         symbol: symbol.to_string(),
//         price: "150.00".to_string(), // Mock price
//         timestamp: chrono::Utc::now().timestamp(),
//     })
// } 