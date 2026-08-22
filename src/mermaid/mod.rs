pub mod ascii_render;
pub mod parser;

pub use ascii_render::render_mermaid_to_lines;
#[allow(unused_imports)]
pub use parser::{parse_mermaid, MermaidDiagram};
