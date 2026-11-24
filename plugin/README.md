# ledmatrix Claude Code plugin

Shows what each Claude Code session is doing on the Framework 16 LED Matrix. Each session gets a
bar, two LEDs high, in the order the sessions started:

| State | Looks like | When |
|---|---|---|
| Working | Dim, with a light moving back and forth | You send a prompt, and while Claude runs tools. |
| Idle | Steady | Claude finished its turn and waits for your next prompt. |
| Needs you | Blinking | Claude, or one of its subagents, waits for a permission or asked you a question. |

Up to 11 sessions fit. With more, the ones that need you are shown first. A session's bar goes
away when it ends, or when its Claude Code process is gone.

Linux only. It runs `inputmodule-control`, built from
[inputmodule-rs](https://github.com/FrameworkComputer/inputmodule-rs).

## Install

The repository is the marketplace. Add it from GitHub, with `#branch` for a branch:

```
/plugin marketplace add FrameworkComputer/inputmodule-rs#inputmodule-rs-claude
/plugin install ledmatrix@inputmodule
```

Installed that way, the plugin has no binary, so it uses `inputmodule-control` from your PATH. It
needs one with `claude-hook`, built from the same branch:

```
cargo install --path inputmodule-control --target "$(rustc -vV | sed -n 's/^host: //p')"
```

Or build the binary into a clone, and add the clone as the marketplace:

```
plugin/build.sh
```

```
/plugin marketplace add /path/to/inputmodule-rs
/plugin install ledmatrix@inputmodule
```

That build is for your machine's architecture and needs libudev.

After new commits, run `/plugin marketplace update inputmodule`; third-party marketplaces don't
update on their own.

To give the LED Matrix to your user without root, install the udev rule in
[`release/50-framework-inputmodule.rules`](https://github.com/FrameworkComputer/inputmodule-rs/blob/main/release/50-framework-inputmodule.rules)
of inputmodule-rs:

```
sudo cp release/50-framework-inputmodule.rules /etc/udev/rules.d/
sudo udevadm control --reload && sudo udevadm trigger
```

Then start it with `/ledmatrix:open`, or turn on autostart.

| Command | Does |
|---|---|
| `/ledmatrix:open` | Starts showing the sessions now. |
| `/ledmatrix:close` | Stops it. The LED Matrix goes to sleep within a minute. |
| `/ledmatrix:autostart on` | Starts it whenever a Claude Code session starts, and now. |
| `/ledmatrix:autostart off` | Stops starting it automatically. |
| `/ledmatrix:autostart` | Shows the setting. |

Autostart is off until you turn it on. The setting is a file,
`~/.config/inputmodule-claude/autostart`.

If the LED Matrix isn't plugged in, it waits for it, and it finds it again after it was unplugged.

If you also have `inputmodule-control claude-hook` in your own Claude Code settings, remove those
hooks: the plugin does the same thing, and both would run.

## How it works

The hooks run `inputmodule-control claude-hook`, which writes each session's state to a file in
`$XDG_RUNTIME_DIR/inputmodule-claude` and exits. It never talks to the device, so it can't slow
Claude down. `inputmodule-control led-matrix --claude` reads those files and draws the bars.

That runs as the systemd user unit `inputmodule-claude` where systemd is available, so it outlives
the session that started it. Logs: `journalctl --user -u inputmodule-claude`. Without systemd it
runs in the background, with its log in `$XDG_RUNTIME_DIR/inputmodule-claude/display.log`. A lock
keeps two sessions from starting two copies.

## Layout

| Path | Contents |
|---|---|
| `../.claude-plugin/marketplace.json` | The marketplace, at the top of the repository, with this one plugin. |
| `ledmatrix/hooks/hooks.json` | Every hook runs `inputmodule-control claude-hook`. `SessionStart` also starts the display if autostart is on. |
| `ledmatrix/skills/` | `/ledmatrix:open`, `/ledmatrix:close` and `/ledmatrix:autostart`. |
| `ledmatrix/scripts/inputmodule-control` | Runs the build for this machine, or the one on PATH. Without either, the hooks do nothing. |
| `ledmatrix/scripts/ledmatrix-claude` | Starts and stops the display, and keeps the autostart setting. |
| `ledmatrix/libexec/` | `inputmodule-control` for this machine, made by `build.sh`. Not in git. |
| `build.sh` | Builds the binary. With a folder argument, also copies the finished marketplace there, to publish as its own repository. |
