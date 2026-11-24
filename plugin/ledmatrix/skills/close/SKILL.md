---
name: close
description: Stop showing the Claude Code sessions on the LED Matrix.
disable-model-invocation: true
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude close)
---

Run exactly this:

```
${CLAUDE_PLUGIN_ROOT}/scripts/ledmatrix-claude close
```

Then tell the user the result in one short sentence, using what it printed. If autostart is on,
the next Claude Code session starts it again; mention that only if the output suggests it. Don't do
anything else.
