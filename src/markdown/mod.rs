pub mod color_parser;
pub mod image_render;
pub mod renderer;

#[allow(unused_imports)]
pub use color_parser::parse_color_tags;
pub use renderer::render_markdown;
