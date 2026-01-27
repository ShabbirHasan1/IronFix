//! FAST protocol client example.
//!
//! This example demonstrates a simple FAST market data client that receives
//! and decodes market data messages from a FAST server.

mod common;

use common::{ExampleConfig, init_logging};
use ironfix_fast::FastDecoder;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::{error, info};

const DEFAULT_PORT: u16 = 9890;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let cfg = ExampleConfig::client();
    let port = std::env::var("FAST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let addr = format!("{}:{}", cfg.host, port);
    info!("Connecting to FAST server at {}", addr);

    let mut socket: TcpStream = TcpStream::connect(&addr).await?;
    info!("Connected to FAST server");

    let mut buf = vec![0u8; 4096];
    let mut data = Vec::new();

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                info!("Server closed connection");
                break;
            }
            Ok(n) => {
                data.extend_from_slice(&buf[..n]);

                // Try to decode messages from buffer
                while let Some(msg) = try_decode_message(&mut data) {
                    info!(
                        "Received: seq={} symbol={} price={:.2} size={}",
                        msg.seq_num, msg.symbol, msg.price, msg.size
                    );
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Decoded market data message.
#[derive(Debug)]
#[allow(dead_code)]
struct MarketDataMessage {
    template_id: u64,
    seq_num: u64,
    timestamp: String,
    symbol: String,
    price: f64,
    size: u64,
}

/// Try to decode a FAST message from the buffer.
/// Returns None if not enough data is available.
fn try_decode_message(data: &mut Vec<u8>) -> Option<MarketDataMessage> {
    if data.is_empty() {
        return None;
    }

    let mut offset = 0;
    let _original_len = data.len();

    // Try to decode the message
    let result = decode_market_data(data, &mut offset);

    match result {
        Ok(msg) => {
            // Remove consumed bytes from buffer
            data.drain(..offset);
            Some(msg)
        }
        Err(_) => {
            // Not enough data or invalid message
            // If we consumed some bytes but failed, we might have a partial message
            // Keep the data for next read
            None
        }
    }
}

/// Decode a FAST-encoded market data message.
fn decode_market_data(data: &[u8], offset: &mut usize) -> Result<MarketDataMessage, &'static str> {
    if data.len() < 7 {
        return Err("Not enough data");
    }

    // Decode presence map
    let _pmap = FastDecoder::decode_uint(data, offset).map_err(|_| "Failed to decode pmap")?;

    // Template ID
    let template_id =
        FastDecoder::decode_uint(data, offset).map_err(|_| "Failed to decode template_id")?;

    // Sequence number
    let seq_num = FastDecoder::decode_uint(data, offset).map_err(|_| "Failed to decode seq_num")?;

    // Timestamp
    let timestamp =
        FastDecoder::decode_ascii(data, offset).map_err(|_| "Failed to decode timestamp")?;

    // Symbol
    let symbol = FastDecoder::decode_ascii(data, offset).map_err(|_| "Failed to decode symbol")?;

    // Price (scaled by 100)
    let scaled_price =
        FastDecoder::decode_uint(data, offset).map_err(|_| "Failed to decode price")?;
    let price = scaled_price as f64 / 100.0;

    // Size
    let size = FastDecoder::decode_uint(data, offset).map_err(|_| "Failed to decode size")?;

    Ok(MarketDataMessage {
        template_id,
        seq_num,
        timestamp,
        symbol,
        price,
        size,
    })
}
