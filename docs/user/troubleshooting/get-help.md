---
title: Report a problem
description: What to include and where to post when Pliego misbehaves or crashes.
order: 20
verified: 0.7.1
---

Post in the [issue tracker](https://github.com/getnodalia/pliego/issues) or in [Discord](https://github.com/getnodalia/pliego/issues), and attach the log file. The log is the most useful thing you can send after a crash.

## What to include

- Your Pliego version and how you installed it (installer, portable, AppImage and so on).
- Your operating system and graphics card.
- What you did, what you expected and what happened.
- The log file `pliego.log`.

## Where the log is

Pliego writes `logs/pliego.log` in its settings folder, beside `ui.json` and `prefs.json`.

| System  | Log folder                                                              |
|---------|-------------------------------------------------------------------------|
| Windows | `%APPDATA%\Pliego\logs\`                                           |
| macOS   | `~/Library/Application Support/Pliego/logs/`                       |
| Linux   | `~/.config/pliego/logs/` (or `$XDG_CONFIG_HOME/pliego/logs/`) |

Each start moves the previous log to `pliego.1.log`, and that one to `pliego.2.log`. The log from a run that crashed survives the next start, so attach `pliego.1.log` if you have restarted since. The file stops growing at 16 MiB.

## More detail in the log

Set the environment variable `RUST_LOG` to `debug` before starting Pliego to log more. The default is `info` for Pliego's own parts and `warn` for everything else.
