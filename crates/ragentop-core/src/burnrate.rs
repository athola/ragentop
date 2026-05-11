//! Cost burn-rate analysis for agent sessions.
//!
//! Pure functions for computing cost trends, rate classification, and
//! token velocity. Inspired by cc-top's burn-rate monitoring.

use crate::types::UsdMicros;
use serde::{Deserialize, Serialize};

/// Snapshot of cost burn-rate for an agent session.
///
/// Cumulative costs use [`UsdMicros`] to avoid f64 drift across thousands
/// of polls; rates (e.g. `hourly_rate`) stay as f64 because they are
/// derived per-poll from a fresh sum and don't accumulate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BurnRate {
    /// Total accumulated cost.
    pub total_cost: UsdMicros,
    /// Current hourly rate in USD/hour (derived, may drift but is recomputed each poll).
    pub hourly_rate: f64,
    /// Direction the rate is moving.
    pub trend: TrendDirection,
    /// Tokens consumed per second.
    pub token_velocity: f64,
    /// Per-model burn rate breakdown.
    pub per_model: Vec<ModelBurnRate>,
    /// Projected daily cost based on current hourly rate.
    pub daily_projection: f64,
    /// Projected monthly cost based on current hourly rate.
    pub monthly_projection: f64,
}

/// Per-model burn rate breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelBurnRate {
    /// Model identifier.
    pub model: String,
    /// Hourly rate for this model (USD/hour).
    pub hourly_rate: f64,
    /// Total cost attributed to this model.
    pub total_cost: UsdMicros,
}

/// Direction a metric is trending over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TrendDirection {
    /// Rate is increasing.
    Up,
    /// Rate is decreasing.
    Down,
    /// Rate is stable (within threshold).
    Flat,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Up => write!(f, "up"),
            Self::Down => write!(f, "down"),
            Self::Flat => write!(f, "flat"),
        }
    }
}

/// Color classification for burn-rate display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RateColor {
    /// Acceptable cost rate.
    Green,
    /// Elevated cost rate - worth watching.
    Yellow,
    /// High cost rate - needs attention.
    Red,
}

impl std::fmt::Display for RateColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Green => write!(f, "green"),
            Self::Yellow => write!(f, "yellow"),
            Self::Red => write!(f, "red"),
        }
    }
}

/// Configurable thresholds for burn-rate color classification.
///
/// Rates below `green_below` are green, below `yellow_below` are yellow,
/// and at or above `yellow_below` are red.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Thresholds {
    /// Hourly rate below this value is classified as green (USD/hour).
    pub green_below: f64,
    /// Hourly rate below this value is classified as yellow (USD/hour).
    /// Rates at or above this are red.
    pub yellow_below: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            green_below: 0.50,
            yellow_below: 2.00,
        }
    }
}

/// Classify an hourly cost rate into a display color.
///
/// Uses the given thresholds to determine the color bucket.
///
/// # Examples
/// ```
/// use ragentop_core::burnrate::{compute_color, RateColor, Thresholds};
///
/// let thresholds = Thresholds::default();
/// assert_eq!(compute_color(0.25, &thresholds), RateColor::Green);
/// assert_eq!(compute_color(1.00, &thresholds), RateColor::Yellow);
/// assert_eq!(compute_color(5.00, &thresholds), RateColor::Red);
/// ```
#[must_use]
pub fn compute_color(hourly_rate: f64, thresholds: &Thresholds) -> RateColor {
    if hourly_rate < thresholds.green_below {
        RateColor::Green
    } else if hourly_rate < thresholds.yellow_below {
        RateColor::Yellow
    } else {
        RateColor::Red
    }
}

/// Determine the trend direction by comparing two time windows.
///
/// A change of less than 5% is considered flat to avoid noise.
///
/// # Examples
/// ```
/// use ragentop_core::burnrate::{compute_trend, TrendDirection};
///
/// assert_eq!(compute_trend(1.5, 1.0), TrendDirection::Up);
/// assert_eq!(compute_trend(0.5, 1.0), TrendDirection::Down);
/// assert_eq!(compute_trend(1.01, 1.0), TrendDirection::Flat);
/// ```
#[must_use]
pub fn compute_trend(current_window: f64, previous_window: f64) -> TrendDirection {
    // Use a threshold slightly above 5% to handle floating-point imprecision
    // (e.g. (1.05 - 1.0) / 1.0 may produce 0.050000000000000044).
    const TREND_THRESHOLD: f64 = 0.05 + 1e-9;

    // Avoid division by zero: if previous is zero, any positive current is "up"
    if previous_window == 0.0 {
        if current_window > 0.0 {
            return TrendDirection::Up;
        }
        return TrendDirection::Flat;
    }

    let ratio = (current_window - previous_window) / previous_window;

    if ratio > TREND_THRESHOLD {
        TrendDirection::Up
    } else if ratio < -TREND_THRESHOLD {
        TrendDirection::Down
    } else {
        TrendDirection::Flat
    }
}

/// Compute projected daily cost from an hourly rate.
///
/// Simply multiplies the hourly rate by 24 hours.
///
/// # Examples
/// ```
/// use ragentop_core::burnrate::compute_daily_projection;
///
/// assert!((compute_daily_projection(1.0) - 24.0).abs() < f64::EPSILON);
/// ```
#[must_use]
pub fn compute_daily_projection(hourly_rate: f64) -> f64 {
    hourly_rate * 24.0
}

/// Compute projected monthly cost from an hourly rate.
///
/// Multiplies the hourly rate by 720 hours (30 days).
///
/// # Examples
/// ```
/// use ragentop_core::burnrate::compute_monthly_projection;
///
/// assert!((compute_monthly_projection(1.0) - 720.0).abs() < f64::EPSILON);
/// ```
#[must_use]
pub fn compute_monthly_projection(hourly_rate: f64) -> f64 {
    hourly_rate * 720.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Thresholds defaults --

    #[test]
    fn thresholds_default_values() {
        let t = Thresholds::default();
        assert!((t.green_below - 0.50).abs() < f64::EPSILON);
        assert!((t.yellow_below - 2.00).abs() < f64::EPSILON);
    }

    // -- compute_color --

    #[test]
    fn color_green_below_threshold() {
        let t = Thresholds::default();
        assert_eq!(compute_color(0.0, &t), RateColor::Green);
        assert_eq!(compute_color(0.25, &t), RateColor::Green);
        assert_eq!(compute_color(0.49, &t), RateColor::Green);
    }

    #[test]
    fn color_yellow_between_thresholds() {
        let t = Thresholds::default();
        assert_eq!(compute_color(0.50, &t), RateColor::Yellow);
        assert_eq!(compute_color(1.00, &t), RateColor::Yellow);
        assert_eq!(compute_color(1.99, &t), RateColor::Yellow);
    }

    #[test]
    fn color_red_at_or_above_threshold() {
        let t = Thresholds::default();
        assert_eq!(compute_color(2.00, &t), RateColor::Red);
        assert_eq!(compute_color(10.00, &t), RateColor::Red);
        assert_eq!(compute_color(100.00, &t), RateColor::Red);
    }

    #[test]
    fn color_custom_thresholds() {
        let t = Thresholds {
            green_below: 1.0,
            yellow_below: 5.0,
        };
        assert_eq!(compute_color(0.5, &t), RateColor::Green);
        assert_eq!(compute_color(3.0, &t), RateColor::Yellow);
        assert_eq!(compute_color(5.0, &t), RateColor::Red);
    }

    #[test]
    fn color_negative_rate_is_green() {
        let t = Thresholds::default();
        assert_eq!(compute_color(-1.0, &t), RateColor::Green);
    }

    // -- compute_trend --

    #[test]
    fn trend_up_when_current_significantly_higher() {
        assert_eq!(compute_trend(1.5, 1.0), TrendDirection::Up);
        assert_eq!(compute_trend(2.0, 1.0), TrendDirection::Up);
    }

    #[test]
    fn trend_down_when_current_significantly_lower() {
        assert_eq!(compute_trend(0.5, 1.0), TrendDirection::Down);
        assert_eq!(compute_trend(0.1, 1.0), TrendDirection::Down);
    }

    #[test]
    fn trend_flat_when_within_5_percent() {
        assert_eq!(compute_trend(1.04, 1.0), TrendDirection::Flat);
        assert_eq!(compute_trend(0.96, 1.0), TrendDirection::Flat);
        assert_eq!(compute_trend(1.0, 1.0), TrendDirection::Flat);
    }

    #[test]
    fn trend_boundary_exactly_5_percent() {
        // Exactly 5% change should be flat (not strictly > 0.05)
        assert_eq!(compute_trend(1.05, 1.0), TrendDirection::Flat);
        assert_eq!(compute_trend(0.95, 1.0), TrendDirection::Flat);
    }

    #[test]
    fn trend_just_over_5_percent() {
        assert_eq!(compute_trend(1.06, 1.0), TrendDirection::Up);
        assert_eq!(compute_trend(0.94, 1.0), TrendDirection::Down);
    }

    #[test]
    fn trend_previous_zero_current_positive() {
        assert_eq!(compute_trend(1.0, 0.0), TrendDirection::Up);
    }

    #[test]
    fn trend_both_zero_is_flat() {
        assert_eq!(compute_trend(0.0, 0.0), TrendDirection::Flat);
    }

    #[test]
    fn trend_previous_zero_current_zero_is_flat() {
        assert_eq!(compute_trend(0.0, 0.0), TrendDirection::Flat);
    }

    // -- Display impls --

    #[test]
    fn trend_direction_display() {
        assert_eq!(TrendDirection::Up.to_string(), "up");
        assert_eq!(TrendDirection::Down.to_string(), "down");
        assert_eq!(TrendDirection::Flat.to_string(), "flat");
    }

    #[test]
    fn rate_color_display() {
        assert_eq!(RateColor::Green.to_string(), "green");
        assert_eq!(RateColor::Yellow.to_string(), "yellow");
        assert_eq!(RateColor::Red.to_string(), "red");
    }

    // -- Serde round-trips --

    #[test]
    fn burn_rate_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let rate = BurnRate {
            total_cost: UsdMicros::from_dollars(1.23),
            hourly_rate: 0.45,
            trend: TrendDirection::Up,
            token_velocity: 150.0,
            per_model: vec![ModelBurnRate {
                model: "claude-opus-4-6".to_string(),
                hourly_rate: 0.45,
                total_cost: UsdMicros::from_dollars(1.23),
            }],
            daily_projection: compute_daily_projection(0.45),
            monthly_projection: compute_monthly_projection(0.45),
        };
        let json = serde_json::to_string(&rate)?;
        let parsed: BurnRate = serde_json::from_str(&json)?;
        assert_eq!(parsed, rate);
        Ok(())
    }

    #[test]
    fn thresholds_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let t = Thresholds::default();
        let json = serde_json::to_string(&t)?;
        let parsed: Thresholds = serde_json::from_str(&json)?;
        assert_eq!(parsed, t);
        Ok(())
    }

    #[test]
    fn trend_direction_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [
            TrendDirection::Up,
            TrendDirection::Down,
            TrendDirection::Flat,
        ] {
            let json = serde_json::to_string(&variant)?;
            let parsed: TrendDirection = serde_json::from_str(&json)?;
            assert_eq!(parsed, variant);
        }
        Ok(())
    }

    #[test]
    fn rate_color_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [RateColor::Green, RateColor::Yellow, RateColor::Red] {
            let json = serde_json::to_string(&variant)?;
            let parsed: RateColor = serde_json::from_str(&json)?;
            assert_eq!(parsed, variant);
        }
        Ok(())
    }

    // -- Projection functions --

    #[test]
    fn daily_projection_from_hourly() {
        assert!((compute_daily_projection(1.0) - 24.0).abs() < f64::EPSILON);
        assert!((compute_daily_projection(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((compute_daily_projection(2.5) - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn monthly_projection_from_hourly() {
        assert!((compute_monthly_projection(1.0) - 720.0).abs() < f64::EPSILON);
        assert!((compute_monthly_projection(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((compute_monthly_projection(2.5) - 1800.0).abs() < f64::EPSILON);
    }

    // -- ModelBurnRate --

    #[test]
    fn model_burn_rate_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mbr = ModelBurnRate {
            model: "claude-opus-4-6".to_string(),
            hourly_rate: 1.5,
            total_cost: UsdMicros::from_dollars(10.0),
        };
        let json = serde_json::to_string(&mbr)?;
        let parsed: ModelBurnRate = serde_json::from_str(&json)?;
        assert_eq!(parsed, mbr);
        Ok(())
    }

    #[test]
    fn burn_rate_with_multiple_models() {
        let rate = BurnRate {
            total_cost: UsdMicros::from_dollars(5.0),
            hourly_rate: 1.0,
            trend: TrendDirection::Flat,
            token_velocity: 100.0,
            per_model: vec![
                ModelBurnRate {
                    model: "claude-opus-4-6".to_string(),
                    hourly_rate: 0.7,
                    total_cost: UsdMicros::from_dollars(3.5),
                },
                ModelBurnRate {
                    model: "claude-haiku-4-5-20251001".to_string(),
                    hourly_rate: 0.3,
                    total_cost: UsdMicros::from_dollars(1.5),
                },
            ],
            daily_projection: compute_daily_projection(1.0),
            monthly_projection: compute_monthly_projection(1.0),
        };
        assert_eq!(rate.per_model.len(), 2);
        assert!((rate.daily_projection - 24.0).abs() < f64::EPSILON);
        assert!((rate.monthly_projection - 720.0).abs() < f64::EPSILON);
    }
}
