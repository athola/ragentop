//! Daemon socket client for communicating with ragentop-daemon.

use ragentop_core::{
    AgentSession, Command, HistoryDepth, Request, Response, SessionId, SessionMetrics,
};
use std::path::Path;
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader};
use tokio::net::UnixStream;

/// Error type for daemon client operations.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Daemon error: {0}")]
    Daemon(String),
}

/// Client for communicating with the ragentop daemon via Unix socket.
#[expect(
    clippy::module_name_repetitions,
    reason = "DaemonClient is clearer than just Client"
)]
pub struct DaemonClient {
    stream: BufReader<UnixStream>,
}

impl DaemonClient {
    /// Connect to the daemon at the given socket path.
    ///
    /// # Errors
    /// Returns `ClientError::Io` if the connection fails.
    pub async fn connect<P: AsRef<Path>>(socket_path: P) -> Result<Self, ClientError> {
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    /// Send a request to the daemon and receive a response.
    ///
    /// # Errors
    /// Returns `ClientError` on I/O, serialization, or daemon errors.
    pub async fn send(&mut self, request: &Request) -> Result<Response, ClientError> {
        let mut json = serde_json::to_string(request)?;
        json.push('\n');

        self.stream.get_mut().write_all(json.as_bytes()).await?;
        self.stream.get_mut().flush().await?;

        let mut line = String::new();
        self.stream.read_line(&mut line).await?;

        let response: Response = serde_json::from_str(&line)?;

        if let Response::Error { message } = &response {
            return Err(ClientError::Daemon(message.clone()));
        }

        Ok(response)
    }

    /// List all active sessions.
    ///
    /// # Errors
    /// Returns `ClientError` on communication or daemon errors.
    pub async fn list_sessions(&mut self) -> Result<Vec<AgentSession>, ClientError> {
        match self.send(&Request::ListSessions).await? {
            Response::Sessions { sessions } => Ok(sessions),
            other => Err(ClientError::Daemon(format!(
                "Unexpected response: {other:?}"
            ))),
        }
    }

    /// Get metrics for a specific session.
    ///
    /// # Errors
    /// Returns `ClientError` on communication or daemon errors.
    pub async fn get_metrics(
        &mut self,
        session_id: &SessionId,
    ) -> Result<SessionMetrics, ClientError> {
        match self
            .send(&Request::GetMetrics {
                session_id: session_id.clone(),
            })
            .await?
        {
            Response::Metrics { metrics, .. } => Ok(metrics),
            other => Err(ClientError::Daemon(format!(
                "Unexpected response: {other:?}"
            ))),
        }
    }

    /// Get command history for a session.
    ///
    /// # Errors
    /// Returns `ClientError` on communication or daemon errors.
    pub async fn get_history(
        &mut self,
        session_id: &SessionId,
        depth: HistoryDepth,
        limit: usize,
    ) -> Result<Vec<Command>, ClientError> {
        match self
            .send(&Request::GetHistory {
                session_id: session_id.clone(),
                depth,
                limit,
            })
            .await?
        {
            Response::History { commands, .. } => Ok(commands),
            other => Err(ClientError::Daemon(format!(
                "Unexpected response: {other:?}"
            ))),
        }
    }

    /// Subscribe to real-time updates from the daemon.
    ///
    /// # Errors
    /// Returns `ClientError` on communication or daemon errors.
    pub async fn subscribe(&mut self) -> Result<(), ClientError> {
        match self.send(&Request::Subscribe).await? {
            Response::Subscribed => Ok(()),
            other => Err(ClientError::Daemon(format!(
                "Unexpected response: {other:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let req = Request::ListSessions;
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("list_sessions"));
        Ok(())
    }

    #[test]
    fn test_get_metrics_request() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let req = Request::GetMetrics {
            session_id: SessionId::new_unchecked("test-123"),
        };
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("get_metrics"));
        assert!(json.contains("test-123"));
        Ok(())
    }

    #[test]
    fn test_get_history_request_serialization(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let req = Request::GetHistory {
            session_id: SessionId::new_unchecked("session-456"),
            depth: HistoryDepth::FullConversation,
            limit: 50,
        };
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("get_history"));
        assert!(json.contains("session-456"));
        assert!(json.contains("full_conversation"));
        assert!(json.contains("50"));
        Ok(())
    }

    #[test]
    fn test_subscribe_request_serialization() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let req = Request::Subscribe;
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("subscribe"));
        Ok(())
    }

    #[test]
    fn test_client_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "socket not found");
        let client_err = ClientError::Io(io_err);
        let msg = client_err.to_string();
        assert!(msg.contains("IO error"));
        assert!(msg.contains("socket not found"));
    }

    #[test]
    fn test_client_error_display_json() {
        let result = serde_json::from_str::<Request>("invalid");
        assert!(result.is_err());
        if let Err(json_err) = result {
            let client_err = ClientError::Json(json_err);
            let msg = client_err.to_string();
            assert!(msg.contains("JSON serialization error"));
        }
    }

    #[test]
    fn test_client_error_display_daemon() {
        let client_err = ClientError::Daemon("Session not found".to_string());
        let msg = client_err.to_string();
        assert!(msg.contains("Daemon error"));
        assert!(msg.contains("Session not found"));
    }

    #[test]
    fn test_client_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let client_err: ClientError = io_err.into();
        assert!(matches!(client_err, ClientError::Io(_)));
    }

    #[test]
    fn test_client_error_from_json() {
        let result = serde_json::from_str::<Request>("{bad}");
        assert!(result.is_err());
        if let Err(json_err) = result {
            let client_err: ClientError = json_err.into();
            assert!(matches!(client_err, ClientError::Json(_)));
        }
    }

    #[tokio::test]
    async fn test_connect_nonexistent_socket() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let result = DaemonClient::connect("/nonexistent/path/to/socket.sock").await;
        assert!(result.is_err());
        if let Err(ClientError::Io(e)) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        } else {
            return Err("Expected ClientError::Io".into());
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_connect_invalid_path_empty() {
        let result = DaemonClient::connect("").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ClientError::Io(_))));
    }

    #[tokio::test]
    async fn test_connect_permission_denied_path() {
        // Try to connect to a path that would require elevated permissions
        let result = DaemonClient::connect("/root/protected.sock").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ClientError::Io(_))));
    }

    #[test]
    fn test_response_error_deserialization() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let json = r#"{"type":"error","message":"Something went wrong"}"#;
        let response: Response = serde_json::from_str(json)?;
        assert!(
            matches!(response, Response::Error { message } if message == "Something went wrong")
        );
        Ok(())
    }

    #[test]
    fn test_response_sessions_deserialization(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"type":"sessions","sessions":[]}"#;
        let response: Response = serde_json::from_str(json)?;
        assert!(matches!(response, Response::Sessions { sessions } if sessions.is_empty()));
        Ok(())
    }

    #[test]
    fn test_response_subscribed_deserialization(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"type":"subscribed"}"#;
        let response: Response = serde_json::from_str(json)?;
        assert!(matches!(response, Response::Subscribed));
        Ok(())
    }

    #[test]
    fn test_history_depth_variants_serialization(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        assert!(serde_json::to_string(&HistoryDepth::ToolCallsOnly)?.contains("tool_calls_only"));
        assert!(serde_json::to_string(&HistoryDepth::WithResponses)?.contains("with_responses"));
        assert!(
            serde_json::to_string(&HistoryDepth::FullConversation)?.contains("full_conversation")
        );
        Ok(())
    }

    #[test]
    fn test_client_error_debug_impl() {
        let err = ClientError::Daemon("test error".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Daemon"));
        assert!(debug_str.contains("test error"));
    }
}
