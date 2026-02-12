use crate::ast::sequence::*;
use crate::layout::{Canvas, Shape, TextAnchor};

const FONT_SIZE: f64 = 14.0;
const CHAR_WIDTH: f64 = 8.4; // monospace: font_size * 0.6
const ROW_HEIGHT: f64 = 40.0;
const PARTICIPANT_PAD: f64 = 20.0;
const PARTICIPANT_HEIGHT: f64 = 36.0;
const PARTICIPANT_MIN_WIDTH: f64 = 80.0;
const PARTICIPANT_SPACING: f64 = 180.0;
const MARGIN_X: f64 = 40.0;
const MARGIN_TOP: f64 = 20.0;
const NOTE_PAD: f64 = 8.0;
const BLOCK_PAD: f64 = 8.0;
const BLOCK_LABEL_HEIGHT: f64 = 20.0;

pub fn layout(diagram: &SequenceDiagram) -> Canvas {
    let n = diagram.participants.len();
    if n == 0 {
        return Canvas {
            width: 100.0,
            height: 100.0,
            shapes: Vec::new(),
            css: css(),
        };
    }

    // Calculate participant box widths
    let participant_widths: Vec<f64> = diagram
        .participants
        .iter()
        .map(|p| {
            let text_width = p.label.len() as f64 * CHAR_WIDTH + PARTICIPANT_PAD * 2.0;
            text_width.max(PARTICIPANT_MIN_WIDTH)
        })
        .collect();

    // Calculate x positions for each participant (center)
    let mut participant_x: Vec<f64> = Vec::with_capacity(n);
    let mut x = MARGIN_X + participant_widths[0] / 2.0;
    participant_x.push(x);
    for _i in 1..n {
        x += PARTICIPANT_SPACING;
        participant_x.push(x);
    }

    let mut shapes: Vec<Shape> = Vec::new();
    let mut y = MARGIN_TOP;

    // Draw participant boxes at top
    for i in 0..n {
        let px = participant_x[i];
        let pw = participant_widths[i];
        shapes.push(Shape::Rect {
            x: px - pw / 2.0,
            y,
            w: pw,
            h: PARTICIPANT_HEIGHT,
            rx: 4.0,
            label: String::new(),
            class: "participant".to_string(),
        });
        shapes.push(Shape::Text {
            x: px,
            y: y + PARTICIPANT_HEIGHT / 2.0 + FONT_SIZE * 0.35,
            content: diagram.participants[i].label.clone(),
            anchor: TextAnchor::Middle,
            class: "participant-text".to_string(),
        });
    }

    y += PARTICIPANT_HEIGHT + 10.0;
    let lifeline_top = y;

    // Layout events
    let participant_ids: Vec<&str> = diagram.participants.iter().map(|p| p.id.as_str()).collect();
    layout_events(
        &diagram.events,
        &participant_ids,
        &participant_x,
        &mut shapes,
        &mut y,
    );

    y += 20.0;

    // Draw participant boxes at bottom
    for i in 0..n {
        let px = participant_x[i];
        let pw = participant_widths[i];
        shapes.push(Shape::Rect {
            x: px - pw / 2.0,
            y,
            w: pw,
            h: PARTICIPANT_HEIGHT,
            rx: 4.0,
            label: String::new(),
            class: "participant".to_string(),
        });
        shapes.push(Shape::Text {
            x: px,
            y: y + PARTICIPANT_HEIGHT / 2.0 + FONT_SIZE * 0.35,
            content: diagram.participants[i].label.clone(),
            anchor: TextAnchor::Middle,
            class: "participant-text".to_string(),
        });
    }

    let lifeline_bottom = y;

    // Draw lifelines
    for i in 0..n {
        shapes.push(Shape::Line {
            x1: participant_x[i],
            y1: lifeline_top,
            x2: participant_x[i],
            y2: lifeline_bottom,
            dashed: true,
            arrow_head: false,
            arrow_tail: false,
            class: "lifeline".to_string(),
        });
    }

    y += PARTICIPANT_HEIGHT + MARGIN_TOP;
    let total_width = participant_x[n - 1] + participant_widths[n - 1] / 2.0 + MARGIN_X;

    Canvas {
        width: total_width,
        height: y,
        shapes,
        css: css(),
    }
}

fn layout_events(
    events: &[Event],
    participant_ids: &[&str],
    participant_x: &[f64],
    shapes: &mut Vec<Shape>,
    y: &mut f64,
) {
    for event in events {
        match event {
            Event::Message(msg) => {
                let from_idx = participant_index(&msg.from, participant_ids);
                let to_idx = participant_index(&msg.to, participant_ids);
                let from_x = participant_x[from_idx];
                let to_x = participant_x[to_idx];

                let dashed = matches!(msg.arrow, ArrowStyle::Dashed | ArrowStyle::DashedOpen | ArrowStyle::DashedCross);
                let arrow_head = !matches!(msg.arrow, ArrowStyle::SolidOpen | ArrowStyle::DashedOpen);

                if from_idx == to_idx {
                    // Self-message: loop out to the right and back
                    let loop_w = 40.0;
                    let loop_h = ROW_HEIGHT * 0.6;
                    let x = from_x;
                    // Right segment
                    shapes.push(Shape::Line {
                        x1: x,
                        y1: *y,
                        x2: x + loop_w,
                        y2: *y,
                        dashed,
                        arrow_head: false,
                        arrow_tail: false,
                        class: "message".to_string(),
                    });
                    // Down segment
                    shapes.push(Shape::Line {
                        x1: x + loop_w,
                        y1: *y,
                        x2: x + loop_w,
                        y2: *y + loop_h,
                        dashed,
                        arrow_head: false,
                        arrow_tail: false,
                        class: "message".to_string(),
                    });
                    // Back left segment with arrow
                    shapes.push(Shape::Line {
                        x1: x + loop_w,
                        y1: *y + loop_h,
                        x2: x,
                        y2: *y + loop_h,
                        dashed,
                        arrow_head,
                        arrow_tail: false,
                        class: "message".to_string(),
                    });
                    // Label
                    if !msg.text.is_empty() {
                        shapes.push(Shape::Text {
                            x: x + loop_w + 5.0,
                            y: *y + loop_h / 2.0 + FONT_SIZE * 0.35,
                            content: msg.text.clone(),
                            anchor: TextAnchor::Start,
                            class: "message-text".to_string(),
                        });
                    }
                    *y += ROW_HEIGHT;
                } else {
                    // Normal message
                    shapes.push(Shape::Line {
                        x1: from_x,
                        y1: *y,
                        x2: to_x,
                        y2: *y,
                        dashed,
                        arrow_head,
                        arrow_tail: false,
                        class: "message".to_string(),
                    });
                    // Label above the line
                    if !msg.text.is_empty() {
                        let mid_x = (from_x + to_x) / 2.0;
                        shapes.push(Shape::Text {
                            x: mid_x,
                            y: *y - 6.0,
                            content: msg.text.clone(),
                            anchor: TextAnchor::Middle,
                            class: "message-text".to_string(),
                        });
                    }
                    *y += ROW_HEIGHT;
                }
            }
            Event::Note(note) => {
                if note.over.is_empty() {
                    *y += ROW_HEIGHT;
                    continue;
                }
                let indices: Vec<usize> = note
                    .over
                    .iter()
                    .map(|id| participant_index(id, participant_ids))
                    .collect();

                // Handle <br/> as newlines
                let lines: Vec<&str> = note.text.split("<br/>").collect();
                let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);
                let text_width = max_line_len as f64 * CHAR_WIDTH + NOTE_PAD * 2.0;
                let text_height = lines.len() as f64 * (FONT_SIZE + 4.0) + NOTE_PAD * 2.0;
                let note_height = text_height.max(ROW_HEIGHT * 0.8);

                let center_x = if indices.len() == 1 {
                    let idx = indices[0];
                    match note.position {
                        NotePosition::Over => participant_x[idx],
                        NotePosition::LeftOf => participant_x[idx] - text_width / 2.0 - 20.0,
                        NotePosition::RightOf => participant_x[idx] + text_width / 2.0 + 20.0,
                    }
                } else {
                    let min_x = indices.iter().map(|&i| participant_x[i]).fold(f64::INFINITY, f64::min);
                    let max_x = indices.iter().map(|&i| participant_x[i]).fold(f64::NEG_INFINITY, f64::max);
                    (min_x + max_x) / 2.0
                };

                // Clamp note so it doesn't go off-canvas left
                let note_x = (center_x - text_width / 2.0).max(4.0);
                let actual_center = note_x + text_width / 2.0;

                shapes.push(Shape::Rect {
                    x: note_x,
                    y: *y - note_height / 2.0,
                    w: text_width,
                    h: note_height,
                    rx: 0.0,
                    label: String::new(),
                    class: "note".to_string(),
                });

                for (i, line) in lines.iter().enumerate() {
                    let line_y = *y - (lines.len() as f64 - 1.0) * (FONT_SIZE + 4.0) / 2.0
                        + i as f64 * (FONT_SIZE + 4.0)
                        + FONT_SIZE * 0.35;
                    shapes.push(Shape::Text {
                        x: actual_center,
                        y: line_y,
                        content: line.to_string(),
                        anchor: TextAnchor::Middle,
                        class: "note-text".to_string(),
                    });
                }
                *y += note_height.max(ROW_HEIGHT);
            }
            Event::Block(block) => {
                let block_start_y = *y - BLOCK_PAD;

                // Block label background
                let label_text = format!("{} {}", block_kind_label(block.kind), block.label);

                *y += BLOCK_LABEL_HEIGHT;

                for (si, section) in block.sections.iter().enumerate() {
                    if si > 0 {
                        // Draw dashed separator line for else/and
                        let sep_y = *y;
                        shapes.push(Shape::Line {
                            x1: MARGIN_X - 20.0,
                            y1: sep_y,
                            x2: participant_x.last().copied().unwrap_or(200.0) + PARTICIPANT_SPACING / 2.0,
                            y2: sep_y,
                            dashed: true,
                            arrow_head: false,
                            arrow_tail: false,
                            class: "block-separator".to_string(),
                        });
                        if let Some(ref lbl) = section.label {
                            shapes.push(Shape::Text {
                                x: MARGIN_X - 10.0,
                                y: sep_y + FONT_SIZE + 2.0,
                                content: format!("[{}]", lbl),
                                anchor: TextAnchor::Start,
                                class: "block-label".to_string(),
                            });
                        }
                        *y += BLOCK_LABEL_HEIGHT;
                    }
                    layout_events(&section.events, participant_ids, participant_x, shapes, y);
                }

                let block_end_y = *y + BLOCK_PAD;
                let block_left = MARGIN_X - 20.0;
                let block_right = participant_x.last().copied().unwrap_or(200.0) + PARTICIPANT_SPACING / 2.0;

                // Block rectangle
                shapes.push(Shape::Rect {
                    x: block_left,
                    y: block_start_y,
                    w: block_right - block_left,
                    h: block_end_y - block_start_y,
                    rx: 0.0,
                    label: String::new(),
                    class: "block".to_string(),
                });

                // Block label tag
                let label_w = label_text.len() as f64 * CHAR_WIDTH + 16.0;
                shapes.push(Shape::Rect {
                    x: block_left,
                    y: block_start_y,
                    w: label_w,
                    h: BLOCK_LABEL_HEIGHT,
                    rx: 0.0,
                    label: String::new(),
                    class: "block-tag".to_string(),
                });
                shapes.push(Shape::Text {
                    x: block_left + 8.0,
                    y: block_start_y + BLOCK_LABEL_HEIGHT / 2.0 + FONT_SIZE * 0.35,
                    content: label_text,
                    anchor: TextAnchor::Start,
                    class: "block-label".to_string(),
                });

                *y = block_end_y + 10.0;
            }
            Event::Activate(_) | Event::Deactivate(_) => {
                // Activations are visual-only; skip for v0.1
            }
        }
    }
}

fn participant_index(id: &str, participant_ids: &[&str]) -> usize {
    participant_ids
        .iter()
        .position(|&p| p == id)
        .unwrap_or(0)
}

fn block_kind_label(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Loop => "loop",
        BlockKind::Alt => "alt",
        BlockKind::Opt => "opt",
        BlockKind::Par => "par",
        BlockKind::Critical => "critical",
        BlockKind::Break => "break",
    }
}

fn css() -> String {
    r#"
    .participant { fill: #e8e8e8; stroke: #333; stroke-width: 1.5; }
    .participant-text { font-family: monospace; font-size: 14px; fill: #333; font-weight: bold; }
    .lifeline { stroke: #999; stroke-width: 1; stroke-dasharray: 6,4; }
    .message { stroke: #333; stroke-width: 1.5; }
    .message-text { font-family: monospace; font-size: 13px; fill: #333; }
    .note { fill: #ffffcc; stroke: #cccc00; stroke-width: 1; }
    .note-text { font-family: monospace; font-size: 12px; fill: #333; }
    .block { fill: none; stroke: #666; stroke-width: 1; }
    .block-tag { fill: #e0e0e0; stroke: #666; stroke-width: 1; }
    .block-label { font-family: monospace; font-size: 12px; fill: #333; font-weight: bold; }
    .block-separator { stroke: #666; stroke-width: 1; stroke-dasharray: 4,4; }
    "#
    .to_string()
}
