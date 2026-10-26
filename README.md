# Agent memory

Local, opt-in memory for AI agents. Each Git repository has its own store;
worktrees share it. Rust standard library only, packaged with Nix.

## Install

From this checkout:

```sh
mkdir -p ~/.local/share/agent-memory ~/.local/bin
nix-build default.nix --out-link ~/.local/share/agent-memory/package
ln -s ~/.local/share/agent-memory/package/bin/agent-memory ~/.local/bin/agent-memory
```

Rebuild with the same `nix-build` command to update. The package link protects
the installation from Nix garbage collection. Git is bundled at runtime.

## Use

Inside a repository you want to enable:

```sh
~/.local/bin/agent-memory init
~/.local/bin/agent-memory note "Use keepalives; when: idle streams disconnect; source: investigation"
~/.local/bin/agent-memory search "idle stream disconnect"
```

Add the [agent instructions](docs/agent-instructions.md) to its `AGENTS.md`.
Memory stays local under the shared Git directory, outside tracked files.
Search returns candidates; verify them against current sources.

- [Commands](docs/commands.md)
- [Optional prompt hooks](docs/hooks.md)
- [Storage and limits](docs/storage.md)
- [Retrieval diagnostic](evaluation/REPORT.md)

## Develop

```sh
nix-shell -p cargo rustc rustfmt git --run 'cargo fmt --check && cargo test --locked'
nix-build default.nix --no-out-link
```

Cargo builds both binaries and runs unit and integration tests. Nix also runs
the retrieval smoke test. `src/` contains the CLI and shared hook/search modules;
`src/bin/agent-memory-eval.rs` is the standalone evaluator; `tests/` exercises
the CLI in disposable repositories. No project memory is needed to run tests.
