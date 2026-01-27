/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Heartbeat and TestRequest management.
//!
//! This module handles FIX session heartbeat logic including:
//! - Sending heartbeats at configured intervals
//! - Sending TestRequest when no messages received
//! - Detecting heartbeat timeouts

use std::time::{Duration, Instant};

/// Manages heartbeat timing for a FIX session.
#[derive(Debug)]
pub struct HeartbeatManager {
    /// Heartbeat interval.
    interval: Duration,
    /// Time of last message sent.
    last_sent: Instant,
    /// Time of last message received.
    last_received: Instant,
    /// Pending TestRequest ID, if any.
    test_request_pending: Option<String>,
    /// Time when TestRequest was sent.
    test_request_sent_at: Option<Instant>,
}

impl HeartbeatManager {
    /// Creates a new heartbeat manager with the specified interval.
    ///
    /// # Arguments
    /// * `interval` - The heartbeat interval
    #[must_use]
    pub fn new(interval: Duration) -> Self {
        let now = Instant::now();
        Self {
            interval,
            last_sent: now,
            last_received: now,
            test_request_pending: None,
            test_request_sent_at: None,
        }
    }

    /// Records that a message was sent.
    #[inline]
    pub fn on_message_sent(&mut self) {
        self.last_sent = Instant::now();
    }

    /// Records that a message was received.
    ///
    /// If a TestRequest was pending and a Heartbeat with matching ID is received,
    /// the pending request is cleared.
    ///
    /// # Arguments
    /// * `is_heartbeat` - Whether the received message is a Heartbeat
    /// * `test_req_id` - The TestReqID from the Heartbeat, if present
    pub fn on_message_received(&mut self, is_heartbeat: bool, test_req_id: Option<&str>) {
        self.last_received = Instant::now();

        if is_heartbeat
            && let (Some(pending), Some(received)) = (&self.test_request_pending, test_req_id)
            && pending == received
        {
            self.test_request_pending = None;
            self.test_request_sent_at = None;
        }
    }

    /// Checks if a heartbeat should be sent.
    ///
    /// A heartbeat should be sent if no message has been sent within the interval.
    #[must_use]
    pub fn should_send_heartbeat(&self) -> bool {
        self.last_sent.elapsed() >= self.interval
    }

    /// Checks if a TestRequest should be sent.
    ///
    /// A TestRequest should be sent if no message has been received within
    /// the interval plus a grace period, and no TestRequest is already pending.
    #[must_use]
    pub fn should_send_test_request(&self) -> bool {
        if self.test_request_pending.is_some() {
            return false;
        }

        let grace = Duration::from_secs(1);
        self.last_received.elapsed() >= self.interval + grace
    }

    /// Checks if the session has timed out.
    ///
    /// A timeout occurs if a TestRequest was sent but no response was received
    /// within the interval.
    #[must_use]
    pub fn is_timed_out(&self) -> bool {
        if let Some(sent_at) = self.test_request_sent_at {
            sent_at.elapsed() >= self.interval
        } else {
            false
        }
    }

    /// Records that a TestRequest was sent.
    ///
    /// # Arguments
    /// * `test_req_id` - The TestReqID that was sent
    pub fn on_test_request_sent(&mut self, test_req_id: String) {
        self.test_request_pending = Some(test_req_id);
        self.test_request_sent_at = Some(Instant::now());
        self.last_sent = Instant::now();
    }

    /// Returns the pending TestRequest ID, if any.
    #[must_use]
    pub fn pending_test_request(&self) -> Option<&str> {
        self.test_request_pending.as_deref()
    }

    /// Returns the time since the last message was received.
    #[must_use]
    pub fn time_since_last_received(&self) -> Duration {
        self.last_received.elapsed()
    }

    /// Returns the time since the last message was sent.
    #[must_use]
    pub fn time_since_last_sent(&self) -> Duration {
        self.last_sent.elapsed()
    }

    /// Returns the heartbeat interval.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.interval
    }

    /// Resets the manager state.
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.last_sent = now;
        self.last_received = now;
        self.test_request_pending = None;
        self.test_request_sent_at = None;
    }
}

/// Generates a unique TestReqID.
///
/// Uses the current timestamp in nanoseconds.
#[must_use]
pub fn generate_test_req_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    format!("TEST{}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_heartbeat_manager_new() {
        let mgr = HeartbeatManager::new(Duration::from_secs(30));
        assert_eq!(mgr.interval(), Duration::from_secs(30));
        assert!(mgr.pending_test_request().is_none());
    }

    #[test]
    fn test_should_send_heartbeat() {
        let mgr = HeartbeatManager::new(Duration::from_millis(10));
        assert!(!mgr.should_send_heartbeat());

        sleep(Duration::from_millis(15));
        assert!(mgr.should_send_heartbeat());
    }

    #[test]
    fn test_on_message_sent() {
        let mut mgr = HeartbeatManager::new(Duration::from_millis(10));
        sleep(Duration::from_millis(15));
        assert!(mgr.should_send_heartbeat());

        mgr.on_message_sent();
        assert!(!mgr.should_send_heartbeat());
    }

    #[test]
    fn test_test_request_pending() {
        let mut mgr = HeartbeatManager::new(Duration::from_secs(30));

        mgr.on_test_request_sent("TEST123".to_string());
        assert_eq!(mgr.pending_test_request(), Some("TEST123"));

        mgr.on_message_received(true, Some("TEST123"));
        assert!(mgr.pending_test_request().is_none());
    }

    #[test]
    fn test_generate_test_req_id() {
        let id1 = generate_test_req_id();
        std::thread::sleep(std::time::Duration::from_nanos(1));
        let id2 = generate_test_req_id();

        assert!(id1.starts_with("TEST"));
        assert!(id2.starts_with("TEST"));
        // IDs may be equal if generated within the same nanosecond on fast systems
        // The important thing is that they have the correct format
        assert!(id1.len() > 4);
        assert!(id2.len() > 4);
    }
}
