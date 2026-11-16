//! Copy controls are decorations, never part of selectable or searchable code.
//! Retain the fenced source so wrapping, highlighting, and resize cannot alter the clipboard.

use super::*;

const LABEL: &str = "[Copy]";
const INLINE_GAP: u16 = 4;
const FEEDBACK_WIDTH: u16 = 10;

pub(super) struct CodeCopyControl {
    pub(super) range: Range<usize>,
    pub(super) row: Option<usize>,
    column: Option<u16>,
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
                Some((range, codes.remove(at)))
            })
            .collect();
        self.with_code_copy_targets(targets)
    }

    pub(super) fn with_code_copy_targets(mut self, targets: Vec<(Range<usize>, Arc<str>)>) -> Self {
        for (range, content) in targets {
            let inline = (content.lines().count() == 1)
                .then(|| self.inline_code_copy_position(&range))
                .flatten();
            // Keep targets at narrow widths so a subsequent resize can restore their buttons.
            let (row, column) = if let Some((row, column)) = inline {
                (Some(row), Some(column))
            } else if usize::from(self.width) >= LABEL.len() {
                let row = self
                    .rows
                    .partition_point(|row| row.source.start < range.start);
                let indent = usize::from(self.width) - LABEL.len();
                self.insert_presentation_row(
                    row,
                    range.start,
                    indent,
                    Line::from(format!("{}{LABEL}", " ".repeat(indent))).cyan(),
                );
                (Some(row), None)
            } else {
                (None, None)
            };
            self.code_copy_controls.push(CodeCopyControl {
                range,
                row,
                column,
                content,
            });
        }
        self
    }

    fn inline_code_copy_position(&self, range: &Range<usize>) -> Option<(usize, u16)> {
        let mut rows = self.rows.iter().enumerate().filter(|(_, row)| {
            !row.source.is_empty() && row.source.start < range.end && row.source.end > range.start
        });
        let (index, row) = rows.next()?;
        if rows.next().is_some() {
            return None;
        }
        let column = self
            .column_for_offset_on_row(index, row.source.end)
            .checked_add(INLINE_GAP)?;
        (column.checked_add(FEEDBACK_WIDTH)? <= self.width).then_some((index, column))
    }

    pub(in crate::transcript_view) fn code_copy_control(
        &self,
        row: usize,
    ) -> Option<(usize, &str)> {
        self.code_copy_controls
            .iter()
            .find(|control| control.row == Some(row))
            .map(|control| (control.range.start, control.content.as_ref()))
    }

    pub(in crate::transcript_view) fn code_copy_columns(
        &self,
        row: usize,
        label_width: u16,
    ) -> Option<Range<u16>> {
        let control = self
            .code_copy_controls
            .iter()
            .find(|control| control.row == Some(row))?;
        let start = match control.column {
            Some(column) => column,
            None => self.width.checked_sub(label_width)?,
        };
        let end = start.checked_add(label_width)?;
        (end <= self.width).then_some(start..end)
    }

    pub(in crate::transcript_view) fn render_code_copy_control(
        &self,
        row: usize,
        area: Rect,
        buffer: &mut Buffer,
        label: Option<Line<'static>>,
    ) {
        let label = label.unwrap_or_else(|| Line::from(LABEL).cyan());
        let Some(columns) = self.code_copy_columns(row, label.width() as u16) else {
            return;
        };
        let target = Rect::new(
            area.x + columns.start,
            area.y,
            columns.end - columns.start,
            1,
        );
        ratatui::widgets::Clear.render(target, buffer);
        label.render(target, buffer);
    }
}
