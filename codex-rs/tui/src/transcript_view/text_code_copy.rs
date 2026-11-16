//! Code-copy headers are presentation rows, never part of selectable or searchable text.
//! Retain the fenced source so wrapping, highlighting, and resize cannot alter the clipboard.

use super::*;

const LABEL: &str = "[Copy]";

pub(super) struct CodeCopyControl {
    pub(super) offset: usize,
    pub(super) row: Option<usize>,
    pub(super) content: Arc<str>,
}

impl TextLayout {
    pub(in crate::transcript_view) fn with_code_copy_controls(self, markdown: &str) -> Self {
        let mut codes = crate::markdown::extract_copy_targets(markdown)
            .into_iter()
            .filter_map(|target| match target {
                crate::markdown::CopyTarget::Code { content, .. } => Some(content),
                crate::markdown::CopyTarget::Quote(_) => None,
            })
            .collect::<Vec<_>>();
        let normalized = |text: &str| {
            crate::markdown::normalize_markdown_for_rendering(text)
                .lines()
                .map(str::trim_end)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_owned()
        };
        let targets = self
            .copy_block_ranges()
            .into_iter()
            .filter(|(_, label, _)| *label == "Code block")
            .filter_map(|(range, _, payload)| {
                let displayed = payload.as_deref().unwrap_or(&self.text[range.clone()]);
                let at = codes
                    .iter()
                    .position(|code| normalized(code) == normalized(displayed))?;
                Some((range.start, codes.remove(at)))
            })
            .collect();
        self.with_code_copy_targets(targets)
    }

    pub(super) fn with_code_copy_targets(mut self, targets: Vec<(usize, Arc<str>)>) -> Self {
        for (offset, content) in targets {
            // Keep targets at narrow widths so a subsequent resize can restore their buttons.
            let row = if usize::from(self.width) >= LABEL.len() {
                let row = self.rows.partition_point(|row| row.source.start < offset);
                let indent = usize::from(self.width) - LABEL.len();
                self.insert_presentation_row(
                    row,
                    offset,
                    indent,
                    Line::from(format!("{}{LABEL}", " ".repeat(indent))).cyan(),
                );
                Some(row)
            } else {
                None
            };
            self.code_copy_controls.push(CodeCopyControl {
                offset,
                row,
                content,
            });
        }
        self
    }

    pub(in crate::transcript_view) fn code_copy_control(
        &self,
        row: usize,
    ) -> Option<(usize, &str)> {
        self.code_copy_controls
            .iter()
            .find(|control| control.row == Some(row))
            .map(|control| (control.offset, control.content.as_ref()))
    }
}
