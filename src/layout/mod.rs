pub mod sequence;

#[derive(Debug, Clone)]
pub struct Canvas {
    pub width: f64,
    pub height: f64,
    pub shapes: Vec<Shape>,
    pub css: String,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        rx: f64,
        label: String,
        class: String,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        dashed: bool,
        arrow_head: bool,
        arrow_tail: bool,
        class: String,
    },
    Text {
        x: f64,
        y: f64,
        content: String,
        anchor: TextAnchor,
        class: String,
    },
    Group {
        shapes: Vec<Shape>,
        class: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}
