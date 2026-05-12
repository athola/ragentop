//! Core domain types - pure data structures.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// A non-negative USD amount with microcurrency precision.
///
/// `1 UsdMicros` = `$0.000001`. Stored as `u64`, so the representable range
/// is `$0` through `$18,446,744,073,709.55`. Sign-free by construction:
/// money cannot go negative, so refunds (if ever needed) would be modeled
/// as separate events rather than negative balances.
///
/// **Why not `f64`?** Accumulating session cost over thousands of API calls
/// drifts under IEEE-754 binary fractions (0.10 + 0.20 != 0.30). `UsdMicros`
/// performs exact integer arithmetic on micro-dollars and only converts to
/// `f64` at display/serde boundaries.
///
/// **Arithmetic:** Use [`Self::saturating_add`] / [`Self::saturating_sub`].
/// The type intentionally does NOT implement `+` / `-` operators to keep
/// over/underflow handling explicit at every call site.
///
/// **Wire format:** Serializes as an `f64` dollar value (e.g. `0.15`) for
/// backward compatibility and human readability. Deserialization rejects
/// NaN and infinities (returns a serde error) and clamps negative inputs
/// to [`Self::ZERO`]. Non-finite values signal a bug or corrupted source,
/// so erroring is preferred over silent ZERO substitution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct UsdMicros(u64);

impl UsdMicros {
    /// `$0.000000`.
    pub const ZERO: Self = Self(0);

    /// Number of micros in one US dollar.
    pub const PER_USD: u64 = 1_000_000;

    /// Constructs from a raw micros count (1 = $0.000001).
    #[must_use]
    pub const fn from_micros(m: u64) -> Self {
        Self(m)
    }

    /// Constructs from whole dollars (e.g. `from_usd(5)` = $5.00).
    ///
    /// Saturates at `Self::MAX` if `d * PER_USD` overflows `u64`.
    #[must_use]
    pub const fn from_usd(d: u64) -> Self {
        Self(d.saturating_mul(Self::PER_USD))
    }

    /// Constructs from an f64 dollar amount.
    ///
    /// Clamps NaN, negative, and non-finite values to [`Self::ZERO`].
    /// Use only at boundaries (config TOML, wire format, legacy callers);
    /// prefer [`Self::from_micros`] / [`Self::from_usd`] internally.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "value is clamped to [0, u64::MAX as f64] before u64 cast; \
                  Self::PER_USD (1_000_000) and u64::MAX as f64 are clamp \
                  bounds, not arithmetic operands requiring exact precision"
    )]
    pub fn from_dollars(d: f64) -> Self {
        if !d.is_finite() || d <= 0.0 {
            return Self::ZERO;
        }
        let micros = (d * Self::PER_USD as f64).clamp(0.0, u64::MAX as f64);
        Self(micros as u64)
    }

    /// Returns the raw micros count.
    #[must_use]
    pub const fn as_micros(self) -> u64 {
        self.0
    }

    /// Converts to f64 dollars for display and wire format.
    ///
    /// Exact for amounts below `2^53` micros (~$9 billion); above that
    /// the conversion is lossy.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "documented loss above 2^53 micros (~$9B)"
    )]
    pub fn as_f64(self) -> f64 {
        (self.0 as f64) / (Self::PER_USD as f64)
    }

    /// Saturating addition.
    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction (clamps at zero).
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

impl std::fmt::Display for UsdMicros {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dollars = self.0 / Self::PER_USD;
        let fract = self.0 % Self::PER_USD;
        write!(f, "${dollars}.{fract:06}")
    }
}

impl Serialize for UsdMicros {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.as_f64().serialize(s)
    }
}

impl<'de> Deserialize<'de> for UsdMicros {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let dollars = f64::deserialize(d)?;
        if dollars.is_nan() {
            return Err(serde::de::Error::custom(
                "UsdMicros cannot be deserialized from NaN",
            ));
        }
        if dollars.is_infinite() {
            return Err(serde::de::Error::custom(
                "UsdMicros cannot be deserialized from infinity",
            ));
        }
        Ok(Self::from_dollars(dollars))
    }
}

/// Supported AI coding agent types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
    Copilot,
    Qwen,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
            Self::Gemini => write!(f, "gemini"),
            Self::Copilot => write!(f, "copilot"),
            Self::Qwen => write!(f, "qwen"),
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
    ///
    /// Callers must ensure the ID is non-empty. This method exists for
    /// performance in internal code paths where the ID has already been
    /// validated or comes from a trusted source.
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

/// Telemetry connection status for an agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum TelemetryStatus {
    /// Successfully connected to telemetry source.
    Connected,
    /// Connection attempt used wrong port/endpoint.
    WrongPort,
    /// Telemetry is not configured for this agent.
    NotConfigured,
    /// Status is not yet determined.
    #[default]
    Unknown,
}

impl std::fmt::Display for TelemetryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connected => write!(f, "connected"),
            Self::WrongPort => write!(f, "wrong_port"),
            Self::NotConfigured => write!(f, "not_configured"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Activity classification based on time since last event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum ActivityStatus {
    /// Event within the last 30 seconds.
    Active,
    /// Event between 30 seconds and 5 minutes ago.
    Idle,
    /// No event for over 5 minutes, or no events at all.
    Done,
}

impl std::fmt::Display for ActivityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Idle => write!(f, "idle"),
            Self::Done => write!(f, "done"),
        }
    }
}

/// 30 seconds: threshold between Active and Idle.
const ACTIVE_THRESHOLD: Duration = Duration::from_secs(30);
/// 5 minutes: threshold between Idle and Done.
const IDLE_THRESHOLD: Duration = Duration::from_mins(5);

/// Classify activity based on elapsed time since last event.
///
/// - Active: last event within 30 seconds
/// - Idle: last event between 30 seconds and 5 minutes
/// - Done: last event over 5 minutes ago, or no event at all
///
/// # Examples
/// ```
/// use ragentop_core::types::{classify_activity, ActivityStatus};
/// use std::time::SystemTime;
///
/// // No event at all -> Done
/// assert_eq!(classify_activity(None, SystemTime::now()), ActivityStatus::Done);
///
/// // Just happened -> Active
/// assert_eq!(
///     classify_activity(Some(SystemTime::now()), SystemTime::now()),
///     ActivityStatus::Active,
/// );
/// ```
#[must_use]
pub fn classify_activity(last_event: Option<SystemTime>, now: SystemTime) -> ActivityStatus {
    last_event.map_or(ActivityStatus::Done, |last| {
        let elapsed = now.duration_since(last).unwrap_or(Duration::ZERO);
        if elapsed <= ACTIVE_THRESHOLD {
            ActivityStatus::Active
        } else if elapsed <= IDLE_THRESHOLD {
            ActivityStatus::Idle
        } else {
            ActivityStatus::Done
        }
    })
}

/// Information about a detected agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
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
    /// When the most recent event was received from this session.
    #[serde(default)]
    pub last_event_at: Option<SystemTime>,
    /// Telemetry connection status.
    #[serde(default)]
    pub telemetry_status: TelemetryStatus,
}

impl AgentSession {
    /// Creates a new `AgentSession` with the required fields; optional fields default to `None`/`Unknown`.
    ///
    /// Optional fields are `pub`, so callers set them by direct assignment after
    /// construction:
    ///
    /// ```
    /// use ragentop_core::{AgentSession, AgentType, SessionId, SessionStatus};
    /// let mut session = AgentSession::new(
    ///     SessionId::new_unchecked("sess-1"),
    ///     AgentType::Claude,
    ///     SessionStatus::Active,
    /// );
    /// session.model = Some("claude-opus-4-7".to_string());
    /// ```
    #[must_use]
    pub const fn new(id: SessionId, agent_type: AgentType, status: SessionStatus) -> Self {
        Self {
            id,
            agent_type,
            model: None,
            session_name: None,
            working_dir: None,
            pane_id: None,
            pid: None,
            started_at: None,
            status,
            last_event_at: None,
            telemetry_status: TelemetryStatus::Unknown,
        }
    }
}

/// Metrics for a session at a point in time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SessionMetrics {
    /// Total tokens used in the session (input + output).
    pub token_count: u64,
    /// Cost in US dollars, if available from the agent.
    ///
    /// Stored as [`UsdMicros`] (microcurrency, u64) to avoid f64 drift on
    /// accumulation. The wire format remains an f64 dollar value for
    /// human-readability and backward compatibility. Construct via
    /// [`UsdMicros::from_dollars`] from f64 inputs, or [`UsdMicros::from_micros`]
    /// when the source already speaks micros.
    pub cost_usd: Option<UsdMicros>,
    /// CPU usage percentage, if measurable.
    ///
    /// Valid range: 0.0 to 100.0 (inclusive), finite.
    /// NaN, infinity, and out-of-range values are invalid.
    pub cpu_percent: Option<f32>,
    /// Elapsed time since session start.
    pub duration: Option<Duration>,
    /// Number of commands/tool calls executed.
    pub command_count: u64,
    /// Lines of code added during this session.
    #[serde(default)]
    pub lines_added: u64,
    /// Lines of code removed during this session.
    #[serde(default)]
    pub lines_removed: u64,
    /// Fraction of API requests served from cache (0.0 - 1.0).
    #[serde(default)]
    pub cache_hit_rate: Option<f64>,
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
                Some(UsdMicros::ZERO)
            } else {
                Some(UsdMicros::from_dollars(v))
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
                lines_added: 0,
                lines_removed: 0,
                cache_hit_rate: None,
            },
            issues,
        )
    }

    /// Validates metrics and returns any issues found.
    ///
    /// Does not modify the metrics. Use [`Self::sanitize`] to fix issues.
    ///
    /// Note: `cost_usd` is no longer checked because [`UsdMicros`] is sign-
    /// free and finite by construction. Only [`cpu_percent`](Self::cpu_percent)
    /// can carry invalid state once a `SessionMetrics` exists.
    #[must_use]
    pub fn validate(&self) -> Vec<MetricsValidationIssue> {
        let mut issues = Vec::new();

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
    /// - NaN/Infinity CPU -> None
    /// - Out-of-range CPU -> clamped to [0, 100]
    ///
    /// `cost_usd` and `token_count` are already invariant-preserving by type,
    /// so they pass through unchanged.
    #[must_use]
    pub fn sanitize(&self) -> Self {
        // Reuse the CPU sanitization in Self::new by feeding a guaranteed-valid
        // f64 cost (the type already guarantees the cost is finite & non-negative,
        // so the round-trip through `Self::new` reports no cost issues).
        let cost_dollars = self.cost_usd.map(UsdMicros::as_f64);
        let (mut sanitized, _) = Self::new(
            self.token_count,
            cost_dollars,
            self.cpu_percent,
            self.duration,
            self.command_count,
        );
        sanitized.lines_added = self.lines_added;
        sanitized.lines_removed = self.lines_removed;
        sanitized.cache_hit_rate = self.cache_hit_rate;
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

    // -- UsdMicros --

    #[test]
    fn usd_micros_zero_is_zero() {
        assert_eq!(UsdMicros::ZERO.as_micros(), 0);
        assert!((UsdMicros::ZERO.as_f64() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn usd_micros_from_dollars_roundtrip_within_precision() {
        let m = UsdMicros::from_dollars(0.15);
        assert_eq!(m.as_micros(), 150_000);
        assert!((m.as_f64() - 0.15).abs() < 1e-9);
    }

    #[test]
    fn usd_micros_from_dollars_clamps_negative_to_zero() {
        assert_eq!(UsdMicros::from_dollars(-5.0), UsdMicros::ZERO);
        assert_eq!(UsdMicros::from_dollars(-0.0001), UsdMicros::ZERO);
    }

    #[test]
    fn usd_micros_from_dollars_clamps_nan_and_infinity_to_zero() {
        assert_eq!(UsdMicros::from_dollars(f64::NAN), UsdMicros::ZERO);
        assert_eq!(UsdMicros::from_dollars(f64::INFINITY), UsdMicros::ZERO);
        assert_eq!(UsdMicros::from_dollars(f64::NEG_INFINITY), UsdMicros::ZERO);
    }

    #[test]
    fn usd_micros_saturating_add_does_not_overflow() {
        let near_max = UsdMicros::from_micros(u64::MAX - 10);
        let plus = UsdMicros::from_micros(100);
        assert_eq!(near_max.saturating_add(plus).as_micros(), u64::MAX);
    }

    #[test]
    fn usd_micros_saturating_sub_clamps_at_zero() {
        let small = UsdMicros::from_dollars(0.10);
        let big = UsdMicros::from_dollars(1.00);
        assert_eq!(small.saturating_sub(big), UsdMicros::ZERO);
    }

    #[test]
    fn usd_micros_accumulates_exactly_across_thousands_of_calls() {
        // The bug we are preventing: 10_000 * $0.10 should be exactly $1000.
        let increment = UsdMicros::from_dollars(0.10);
        let mut total = UsdMicros::ZERO;
        for _ in 0..10_000 {
            total = total.saturating_add(increment);
        }
        assert_eq!(total.as_micros(), 1_000_000_000); // $1000.000000
        assert!((total.as_f64() - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn usd_micros_display_pads_to_six_digits() {
        assert_eq!(UsdMicros::from_micros(150_000).to_string(), "$0.150000");
        assert_eq!(UsdMicros::from_micros(1).to_string(), "$0.000001");
        assert_eq!(UsdMicros::from_usd(5).to_string(), "$5.000000");
    }

    #[test]
    fn usd_micros_serde_uses_f64_dollars_format() -> Result<(), Box<dyn std::error::Error>> {
        // Wire-format backward-compatibility: ten cents -> 0.1, not 100000.
        let m = UsdMicros::from_dollars(0.10);
        let json = serde_json::to_string(&m)?;
        assert_eq!(json, "0.1");
        let parsed: UsdMicros = serde_json::from_str("0.1")?;
        assert_eq!(parsed, m);
        Ok(())
    }

    #[test]
    fn usd_micros_deserialize_clamps_invalid_inputs() -> Result<(), Box<dyn std::error::Error>> {
        let negative: UsdMicros = serde_json::from_str("-1.0")?;
        assert_eq!(negative, UsdMicros::ZERO);
        let zero: UsdMicros = serde_json::from_str("0.0")?;
        assert_eq!(zero, UsdMicros::ZERO);
        Ok(())
    }

    #[test]
    fn usd_micros_deserialize_errors_on_nan() {
        use serde::de::IntoDeserializer;
        let de: serde::de::value::F64Deserializer<serde::de::value::Error> =
            f64::NAN.into_deserializer();
        assert!(UsdMicros::deserialize(de).is_err());
    }

    #[test]
    fn usd_micros_deserialize_errors_on_infinity() {
        use serde::de::IntoDeserializer;
        let de: serde::de::value::F64Deserializer<serde::de::value::Error> =
            f64::INFINITY.into_deserializer();
        assert!(UsdMicros::deserialize(de).is_err());
    }

    #[test]
    fn usd_micros_deserialize_errors_on_neg_infinity() {
        use serde::de::IntoDeserializer;
        let de: serde::de::value::F64Deserializer<serde::de::value::Error> =
            f64::NEG_INFINITY.into_deserializer();
        assert!(UsdMicros::deserialize(de).is_err());
    }

    #[test]
    fn usd_micros_from_dollars_f64_max_saturates() {
        // f64::MAX * PER_USD overflows to INFINITY; clamp must pin to
        // u64::MAX micros, not wrap or panic.
        let m = UsdMicros::from_dollars(f64::MAX);
        assert_eq!(m.as_micros(), u64::MAX);
    }

    #[test]
    fn usd_micros_from_dollars_huge_finite_saturates() {
        // 1e20 dollars > u64::MAX micros (~1.84e19); must saturate.
        let m = UsdMicros::from_dollars(1e20);
        assert_eq!(m.as_micros(), u64::MAX);
    }

    #[test]
    fn usd_micros_ordering() {
        assert!(UsdMicros::from_dollars(0.5) < UsdMicros::from_dollars(1.0));
        assert!(UsdMicros::from_dollars(2.0) > UsdMicros::from_dollars(1.0));
    }

    // -- AgentType --

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Claude.to_string(), "claude");
        assert_eq!(AgentType::Codex.to_string(), "codex");
    }

    #[test]
    fn test_agent_type_serde() -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(&AgentType::Claude)?;
        assert_eq!(json, "\"claude\"");
        let parsed: AgentType = serde_json::from_str("\"gemini\"")?;
        assert_eq!(parsed, AgentType::Gemini);
        Ok(())
    }

    #[test]
    fn test_session_id_valid() -> Result<(), Box<dyn std::error::Error>> {
        let id = SessionId::new("session-123")?;
        assert_eq!(id.as_str(), "session-123");
        assert!(SessionId::new("abc_DEF-123").is_ok());
        Ok(())
    }

    #[test]
    fn test_session_id_invalid() {
        assert!(SessionId::new("").is_err());
        assert!(SessionId::new("a".repeat(256)).is_err());
        assert!(SessionId::new("foo bar").is_err());
        assert!(SessionId::new("foo;bar").is_err());
    }

    #[test]
    fn test_session_id_boundary_255_chars_valid() -> Result<(), Box<dyn std::error::Error>> {
        let id_255 = "a".repeat(255);
        assert!(SessionId::new(&id_255).is_ok());
        assert_eq!(SessionId::new(&id_255)?.as_str().len(), 255);
        Ok(())
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
            Some(Duration::from_mins(1)),
            42,
        );
        assert!(issues.is_empty());
        assert_eq!(metrics.cost_usd, Some(UsdMicros::from_dollars(0.15)));
        assert_eq!(metrics.cpu_percent, Some(25.5));
        assert!(metrics.is_valid());
    }

    #[test]
    fn test_session_metrics_negative_cost() {
        let (metrics, issues) = SessionMetrics::new(0, Some(-5.0), None, None, 0);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], MetricsValidationIssue::NegativeCost(_)));
        // Negative cost is clamped to ZERO
        assert_eq!(metrics.cost_usd, Some(UsdMicros::ZERO));
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
    fn test_session_metrics_validate_cpu_only() {
        // cost_usd cannot be invalid post-construction (UsdMicros is sign-free
        // and finite by type). Only cpu_percent can carry invalid state.
        let metrics = SessionMetrics {
            token_count: 0,
            cost_usd: Some(UsdMicros::from_dollars(1.0)),
            cpu_percent: Some(200.0),
            duration: None,
            command_count: 0,
            lines_added: 0,
            lines_removed: 0,
            cache_hit_rate: None,
        };
        let issues = metrics.validate();
        assert_eq!(issues.len(), 1);
        assert!(!metrics.is_valid());
        assert!(matches!(
            issues[0],
            MetricsValidationIssue::CpuOutOfRange(_)
        ));
    }

    #[test]
    fn test_session_metrics_sanitize_passes_through_cost_clamps_cpu() {
        // cost_usd is already UsdMicros (invariant-safe), so sanitize is a
        // pass-through for cost. Out-of-range cpu gets clamped to [0,100].
        let metrics = SessionMetrics {
            token_count: 100,
            cost_usd: Some(UsdMicros::from_dollars(2.50)),
            cpu_percent: Some(150.0),
            duration: Some(Duration::from_secs(30)),
            command_count: 5,
            lines_added: 42,
            lines_removed: 10,
            cache_hit_rate: Some(0.75),
        };
        let sanitized = metrics.sanitize();
        assert_eq!(sanitized.cost_usd, Some(UsdMicros::from_dollars(2.50)));
        assert_eq!(sanitized.cpu_percent, Some(100.0));
        assert!(sanitized.is_valid());
        // Other fields preserved
        assert_eq!(sanitized.token_count, 100);
        assert_eq!(sanitized.command_count, 5);
        assert_eq!(sanitized.lines_added, 42);
        assert_eq!(sanitized.lines_removed, 10);
        assert_eq!(sanitized.cache_hit_rate, Some(0.75));
    }

    #[test]
    fn test_session_metrics_boundary_values() {
        // Zero cost is valid
        let (metrics, issues) = SessionMetrics::new(0, Some(0.0), Some(0.0), None, 0);
        assert!(issues.is_empty());
        assert_eq!(metrics.cost_usd, Some(UsdMicros::ZERO));
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

    #[test]
    fn test_session_metrics_new_fields_default() {
        let (metrics, _) = SessionMetrics::new(100, Some(1.0), None, None, 5);
        assert_eq!(metrics.lines_added, 0);
        assert_eq!(metrics.lines_removed, 0);
        assert_eq!(metrics.cache_hit_rate, None);
    }

    // -- TelemetryStatus --

    #[test]
    fn telemetry_status_default_is_unknown() {
        assert_eq!(TelemetryStatus::default(), TelemetryStatus::Unknown);
    }

    #[test]
    fn telemetry_status_display() {
        assert_eq!(TelemetryStatus::Connected.to_string(), "connected");
        assert_eq!(TelemetryStatus::WrongPort.to_string(), "wrong_port");
        assert_eq!(TelemetryStatus::NotConfigured.to_string(), "not_configured");
        assert_eq!(TelemetryStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn telemetry_status_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let variants = [
            TelemetryStatus::Connected,
            TelemetryStatus::WrongPort,
            TelemetryStatus::NotConfigured,
            TelemetryStatus::Unknown,
        ];
        for v in variants {
            let json = serde_json::to_string(&v)?;
            let parsed: TelemetryStatus = serde_json::from_str(&json)?;
            assert_eq!(parsed, v);
        }
        Ok(())
    }

    // -- ActivityStatus --

    #[test]
    fn activity_status_display() {
        assert_eq!(ActivityStatus::Active.to_string(), "active");
        assert_eq!(ActivityStatus::Idle.to_string(), "idle");
        assert_eq!(ActivityStatus::Done.to_string(), "done");
    }

    #[test]
    fn activity_status_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for v in [
            ActivityStatus::Active,
            ActivityStatus::Idle,
            ActivityStatus::Done,
        ] {
            let json = serde_json::to_string(&v)?;
            let parsed: ActivityStatus = serde_json::from_str(&json)?;
            assert_eq!(parsed, v);
        }
        Ok(())
    }

    // -- classify_activity --

    #[test]
    fn classify_activity_none_is_done() {
        assert_eq!(
            classify_activity(None, SystemTime::now()),
            ActivityStatus::Done
        );
    }

    #[test]
    fn classify_activity_just_now_is_active() {
        let now = SystemTime::now();
        assert_eq!(classify_activity(Some(now), now), ActivityStatus::Active);
    }

    #[test]
    fn classify_activity_10_sec_ago_is_active() {
        let now = SystemTime::now();
        let last = now - Duration::from_secs(10);
        assert_eq!(classify_activity(Some(last), now), ActivityStatus::Active);
    }

    #[test]
    fn classify_activity_30_sec_boundary_is_active() {
        let now = SystemTime::now();
        let last = now - Duration::from_secs(30);
        assert_eq!(classify_activity(Some(last), now), ActivityStatus::Active);
    }

    #[test]
    fn classify_activity_31_sec_is_idle() {
        let now = SystemTime::now();
        let last = now - Duration::from_secs(31);
        assert_eq!(classify_activity(Some(last), now), ActivityStatus::Idle);
    }

    #[test]
    fn classify_activity_2_min_is_idle() {
        let now = SystemTime::now();
        let last = now - Duration::from_mins(2);
        assert_eq!(classify_activity(Some(last), now), ActivityStatus::Idle);
    }

    #[test]
    fn classify_activity_5_min_boundary_is_idle() {
        let now = SystemTime::now();
        let last = now - Duration::from_mins(5);
        assert_eq!(classify_activity(Some(last), now), ActivityStatus::Idle);
    }

    #[test]
    fn classify_activity_301_sec_is_done() {
        let now = SystemTime::now();
        let last = now - Duration::from_secs(301);
        assert_eq!(classify_activity(Some(last), now), ActivityStatus::Done);
    }

    #[test]
    fn classify_activity_1_hour_is_done() {
        let now = SystemTime::now();
        let last = now - Duration::from_hours(1);
        assert_eq!(classify_activity(Some(last), now), ActivityStatus::Done);
    }

    // -- AgentSession with new fields --

    #[test]
    fn agent_session_new_fields_serde() -> Result<(), Box<dyn std::error::Error>> {
        let session = AgentSession {
            id: SessionId::new_unchecked("test"),
            agent_type: AgentType::Claude,
            model: None,
            session_name: None,
            working_dir: None,
            pane_id: None,
            pid: None,
            started_at: None,
            status: SessionStatus::Active,
            last_event_at: Some(SystemTime::now()),
            telemetry_status: TelemetryStatus::Connected,
        };
        let json = serde_json::to_string(&session)?;
        let parsed: AgentSession = serde_json::from_str(&json)?;
        assert_eq!(parsed.telemetry_status, TelemetryStatus::Connected);
        assert!(parsed.last_event_at.is_some());
        Ok(())
    }

    #[test]
    fn agent_session_deserialize_without_new_fields() -> Result<(), Box<dyn std::error::Error>> {
        // Simulate JSON from an older version without the new fields
        let json = r#"{
            "id": "test",
            "agent_type": "claude",
            "model": null,
            "session_name": null,
            "working_dir": null,
            "pane_id": null,
            "pid": null,
            "started_at": null,
            "status": "active"
        }"#;
        let parsed: AgentSession = serde_json::from_str(json)?;
        assert_eq!(parsed.last_event_at, None);
        assert_eq!(parsed.telemetry_status, TelemetryStatus::Unknown);
        Ok(())
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
