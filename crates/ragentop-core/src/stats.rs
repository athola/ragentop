//! Dashboard aggregate statistics.
//!
//! Pure data structures for rich aggregate statistics displayed on the
//! monitoring dashboard. Inspired by cc-top's dashboard stats.

use crate::types::UsdMicros;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregate statistics for the monitoring dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DashboardStats {
    /// Total lines of code added across all sessions.
    pub lines_added: u64,
    /// Total lines of code removed across all sessions.
    pub lines_removed: u64,
    /// Tool acceptance rates (tool name -> rate 0.0-1.0).
    pub tool_acceptance: HashMap<String, f64>,
    /// Cache efficiency ratio (0.0-1.0).
    pub cache_efficiency: f64,
    /// Average API latency in seconds.
    pub avg_api_latency_secs: f64,
    /// Per-model cost and token breakdown.
    pub model_breakdown: Vec<ModelStats>,
    /// Most-used tools, sorted by count descending.
    pub top_tools: Vec<ToolUsage>,
    /// Overall error rate (0.0-1.0).
    pub error_rate: f64,
    /// API latency percentiles.
    pub latency_percentiles: LatencyPercentiles,
    /// Token usage by category (`input`/`output`/`cache_read`/`cache_creation`).
    pub token_breakdown: HashMap<String, u64>,
    /// Estimated cost savings from cache hits (USD).
    pub cache_savings_usd: f64,
}

impl Default for DashboardStats {
    fn default() -> Self {
        Self {
            lines_added: 0,
            lines_removed: 0,
            tool_acceptance: HashMap::new(),
            cache_efficiency: 0.0,
            avg_api_latency_secs: 0.0,
            model_breakdown: Vec::new(),
            top_tools: Vec::new(),
            error_rate: 0.0,
            latency_percentiles: LatencyPercentiles::default(),
            token_breakdown: HashMap::new(),
            cache_savings_usd: 0.0,
        }
    }
}

/// Per-model cost and token statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelStats {
    /// Model identifier (e.g., "claude-sonnet-4-5-20250929").
    pub model: String,
    /// Total cost attributed to this model.
    pub total_cost: UsdMicros,
    /// Total tokens consumed by this model.
    pub total_tokens: u64,
}

/// Tool usage count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ToolUsage {
    /// Name of the tool.
    pub tool_name: String,
    /// Number of times the tool was invoked.
    pub count: u64,
}

/// API latency percentile values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LatencyPercentiles {
    /// 50th percentile (median) latency in seconds.
    pub p50: f64,
    /// 95th percentile latency in seconds.
    pub p95: f64,
    /// 99th percentile latency in seconds.
    pub p99: f64,
}

impl Default for LatencyPercentiles {
    fn default() -> Self {
        Self {
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Default impls --

    #[test]
    fn dashboard_stats_default() {
        let stats = DashboardStats::default();
        assert_eq!(stats.lines_added, 0);
        assert_eq!(stats.lines_removed, 0);
        assert!(stats.tool_acceptance.is_empty());
        assert!((stats.cache_efficiency).abs() < f64::EPSILON);
        assert!((stats.avg_api_latency_secs).abs() < f64::EPSILON);
        assert!(stats.model_breakdown.is_empty());
        assert!(stats.top_tools.is_empty());
        assert!((stats.error_rate).abs() < f64::EPSILON);
        assert!(stats.token_breakdown.is_empty());
        assert!((stats.cache_savings_usd).abs() < f64::EPSILON);
    }

    #[test]
    fn latency_percentiles_default() {
        let p = LatencyPercentiles::default();
        assert!((p.p50).abs() < f64::EPSILON);
        assert!((p.p95).abs() < f64::EPSILON);
        assert!((p.p99).abs() < f64::EPSILON);
    }

    // -- Basic construction --

    #[test]
    fn model_stats_construction() {
        let ms = ModelStats {
            model: "claude-opus-4-6".to_string(),
            total_cost: UsdMicros::from_dollars(12.50),
            total_tokens: 500_000,
        };
        assert_eq!(ms.model, "claude-opus-4-6");
        assert_eq!(ms.total_cost, UsdMicros::from_dollars(12.50));
        assert_eq!(ms.total_tokens, 500_000);
    }

    #[test]
    fn tool_usage_construction() {
        let tu = ToolUsage {
            tool_name: "Bash".to_string(),
            count: 42,
        };
        assert_eq!(tu.tool_name, "Bash");
        assert_eq!(tu.count, 42);
    }

    #[test]
    fn dashboard_stats_with_data() {
        let mut stats = DashboardStats {
            lines_added: 100,
            lines_removed: 20,
            cache_efficiency: 0.85,
            error_rate: 0.02,
            ..DashboardStats::default()
        };
        stats.model_breakdown.push(ModelStats {
            model: "claude-opus-4-6".to_string(),
            total_cost: UsdMicros::from_dollars(5.0),
            total_tokens: 100_000,
        });
        stats.top_tools.push(ToolUsage {
            tool_name: "Read".to_string(),
            count: 50,
        });
        stats.token_breakdown.insert("input".to_string(), 80_000);
        stats.token_breakdown.insert("output".to_string(), 20_000);

        assert_eq!(stats.lines_added, 100);
        assert_eq!(stats.model_breakdown.len(), 1);
        assert_eq!(stats.top_tools.len(), 1);
        assert_eq!(stats.token_breakdown.len(), 2);
    }

    // -- Serde round-trips --

    #[test]
    fn dashboard_stats_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let stats = DashboardStats::default();
        let json = serde_json::to_string(&stats)?;
        let parsed: DashboardStats = serde_json::from_str(&json)?;
        assert_eq!(parsed.lines_added, 0);
        assert!((parsed.error_rate).abs() < f64::EPSILON);
        Ok(())
    }

    #[test]
    fn model_stats_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let ms = ModelStats {
            model: "test-model".to_string(),
            total_cost: UsdMicros::from_dollars(1.23),
            total_tokens: 456,
        };
        let json = serde_json::to_string(&ms)?;
        let parsed: ModelStats = serde_json::from_str(&json)?;
        assert_eq!(parsed, ms);
        Ok(())
    }

    #[test]
    fn tool_usage_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let tu = ToolUsage {
            tool_name: "Grep".to_string(),
            count: 10,
        };
        let json = serde_json::to_string(&tu)?;
        let parsed: ToolUsage = serde_json::from_str(&json)?;
        assert_eq!(parsed, tu);
        Ok(())
    }

    #[test]
    fn latency_percentiles_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let p = LatencyPercentiles {
            p50: 0.5,
            p95: 1.2,
            p99: 3.0,
        };
        let json = serde_json::to_string(&p)?;
        let parsed: LatencyPercentiles = serde_json::from_str(&json)?;
        assert_eq!(parsed, p);
        Ok(())
    }

    // -- Validation-style tests --

    #[test]
    fn latency_percentiles_ordering_is_sensible() {
        let p = LatencyPercentiles {
            p50: 0.5,
            p95: 1.2,
            p99: 3.0,
        };
        assert!(p.p50 <= p.p95);
        assert!(p.p95 <= p.p99);
    }

    #[test]
    fn error_rate_clamped_range() {
        let stats = DashboardStats {
            error_rate: 0.5,
            ..DashboardStats::default()
        };
        assert!((0.0..=1.0).contains(&stats.error_rate));
    }
}
