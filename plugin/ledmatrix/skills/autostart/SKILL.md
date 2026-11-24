---
name: autostart
description: Turn on or off starting the LED Matrix display when a Claude Code session starts.
argument-hint: "[on | off]"
disable-model-invocation: true
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude autostart *), Bash(${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude autostart), Bash(${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude open)
---

The user's argument is: "$ARGUMENTS"

If the argument is `on` or `off`, run exactly this, with the argument in place of ARG:

```
${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude autostart ARG
```

If the argument is empty, run the same command with no argument, which shows the current setting.
If it is anything else, don't run anything; tell the user to use `on` or `off`.

If you turned autostart on, also run this, so it starts now as well:

```
${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude open
```

Then tell the user the result in one short sentence, using what the commands printed. Don't do
anything else.
