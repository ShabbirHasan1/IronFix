/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/1/26
******************************************************************************/

//! Application callback interface.
//!
//! This module defines the callback interface for handling FIX messages,
//! following the QuickFIX pattern with async support.

use async_trait::async_trait;
use ironfix_core::message::{OwnedMessage, RawMessage};

/// Session identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId {
    /// BeginString (FIX version).
    pub begin_string: String,
    /// Sender CompID.
    pub sender_comp_id: String,
    /// Target CompID.
    pub target_comp_id: String,
    /// Optional sender sub ID.
    pub sender_sub_id: Option<String>,
    /// Optional target sub ID.
    pub target_sub_id: Option<String>,
}

impl SessionId {
    /// Creates a new session ID.
    #[must_use]
    pub fn new(
        begin_string: impl Into<String>,
        sender_comp_id: impl Into<String>,
        target_comp_id: impl Into<String>,
    ) -> Self {
        Self {
            begin_string: begin_string.into(),
            sender_comp_id: sender_comp_id.into(),
            target_comp_id: target_comp_id.into(),
            sender_sub_id: None,
            target_sub_id: None,
        }
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
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}->{}",
            self.begin_string, self.sender_comp_id, self.target_comp_id
        )
    }
}

/// Reason for rejecting a message.
#[derive(Debug, Clone)]
pub struct RejectReason {
    /// Rejection reason code.
    pub code: u32,
    /// Human-readable rejection text.
    pub text: String,
    /// Reference tag that caused the rejection.
    pub ref_tag: Option<u32>,
}

impl RejectReason {
    /// Creates a new rejection reason.
    #[must_use]
    pub fn new(code: u32, text: impl Into<String>) -> Self {
        Self {
            code,
            text: text.into(),
            ref_tag: None,
        }
    }

    /// Sets the reference tag.
    #[must_use]
    pub const fn with_ref_tag(mut self, tag: u32) -> Self {
        self.ref_tag = Some(tag);
        self
    }
}

/// Application callback interface for handling FIX messages.
///
/// Implement this trait to receive callbacks for session events
/// and message processing.
#[async_trait]
pub trait Application: Send + Sync {
    /// Called when a session is created.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier
    async fn on_create(&self, session_id: &SessionId);

    /// Called on successful logon.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier
    async fn on_logon(&self, session_id: &SessionId);

    /// Called on logout.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier
    async fn on_logout(&self, session_id: &SessionId);

    /// Called before sending an admin message.
    ///
    /// Allows modification of outgoing admin messages (Logon, Heartbeat, etc.).
    ///
    /// # Arguments
    /// * `message` - The message to be sent (mutable)
    /// * `session_id` - The session identifier
    async fn to_admin(&self, message: &mut OwnedMessage, session_id: &SessionId);

    /// Called when an admin message is received.
    ///
    /// # Arguments
    /// * `message` - The received message
    /// * `session_id` - The session identifier
    ///
    /// # Returns
    /// `Ok(())` to accept, `Err(RejectReason)` to reject.
    #[allow(clippy::wrong_self_convention)]
    async fn from_admin(
        &self,
        message: &RawMessage<'_>,
        session_id: &SessionId,
    ) -> Result<(), RejectReason>;

    /// Called before sending an application message.
    ///
    /// Allows modification of outgoing application messages.
    ///
    /// # Arguments
    /// * `message` - The message to be sent (mutable)
    /// * `session_id` - The session identifier
    async fn to_app(&self, message: &mut OwnedMessage, session_id: &SessionId);

    /// Called when an application message is received.
    ///
    /// # Arguments
    /// * `message` - The received message
    /// * `session_id` - The session identifier
    ///
    /// # Returns
    /// `Ok(())` to accept, `Err(RejectReason)` to reject.
    #[allow(clippy::wrong_self_convention)]
    async fn from_app(
        &self,
        message: &RawMessage<'_>,
        session_id: &SessionId,
    ) -> Result<(), RejectReason>;
}

/// Default no-op application implementation.
#[derive(Debug, Default)]
pub struct NoOpApplication;

#[async_trait]
impl Application for NoOpApplication {
    async fn on_create(&self, _session_id: &SessionId) {}

    async fn on_logon(&self, _session_id: &SessionId) {}

    async fn on_logout(&self, _session_id: &SessionId) {}

    async fn to_admin(&self, _message: &mut OwnedMessage, _session_id: &SessionId) {}

    async fn from_admin(
        &self,
        _message: &RawMessage<'_>,
        _session_id: &SessionId,
    ) -> Result<(), RejectReason> {
        Ok(())
    }

    async fn to_app(&self, _message: &mut OwnedMessage, _session_id: &SessionId) {}

    async fn from_app(
        &self,
        _message: &RawMessage<'_>,
        _session_id: &SessionId,
    ) -> Result<(), RejectReason> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id() {
        let id = SessionId::new("FIX.4.4", "SENDER", "TARGET");
        assert_eq!(id.begin_string, "FIX.4.4");
        assert_eq!(id.sender_comp_id, "SENDER");
        assert_eq!(id.target_comp_id, "TARGET");
        assert_eq!(id.to_string(), "FIX.4.4:SENDER->TARGET");
    }

    #[test]
    fn test_reject_reason() {
        let reason = RejectReason::new(1, "Invalid tag").with_ref_tag(35);
        assert_eq!(reason.code, 1);
        assert_eq!(reason.text, "Invalid tag");
        assert_eq!(reason.ref_tag, Some(35));
    }

    #[tokio::test]
    async fn test_noop_application() {
        let app = NoOpApplication;
        let session_id = SessionId::new("FIX.4.4", "SENDER", "TARGET");

        app.on_create(&session_id).await;
        app.on_logon(&session_id).await;
        app.on_logout(&session_id).await;
    }
}
