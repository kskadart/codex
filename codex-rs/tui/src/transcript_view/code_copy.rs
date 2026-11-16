//! Feedback belongs to the clicked code block and its clipboard ticket, not its screen row.

use super::*;
use crate::clipboard_copy::CopyStatus;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CodeCopyTarget {
    pub(super) key: EntryKey,
    pub(super) offset: usize,
}

pub(super) struct CodeCopyFeedback {
    target: CodeCopyTarget,
    result: Result<CopyStatus, ()>,
    pub(super) expires_at: Option<Instant>,
}

impl CodeCopyFeedback {
    fn is_visible(&self, now: Instant) -> bool {
        self.expires_at.is_none_or(|expiry| expiry > now)
    }

    fn label(&self, width: u16) -> Line<'static> {
        let (label, compact) = match self.result {
            Ok(CopyStatus::Confirmed) => ("[Copied]", "[Done]"),
            Ok(CopyStatus::Pending(_)) => ("[Copying…]", "[Wait]"),
            Ok(CopyStatus::Unconfirmed) => ("[Sent]", "[Sent]"),
            Ok(CopyStatus::Busy) => ("[Busy]", "[Busy]"),
            Err(()) => ("[Failed]", "[Fail]"),
        };
        let label = if Line::from(label).width() <= usize::from(width) {
            label
        } else {
            compact
        };
        let line = Line::from(label).bold().reversed();
        match self.result {
            Ok(CopyStatus::Confirmed) => line.green(),
            Err(()) => line.red(),
            _ => line.cyan(),
        }
    }
}

impl TranscriptView {
    pub(super) fn code_copy_hit_at(&self, column: u16, row: u16) -> Option<(CodeCopyTarget, &str)> {
        if !self.area.contains((column, row).into()) {
            return None;
        }
        let visible = self.visible.get(usize::from(row - self.area.y))?;
        let (offset, content) = visible.layout.code_copy_control(visible.row)?;
        let target = CodeCopyTarget {
            key: visible.key,
            offset,
        };
        let width = self
            .code_copy_feedback
            .as_ref()
            .filter(|feedback| feedback.target == target && feedback.is_visible(Instant::now()))
            .map_or(
                /*default*/ 6,
                |feedback| feedback.label(self.area.width).width(),
            );
        let start = self.area.right().checked_sub(u16::try_from(width).ok()?)?;
        (column >= start).then_some((target, content))
    }

    pub(super) fn record_code_copy_result(
        &mut self,
        target: CodeCopyTarget,
        result: &Result<CopyStatus, String>,
    ) {
        // A rejected second request must not steal the first request's pending ticket.
        if result == &Ok(CopyStatus::Busy)
            && self
                .code_copy_feedback
                .as_ref()
                .is_some_and(|feedback| matches!(feedback.result, Ok(CopyStatus::Pending(_))))
        {
            return;
        }
        self.code_copy_feedback = Some(CodeCopyFeedback {
            target,
            result: result.as_ref().copied().map_err(|_| ()),
            expires_at: (!matches!(result, Ok(CopyStatus::Pending(_))))
                .then(|| Instant::now() + Duration::from_secs(/*secs*/ 2)),
        });
    }

    pub(super) fn finish_code_copy(
        &mut self,
        completion: &(u64, crate::clipboard_copy::worker::CopyResult),
    ) {
        if let Some(feedback) = &self.code_copy_feedback
            && feedback.result == Ok(CopyStatus::Pending(completion.0))
        {
            self.record_code_copy_result(feedback.target, &completion.1);
        }
    }

    pub(super) fn render_code_copy_feedback(
        &self,
        key: EntryKey,
        layout: &TextLayout,
        row: usize,
        area: Rect,
        buffer: &mut Buffer,
    ) {
        let Some(feedback) = &self.code_copy_feedback else {
            return;
        };
        if feedback.target.key != key
            || !feedback.is_visible(Instant::now())
            || layout.code_copy_control(row).map(|(offset, _)| offset)
                != Some(feedback.target.offset)
        {
            return;
        }
        Clear.render(area, buffer);
        let label = feedback.label(area.width);
        let width = label.width() as u16;
        label.render(Rect::new(area.right() - width, area.y, width, 1), buffer);
    }
}
