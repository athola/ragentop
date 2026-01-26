//! Parsers for Claude Code data files.

use ragentop_core::Result;
use serde::Deserialize;

/// Parsed stats cache data.
#[derive(Debug, Clone)]
pub struct StatsCache {
    pub total_tokens: u64,
    pub total_cost: Option<f64>,
    pub model: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawStatsCache {
    total_tokens: Option<u64>,
    total_cost: Option<f64>,
    model: Option<String>,
    session_id: Option<String>,
}

/// Parses a stats-cache.json file.
///
/// # Errors
/// Returns an error if JSON parsing fails.
pub fn parse_stats_cache(contents: &str) -> Result<StatsCache> {
    let raw: RawStatsCache = serde_json::from_str(contents)?;
    Ok(StatsCache {
        total_tokens: raw.total_tokens.unwrap_or(0),
        total_cost: raw.total_cost,
        model: raw.model,
        session_id: raw.session_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stats_cache() {
        let fixture = include_str!("../tests/fixtures/stats_cache.json");
        let stats = parse_stats_cache(fixture).unwrap();
        assert_eq!(stats.total_tokens, 45231);
        assert_eq!(stats.total_cost, Some(1.82));
        assert_eq!(stats.model, Some("claude-sonnet-4-20250514".to_string()));
    }

    #[test]
    fn test_parse_stats_cache_minimal() {
        let json = r#"{"totalTokens": 100}"#;
        let stats = parse_stats_cache(json).unwrap();
        assert_eq!(stats.total_tokens, 100);
        assert!(stats.total_cost.is_none());
    }
}
