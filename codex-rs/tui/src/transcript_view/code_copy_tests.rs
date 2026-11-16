//! Exercise the visible control, mouse routing, and literal clipboard payload together.

use super::*;
use crate::clipboard_copy::CopyFormat;
use crate::clipboard_copy::CopyStatus;
use crate::history_cell::AgentMarkdownCell;
use crate::transcript_view::tests::render;
use crate::transcript_view::tests::text;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseButton;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use pretty_assertions::assert_eq;
use std::path::Path;

fn response(markdown: &str) -> Vec<Arc<dyn HistoryCell>> {
    vec![Arc::new(AgentMarkdownCell::new(
        markdown.to_owned(),
        Path::new("/"),
    ))]
}

fn buttons(view: &TranscriptView) -> Vec<(u16, u16, String)> {
    view.visible
        .iter()
        .enumerate()
        .filter_map(|(row, _)| {
            let column = view.area.width - 1;
            view.code_copy_at(view.area.x + column, view.area.y + row as u16)
                .map(|content| {
                    (
                        view.area.x + column,
                        view.area.y + row as u16,
                        content.into(),
                    )
                })
        })
        .collect()
}

fn click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn copy_button_copies_original_command_as_plain_text() {
    let command = "ssh -t -p 12345 user@example.org 'printf \"hello world\\n\"'\n";
    let cells = response(&format!("Run this command:\n\n```bash\n{command}```"));
    let mut view = TranscriptView::default();
    let buffer = render(&mut view, &cells, /*width*/ 76, /*height*/ 8);
    insta::assert_snapshot!("code_copy_button", text(&buffer));
    let [(column, row, _)] = buttons(&view).try_into().unwrap();
    let Some(ViewAction::CopyCode { text: payload, .. }) =
        view.handle_mouse(click(column, row), &cells)
    else {
        panic!("clicking the visible Copy button should request a clipboard copy");
    };
    assert_eq!(payload, command);
    view.copy_selected_text_with(
        &cells,
        &payload,
        /*clear_selection*/ true,
        |text, format| {
            assert_eq!(text, command);
            assert!(matches!(format, CopyFormat::PlainText));
            Ok(CopyStatus::Confirmed)
        },
    )
    .unwrap();
    assert!(!view.layout(&cells, 0).unwrap().text().contains("[Copy]"));
    assert!(view.is_following());
}

#[test]
fn separate_buttons_keep_identical_blocks_and_untyped_fences_distinct() {
    let cells = response(
        "```sh\necho 'one'\n```\n\n```\necho 'one'\n```\n\n```python\nif True:\n    print(\"two\")  \n```\n",
    );
    let mut view = TranscriptView::default();
    render(&mut view, &cells, /*width*/ 40, /*height*/ 20);
    let targets = buttons(&view);
    assert_eq!(targets.len(), 3);
    for ((column, row, _), expected) in targets.into_iter().zip([
        "echo 'one'\n",
        "echo 'one'\n",
        "if True:\n    print(\"two\")  \n",
    ]) {
        assert!(matches!(
            view.handle_mouse(click(column, row), &cells),
            Some(ViewAction::CopyCode { text: payload, .. }) if payload == expected
        ));
    }
}

#[test]
fn wrapped_command_copy_survives_resize_and_preserves_crlf() {
    let command = "printf '%s\\n' 'a long argument with spaces and quotes'\r\n\tprintf done\r\n";
    let cells = response(&format!("```sh\r\n{command}```\r\n"));
    let mut view = TranscriptView::default();
    render(&mut view, &cells, /*width*/ 20, /*height*/ 20);
    let layout = view.layout(&cells, 0).unwrap();
    let narrow = layout.rewrap(/*width*/ 5);
    assert!((0..narrow.row_count()).all(|row| narrow.code_copy_control(row).is_none()));
    let wide = narrow.rewrap(/*width*/ 60).with_leading_separator();
    let payloads = (0..wide.row_count())
        .filter_map(|row| wide.code_copy_control(row).map(|(_, content)| content))
        .collect::<Vec<_>>();
    assert_eq!(payloads, vec![command]);
    assert_eq!(layout.text(), wide.text());
    render(&mut view, &cells, /*width*/ 32, /*height*/ 20);
    let [(column, row, _)] = buttons(&view).try_into().unwrap();
    assert!(matches!(
        view.handle_mouse(click(column, row), &cells),
        Some(ViewAction::CopyCode { text: payload, .. }) if payload == command
    ));
}

#[test]
fn copy_button_hit_testing_tracks_scrolling_and_offset_viewports() {
    let cells =
        response("Before\n\n```sh\necho first\n```\n\nMiddle\n\n```sh\necho second\n```\n\nAfter");
    let mut view = TranscriptView::default();
    let area = Rect::new(
        /*x*/ 7, /*y*/ 3, /*width*/ 30, /*height*/ 5,
    );
    let mut buffer = Buffer::empty(Rect::new(0, 0, 50, 12));
    view.render(area, &mut buffer, &cells);
    let targets = buttons(&view);
    assert!(!targets.is_empty());
    for (column, row, expected) in targets {
        assert!(matches!(
            view.handle_mouse(click(column, row), &cells),
            Some(ViewAction::CopyCode { text: payload, .. }) if payload == expected
        ));
    }
    view.jump_to_beginning(&cells);
    view.render(area, &mut buffer, &cells);
    let [(column, row, expected)] = buttons(&view).try_into().unwrap();
    assert_eq!(expected, "echo first\n");
    assert!(matches!(
        view.handle_mouse(click(column, row), &cells),
        Some(ViewAction::CopyCode { text: payload, .. }) if payload == expected
    ));
}

#[test]
fn modifier_clicks_and_code_text_do_not_activate_copy_buttons() {
    let cells = response("```sh\necho hello\n```");
    let mut view = TranscriptView::default();
    render(&mut view, &cells, /*width*/ 30, /*height*/ 8);
    let [(column, row, _)] = buttons(&view).try_into().unwrap();
    let mut modified = click(column, row);
    modified.modifiers = KeyModifiers::SHIFT;
    assert!(!matches!(
        view.handle_mouse(modified, &cells),
        Some(ViewAction::Copy(_) | ViewAction::CopyCode { .. })
    ));
    assert!(!matches!(
        view.handle_mouse(click(3, row + 1), &cells),
        Some(ViewAction::Copy(_) | ViewAction::CopyCode { .. })
    ));
}

#[test]
fn raw_transcript_and_user_prompts_do_not_gain_copy_headers() {
    let mut cells = response("```sh\necho hello\n```");
    let mut view = TranscriptView {
        mode: HistoryRenderMode::Raw,
        ..Default::default()
    };
    let buffer = render(&mut view, &cells, /*width*/ 30, /*height*/ 8);
    assert!(buttons(&view).is_empty());
    assert!(!text(&buffer).contains("[Copy]"));
    cells = vec![Arc::new(crate::history_cell::new_user_prompt(
        "```sh\necho user\n```".into(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))];
    view.mode = HistoryRenderMode::Rich;
    render(&mut view, &cells, /*width*/ 30, /*height*/ 8);
    assert!(buttons(&view).is_empty());
}

#[test]
fn clicking_a_button_replaces_copy_mode_payload_with_the_clicked_block() {
    let cells = response("A response\n\n```sh\necho 'Привет 👋'\n```");
    let mut view = TranscriptView::default();
    render(&mut view, &cells, /*width*/ 40, /*height*/ 10);
    view.begin_copy_mode(&cells, /*guard*/ None);
    render(&mut view, &cells, /*width*/ 40, /*height*/ 10);
    let [(column, row, _)] = buttons(&view).try_into().unwrap();
    let Some(ViewAction::CopyCode { text: payload, .. }) =
        view.handle_mouse(click(column, row), &cells)
    else {
        panic!("copy button should work while the response is selected");
    };
    assert!(view.copy_mode.is_none());
    assert!(view.selection.is_none());
    view.copy_selected_text_with(
        &cells,
        &payload,
        /*clear_selection*/ true,
        |text, format| {
            assert_eq!(text, "echo 'Привет 👋'\n");
            assert!(matches!(format, CopyFormat::PlainText));
            Ok(CopyStatus::Confirmed)
        },
    )
    .unwrap();
}

fn target_for_button(
    view: &mut TranscriptView,
    cells: &[Arc<dyn HistoryCell>],
    index: usize,
) -> super::code_copy::CodeCopyTarget {
    let (column, row, _) = buttons(view).remove(index);
    let Some(ViewAction::CopyCode { target, .. }) = view.handle_mouse(click(column, row), cells)
    else {
        panic!("the Copy button should retain its source identity");
    };
    target
}

#[test]
fn button_feedback_tracks_the_clicked_block_and_its_clipboard_ticket() {
    use ratatui::style::Color;
    use ratatui::style::Modifier;

    let cells = response("```sh\necho same\n```\n\n```sh\necho same\n```");
    let mut view = TranscriptView::default();
    render(&mut view, &cells, /*width*/ 40, /*height*/ 12);
    let target = target_for_button(&mut view, &cells, /*index*/ 1);
    view.record_code_copy_result(target, &Ok(CopyStatus::Pending(17)));
    let pending = render(&mut view, &cells, /*width*/ 40, /*height*/ 12);
    assert_eq!(text(&pending).matches("[Copying…]").count(), 1);
    assert_eq!(text(&pending).matches("[Copy]").count(), 1);
    view.finish_copy(
        &cells,
        &(16, Ok(CopyStatus::Confirmed)),
        /*current*/ true,
    );
    let stale = render(&mut view, &cells, /*width*/ 40, /*height*/ 12);
    assert!(text(&stale).contains("[Copying…]"));
    view.finish_copy(
        &cells,
        &(17, Ok(CopyStatus::Confirmed)),
        /*current*/ true,
    );
    let confirmed = render(&mut view, &cells, /*width*/ 40, /*height*/ 12);
    assert_eq!(text(&confirmed).matches("[Copied]").count(), 1);
    assert_eq!(text(&confirmed).matches("[Copy]").count(), 1);
    let (_, row, _) = buttons(&view).remove(1);
    assert_eq!(confirmed[(39, row)].fg, Color::Green);
    assert!(!confirmed[(0, row)].modifier.contains(Modifier::REVERSED));
    assert!(
        confirmed[(39, row)]
            .modifier
            .contains(Modifier::BOLD | Modifier::REVERSED)
    );
    // The longer feedback label remains clickable all the way to its left edge.
    assert!(matches!(
        view.handle_mouse(click(32, row), &cells),
        Some(ViewAction::CopyCode { target: clicked, .. }) if clicked == target
    ));
    assert!(!view.layout(&cells, 0).unwrap().text().contains("[Copied]"));
}

#[test]
fn unconfirmed_or_failed_delivery_never_shows_a_successful_button() {
    use ratatui::style::Color;

    let cells = response("```sh\necho hello\n```");
    let mut view = TranscriptView::default();
    render(&mut view, &cells, /*width*/ 30, /*height*/ 8);
    let target = target_for_button(&mut view, &cells, /*index*/ 0);
    for (result, label, color) in [
        (Ok(CopyStatus::Unconfirmed), "[Sent]", Color::Cyan),
        (Err("clipboard unavailable".into()), "[Failed]", Color::Red),
    ] {
        view.record_code_copy_result(target, &result);
        let buffer = render(&mut view, &cells, /*width*/ 30, /*height*/ 8);
        assert!(text(&buffer).contains(label));
        assert!(!text(&buffer).contains("[Copied]"));
        let (_, row, _) = buttons(&view).remove(0);
        assert_eq!(buffer[(29, row)].fg, color);
    }
}

#[test]
fn feedback_reflows_and_resets_even_without_space_for_footer_feedback() {
    use std::time::Duration;
    use std::time::Instant;

    let cells = response("```sh\necho hello\n```");
    let mut view = TranscriptView::default();
    render(&mut view, &cells, /*width*/ 30, /*height*/ 20);
    let target = target_for_button(&mut view, &cells, /*index*/ 0);
    view.record_code_copy_result(target, &Ok(CopyStatus::Confirmed));
    let narrow = render(&mut view, &cells, /*width*/ 6, /*height*/ 20);
    assert!(text(&narrow).contains("[Done]"));
    let wide = render(&mut view, &cells, /*width*/ 30, /*height*/ 20);
    assert!(text(&wide).contains("[Copied]"));
    let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 20));
    let delay = view
        .render_composer_gap(None, None, &mut buffer, Instant::now())
        .unwrap();
    assert!(delay <= Duration::from_secs(2));
    view.render_composer_gap(
        None,
        None,
        &mut buffer,
        Instant::now() + Duration::from_secs(3),
    );
    let reset = render(&mut view, &cells, /*width*/ 30, /*height*/ 20);
    assert!(text(&reset).contains("[Copy]"));
    assert!(!text(&reset).contains("[Copied]"));
}

#[test]
fn a_busy_second_button_does_not_steal_pending_feedback() {
    let cells = response("```sh\necho first\n```\n\n```sh\necho second\n```");
    let mut view = TranscriptView::default();
    render(&mut view, &cells, /*width*/ 40, /*height*/ 12);
    let first = target_for_button(&mut view, &cells, /*index*/ 0);
    view.record_code_copy_result(first, &Ok(CopyStatus::Pending(42)));
    let second = target_for_button(&mut view, &cells, /*index*/ 1);
    view.record_code_copy_result(second, &Ok(CopyStatus::Busy));
    view.finish_copy(
        &cells,
        &(42, Ok(CopyStatus::Confirmed)),
        /*current*/ true,
    );
    let buffer = render(&mut view, &cells, /*width*/ 40, /*height*/ 12);
    let (_, first_row, _) = buttons(&view).remove(0);
    let (_, second_row, _) = buttons(&view).remove(1);
    assert!(
        text(&buffer)
            .lines()
            .nth(usize::from(first_row))
            .unwrap()
            .contains("[Copied]")
    );
    assert!(
        text(&buffer)
            .lines()
            .nth(usize::from(second_row))
            .unwrap()
            .contains("[Copy]")
    );
}
