---
name: open
description: Start showing the Claude Code sessions on the LED Matrix, if it isn't already running.
disable-model-invocation: true
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude open)
---

Run exactly this:

```
${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude open
```

Then tell the user the result in one short sentence, using what it printed. Don't do anything else.
