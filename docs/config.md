# Configuration

For basic configuration instructions, see [this documentation](https://developers.openai.com/codex/config-basic).

For advanced configuration instructions, see [this documentation](https://developers.openai.com/codex/config-advanced).

For a full configuration reference, see [this documentation](https://developers.openai.com/codex/config-reference).

## Copying code blocks

In the fullscreen transcript, completed assistant responses and plans show a
`[Copy]` control above each nonempty fenced code block. Click it to copy the
original code as plain text, without Markdown fences or surrounding prose.
Quotes, indentation, and line endings are preserved. The existing clipboard
feedback reports whether the copy succeeded.
The clicked control also changes to a highlighted `[Copying…]` while pending,
green `[Copied]` after confirmed delivery, or red `[Failed]` on error. A
terminal-only request shows `[Sent]`; verify delivery by pasting.
Feedback resets to `[Copy]` after two seconds without changing the block's layout.

These controls require terminal mouse support and `[tui].fullscreen_transcript = true`
(the default). They are hidden in raw transcript mode and when the pane is too
narrow to display the complete label. `/copy` remains available for keyboard use
and native scrollback sessions.

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.
