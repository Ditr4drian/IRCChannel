# Arquitectura (para desarrolladores)

## Architecture

```
main / runtime          composition root (tokio select: keys + IRC + tick)
        │
        ├─ ui/          ratatui widgets (render AppState only)
        ├─ app/         TEA: state · events · update · commands
        ├─ irc/         TCP adapter + codec (no UI)
        ├─ domain/      pure model (Channel, ChatLine, User)
        └─ config/      default.toml → XDG file → env
```

Dependency rule: **ui/irc → app → domain**. Protocol and presentation never import each other.

| Crate module | Extend when… |
|---|---|
| `irc/codec` | new message tags / CAP |
| `irc/client` | TLS, reconnect, SASL |
| `irc/event` | map more numerics |
| `app/update` | slash commands, hotkeys |
| `domain` | backlog, highlights, ignores |
| `ui/theme` | user themes |
| `ui/widgets` | nick list, multi-pane |

## Build & run

```bash
cd ~/Projects/IRCChannel
cargo run --release
```

Requires reachability to the IRC host (DITRA VPN).

```bash
cargo test
cargo run   # debug
```

## Configuration

Precedence: embedded `config/default.toml` → `~/.config/ircchannel/config.toml` → env.

| Env | Meaning |
|---|---|
| `IRC_HOST` | server host |
| `IRC_PORT` | port (default 6667) |
| `IRC_NICK` | nickname |
| `IRC_USER` | username |
| `IRC_REALNAME` | realname |
| `IRC_CHANNELS` | comma-separated channels |
| `IRC_PASSWORD` | server PASS |
| `IRC_TZ` | IANA timezone for timestamps (default `America/Mexico_City`) |
| `RUST_LOG` | tracing filter (file log) |

Logs: `~/.local/share/ircchannel/ircchannel.log` (XDG).

## Keys & commands

| Input | Action |
|---|---|
| text + Enter | `PRIVMSG` active buffer |
| `Tab` / `Shift+Tab` | cycle channel buffers |
| `PageUp` / `PageDown` | scroll history |
| `Ctrl+C` / `Ctrl+Q` | quit |
| `/join #chan` | join |
| `/part` | leave active |
| `/msg nick text` | query / privmsg |
| `/nick new` | change nick |
| `/raw …` | send raw line |
| `/buffer #chan` | focus buffer |
| `/quit` | disconnect + exit |
| `/help` | list commands |

## Roadmap (modular hooks already in place)

1. TLS listener module (`irc/tls.rs`)
2. Reconnect policy in `runtime`
3. SASL / NickServ in `irc/client` registration
4. Nick list pane widget
5. Persistent logs per channel in `domain`/storage adapter

## Releases & changelog

Versions are driven by [Conventional Commits](https://www.conventionalcommits.org) and [git-cliff](https://git-cliff.org) (`cliff.toml`).

| Branch / event | Workflow | What happens |
|---|---|---|
| push / PR to `dev` or `main` | `tests.yml` | `cargo fmt --check`, `clippy -D warnings`, `cargo test` |
| push to `dev` | `changelog.yml` | refreshes `[Unreleased]` in `CHANGELOG.md` + `docs/changelog.json` |
| push to `main` | `release.yml` | bumps version (Cargo.toml, Cargo.lock, `VERSION`), writes changelog, tags `vX.Y.Z`, publishes GitHub Release with the Linux binary |
| manual dispatch | `release.yml` | same, optionally forcing a version |

Bump rules: `feat` → minor, `fix`/`perf` → patch, `!` or `BREAKING CHANGE` → major (while `0.x`, git-cliff keeps majors at minor).
Only `feat`, `fix`, `perf`, `refactor` and `docs` appear in the changelog; `chore`, `ci`, `test`, `style`, `build` are skipped.

Workflow: branch from `dev` → PR to `dev` → when ready, PR `dev` → `main` to cut a release.
