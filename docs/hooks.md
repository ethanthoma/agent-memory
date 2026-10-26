# Optional prompt recall

Enable memory with `agent-memory init`, then merge this handler into the
appropriate settings file. Preserve existing settings and hooks.

| Agent | Settings file | Subcommand |
| --- | --- | --- |
| Codex | `.codex/hooks.json` or `~/.codex/hooks.json` | `codex-hook` |
| Claude Code | `.claude/settings.local.json` | `claude-hook` |

```json
{
  "hooks": {
    "UserPromptSubmit": [{
      "hooks": [{
        "type": "command",
        "command": "\"$HOME/.local/bin/agent-memory\" codex-hook",
        "timeout": 5
      }]
    }]
  }
}
```

For Claude Code, replace `codex-hook` with `claude-hook`. For Codex, review and
trust the hook with `/hooks`; changing its definition requires another review.
Codex loads project hooks only for trusted projects.

Hooks check the event directory against the process directory. Claude also
checks its session project. They search only the enabled repository's shared
store and inject up to three candidates within 2048 search-context bytes.
Notes enter model context as untrusted evidence; verify sources and relevance.
Errors, empty queries, and disabled stores stay silent and do not block prompts.
Remove the handler to disable recall.

For a non-Git Codex workspace, set `AGENT_MEMORY_WORKSPACE` to its exact directory
and `AGENT_MEMORY_STORE_REPOSITORY` to a separate enabled memory repository in
the hook command. The mapping applies only when Codex runs from that exact
workspace; other repositories use their own stores.
