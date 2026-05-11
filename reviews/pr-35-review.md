# PR #35 Round 3 Review — Post-Fix Regression Scan

**Branch**: `mvp-0.1.0` → `master` (38 files, +3,568 / -450)
**Scope mode**: flexible (RED ZONE branch, 3 prior review rounds, mergeable=CLEAN)
**Focus**: regressions in the most recent fix commits (`a714541`, `2ff1315`, `454db6e`). Did NOT re-review what prior rounds already covered.

## Verdict

**No blocking issues. Ready to merge.** CI is green across 5 checks (Check, Clippy, Coverage, Format, Test). All 7 prior review threads resolved. The C7 UsdMicros migration is properly anchored by tests. Adapter detector unification is regression-anchored by per-adapter tests.

The findings below are all **non-blocking** and routed to backlog issues. None justifies a fourth fix round on a 105-day-old branch.

## Blocking Issues

None.

## Non-Blocking Improvements (issues to be created)

| ID | File:Line | Category | Description |
|----|-----------|----------|-------------|
| N1 | `crates/ragentop-cli/src/main.rs:120-127` | Test gap | The new `detect_sessions` → `bool` flag → `ExitCode::FAILURE` propagation (C4/C5) has no integration test. Revert would compile and ship silently. `ragentop-cli` has 0 tests total. |
| N2 | `crates/ragentop-daemon/src/zellij.rs:61-133` | Test gap | The new `rename_pane` control flow — `target_exists` lookup, `max_cycles` loop, cycle-detection, post-loop verification — is untested. Only `parse_pane_line` / `validate_*` helpers have tests. Suggest refactoring into a pure helper that takes function pointers so it can be mocked. |
| N3 | `crates/ragentop-core/src/pricing.rs:105` | Silent failure | `compute_cost` calls `UsdMicros::from_dollars(...)` which silently clamps NaN/negative to ZERO with no `tracing::warn!`. If a user supplies a malformed `pricing.*` in TOML, the cost ledger reports $0, burnrate stays green, alerts never fire. Suggest validating `pricing.*` once at config-load (return `Err`) rather than at every `compute_cost`. |
| N4 | `crates/ragentop-core/src/types.rs:115-122` | Silent failure | `UsdMicros::Deserialize` routes through `from_dollars` and clamps NaN/negative silently. Low risk today (no `UsdMicros` is persisted to disk), but the contract should `serde` error on non-finite. |
| N5 | `crates/ragentop-core/src/alert.rs:255` | Type-safety regression | `check_session_cost(total_cost: f64, threshold: f64)` takes raw `f64` while the canonical accumulator is now `UsdMicros`. No current caller wires this up, so no active confusion — but the next caller must call `.as_f64()`, losing the C7 type-safety guarantee. Suggest `(total_cost: UsdMicros, threshold: f64)` before first real caller exists. |
| N6 | `crates/ragentop-web/src/components/session_detail.rs:14` | Type drift | `SessionDetails.cost_usd: Option<f64>` diverges from `SessionMetrics.cost_usd: Option<UsdMicros>`. Harmless with mock data; needs a conversion shim when real data flows through `client.get_metrics()`. |
| N7 | `crates/ragentop-core/src/types.rs:46-72` | Test gap | `from_dollars(f64::MAX)` clamp branch not directly asserted. `is_finite()` short-circuits `INFINITY`/`NaN`, so risk is low, but a finite-but-huge input has no pinning test. |

## Confirmations (audited and clean)

- **UsdMicros newtype is robustly tested** at `types.rs:619-707`: ZERO, roundtrip, negative → ZERO, NaN/INFINITY/NEG_INFINITY → ZERO, `saturating_add` at `u64::MAX-10`, `saturating_sub` underflow, 10k-call exact accumulation, `Display` six-digit padding, serde `f64`-format roundtrip, deserialize clamping, `Ord`.
- **f64 → UsdMicros migration is complete** across the five cumulative-cost fields: `SessionMetrics.cost_usd`, `BurnRate.total_cost`, `ModelBurnRate.total_cost`, `ModelStats.total_cost`, `compute_cost` return.
- **Adapter detector parity** — claude, codex, copilot, qwen, gemini all use direct field assignment (no remaining `with_*` builders) and uniformly `match` with `tracing::warn!` on file/parse errors instead of silent skips.
- **Silent-failure surface decreased**, not increased: every `let _ = ...` / bare `.ok()` / unbalanced `if let Ok` was removed by `a714541`; new error paths in qwen parser, codex/copilot/qwen detectors, ragentop-cli, and daemon/zellij.rs all log structured warnings.
- **Pricing arithmetic** is regression-anchored by `compute_cost_accumulates_exactly_across_many_calls` (1000 × $0.018 = exact 18M micros) and `compute_cost_all_token_types` (four distinct rates → swap-prompt/completion or `+`→`-` would flip the result).
- **CI**: Check + Clippy + Coverage + Format + Test all pass on the tip.

## PR Description Drift (must fix before merge)

The PR body currently lists adapters as **"Claude, Codex, Aider, Copilot, Goose"** but the actual adapters in `crates/adapters/` are **claude, codex, copilot, gemini, qwen**. Also says 318 tests; actual count is 338. Description has been updated to reflect reality.

## Out-of-Scope (deferred to v0.2.0+)

None new. Prior rounds already routed deferred items.
