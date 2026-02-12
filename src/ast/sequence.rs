#[derive(Debug, Clone)]
pub struct SequenceDiagram {
    pub participants: Vec<Participant>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum Event {
    Message(Message),
    Note(Note),
    Block(Block),
    Activate(String),
    Deactivate(String),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub text: String,
    pub arrow: ArrowStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArrowStyle {
    Solid,       // ->>
    SolidOpen,   // ->
    Dashed,      // -->>
    DashedOpen,  // -->
    SolidCross,  // -x
    DashedCross, // --x
}

#[derive(Debug, Clone)]
pub struct Note {
    pub over: Vec<String>,
    pub text: String,
    pub position: NotePosition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotePosition {
    Over,
    LeftOf,
    RightOf,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub kind: BlockKind,
    pub label: String,
    pub sections: Vec<BlockSection>,
}

#[derive(Debug, Clone)]
pub struct BlockSection {
    pub label: Option<String>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockKind {
    Loop,
    Alt,
    Opt,
    Par,
    Critical,
    Break,
}
