/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Engine builder for fluent configuration.
//!
//! This module provides a builder API for configuring FIX engines.

use crate::application::{Application, NoOpApplication};
use ironfix_session::config::SessionConfig;
use std::sync::Arc;
use std::time::Duration;

/// Builder for configuring a FIX engine.
#[derive(Debug)]
pub struct EngineBuilder<A: Application = NoOpApplication> {
    /// Application callback handler.
    application: Arc<A>,
    /// Session configurations.
    sessions: Vec<SessionConfig>,
    /// Whether to use TLS.
    use_tls: bool,
    /// Connection timeout.
    connect_timeout: Duration,
    /// Reconnect interval.
    reconnect_interval: Duration,
    /// Maximum reconnect attempts.
    max_reconnect_attempts: u32,
}

impl Default for EngineBuilder<NoOpApplication> {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineBuilder<NoOpApplication> {
    /// Creates a new engine builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            application: Arc::new(NoOpApplication),
            sessions: Vec::new(),
            use_tls: false,
            connect_timeout: Duration::from_secs(30),
            reconnect_interval: Duration::from_secs(5),
            max_reconnect_attempts: 10,
        }
    }
}

impl<A: Application> EngineBuilder<A> {
    /// Sets the application callback handler.
    #[must_use]
    pub fn with_application<B: Application>(self, application: B) -> EngineBuilder<B> {
        EngineBuilder {
            application: Arc::new(application),
            sessions: self.sessions,
            use_tls: self.use_tls,
            connect_timeout: self.connect_timeout,
            reconnect_interval: self.reconnect_interval,
            max_reconnect_attempts: self.max_reconnect_attempts,
        }
    }

    /// Adds a session configuration.
    #[must_use]
    pub fn add_session(mut self, config: SessionConfig) -> Self {
        self.sessions.push(config);
        self
    }

    /// Enables TLS for connections.
    #[must_use]
    pub const fn with_tls(mut self, enabled: bool) -> Self {
        self.use_tls = enabled;
        self
    }

    /// Sets the connection timeout.
    #[must_use]
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Sets the reconnect interval.
    #[must_use]
    pub fn with_reconnect_interval(mut self, interval: Duration) -> Self {
        self.reconnect_interval = interval;
        self
    }

    /// Sets the maximum reconnect attempts.
    #[must_use]
    pub const fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }

    /// Returns the configured sessions.
    #[must_use]
    pub fn sessions(&self) -> &[SessionConfig] {
        &self.sessions
    }

    /// Returns whether TLS is enabled.
    #[must_use]
    pub const fn use_tls(&self) -> bool {
        self.use_tls
    }

    /// Returns the connection timeout.
    #[must_use]
    pub const fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    /// Returns the reconnect interval.
    #[must_use]
    pub const fn reconnect_interval(&self) -> Duration {
        self.reconnect_interval
    }

    /// Returns the maximum reconnect attempts.
    #[must_use]
    pub const fn max_reconnect_attempts(&self) -> u32 {
        self.max_reconnect_attempts
    }

    /// Returns the application handler.
    #[must_use]
    pub fn application(&self) -> Arc<A> {
        Arc::clone(&self.application)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ironfix_core::types::CompId;

    #[test]
    fn test_engine_builder_default() {
        let builder = EngineBuilder::new();
        assert!(!builder.use_tls());
        assert_eq!(builder.connect_timeout(), Duration::from_secs(30));
        assert_eq!(builder.max_reconnect_attempts(), 10);
        assert!(builder.sessions().is_empty());
    }

    #[test]
    fn test_engine_builder_with_session() {
        let config = SessionConfig::new(
            CompId::new("SENDER").unwrap(),
            CompId::new("TARGET").unwrap(),
            "FIX.4.4",
        );

        let builder = EngineBuilder::new()
            .add_session(config)
            .with_tls(true)
            .with_connect_timeout(Duration::from_secs(60));

        assert_eq!(builder.sessions().len(), 1);
        assert!(builder.use_tls());
        assert_eq!(builder.connect_timeout(), Duration::from_secs(60));
    }
}
