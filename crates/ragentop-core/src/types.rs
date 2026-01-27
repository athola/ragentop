//! Core domain types - pure data structures.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Supported AI coding agent types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
    Copilot,
    Qwen,
    Glm,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
            Self::Gemini => write!(f, "gemini"),
            Self::Copilot => write!(f, "copilot"),
            Self::Qwen => write!(f, "qwen"),
            Self::Glm => write!(f, "glm"),
        }
    }
}

/// Unique identifier for an agent session.
///
/// This is an opaque identifier - use [`SessionId::new`] to create and
/// [`SessionId::as_str`] to access the underlying value.
///
/// # Constraints
/// - Must not be empty
/// - Maximum 255 characters
/// - Only alphanumeric characters, dashes (`-`), and underscores (`_`) allowed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Creates a new session ID with validation.
    ///
    /// # Errors
    /// Returns an error if the ID is empty, exceeds 255 characters, or contains
    /// characters other than alphanumeric, dash, or underscore.
    ///
    /// # Examples
    /// ```
    /// use ragentop_core::SessionId;
    /// let id = SessionId::new("session-123").unwrap();
    /// assert_eq!(id.as_str(), "session-123");
    /// ```
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        let id = id.into();
        if id.is_empty() {
            return Err("session ID cannot be empty");
        }
        if id.len() > 255 {
            return Err("session ID cannot exceed 255 characters");
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "session ID can only contain alphanumeric characters, dashes, and underscores",
            );
        }
        Ok(Self(id))
    }

    /// Creates a session ID without validation.
    ///
    /// # Safety
    /// This bypasses validation. Use only when the ID is known to be valid
    /// (e.g., from trusted internal sources or deserialization).
    #[must_use]
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the session ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Current status of an agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Idle,
    Paused,
}

/// Information about a detected agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: SessionId,
    pub agent_type: AgentType,
    pub model: Option<String>,
    pub session_name: Option<String>,
    pub working_dir: Option<PathBuf>,
    pub pane_id: Option<String>,
    pub pid: Option<u32>,
    pub started_at: Option<SystemTime>,
    pub status: SessionStatus,
}

/// Metrics for a session at a point in time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total tokens used in the session (input + output).
    pub token_count: u64,
    /// Cost in US dollars, if available from the agent.
    ///
    /// Valid range: non-negative finite values (>= 0.0).
    /// NaN, infinity, and negative values are invalid.
    pub cost_usd: Option<f64>,
    /// CPU usage percentage, if measurable.
    ///
    /// Valid range: 0.0 to 100.0 (inclusive), finite.
    /// NaN, infinity, and out-of-range values are invalid.
    pub cpu_percent: Option<f32>,
    /// Elapsed time since session start.
    pub duration: Option<Duration>,
    /// Number of commands/tool calls executed.
    pub command_count: u64,
}

/// Validation issue found in metrics.
#[derive(Debug, Clone, PartialEq)]
pub enum MetricsValidationIssue {
    /// `cost_usd` was negative.
    NegativeCost(f64),
    /// `cost_usd` was NaN or infinity.
    InvalidCost(f64),
    /// `cpu_percent` was outside 0-100 range.
    CpuOutOfRange(f32),
    /// `cpu_percent` was NaN or infinity.
    InvalidCpu(f32),
}

impl std::fmt::Display for MetricsValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeCost(v) => write!(f, "negative cost_usd: {v}"),
            Self::InvalidCost(v) => write!(f, "invalid cost_usd (NaN/Inf): {v}"),
            Self::CpuOutOfRange(v) => write!(f, "cpu_percent out of range [0,100]: {v}"),
            Self::InvalidCpu(v) => write!(f, "invalid cpu_percent (NaN/Inf): {v}"),
        }
    }
}

impl SessionMetrics {
    /// Creates new metrics with validation.
    ///
    /// Returns the metrics and any validation issues found.
    /// Invalid values are sanitized (clamped or set to None).
    #[must_use]
    pub fn new(
        token_count: u64,
        cost_usd: Option<f64>,
        cpu_percent: Option<f32>,
        duration: Option<Duration>,
        command_count: u64,
    ) -> (Self, Vec<MetricsValidationIssue>) {
        let mut issues = Vec::new();

        let sanitized_cost = cost_usd.and_then(|v| {
            if !v.is_finite() {
                issues.push(MetricsValidationIssue::InvalidCost(v));
                None
            } else if v < 0.0 {
                issues.push(MetricsValidationIssue::NegativeCost(v));
                Some(0.0)
            } else {
                Some(v)
            }
        });

        let sanitized_cpu = cpu_percent.and_then(|v| {
            if !v.is_finite() {
                issues.push(MetricsValidationIssue::InvalidCpu(v));
                None
            } else if !(0.0..=100.0).contains(&v) {
                issues.push(MetricsValidationIssue::CpuOutOfRange(v));
                Some(v.clamp(0.0, 100.0))
            } else {
                Some(v)
            }
        });

        (
            Self {
                token_count,
                cost_usd: sanitized_cost,
                cpu_percent: sanitized_cpu,
                duration,
                command_count,
            },
            issues,
        )
    }

    /// Validates metrics and returns any issues found.
    ///
    /// Does not modify the metrics. Use [`Self::sanitize`] to fix issues.
    #[must_use]
    pub fn validate(&self) -> Vec<MetricsValidationIssue> {
        let mut issues = Vec::new();

        if let Some(cost) = self.cost_usd {
            if !cost.is_finite() {
                issues.push(MetricsValidationIssue::InvalidCost(cost));
            } else if cost < 0.0 {
                issues.push(MetricsValidationIssue::NegativeCost(cost));
            }
        }

        if let Some(cpu) = self.cpu_percent {
            if !cpu.is_finite() {
                issues.push(MetricsValidationIssue::InvalidCpu(cpu));
            } else if !(0.0..=100.0).contains(&cpu) {
                issues.push(MetricsValidationIssue::CpuOutOfRange(cpu));
            }
        }

        issues
    }

    /// Returns true if all numeric values are valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// Returns a sanitized copy with invalid values fixed.
    ///
    /// - NaN/Infinity cost → None
    /// - Negative cost → 0.0
    /// - NaN/Infinity CPU → None
    /// - Out-of-range CPU → clamped to [0, 100]
    #[must_use]
    pub fn sanitize(&self) -> Self {
        let (sanitized, _) = Self::new(
            self.token_count,
            self.cost_usd,
            self.cpu_percent,
            self.duration,
            self.command_count,
        );
        sanitized
    }
}

/// A command executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub timestamp: SystemTime,
    pub tool: String,
    pub args: String,
    pub status: CommandStatus,
    pub result_summary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Success,
    Failed,
    Running,
}

/// History depth for command queries.
///
/// Controls how much detail is returned when fetching command history.
/// Higher levels include more context but consume more bandwidth/memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryDepth {
    /// Tool calls only - just the command names and arguments (level 1)
    ToolCallsOnly,
    /// Tool calls with abbreviated responses - includes truncated output (level 2)
    #[default]
    WithResponses,
    /// Full conversation turns - complete input/output (level 3)
    FullConversation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Claude.to_string(), "claude");
        assert_eq!(AgentType::Codex.to_string(), "codex");
    }

    #[test]
    fn test_agent_type_serde() {
        let json = serde_json::to_string(&AgentType::Claude).unwrap();
        assert_eq!(json, "\"claude\"");
        let parsed: AgentType = serde_json::from_str("\"gemini\"").unwrap();
        assert_eq!(parsed, AgentType::Gemini);
    }

    #[test]
    fn test_session_id_valid() {
        let id = SessionId::new("session-123").unwrap();
        assert_eq!(id.as_str(), "session-123");
        assert!(SessionId::new("abc_DEF-123").is_ok());
    }

    #[test]
    fn test_session_id_invalid() {
        assert!(SessionId::new("").is_err());
        assert!(SessionId::new("a".repeat(256)).is_err());
        assert!(SessionId::new("foo bar").is_err());
        assert!(SessionId::new("foo;bar").is_err());
    }

    #[test]
    fn test_session_id_boundary_255_chars_valid() {
        let id_255 = "a".repeat(255);
        assert!(SessionId::new(&id_255).is_ok());
        assert_eq!(SessionId::new(&id_255).unwrap().as_str().len(), 255);
    }

    #[test]
    fn test_session_id_boundary_256_chars_invalid() {
        let id_256 = "a".repeat(256);
        assert!(SessionId::new(&id_256).is_err());
    }

    #[test]
    fn test_session_id_single_char_valid() {
        assert!(SessionId::new("a").is_ok());
        assert!(SessionId::new("_").is_ok());
        assert!(SessionId::new("-").is_ok());
    }

    #[test]
    fn test_session_id_unchecked() {
        let id = SessionId::new_unchecked("any-value");
        assert_eq!(id.as_str(), "any-value");
    }

    #[test]
    fn test_history_depth_default() {
        assert_eq!(HistoryDepth::default(), HistoryDepth::WithResponses);
    }

    #[test]
    fn test_session_metrics_valid() {
        let (metrics, issues) = SessionMetrics::new(
            1000,
            Some(0.15),
            Some(25.5),
            Some(Duration::from_secs(60)),
            42,
        );
        assert!(issues.is_empty());
        assert_eq!(metrics.cost_usd, Some(0.15));
        assert_eq!(metrics.cpu_percent, Some(25.5));
        assert!(metrics.is_valid());
    }

    #[test]
    fn test_session_metrics_negative_cost() {
        let (metrics, issues) = SessionMetrics::new(0, Some(-5.0), None, None, 0);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], MetricsValidationIssue::NegativeCost(_)));
        // Negative cost is clamped to 0
        assert_eq!(metrics.cost_usd, Some(0.0));
    }

    #[test]
    fn test_session_metrics_nan_cost() {
        let (metrics, issues) = SessionMetrics::new(0, Some(f64::NAN), None, None, 0);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], MetricsValidationIssue::InvalidCost(_)));
        // NaN is replaced with None
        assert_eq!(metrics.cost_usd, None);
    }

    #[test]
    fn test_session_metrics_infinity_cost() {
        let (metrics, issues) = SessionMetrics::new(0, Some(f64::INFINITY), None, None, 0);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], MetricsValidationIssue::InvalidCost(_)));
        assert_eq!(metrics.cost_usd, None);
    }

    #[test]
    fn test_session_metrics_cpu_out_of_range() {
        // Above 100
        let (metrics, issues) = SessionMetrics::new(0, None, Some(150.0), None, 0);
        assert_eq!(issues.len(), 1);
        assert!(matches!(
            issues[0],
            MetricsValidationIssue::CpuOutOfRange(_)
        ));
        assert_eq!(metrics.cpu_percent, Some(100.0)); // Clamped

        // Below 0
        let (metrics, issues) = SessionMetrics::new(0, None, Some(-10.0), None, 0);
        assert_eq!(issues.len(), 1);
        assert_eq!(metrics.cpu_percent, Some(0.0)); // Clamped
    }

    #[test]
    fn test_session_metrics_nan_cpu() {
        let (metrics, issues) = SessionMetrics::new(0, None, Some(f32::NAN), None, 0);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], MetricsValidationIssue::InvalidCpu(_)));
        assert_eq!(metrics.cpu_percent, None);
    }

    #[test]
    fn test_session_metrics_validate() {
        let metrics = SessionMetrics {
            token_count: 0,
            cost_usd: Some(-1.0),
            cpu_percent: Some(200.0),
            duration: None,
            command_count: 0,
        };
        let issues = metrics.validate();
        assert_eq!(issues.len(), 2);
        assert!(!metrics.is_valid());
    }

    #[test]
    fn test_session_metrics_sanitize() {
        let metrics = SessionMetrics {
            token_count: 100,
            cost_usd: Some(-1.0),
            cpu_percent: Some(150.0),
            duration: Some(Duration::from_secs(30)),
            command_count: 5,
        };
        let sanitized = metrics.sanitize();
        assert_eq!(sanitized.cost_usd, Some(0.0));
        assert_eq!(sanitized.cpu_percent, Some(100.0));
        assert!(sanitized.is_valid());
        // Other fields preserved
        assert_eq!(sanitized.token_count, 100);
        assert_eq!(sanitized.command_count, 5);
    }

    #[test]
    fn test_session_metrics_boundary_values() {
        // Zero cost is valid
        let (metrics, issues) = SessionMetrics::new(0, Some(0.0), Some(0.0), None, 0);
        assert!(issues.is_empty());
        assert_eq!(metrics.cost_usd, Some(0.0));
        assert_eq!(metrics.cpu_percent, Some(0.0));

        // 100% CPU is valid
        let (metrics, issues) = SessionMetrics::new(0, None, Some(100.0), None, 0);
        assert!(issues.is_empty());
        assert_eq!(metrics.cpu_percent, Some(100.0));
    }

    #[test]
    fn test_metrics_validation_issue_display() {
        assert_eq!(
            MetricsValidationIssue::NegativeCost(-5.0).to_string(),
            "negative cost_usd: -5"
        );
        assert_eq!(
            MetricsValidationIssue::CpuOutOfRange(150.0).to_string(),
            "cpu_percent out of range [0,100]: 150"
        );
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn session_id_valid_chars_accepted(s in "[a-zA-Z0-9_-]{1,255}") {
                prop_assert!(SessionId::new(&s).is_ok());
            }

            #[test]
            fn session_id_rejects_spaces(s in "[a-zA-Z0-9_-]+ [a-zA-Z0-9_-]+") {
                prop_assert!(SessionId::new(&s).is_err());
            }

            #[test]
            fn session_metrics_sanitize_always_valid(
                token_count in any::<u64>(),
                cost in prop::option::of(-1000.0f64..1000.0),
                cpu in prop::option::of(-100.0f32..200.0),
            ) {
                let (metrics, _) = SessionMetrics::new(token_count, cost, cpu, None, 0);
                prop_assert!(metrics.is_valid());
            }
        }
    }
}
