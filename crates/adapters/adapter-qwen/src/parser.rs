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
#[non_exhaustive]
struct QwenLogFile {
    #[serde(default)]
    request: Option<Request>,
    #[serde(default)]
    timestamp: Option<f64>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize)]
#[non_exhaustive]
#[expect(
    dead_code,
    clippy::struct_field_names,
    reason = "Fields needed for serde deserialization"
)]
struct Usage {
    #[serde(default)]
    completion_tokens: u64,
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct Request {
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct ToolCall {
    #[serde(rename = "type", default)]
    call_type: Option<String>,
    #[serde(default)]
    function: Option<FunctionCall>,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct FunctionCall {
    #[serde(default)]
    arguments: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

/// Aggregates metrics from all Qwen log files in the logs directory.
///
/// # Errors
/// Returns an error if the filesystem cannot be read.
#[inline]
pub fn aggregate_metrics(config_dir: &Path) -> Result<SessionMetrics> {
    let logs_dir = config_dir.join("logs").join("openai");
    if !logs_dir.exists() {
        return Ok(SessionMetrics::default());
    }

    let mut total_tokens: u64 = 0;
    let mut command_count: u64 = 0;

    for dir_entry in std::fs::read_dir(&logs_dir)? {
        let dir_entry = dir_entry?;
        let file_path = dir_entry.path();
        let Some(ext) = file_path.extension() else {
            continue;
        };
        if ext != "json" {
            continue;
        }
        let contents = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    path = %file_path.display(),
                    error = %e,
                    "skipping unreadable qwen log file during metric aggregation"
                );
                continue;
            }
        };
        let log_file = match serde_json::from_str::<QwenLogFile>(&contents) {
            Ok(lf) => lf,
            Err(e) => {
                tracing::warn!(
                    path = %file_path.display(),
                    error = %e,
                    "skipping malformed qwen log file during metric aggregation"
                );
                continue;
            }
        };
        if let Some(usage) = &log_file.usage {
            total_tokens += usage.total_tokens;
        }
        if let Some(req) = &log_file.request {
            if let Some(tools) = &req.tool_calls {
                command_count += tools.len() as u64;
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
#[inline]
pub fn parse_history(config_dir: &Path, limit: usize) -> Result<Vec<Command>> {
    let logs_dir = config_dir.join("logs").join("openai");
    if !logs_dir.exists() {
        return Ok(vec![]);
    }

    let mut commands = Vec::new();

    // Collect log files sorted by mtime (newest first)
    let mut entries: Vec<_> = std::fs::read_dir(&logs_dir)?
        .flatten()
        .filter(|ent| ent.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by(|left, right| {
        let mtime_left = left
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let mtime_right = right
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        mtime_right.cmp(&mtime_left)
    });

    for dir_entry in entries {
        if commands.len() >= limit {
            break;
        }
        let file_path = dir_entry.path();
        let contents = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    path = %file_path.display(),
                    error = %e,
                    "skipping unreadable qwen log file during history parse"
                );
                continue;
            }
        };
        let log_file = match serde_json::from_str::<QwenLogFile>(&contents) {
            Ok(lf) => lf,
            Err(e) => {
                tracing::warn!(
                    path = %file_path.display(),
                    error = %e,
                    "skipping malformed qwen log file during history parse"
                );
                continue;
            }
        };
        let timestamp = log_file.timestamp.map_or_else(
            || {
                file_path
                    .metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or_else(|_| SystemTime::now())
            },
            |ts_val| SystemTime::UNIX_EPOCH + Duration::from_secs_f64(ts_val),
        );

        let Some(req) = &log_file.request else {
            continue;
        };
        let Some(tool_calls) = &req.tool_calls else {
            continue;
        };
        for tool_call in tool_calls {
            if commands.len() >= limit {
                break;
            }
            let tool = tool_call
                .function
                .as_ref()
                .and_then(|func| func.name.clone())
                .unwrap_or_else(|| {
                    tool_call
                        .call_type
                        .clone()
                        .unwrap_or_else(|| "unknown".to_owned())
                });
            let args = tool_call
                .function
                .as_ref()
                .and_then(|func| func.arguments.clone())
                .unwrap_or_default();
            commands.push(Command {
                args,
                result_summary: None,
                status: CommandStatus::Success,
                timestamp,
                tool,
            });
        }
    }
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_log_file(
        dir_path: &Path,
        name: &str,
        content: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        fs::write(dir_path.join(name), content)?;
        Ok(())
    }

    #[test]
    fn test_aggregate_metrics_sums_tokens() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let qwen_dir = temp_dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir)?;

        make_log_file(
            &logs_dir,
            "log1.json",
            r#"{"usage": {"prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150}}"#,
        )?;
        make_log_file(
            &logs_dir,
            "log2.json",
            r#"{"usage": {"prompt_tokens": 200, "completion_tokens": 100, "total_tokens": 300}}"#,
        )?;

        let metrics = aggregate_metrics(&qwen_dir)?;
        assert_eq!(metrics.token_count, 450);
        Ok(())
    }

    #[test]
    fn test_aggregate_metrics_counts_tool_calls(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let qwen_dir = temp_dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir)?;

        make_log_file(
            &logs_dir,
            "log1.json",
            r#"{"request": {"tool_calls": [{"type": "function", "function": {"name": "bash", "arguments": "ls"}}, {"type": "function", "function": {"name": "read", "arguments": "file.txt"}}]}}"#,
        )?;

        let metrics = aggregate_metrics(&qwen_dir)?;
        assert_eq!(metrics.command_count, 2);
        Ok(())
    }

    #[test]
    fn test_aggregate_metrics_empty_dir() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let metrics = aggregate_metrics(temp_dir.path())?;
        assert_eq!(metrics.token_count, 0);
        Ok(())
    }

    #[test]
    fn test_parse_history_extracts_tool_calls(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let qwen_dir = temp_dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir)?;

        make_log_file(
            &logs_dir,
            "log1.json",
            r#"{"timestamp": 1706000000.0, "request": {"tool_calls": [{"type": "function", "function": {"name": "bash", "arguments": "ls -la"}}]}}"#,
        )?;

        let cmds = parse_history(&qwen_dir, 10)?;
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].tool, "bash");
        assert_eq!(cmds[0].args, "ls -la");
        Ok(())
    }

    #[test]
    fn test_parse_history_respects_limit() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let qwen_dir = temp_dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir)?;

        make_log_file(
            &logs_dir,
            "log1.json",
            r#"{"request": {"tool_calls": [{"function": {"name": "a"}}, {"function": {"name": "b"}}, {"function": {"name": "c"}}]}}"#,
        )?;

        let cmds = parse_history(&qwen_dir, 2)?;
        assert_eq!(cmds.len(), 2);
        Ok(())
    }

    #[test]
    fn test_parse_history_missing_dir() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let cmds = parse_history(temp_dir.path(), 10)?;
        assert!(cmds.is_empty());
        Ok(())
    }

    #[test]
    fn test_aggregate_metrics_skips_malformed(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let qwen_dir = temp_dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir)?;
        make_log_file(&logs_dir, "bad.json", "not json {{{")?;
        make_log_file(
            &logs_dir,
            "good.json",
            r#"{"usage": {"total_tokens": 100}}"#,
        )?;

        let metrics = aggregate_metrics(&qwen_dir)?;
        assert_eq!(metrics.token_count, 100);
        Ok(())
    }
}
