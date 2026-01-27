/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Session configuration.
//!
//! This module provides configuration options for FIX sessions.

use ironfix_core::types::CompId;
use std::time::Duration;

/// Configuration for a FIX session.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Sender CompID (tag 49).
    pub sender_comp_id: CompId,
    /// Target CompID (tag 56).
    pub target_comp_id: CompId,
    /// FIX version BeginString (e.g., "FIX.4.4").
    pub begin_string: String,
    /// Heartbeat interval in seconds.
    pub heartbeat_interval: Duration,
    /// Whether to reset sequence numbers on logon.
    pub reset_on_logon: bool,
    /// Whether to reset sequence numbers on logout.
    pub reset_on_logout: bool,
    /// Whether to reset sequence numbers on disconnect.
    pub reset_on_disconnect: bool,
    /// Maximum message size in bytes.
    pub max_message_size: usize,
    /// Logon timeout duration.
    pub logon_timeout: Duration,
    /// Logout timeout duration.
    pub logout_timeout: Duration,
    /// Whether to validate incoming message checksums.
    pub validate_checksum: bool,
    /// Whether to validate incoming message length.
    pub validate_length: bool,
    /// Optional sender sub ID (tag 50).
    pub sender_sub_id: Option<String>,
    /// Optional target sub ID (tag 57).
    pub target_sub_id: Option<String>,
    /// Optional sender location ID (tag 142).
    pub sender_location_id: Option<String>,
    /// Optional target location ID (tag 143).
    pub target_location_id: Option<String>,
}

impl SessionConfig {
    /// Creates a new session configuration with required fields.
    ///
    /// # Arguments
    /// * `sender_comp_id` - The sender CompID
    /// * `target_comp_id` - The target CompID
    /// * `begin_string` - The FIX version string
    #[must_use]
    pub fn new(
        sender_comp_id: CompId,
        target_comp_id: CompId,
        begin_string: impl Into<String>,
    ) -> Self {
        Self {
            sender_comp_id,
            target_comp_id,
            begin_string: begin_string.into(),
            heartbeat_interval: Duration::from_secs(30),
            reset_on_logon: false,
            reset_on_logout: false,
            reset_on_disconnect: false,
            max_message_size: 1024 * 1024, // 1MB
            logon_timeout: Duration::from_secs(10),
            logout_timeout: Duration::from_secs(10),
            validate_checksum: true,
            validate_length: true,
            sender_sub_id: None,
            target_sub_id: None,
            sender_location_id: None,
            target_location_id: None,
        }
    }

    /// Sets the heartbeat interval.
    #[must_use]
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Sets whether to reset sequence numbers on logon.
    #[must_use]
    pub const fn with_reset_on_logon(mut self, reset: bool) -> Self {
        self.reset_on_logon = reset;
        self
    }

    /// Sets the maximum message size.
    #[must_use]
    pub const fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Sets the logon timeout.
    #[must_use]
    pub fn with_logon_timeout(mut self, timeout: Duration) -> Self {
        self.logon_timeout = timeout;
        self
    }

    /// Sets the sender sub ID.
    #[must_use]
    pub fn with_sender_sub_id(mut self, sub_id: impl Into<String>) -> Self {
        self.sender_sub_id = Some(sub_id.into());
        self
    }

    /// Sets the target sub ID.
    #[must_use]
    pub fn with_target_sub_id(mut self, sub_id: impl Into<String>) -> Self {
        self.target_sub_id = Some(sub_id.into());
        self
    }

    /// Returns the heartbeat interval in seconds.
    #[must_use]
    pub fn heartbeat_interval_secs(&self) -> u64 {
        self.heartbeat_interval.as_secs()
    }
}

/// Builder for session configuration.
#[derive(Debug, Default)]
pub struct SessionConfigBuilder {
    sender_comp_id: Option<CompId>,
    target_comp_id: Option<CompId>,
    begin_string: Option<String>,
    heartbeat_interval: Option<Duration>,
    reset_on_logon: bool,
    max_message_size: Option<usize>,
}

impl SessionConfigBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the sender CompID.
    #[must_use]
    pub fn sender_comp_id(mut self, id: CompId) -> Self {
        self.sender_comp_id = Some(id);
        self
    }

    /// Sets the target CompID.
    #[must_use]
    pub fn target_comp_id(mut self, id: CompId) -> Self {
        self.target_comp_id = Some(id);
        self
    }

    /// Sets the FIX version.
    #[must_use]
    pub fn begin_string(mut self, version: impl Into<String>) -> Self {
        self.begin_string = Some(version.into());
        self
    }

    /// Sets the heartbeat interval.
    #[must_use]
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = Some(interval);
        self
    }

    /// Sets whether to reset on logon.
    #[must_use]
    pub const fn reset_on_logon(mut self, reset: bool) -> Self {
        self.reset_on_logon = reset;
        self
    }

    /// Builds the configuration.
    ///
    /// # Panics
    /// Panics if required fields are not set.
    #[must_use]
    pub fn build(self) -> SessionConfig {
        let sender = self.sender_comp_id.expect("sender_comp_id is required");
        let target = self.target_comp_id.expect("target_comp_id is required");
        let begin_string = self.begin_string.unwrap_or_else(|| "FIX.4.4".to_string());

        let mut config = SessionConfig::new(sender, target, begin_string);

        if let Some(interval) = self.heartbeat_interval {
            config.heartbeat_interval = interval;
        }
        config.reset_on_logon = self.reset_on_logon;
        if let Some(size) = self.max_message_size {
            config.max_message_size = size;
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_new() {
        let sender = CompId::new("SENDER").unwrap();
        let target = CompId::new("TARGET").unwrap();
        let config = SessionConfig::new(sender, target, "FIX.4.4");

        assert_eq!(config.sender_comp_id.as_str(), "SENDER");
        assert_eq!(config.target_comp_id.as_str(), "TARGET");
        assert_eq!(config.begin_string, "FIX.4.4");
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_session_config_builder() {
        let config = SessionConfigBuilder::new()
            .sender_comp_id(CompId::new("SENDER").unwrap())
            .target_comp_id(CompId::new("TARGET").unwrap())
            .begin_string("FIX.4.2")
            .heartbeat_interval(Duration::from_secs(60))
            .reset_on_logon(true)
            .build();

        assert_eq!(config.begin_string, "FIX.4.2");
        assert_eq!(config.heartbeat_interval, Duration::from_secs(60));
        assert!(config.reset_on_logon);
    }
}
