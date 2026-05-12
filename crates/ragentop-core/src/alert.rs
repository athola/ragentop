//! Alert rules and deduplication for agent monitoring.
//!
//! Pure types and functions for defining alert conditions, severity levels,
//! and deduplication logic.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use crate::{SessionId, UsdMicros};

/// An alert that has been triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Alert {
    /// Which rule triggered this alert.
    pub rule: AlertRule,
    /// How severe this alert is.
    pub severity: Severity,
    /// Human-readable description.
    pub message: String,
    /// The session that triggered it, if applicable.
    pub session_id: Option<SessionId>,
    /// When the alert was triggered.
    pub fired_at: SystemTime,
}

/// Classification of alert rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AlertRule {
    /// Cost is increasing rapidly.
    CostSurge,
    /// Token consumption is unusually high.
    RunawayTokens,
    /// Agent appears to be in a loop.
    LoopDetection,
    /// Many errors in a short period.
    ErrorStorm,
    /// Session has been idle too long.
    StaleSession,
    /// Context window is nearly full.
    ContextPressure,
    /// High tool rejection rate.
    HighRejection,
    /// Session cost exceeds configured threshold.
    SessionCost,
}

impl std::fmt::Display for AlertRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CostSurge => write!(f, "cost_surge"),
            Self::RunawayTokens => write!(f, "runaway_tokens"),
            Self::LoopDetection => write!(f, "loop_detection"),
            Self::ErrorStorm => write!(f, "error_storm"),
            Self::StaleSession => write!(f, "stale_session"),
            Self::ContextPressure => write!(f, "context_pressure"),
            Self::HighRejection => write!(f, "high_rejection"),
            Self::SessionCost => write!(f, "session_cost"),
        }
    }
}

/// Alert severity level, ordered from most to least severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Severity {
    /// Informational - no action required.
    Info,
    /// Potential issue - should be monitored.
    Warning,
    /// Serious problem - needs immediate attention.
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Thresholds for triggering alerts.
///
/// Defaults are tuned for typical interactive coding sessions; override via
/// `[alerts]` in `config.toml` for production workloads.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(default)]
pub struct AlertThresholds {
    /// Cost increase per hour that triggers a surge alert (USD).
    pub cost_surge_per_hour: f64,
    /// Token velocity (tokens/sec) that indicates runaway consumption.
    pub runaway_token_velocity: f64,
    /// Number of identical tool calls that indicates a loop.
    pub loop_threshold: u32,
    /// Number of errors in a window that triggers a storm alert.
    pub error_storm_count: u32,
    /// Hours of inactivity before a session is considered stale.
    pub stale_hours: u32,
    /// Context usage percentage that triggers pressure alert.
    pub context_pressure_pct: u8,
    /// Tool rejection rate (0.0-1.0) that triggers high rejection alert.
    pub rejection_rate: f64,
    /// Session cost threshold (USD) that triggers a session cost alert.
    pub session_cost_threshold: f64,
    /// Minutes of sustained high token velocity before triggering runaway alert.
    pub runaway_token_sustained_minutes: u32,
    /// Window in minutes for loop detection.
    pub loop_detector_window_minutes: u32,
    /// Window in minutes for high rejection rate detection.
    pub high_rejection_window_minutes: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cost_surge_per_hour: 2.0,
            runaway_token_velocity: 50_000.0,
            loop_threshold: 3,
            error_storm_count: 10,
            stale_hours: 2,
            context_pressure_pct: 80,
            rejection_rate: 0.50,
            session_cost_threshold: 5.0,
            runaway_token_sustained_minutes: 2,
            loop_detector_window_minutes: 5,
            high_rejection_window_minutes: 5,
        }
    }
}

impl AlertThresholds {
    /// Validate that all threshold values are sensible.
    ///
    /// Returns `Ok(())` if all fields are valid, or `Err(String)` describing
    /// the first invalid field found.
    ///
    /// # Errors
    ///
    /// Returns an error string if any f64 field is negative, NaN, or infinite,
    /// if `rejection_rate` is outside `[0.0, 1.0]`, or if `context_pressure_pct`
    /// exceeds 100.
    pub fn validate(&self) -> Result<(), String> {
        if !self.cost_surge_per_hour.is_finite() || self.cost_surge_per_hour < 0.0 {
            return Err(format!(
                "cost_surge_per_hour must be finite and non-negative, got {}",
                self.cost_surge_per_hour
            ));
        }
        if !self.runaway_token_velocity.is_finite() || self.runaway_token_velocity < 0.0 {
            return Err(format!(
                "runaway_token_velocity must be finite and non-negative, got {}",
                self.runaway_token_velocity
            ));
        }
        if !self.rejection_rate.is_finite()
            || self.rejection_rate < 0.0
            || self.rejection_rate > 1.0
        {
            return Err(format!(
                "rejection_rate must be finite and between 0.0 and 1.0, got {}",
                self.rejection_rate
            ));
        }
        if !self.session_cost_threshold.is_finite() || self.session_cost_threshold < 0.0 {
            return Err(format!(
                "session_cost_threshold must be finite and non-negative, got {}",
                self.session_cost_threshold
            ));
        }
        if self.context_pressure_pct > 100 {
            return Err(format!(
                "context_pressure_pct must be <= 100, got {}",
                self.context_pressure_pct
            ));
        }
        Ok(())
    }
}

impl Alert {
    /// Creates a new `Alert`.
    #[must_use]
    pub fn new(
        rule: AlertRule,
        severity: Severity,
        message: impl Into<String>,
        session_id: Option<SessionId>,
        fired_at: SystemTime,
    ) -> Self {
        Self {
            rule,
            severity,
            message: message.into(),
            session_id,
            fired_at,
        }
    }
}

/// Determine whether a candidate alert should be suppressed as a duplicate.
///
/// An alert is considered a duplicate if there is an existing alert with:
/// - The same rule
/// - The same session ID
/// - Fired within the deduplication window
///
/// This prevents alert fatigue from repeated identical alerts.
///
/// # Examples
/// ```
/// use ragentop_core::alert::{should_dedup, Alert, AlertRule, Severity};
/// use std::time::{Duration, SystemTime};
///
/// let existing = vec![
///     Alert::new(AlertRule::CostSurge, Severity::Warning, "cost surge", None, SystemTime::now()),
/// ];
/// let candidate = Alert::new(
///     AlertRule::CostSurge, Severity::Warning, "cost surge again", None, SystemTime::now(),
/// );
/// assert!(should_dedup(&existing, &candidate, Duration::from_mins(5)));
/// ```
#[must_use]
pub fn should_dedup(existing: &[Alert], candidate: &Alert, window: Duration) -> bool {
    existing.iter().any(|alert| {
        // Same rule
        alert.rule == candidate.rule
            // Same session (both None counts as same)
            && alert.session_id == candidate.session_id
            // Within dedup window: candidate fired_at - alert fired_at < window
            && candidate
                .fired_at
                .duration_since(alert.fired_at)
                .is_ok_and(|elapsed| elapsed < window)
    })
}

/// Check if a session's total cost exceeds the configured threshold.
///
/// `total_cost` is a [`UsdMicros`] so accumulated session cost flows
/// through this function without round-tripping through `f64` (and
/// without losing the exact-micro-dollar guarantee). `threshold` stays
/// `f64` because it comes from human-authored TOML (`[alerts]
/// session_cost_threshold`).
///
/// Returns `Some(AlertRule::SessionCost)` if the cost exceeds the threshold,
/// `None` otherwise.
///
/// # Examples
/// ```
/// use ragentop_core::UsdMicros;
/// use ragentop_core::alert::{check_session_cost, AlertRule};
///
/// assert!(check_session_cost(UsdMicros::from_dollars(6.0), 5.0).is_some());
/// assert!(check_session_cost(UsdMicros::from_dollars(4.0), 5.0).is_none());
/// ```
#[must_use]
pub fn check_session_cost(total_cost: UsdMicros, threshold: f64) -> Option<AlertRule> {
    if total_cost.as_f64() > threshold {
        Some(AlertRule::SessionCost)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_alert(rule: AlertRule, session_id: Option<SessionId>, fired_at: SystemTime) -> Alert {
        Alert {
            rule,
            severity: Severity::Warning,
            message: format!("{rule} alert"),
            session_id,
            fired_at,
        }
    }

    // -- AlertThresholds defaults --

    #[test]
    fn thresholds_default_values() {
        let t = AlertThresholds::default();
        assert!((t.cost_surge_per_hour - 2.0).abs() < f64::EPSILON);
        assert!((t.runaway_token_velocity - 50_000.0).abs() < f64::EPSILON);
        assert_eq!(t.loop_threshold, 3);
        assert_eq!(t.error_storm_count, 10);
        assert_eq!(t.stale_hours, 2);
        assert_eq!(t.context_pressure_pct, 80);
        assert!((t.rejection_rate - 0.50).abs() < f64::EPSILON);
        assert!((t.session_cost_threshold - 5.0).abs() < f64::EPSILON);
        assert_eq!(t.runaway_token_sustained_minutes, 2);
        assert_eq!(t.loop_detector_window_minutes, 5);
        assert_eq!(t.high_rejection_window_minutes, 5);
    }

    // -- Severity ordering --

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Critical);
        assert!(Severity::Info < Severity::Critical);
    }

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Info.to_string(), "info");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Critical.to_string(), "critical");
    }

    // -- AlertRule display --

    #[test]
    fn alert_rule_display() {
        assert_eq!(AlertRule::CostSurge.to_string(), "cost_surge");
        assert_eq!(AlertRule::RunawayTokens.to_string(), "runaway_tokens");
        assert_eq!(AlertRule::LoopDetection.to_string(), "loop_detection");
        assert_eq!(AlertRule::ErrorStorm.to_string(), "error_storm");
        assert_eq!(AlertRule::StaleSession.to_string(), "stale_session");
        assert_eq!(AlertRule::ContextPressure.to_string(), "context_pressure");
        assert_eq!(AlertRule::HighRejection.to_string(), "high_rejection");
        assert_eq!(AlertRule::SessionCost.to_string(), "session_cost");
    }

    // -- should_dedup --

    #[test]
    fn dedup_same_rule_same_session_within_window() {
        let now = SystemTime::now();
        let existing = vec![make_alert(AlertRule::CostSurge, None, now)];
        let candidate = make_alert(AlertRule::CostSurge, None, now);
        assert!(should_dedup(&existing, &candidate, Duration::from_mins(5)));
    }

    #[test]
    fn no_dedup_different_rule() {
        let now = SystemTime::now();
        let existing = vec![make_alert(AlertRule::CostSurge, None, now)];
        let candidate = make_alert(AlertRule::ErrorStorm, None, now);
        assert!(!should_dedup(&existing, &candidate, Duration::from_mins(5)));
    }

    #[test]
    fn no_dedup_different_session() -> Result<(), Box<dyn std::error::Error>> {
        let now = SystemTime::now();
        let sid1 = SessionId::new("s1")?;
        let sid2 = SessionId::new("s2")?;
        let existing = vec![make_alert(AlertRule::CostSurge, Some(sid1), now)];
        let candidate = make_alert(AlertRule::CostSurge, Some(sid2), now);
        assert!(!should_dedup(&existing, &candidate, Duration::from_mins(5)));
        Ok(())
    }

    #[test]
    fn no_dedup_outside_window() {
        let past = SystemTime::UNIX_EPOCH;
        let now = SystemTime::now();
        let existing = vec![make_alert(AlertRule::CostSurge, None, past)];
        let candidate = make_alert(AlertRule::CostSurge, None, now);
        assert!(!should_dedup(&existing, &candidate, Duration::from_mins(5)));
    }

    #[test]
    fn dedup_empty_existing_returns_false() {
        let candidate = make_alert(AlertRule::CostSurge, None, SystemTime::now());
        assert!(!should_dedup(&[], &candidate, Duration::from_mins(5)));
    }

    #[test]
    fn dedup_with_session_ids_matching() -> Result<(), Box<dyn std::error::Error>> {
        let now = SystemTime::now();
        let sid = SessionId::new("sess-1")?;
        let existing = vec![make_alert(AlertRule::LoopDetection, Some(sid.clone()), now)];
        let candidate = make_alert(AlertRule::LoopDetection, Some(sid), now);
        assert!(should_dedup(&existing, &candidate, Duration::from_mins(5)));
        Ok(())
    }

    #[test]
    fn dedup_one_none_one_some_session() -> Result<(), Box<dyn std::error::Error>> {
        let now = SystemTime::now();
        let sid = SessionId::new("sess-1")?;
        let existing = vec![make_alert(AlertRule::CostSurge, None, now)];
        let candidate = make_alert(AlertRule::CostSurge, Some(sid), now);
        assert!(!should_dedup(&existing, &candidate, Duration::from_mins(5)));
        Ok(())
    }

    // -- Serde round-trips --

    #[test]
    fn alert_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let alert = make_alert(AlertRule::ErrorStorm, None, SystemTime::now());
        let json = serde_json::to_string(&alert)?;
        let parsed: Alert = serde_json::from_str(&json)?;
        assert_eq!(parsed.rule, AlertRule::ErrorStorm);
        assert_eq!(parsed.severity, Severity::Warning);
        Ok(())
    }

    #[test]
    fn alert_rule_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let rules = [
            AlertRule::CostSurge,
            AlertRule::RunawayTokens,
            AlertRule::LoopDetection,
            AlertRule::ErrorStorm,
            AlertRule::StaleSession,
            AlertRule::ContextPressure,
            AlertRule::HighRejection,
            AlertRule::SessionCost,
        ];
        for rule in rules {
            let json = serde_json::to_string(&rule)?;
            let parsed: AlertRule = serde_json::from_str(&json)?;
            assert_eq!(parsed, rule);
        }
        Ok(())
    }

    #[test]
    fn severity_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for severity in [Severity::Info, Severity::Warning, Severity::Critical] {
            let json = serde_json::to_string(&severity)?;
            let parsed: Severity = serde_json::from_str(&json)?;
            assert_eq!(parsed, severity);
        }
        Ok(())
    }

    #[test]
    fn alert_thresholds_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let t = AlertThresholds::default();
        let json = serde_json::to_string(&t)?;
        let parsed: AlertThresholds = serde_json::from_str(&json)?;
        assert_eq!(parsed, t);
        Ok(())
    }

    // -- check_session_cost --

    #[test]
    fn session_cost_exceeds_threshold() {
        assert_eq!(
            check_session_cost(UsdMicros::from_dollars(6.0), 5.0),
            Some(AlertRule::SessionCost)
        );
    }

    #[test]
    fn session_cost_below_threshold() {
        assert_eq!(check_session_cost(UsdMicros::from_dollars(4.0), 5.0), None);
    }

    #[test]
    fn session_cost_exactly_at_threshold() {
        assert_eq!(check_session_cost(UsdMicros::from_dollars(5.0), 5.0), None);
    }

    #[test]
    fn session_cost_zero() {
        assert_eq!(check_session_cost(UsdMicros::ZERO, 5.0), None);
    }

    #[test]
    fn session_cost_just_above_threshold() {
        assert_eq!(
            check_session_cost(UsdMicros::from_dollars(5.01), 5.0),
            Some(AlertRule::SessionCost)
        );
    }

    #[test]
    fn session_cost_accepts_typed_usd_micros() {
        // Compile-time guarantee: the function takes UsdMicros, not raw f64.
        // Wiring a SessionMetrics.cost_usd straight through (without
        // .as_f64() at the call site) is the desired ergonomic.
        let total = UsdMicros::from_dollars(7.5);
        assert!(check_session_cost(total, 5.0).is_some());
    }

    // -- AlertThresholds::validate --

    #[test]
    fn validate_default_thresholds_ok() {
        let t = AlertThresholds::default();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn validate_negative_cost_surge() {
        let t = AlertThresholds {
            cost_surge_per_hour: -1.0,
            ..AlertThresholds::default()
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn validate_nan_session_cost() {
        let t = AlertThresholds {
            session_cost_threshold: f64::NAN,
            ..AlertThresholds::default()
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn validate_infinite_token_velocity() {
        let t = AlertThresholds {
            runaway_token_velocity: f64::INFINITY,
            ..AlertThresholds::default()
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn validate_rejection_rate_above_one() {
        let t = AlertThresholds {
            rejection_rate: 1.5,
            ..AlertThresholds::default()
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn validate_rejection_rate_negative() {
        let t = AlertThresholds {
            rejection_rate: -0.1,
            ..AlertThresholds::default()
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn validate_context_pressure_over_100() {
        let t = AlertThresholds {
            context_pressure_pct: 101,
            ..AlertThresholds::default()
        };
        assert!(t.validate().is_err());
    }
}
