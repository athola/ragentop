//! Model pricing data and cost computation.
//!
//! Pure data and functions for per-model token pricing and cost calculation.
//! Based on cc-top's pricing defaults (2025 pricing).

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

/// Returns default pricing for known Claude models.
///
/// Based on cc-top defaults (2025 pricing).
#[must_use]
pub fn default_pricing() -> Vec<(&'static str, ModelPricing)> {
    vec![
        (
            "claude-sonnet-4-5-20250929",
            ModelPricing {
                input_per_million: 3.00,
                output_per_million: 15.00,
                cache_read_per_million: 0.30,
                cache_creation_per_million: 3.75,
            },
        ),
        (
            "claude-opus-4-6",
            ModelPricing {
                input_per_million: 5.00,
                output_per_million: 25.00,
                cache_read_per_million: 0.50,
                cache_creation_per_million: 6.25,
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
///
/// # Examples
/// ```
/// use ragentop_core::pricing::{compute_cost, default_pricing};
///
/// let pricing_list = default_pricing();
/// let (_, pricing) = &pricing_list[0]; // sonnet
/// let cost = compute_cost(1_000_000, 1_000_000, 0, 0, pricing);
/// assert!((cost - 18.00).abs() < 0.001);
/// ```
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn compute_cost(
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    pricing: &ModelPricing,
) -> f64 {
    let scale = 1_000_000.0_f64;
    let input_cost = (input_tokens as f64 / scale) * pricing.input_per_million;
    let output_cost = (output_tokens as f64 / scale) * pricing.output_per_million;
    let cache_read_cost = (cache_read_tokens as f64 / scale) * pricing.cache_read_per_million;
    let cache_creation_cost =
        (cache_creation_tokens as f64 / scale) * pricing.cache_creation_per_million;
    input_cost + output_cost + cache_read_cost + cache_creation_cost
}

/// Returns default context window limits for known Claude models.
#[must_use]
pub fn default_context_limits() -> Vec<(&'static str, usize)> {
    vec![
        ("claude-sonnet-4-5-20250929", 200_000),
        ("claude-opus-4-6", 200_000),
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
        assert!(names.contains(&"claude-sonnet-4-5-20250929"));
        assert!(names.contains(&"claude-opus-4-6"));
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
        assert!((cost).abs() < f64::EPSILON);
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
        assert!((cost - 18.00).abs() < 0.001);
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
        assert!((cost - 4.05).abs() < 0.001);
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
        assert!((cost - 10.00).abs() < 0.001);
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
        assert!((cost - expected).abs() < 0.001);
    }

    // -- default_context_limits --

    #[test]
    fn default_context_limits_has_three_models() {
        let limits = default_context_limits();
        assert_eq!(limits.len(), 3);
    }

    #[test]
    fn default_context_limits_all_200k() {
        let limits = default_context_limits();
        for (_, window) in &limits {
            assert_eq!(*window, 200_000);
        }
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
