pub mod sequence;

use crate::ast::sequence::SequenceDiagram;

pub enum Diagram {
    Sequence(SequenceDiagram),
}

pub fn parse(input: &str) -> Result<Diagram, String> {
    let trimmed = input.trim();
    if trimmed.starts_with("sequenceDiagram") {
        let body = &trimmed["sequenceDiagram".len()..];
        Ok(Diagram::Sequence(sequence::parse(body)?))
    } else {
        Err(format!("unsupported diagram type"))
    }
}
