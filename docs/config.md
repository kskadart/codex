# Configuration

For basic configuration instructions, see [this documentation](https://developers.openai.com/codex/config-basic).

For advanced configuration instructions, see [this documentation](https://developers.openai.com/codex/config-advanced).

For a full configuration reference, see [this documentation](https://developers.openai.com/codex/config-reference).

## Copying code blocks

In the fullscreen transcript, completed assistant responses and plans show a
`[Copy]` control beside the last displayed line of each fenced code block. When
that line has insufficient room, the control appears immediately below the block.
Inline controls reserve room for the widest feedback label so the command is
never covered.
Placement is recalculated when the pane is resized. Click a control to copy the
original code as plain text, without Markdown fences or surrounding prose.
Quotes, indentation, and internal line endings are preserved. Trailing line endings
are removed so pasting does not submit the final command. The existing clipboard
feedback reports whether the copy succeeded.
The clicked control also changes to a highlighted `[Copying…]` while pending,
green `[Copied]` after confirmed delivery, or red `[Failed]` on error. A
terminal-only request shows `[Sent]`; verify delivery by pasting.
Feedback resets to `[Copy]` after two seconds without changing the block's layout.
Visual wrapping never adds newlines to the copied source text.

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
