# Storage and limits

`<git-common-dir>/agent-memory/` holds `identity`, `journal`, `pins`, `summary`,
and `lock`. Worktrees share this store; separately initialized repositories
have different identities. Back it up explicitly: Git pushes do not include it.

Journal line number minus one is the note ID. Erasure leaves a blank slot, so
IDs are never reused. Normal writes preserve existing notes; correction and
erasure replace content. Deletion cannot remove prior tool output or backups.

Startup reads pins, a single agent-written rolling summary, and recent notes.
Uncovered older history is reported and remains searchable. A summary becomes
due after 64 slots beyond its coverage. Its accuracy is the agent's responsibility.

Operations use an exclusive, nonblocking lock; retry if busy. Writes sync a
temporary file, rename it atomically, and sync the directory. Corrections and
erasures clear summaries first, so an interruption may leave a cleared summary
without losing notes.

The journal is capped at 16 MiB and 65,536 ID slots. Operations load and rewrite
bounded files: use this for curated knowledge rather than transcripts. There
is no embedding runtime, background summarizer, or cloud sync. Prompt hooks
are installed separately and are opt-in.
