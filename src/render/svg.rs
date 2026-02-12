use crate::layout::{Canvas, Shape, TextAnchor};

pub fn render(canvas: &Canvas) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
        canvas.width, canvas.height, canvas.width, canvas.height
    ));
    out.push('\n');

    // Defs for arrowhead marker
    out.push_str(r##"  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto">
      <polygon points="0 0, 10 3.5, 0 7" fill="#333" />
    </marker>
  </defs>
"##);

    // Embedded CSS
    out.push_str("  <style>\n");
    out.push_str(&canvas.css);
    out.push_str("  </style>\n");

    // Render shapes
    for shape in &canvas.shapes {
        render_shape(shape, &mut out, 1);
    }

    out.push_str("</svg>\n");
    out
}

fn render_shape(shape: &Shape, out: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    match shape {
        Shape::Rect { x, y, w, h, rx, label: _, class } => {
            out.push_str(&format!(
                r#"{}<rect x="{}" y="{}" width="{}" height="{}" rx="{}" class="{}" />"#,
                indent, x, y, w, h, rx, class
            ));
            out.push('\n');
        }
        Shape::Line { x1, y1, x2, y2, dashed, arrow_head, arrow_tail: _, class } => {
            let dash = if *dashed { r#" stroke-dasharray="6,4""# } else { "" };
            let marker = if *arrow_head { r#" marker-end="url(#arrowhead)""# } else { "" };
            out.push_str(&format!(
                r#"{}<line x1="{}" y1="{}" x2="{}" y2="{}" class="{}"{}{} />"#,
                indent, x1, y1, x2, y2, class, dash, marker
            ));
            out.push('\n');
        }
        Shape::Text { x, y, content, anchor, class } => {
            let anchor_str = match anchor {
                TextAnchor::Start => "start",
                TextAnchor::Middle => "middle",
                TextAnchor::End => "end",
            };
            out.push_str(&format!(
                r#"{}<text x="{}" y="{}" text-anchor="{}" class="{}">{}</text>"#,
                indent, x, y, anchor_str, class, escape_xml(content)
            ));
            out.push('\n');
        }
        Shape::Group { shapes, class } => {
            out.push_str(&format!(r#"{}<g class="{}">"#, indent, class));
            out.push('\n');
            for s in shapes {
                render_shape(s, out, depth + 1);
            }
            out.push_str(&format!("{}</g>\n", indent));
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
