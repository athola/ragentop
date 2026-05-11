//! Model pricing data and cost computation.
//!
//! Pure data and functions for per-model token pricing and cost calculation.
//! Defaults cover the current Claude 4.x lineup (Opus 4.7, Sonnet 4.6,
//! Haiku 4.5) using Anthropic's published per-million-token rates. Prices
//! are USD per million tokens and should be re-verified against the
//! Anthropic console before billing or alert thresholds depend on them.

use crate::types::UsdMicros;
use serde::{Deserialize, Serialize};

/// Per-million-token pricing for a model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelPricing {
    /// Cost per million input tokens (USD).
    pub input_per_million: f64,
    /// Cost per million output tokens (USD).
    pub output_per_million: f64,
    /// Cost per million cache-read tokens (USD).
    pub cache_read_per_million: f64,
    /// Cost per million cache-creation tokens (USD).
    pub cache_creation_per_million: f64,
}

/// Model context window limits in tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelLimits {
    /// Maximum context window size in tokens.
    pub context_window: usize,
}

/// Returns default pricing for the current Claude 4.x model lineup.
///
/// Covers Opus 4.7, Sonnet 4.6, and Haiku 4.5 — the three Claude models
/// available as of the 0.1.0 release. Cache rates follow Anthropic's
/// standard 10x cache-read discount and 1.25x cache-creation surcharge
/// versus input pricing. Re-verify against the Anthropic pricing page
/// before relying on these numbers for cost alerts.
#[must_use]
pub fn default_pricing() -> Vec<(&'static str, ModelPricing)> {
    vec![
        (
            "claude-opus-4-7",
            ModelPricing {
                input_per_million: 15.00,
                output_per_million: 75.00,
                cache_read_per_million: 1.50,
                cache_creation_per_million: 18.75,
            },
        ),
        (
            "claude-sonnet-4-6",
            ModelPricing {
                input_per_million: 3.00,
                output_per_million: 15.00,
                cache_read_per_million: 0.30,
                cache_creation_per_million: 3.75,
            },
        ),
        (
            "claude-haiku-4-5-20251001",
            ModelPricing {
                input_per_million: 1.00,
                output_per_million: 5.00,
                cache_read_per_million: 0.10,
                cache_creation_per_million: 1.25,
            },
        ),
    ]
}

/// Compute cost from token counts and pricing.
///
/// All token counts are absolute values; pricing is per-million tokens.
/// Returns [`UsdMicros`] so callers accumulate exact micro-dollar totals
/// rather than drifting f64 sums.
///
/// # Examples
/// ```
/// use ragentop_core::pricing::{compute_cost, default_pricing};
///
/// let pricing_list = default_pricing();
/// let (_, opus) = &pricing_list[0]; // opus 4.7
/// // 1M input + 1M output at Opus rates = $15 + $75 = $90
/// let cost = compute_cost(1_000_000, 1_000_000, 0, 0, opus);
/// assert!((cost.as_f64() - 90.00).abs() < 0.001);
/// ```
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn compute_cost(
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    pricing: &ModelPricing,
) -> UsdMicros {
    let scale = 1_000_000.0_f64;
    let input_cost = (input_tokens as f64 / scale) * pricing.input_per_million;
    let output_cost = (output_tokens as f64 / scale) * pricing.output_per_million;
    let cache_read_cost = (cache_read_tokens as f64 / scale) * pricing.cache_read_per_million;
    let cache_creation_cost =
        (cache_creation_tokens as f64 / scale) * pricing.cache_creation_per_million;
    UsdMicros::from_dollars(input_cost + output_cost + cache_read_cost + cache_creation_cost)
}

/// Returns default context window limits for the current Claude 4.x lineup.
///
/// Opus 4.7 ships with a 1M-token window (GA); Sonnet 4.6 and Haiku 4.5
/// retain the standard 200K window. Models authorize their own ceiling at
/// request time — these defaults are used for warning thresholds and
/// dashboard utilization bars.
#[must_use]
pub fn default_context_limits() -> Vec<(&'static str, usize)> {
    vec![
        ("claude-opus-4-7", 1_000_000),
        ("claude-sonnet-4-6", 200_000),
        ("claude-haiku-4-5-20251001", 200_000),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- default_pricing --

    #[test]
    fn default_pricing_has_three_models() {
        let pricing = default_pricing();
        assert_eq!(pricing.len(), 3);
    }

    #[test]
    fn default_pricing_contains_known_models() {
        let pricing = default_pricing();
        let names: Vec<&str> = pricing.iter().map(|(name, _)| *name).collect();
        assert!(names.contains(&"claude-opus-4-7"));
        assert!(names.contains(&"claude-sonnet-4-6"));
        assert!(names.contains(&"claude-haiku-4-5-20251001"));
    }

    // -- compute_cost --

    #[test]
    fn compute_cost_zero_tokens() {
        let pricing = ModelPricing {
            input_per_million: 3.00,
            output_per_million: 15.00,
            cache_read_per_million: 0.30,
            cache_creation_per_million: 3.75,
        };
        let cost = compute_cost(0, 0, 0, 0, &pricing);
        assert_eq!(cost, UsdMicros::ZERO);
    }

    #[test]
    fn compute_cost_known_values() {
        let pricing = ModelPricing {
            input_per_million: 3.00,
            output_per_million: 15.00,
            cache_read_per_million: 0.30,
            cache_creation_per_million: 3.75,
        };
        // 1M input + 1M output = $3 + $15 = $18
        let cost = compute_cost(1_000_000, 1_000_000, 0, 0, &pricing);
        assert!((cost.as_f64() - 18.00).abs() < 0.001);
    }

    #[test]
    fn compute_cost_cache_pricing() {
        let pricing = ModelPricing {
            input_per_million: 3.00,
            output_per_million: 15.00,
            cache_read_per_million: 0.30,
            cache_creation_per_million: 3.75,
        };
        // 1M cache_read + 1M cache_creation = $0.30 + $3.75 = $4.05
        let cost = compute_cost(0, 0, 1_000_000, 1_000_000, &pricing);
        assert!((cost.as_f64() - 4.05).abs() < 0.001);
    }

    #[test]
    fn compute_cost_partial_tokens() {
        let pricing = ModelPricing {
            input_per_million: 10.00,
            output_per_million: 10.00,
            cache_read_per_million: 0.0,
            cache_creation_per_million: 0.0,
        };
        // 500_000 input = $5, 500_000 output = $5 -> $10 total
        let cost = compute_cost(500_000, 500_000, 0, 0, &pricing);
        assert!((cost.as_f64() - 10.00).abs() < 0.001);
    }

    #[test]
    fn compute_cost_all_token_types() {
        let pricing = ModelPricing {
            input_per_million: 5.00,
            output_per_million: 25.00,
            cache_read_per_million: 0.50,
            cache_creation_per_million: 6.25,
        };
        // 100k each: input=$0.50, output=$2.50, cache_read=$0.05, cache_creation=$0.625
        let cost = compute_cost(100_000, 100_000, 100_000, 100_000, &pricing);
        let expected = 0.50 + 2.50 + 0.05 + 0.625;
        assert!((cost.as_f64() - expected).abs() < 0.001);
    }

    #[test]
    fn compute_cost_accumulates_exactly_across_many_calls() {
        let pricing = ModelPricing {
            input_per_million: 3.00,
            output_per_million: 15.00,
            cache_read_per_million: 0.30,
            cache_creation_per_million: 3.75,
        };
        // 1,000 calls each costing $0.018 (6k input @ $3/M). With f64,
        // 1000 * 0.018 drifts (0.018 has no exact binary representation);
        // with UsdMicros it accumulates to exactly $18.000000.
        let one_call = compute_cost(6_000, 0, 0, 0, &pricing);
        assert_eq!(one_call.as_micros(), 18_000);
        let mut total = UsdMicros::ZERO;
        for _ in 0..1_000 {
            total = total.saturating_add(one_call);
        }
        assert_eq!(total.as_micros(), 18_000_000);
    }

    // -- default_context_limits --

    #[test]
    fn default_context_limits_has_three_models() {
        let limits = default_context_limits();
        assert_eq!(limits.len(), 3);
    }

    #[test]
    fn default_context_limits_match_lineup() {
        let limits = default_context_limits();
        let map: std::collections::HashMap<&str, usize> = limits.into_iter().collect();
        // Opus 4.7 has the 1M context window; Sonnet/Haiku stay at 200K.
        assert_eq!(map.get("claude-opus-4-7"), Some(&1_000_000));
        assert_eq!(map.get("claude-sonnet-4-6"), Some(&200_000));
        assert_eq!(map.get("claude-haiku-4-5-20251001"), Some(&200_000));
    }

    // -- Serde round-trips --

    #[test]
    fn model_pricing_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let pricing = ModelPricing {
            input_per_million: 3.00,
            output_per_million: 15.00,
            cache_read_per_million: 0.30,
            cache_creation_per_million: 3.75,
        };
        let json = serde_json::to_string(&pricing)?;
        let parsed: ModelPricing = serde_json::from_str(&json)?;
        assert_eq!(parsed, pricing);
        Ok(())
    }

    #[test]
    fn model_limits_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let limits = ModelLimits {
            context_window: 200_000,
        };
        let json = serde_json::to_string(&limits)?;
        let parsed: ModelLimits = serde_json::from_str(&json)?;
        assert_eq!(parsed, limits);
        Ok(())
    }
}
