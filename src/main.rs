use dotenv::dotenv;
use std::env;
use std::time::Duration;
use web3::{
    futures::{StreamExt, TryStreamExt},
    transports::WebSocket,
    types::{Transaction, H160},
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    
    let ws_url = env::var("INFURA_WS").expect("INFURA_WS not set in .env");
    let wallet_address: H160 = env::var("WALLET_ADDRESS")
        .expect("WALLET_ADDRESS not set in .env")
        .parse()?;

    loop {
        match connect_and_listen(&ws_url, wallet_address).await {
            Ok(_) => break, // If successful, exit the loop
            Err(e) => {
                eprintln!("Error: {:?}. Reconnecting in 5 seconds...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}

async fn connect_and_listen(ws_url: &str, wallet_address: H160) -> eyre::Result<()> {
    println!("Connecting to WebSocket...");
    let transport = WebSocket::new(ws_url).await?;
    let web3 = web3::Web3::new(transport);

    // Subscribe to pending transactions instead of logs
    let mut sub = web3.eth_subscribe().subscribe_new_pending_transactions().await?;
    
    println!("Listening for transactions involving: {:?}", wallet_address);

    while let Some(tx_hash) = sub.try_next().await? {
        if let Ok(Some(tx)) = web3.eth().transaction(web3::types::TransactionId::Hash(tx_hash)).await {
            // Check if our wallet is involved
            if tx.to == Some(wallet_address ){
                println!("Incoming Transaction: {:?}", tx.hash);
                println!("From: {:?}, To: {:?}, Value: {:?}", tx.from, tx.to, tx.value);
            }
        }
    }

    Ok(())
}
