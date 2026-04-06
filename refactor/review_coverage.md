# Review Coverage

Purpose: record which module review files exist in `refactor/` and clarify that this review set is partly historical.

## Current Meaning

- the review files in `refactor/` are a review archive from the refactor process
- some review files cover modules that have since been removed
- the current live module inventory is described in [`refactor/repo_modules.md`](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/repo_modules.md)

## Review Files Present

Active-scope modules:

- `query` → `refactor/query_review.md`
- `api` → `refactor/api_review.md`
- `core` → `refactor/core_review.md`
- `tools` → `refactor/tools_review.md`
- `tui` → `refactor/tui_review.md`
- `commands` → `refactor/commands_review.md`
- `cli` → `refactor/cli_review.md`
- `mcp` → `refactor/mcp_review.md`
- `plugins` → `refactor/plugins_review.md`

Historical removed-scope modules:

- `bridge` → `refactor/bridge_review.md`
- `acp` → `refactor/acp_review.md`
- `buddy` → `refactor/buddy_review.md`

## Conclusion

The review archive is complete for both:

- the current active module set
- the removed modules that were explicitly reviewed during the refactor

If new modules are added later, add a matching `refactor/*_review.md` file only if a fresh review pass is actually performed.
