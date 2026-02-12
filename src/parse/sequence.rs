use crate::ast::sequence::*;

pub fn parse(input: &str) -> Result<SequenceDiagram, String> {
    let mut participants: Vec<Participant> = Vec::new();
    let mut participant_ids: Vec<String> = Vec::new();
    let lines: Vec<&str> = input.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let events = parse_events(&lines, &mut participants, &mut participant_ids, &mut 0)?;
    Ok(SequenceDiagram { participants, events })
}

fn parse_events(
    lines: &[&str],
    participants: &mut Vec<Participant>,
    participant_ids: &mut Vec<String>,
    pos: &mut usize,
) -> Result<Vec<Event>, String> {
    let mut events = Vec::new();
    while *pos < lines.len() {
        let line = lines[*pos];

        if line == "end" {
            return Ok(events);
        }

        if let Some(rest) = strip_block_keyword(line) {
            let (kind, label) = rest;
            *pos += 1;
            let sections = parse_block_sections(lines, participants, participant_ids, pos, kind)?;
            events.push(Event::Block(Block { kind, label, sections }));
            // *pos now points past the "end"
            continue;
        }

        if line.starts_with("participant ") {
            parse_participant(&line["participant ".len()..], participants, participant_ids);
            *pos += 1;
            continue;
        }

        if line.starts_with("actor ") {
            parse_participant(&line["actor ".len()..], participants, participant_ids);
            *pos += 1;
            continue;
        }

        if line.starts_with("Note ") || line.starts_with("note ") {
            let note = parse_note(&line[5..], participants, participant_ids)?;
            events.push(Event::Note(note));
            *pos += 1;
            continue;
        }

        if line.starts_with("activate ") {
            let id = line["activate ".len()..].trim().to_string();
            ensure_participant(&id, participants, participant_ids);
            events.push(Event::Activate(id));
            *pos += 1;
            continue;
        }

        if line.starts_with("deactivate ") {
            let id = line["deactivate ".len()..].trim().to_string();
            ensure_participant(&id, participants, participant_ids);
            events.push(Event::Deactivate(id));
            *pos += 1;
            continue;
        }

        // Try message parse
        if let Some(msg) = parse_message(line, participants, participant_ids) {
            events.push(Event::Message(msg));
            *pos += 1;
            continue;
        }

        // Skip unrecognized lines
        *pos += 1;
    }
    Ok(events)
}

fn parse_block_sections(
    lines: &[&str],
    participants: &mut Vec<Participant>,
    participant_ids: &mut Vec<String>,
    pos: &mut usize,
    kind: BlockKind,
) -> Result<Vec<BlockSection>, String> {
    let mut sections = Vec::new();
    let mut current_events = Vec::new();
    let mut current_label: Option<String> = None;

    while *pos < lines.len() {
        let line = lines[*pos];

        if line == "end" {
            sections.push(BlockSection {
                label: current_label.take(),
                events: current_events,
            });
            *pos += 1;
            return Ok(sections);
        }

        // Handle "else" for alt blocks
        if (kind == BlockKind::Alt && line.starts_with("else"))
            || (kind == BlockKind::Par && line.starts_with("and"))
        {
            sections.push(BlockSection {
                label: current_label.take(),
                events: current_events,
            });
            current_events = Vec::new();
            let separator = if line.starts_with("else") { "else" } else { "and" };
            let rest = line[separator.len()..].trim();
            current_label = if rest.is_empty() { None } else { Some(rest.to_string()) };
            *pos += 1;
            continue;
        }

        // Recursively parse nested blocks
        if let Some((nested_kind, nested_label)) = strip_block_keyword(line) {
            *pos += 1;
            let nested_sections =
                parse_block_sections(lines, participants, participant_ids, pos, nested_kind)?;
            current_events.push(Event::Block(Block {
                kind: nested_kind,
                label: nested_label,
                sections: nested_sections,
            }));
            continue;
        }

        // Parse events inside block
        if line.starts_with("Note ") || line.starts_with("note ") {
            let note = parse_note(&line[5..], participants, participant_ids)?;
            current_events.push(Event::Note(note));
            *pos += 1;
            continue;
        }

        if let Some(msg) = parse_message(line, participants, participant_ids) {
            current_events.push(Event::Message(msg));
            *pos += 1;
            continue;
        }

        if line.starts_with("activate ") {
            let id = line["activate ".len()..].trim().to_string();
            ensure_participant(&id, participants, participant_ids);
            current_events.push(Event::Activate(id));
            *pos += 1;
            continue;
        }

        if line.starts_with("deactivate ") {
            let id = line["deactivate ".len()..].trim().to_string();
            ensure_participant(&id, participants, participant_ids);
            current_events.push(Event::Deactivate(id));
            *pos += 1;
            continue;
        }

        *pos += 1;
    }

    Err("unexpected end of input: missing 'end' for block".to_string())
}

fn strip_block_keyword(line: &str) -> Option<(BlockKind, String)> {
    let keywords = [
        ("loop ", BlockKind::Loop),
        ("alt ", BlockKind::Alt),
        ("opt ", BlockKind::Opt),
        ("par ", BlockKind::Par),
        ("critical ", BlockKind::Critical),
        ("break ", BlockKind::Break),
    ];
    for (prefix, kind) in &keywords {
        if line.starts_with(prefix) {
            return Some((*kind, line[prefix.len()..].trim().to_string()));
        }
    }
    // Bare keywords without labels
    let bare = [
        ("loop", BlockKind::Loop),
        ("alt", BlockKind::Alt),
        ("opt", BlockKind::Opt),
        ("par", BlockKind::Par),
        ("critical", BlockKind::Critical),
        ("break", BlockKind::Break),
    ];
    for (keyword, kind) in &bare {
        if line == *keyword {
            return Some((*kind, String::new()));
        }
    }
    None
}

fn parse_participant(rest: &str, participants: &mut Vec<Participant>, ids: &mut Vec<String>) {
    let (id, label) = if let Some(as_pos) = rest.find(" as ") {
        let id = rest[..as_pos].trim().to_string();
        let label = rest[as_pos + 4..].trim().to_string();
        (id, label)
    } else {
        let id = rest.trim().to_string();
        let label = id.clone();
        (id, label)
    };
    if !ids.contains(&id) {
        ids.push(id.clone());
        participants.push(Participant { id, label });
    }
}

fn ensure_participant(id: &str, participants: &mut Vec<Participant>, ids: &mut Vec<String>) {
    if !ids.contains(&id.to_string()) {
        ids.push(id.to_string());
        participants.push(Participant {
            id: id.to_string(),
            label: id.to_string(),
        });
    }
}

fn parse_note(
    rest: &str,
    participants: &mut Vec<Participant>,
    participant_ids: &mut Vec<String>,
) -> Result<Note, String> {
    // "over X: text" or "over X,Y: text" or "left of X: text" or "right of X: text"
    let (position, after_pos) = if rest.starts_with("over ") {
        (NotePosition::Over, &rest[5..])
    } else if rest.starts_with("left of ") {
        (NotePosition::LeftOf, &rest[8..])
    } else if rest.starts_with("right of ") {
        (NotePosition::RightOf, &rest[9..])
    } else {
        return Err(format!("invalid note syntax: {}", rest));
    };

    let colon_pos = after_pos
        .find(':')
        .ok_or_else(|| format!("note missing colon: {}", rest))?;

    let participant_part = after_pos[..colon_pos].trim();
    let text = after_pos[colon_pos + 1..].trim().to_string();

    let over: Vec<String> = participant_part
        .split(',')
        .map(|s| {
            let id = s.trim().to_string();
            ensure_participant(&id, participants, participant_ids);
            id
        })
        .collect();

    Ok(Note { over, text, position })
}

// Arrow patterns:
//   -->>  dashed arrowhead
//   -->   dashed open
//   --x   dashed cross
//   ->>   solid arrowhead
//   ->    solid open
//   -x    solid cross
fn parse_message(
    line: &str,
    participants: &mut Vec<Participant>,
    participant_ids: &mut Vec<String>,
) -> Option<Message> {
    // Find arrow pattern in line
    let arrows = [
        ("-->>", ArrowStyle::Dashed),
        ("--x", ArrowStyle::DashedCross),
        ("-->", ArrowStyle::DashedOpen),
        ("->>", ArrowStyle::Solid),
        ("-x", ArrowStyle::SolidCross),
        ("->", ArrowStyle::SolidOpen),
    ];

    for (pattern, style) in &arrows {
        if let Some(arrow_pos) = line.find(pattern) {
            let from = line[..arrow_pos].trim();
            let after_arrow = &line[arrow_pos + pattern.len()..];

            // After arrow: "Target: text"
            let colon_pos = after_arrow.find(':')?;
            let to = after_arrow[..colon_pos].trim();
            let text = after_arrow[colon_pos + 1..].trim();

            if from.is_empty() || to.is_empty() {
                return None;
            }

            ensure_participant(from, participants, participant_ids);
            ensure_participant(to, participants, participant_ids);

            return Some(Message {
                from: from.to_string(),
                to: to.to_string(),
                text: text.to_string(),
                arrow: *style,
            });
        }
    }
    None
}
