//! Common utilities shared across examples.

#![allow(dead_code)]

use std::env;

/// Default server port.
pub const DEFAULT_PORT: u16 = 9876;

/// Default server host.
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Example configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct ExampleConfig {
    /// Server hostname.
    pub host: String,
    /// Server port.
    pub port: u16,
    /// Sender CompID.
    pub sender_comp_id: String,
    /// Target CompID.
    pub target_comp_id: String,
    /// Heartbeat interval in seconds.
    pub heartbeat_interval: u64,
}

impl ExampleConfig {
    /// Creates a new configuration for a client.
    #[must_use]
    pub fn client() -> Self {
        Self {
            host: env::var("FIX_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string()),
            port: env::var("FIX_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(DEFAULT_PORT),
            sender_comp_id: env::var("FIX_SENDER").unwrap_or_else(|_| "CLIENT".to_string()),
            target_comp_id: env::var("FIX_TARGET").unwrap_or_else(|_| "SERVER".to_string()),
            heartbeat_interval: 30,
        }
    }

    /// Creates a new configuration for a server.
    #[must_use]
    pub fn server() -> Self {
        Self {
            host: env::var("FIX_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string()),
            port: env::var("FIX_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(DEFAULT_PORT),
            sender_comp_id: env::var("FIX_SENDER").unwrap_or_else(|_| "SERVER".to_string()),
            target_comp_id: env::var("FIX_TARGET").unwrap_or_else(|_| "CLIENT".to_string()),
            heartbeat_interval: 30,
        }
    }

    /// Returns the socket address string.
    #[must_use]
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Initializes logging for examples.
pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .try_init();
}

/// Format a timestamp in FIX format.
pub fn format_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let (s, ms) = (d.as_secs(), d.subsec_millis());
    let (mut y, mut rd) = (1970u64, s / 86400);
    loop {
        let dy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if rd < dy {
            break;
        }
        rd -= dy;
        y += 1;
    }
    let dm = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let (mut m, mut day) = (1u64, rd);
    for days in dm {
        if day < days {
            break;
        }
        day -= days;
        m += 1;
    }
    let tod = s % 86400;
    format!(
        "{:04}{:02}{:02}-{:02}:{:02}:{:02}.{:03}",
        y,
        m,
        day + 1,
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60,
        ms
    )
}

/// SOH delimiter constant.
pub const SOH: u8 = 0x01;

/// Try to decode a complete FIX message from buffer.
pub fn try_decode_message(buf: &[u8]) -> Option<usize> {
    if buf.len() < 20 {
        return None;
    }
    if let Some(i) = buf.windows(2).position(|w| w == b"9=")
        && let Some(j) = buf[i..].iter().position(|&x| x == SOH)
        && let Ok(len_str) = std::str::from_utf8(&buf[i + 2..i + j])
        && let Ok(len) = len_str.parse::<usize>()
    {
        let total = i + j + 1 + len + 7;
        if buf.len() >= total {
            return Some(total);
        }
    }
    None
}
