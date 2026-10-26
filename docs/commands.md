# Commands

Run `agent-memory` inside a Git repository or worktree. Only `init` creates a
store; rerunning it preserves notes. Text arguments are single lines of UTF-8.

| Command | Behavior |
| --- | --- |
| `init` | Enable memory and create a stable random store identity. |
| `wake` | Read pins, the rolling summary, and the latest 16 note slots. |
| `note "text"` | Save up to 512 bytes; print the stable note ID. |
| `recall "text"` | Literal case-insensitive substring lookup; newest 64 matches. |
| `search "task"` | BM25 keyword ranking; up to three candidates within 2048 bytes. |
| `pin <id>` / `unpin <id>` | Add or remove a pin; at most 16 pins. |
| `summarize <through_id> "text"` | Replace the overview, up to 4096 bytes. |
| `correct <id> "text"` | Replace content, keeping its ID and pin; clear the summary. |
| `forget <id>` | Erase content and pin, keeping its ID slot; clear the summary. |
| `info` | JSON store identity and live note count. |
| `get <id>` | Current note as JSON, including its `memory://<store>/notes/<id>` URI. |
| `search-json "task"` | Ranked candidates as JSON with resource URIs. |
| `codex-hook` / `claude-hook` | [Optional prompt recall](hooks.md). |

Resource reads require an identity created by `init`; they never initialize
or migrate storage. Corrections change the returned content; deleted notes
fail to resolve. JSON marks notes as untrusted data.

The separate `agent-memory-eval evaluation` command runs the fictional retrieval
diagnostic without accessing project memory. See [its report](../evaluation/REPORT.md)
for optional external vectors and limitations.
