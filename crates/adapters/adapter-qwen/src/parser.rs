//! Parsers for Qwen CLI JSON log files.
//!
//! Qwen logs API interactions in ~/.qwen/logs/openai/*.json.
//! Each file contains an OpenAI-compatible API response with
//! usage data (token counts) and `tool_calls` in the request.

use ragentop_core::{Command, CommandStatus, Result, SessionMetrics};
use serde::Deserialize;
use std::path::Path;
use std::time::{Duration, SystemTime};

#[derive(Deserialize)]
struct QwenLogFile {
    #[serde(default)]
    usage: Option<Usage>,
    #[serde(default)]
    request: Option<Request>,
    #[serde(default)]
    timestamp: Option<f64>,
}

#[derive(Deserialize)]
#[allow(dead_code, clippy::struct_field_names)]
struct Usage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
}

#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize)]
struct ToolCall {
    #[serde(rename = "type", default)]
    call_type: Option<String>,
    #[serde(default)]
    function: Option<FunctionCall>,
}

#[derive(Deserialize)]
struct FunctionCall {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

/// Aggregates metrics from all Qwen log files in the logs directory.
///
/// # Errors
/// Returns an error if the filesystem cannot be read.
pub fn aggregate_metrics(config_dir: &Path) -> Result<SessionMetrics> {
    let logs_dir = config_dir.join("logs").join("openai");
    if !logs_dir.exists() {
        return Ok(SessionMetrics::default());
    }

    let mut total_tokens: u64 = 0;
    let mut command_count: u64 = 0;

    for entry in std::fs::read_dir(&logs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(log) = serde_json::from_str::<QwenLogFile>(&contents) {
                    if let Some(usage) = &log.usage {
                        total_tokens += usage.total_tokens;
                    }
                    if let Some(req) = &log.request {
                        if let Some(tools) = &req.tool_calls {
                            command_count += tools.len() as u64;
                        }
                    }
                }
            }
        }
    }

    let (metrics, _) = SessionMetrics::new(total_tokens, None, None, None, command_count);
    Ok(metrics)
}

/// Parses command history (tool calls) from Qwen log files.
///
/// # Errors
/// Returns an error if the filesystem cannot be read.
pub fn parse_history(config_dir: &Path, limit: usize) -> Result<Vec<Command>> {
    let logs_dir = config_dir.join("logs").join("openai");
    if !logs_dir.exists() {
        return Ok(vec![]);
    }

    let mut commands = Vec::new();

    // Collect log files sorted by mtime (newest first)
    let mut entries: Vec<_> = std::fs::read_dir(&logs_dir)?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by(|a, b| {
        let ma = a
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let mb = b
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        mb.cmp(&ma)
    });

    for entry in entries {
        if commands.len() >= limit {
            break;
        }
        let path = entry.path();
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(log) = serde_json::from_str::<QwenLogFile>(&contents) {
                let timestamp = log.timestamp.map_or_else(
                    || {
                        path.metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or_else(|_| SystemTime::now())
                    },
                    |ts| SystemTime::UNIX_EPOCH + Duration::from_secs_f64(ts),
                );

                if let Some(req) = &log.request {
                    if let Some(tool_calls) = &req.tool_calls {
                        for tc in tool_calls {
                            if commands.len() >= limit {
                                break;
                            }
                            let tool = tc
                                .function
                                .as_ref()
                                .and_then(|f| f.name.clone())
                                .unwrap_or_else(|| {
                                    tc.call_type
                                        .clone()
                                        .unwrap_or_else(|| "unknown".to_string())
                                });
                            let args = tc
                                .function
                                .as_ref()
                                .and_then(|f| f.arguments.clone())
                                .unwrap_or_default();
                            commands.push(Command {
                                timestamp,
                                tool,
                                args,
                                status: CommandStatus::Success,
                                result_summary: None,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_log_file(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn test_aggregate_metrics_sums_tokens() {
        let dir = tempdir().unwrap();
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir).unwrap();

        make_log_file(
            &logs_dir,
            "log1.json",
            r#"{
            "usage": {"prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150}
        }"#,
        );
        make_log_file(
            &logs_dir,
            "log2.json",
            r#"{
            "usage": {"prompt_tokens": 200, "completion_tokens": 100, "total_tokens": 300}
        }"#,
        );

        let metrics = aggregate_metrics(&qwen_dir).unwrap();
        assert_eq!(metrics.token_count, 450);
    }

    #[test]
    fn test_aggregate_metrics_counts_tool_calls() {
        let dir = tempdir().unwrap();
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir).unwrap();

        make_log_file(
            &logs_dir,
            "log1.json",
            r#"{
            "request": {
                "tool_calls": [
                    {"type": "function", "function": {"name": "bash", "arguments": "ls"}},
                    {"type": "function", "function": {"name": "read", "arguments": "file.txt"}}
                ]
            }
        }"#,
        );

        let metrics = aggregate_metrics(&qwen_dir).unwrap();
        assert_eq!(metrics.command_count, 2);
    }

    #[test]
    fn test_aggregate_metrics_empty_dir() {
        let dir = tempdir().unwrap();
        let metrics = aggregate_metrics(dir.path()).unwrap();
        assert_eq!(metrics.token_count, 0);
    }

    #[test]
    fn test_parse_history_extracts_tool_calls() {
        let dir = tempdir().unwrap();
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir).unwrap();

        make_log_file(
            &logs_dir,
            "log1.json",
            r#"{
            "timestamp": 1706000000.0,
            "request": {
                "tool_calls": [
                    {"type": "function", "function": {"name": "bash", "arguments": "ls -la"}}
                ]
            }
        }"#,
        );

        let cmds = parse_history(&qwen_dir, 10).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].tool, "bash");
        assert_eq!(cmds[0].args, "ls -la");
    }

    #[test]
    fn test_parse_history_respects_limit() {
        let dir = tempdir().unwrap();
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir).unwrap();

        make_log_file(
            &logs_dir,
            "log1.json",
            r#"{
            "request": {
                "tool_calls": [
                    {"function": {"name": "a"}},
                    {"function": {"name": "b"}},
                    {"function": {"name": "c"}}
                ]
            }
        }"#,
        );

        let cmds = parse_history(&qwen_dir, 2).unwrap();
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_parse_history_missing_dir() {
        let dir = tempdir().unwrap();
        let cmds = parse_history(dir.path(), 10).unwrap();
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_aggregate_metrics_skips_malformed() {
        let dir = tempdir().unwrap();
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir).unwrap();
        make_log_file(&logs_dir, "bad.json", "not json {{{");
        make_log_file(
            &logs_dir,
            "good.json",
            r#"{"usage": {"total_tokens": 100}}"#,
        );

        let metrics = aggregate_metrics(&qwen_dir).unwrap();
        assert_eq!(metrics.token_count, 100);
    }
}
