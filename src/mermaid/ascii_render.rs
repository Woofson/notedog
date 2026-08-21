use crate::mermaid::parser::{DiagramDirection, MermaidDiagram, MermaidNode, NodeShape};
use crate::theme::Theme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;

pub fn render_mermaid_to_lines(diagram: &MermaidDiagram, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let title_span = Span::styled(
        match diagram.direction {
            DiagramDirection::TopDown => " 📊 [Flowchart: Top-Down] ",
            DiagramDirection::LeftRight => " 📊 [Flowchart: Left-Right] ",
        },
        Style::default()
            .fg(theme.secondary)
            .add_modifier(Modifier::BOLD),
    );
    lines.push(Line::from(vec![title_span]));
    lines.push(Line::from(""));

    if diagram.nodes.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            " (Empty Diagram)",
            Style::default().fg(Color::DarkGray),
        )]));
        return lines;
    }

    match diagram.direction {
        DiagramDirection::TopDown => render_top_down(diagram, theme, &mut lines),
        DiagramDirection::LeftRight => render_left_right(diagram, theme, &mut lines),
    }

    lines
}

fn render_top_down(diagram: &MermaidDiagram, theme: &Theme, lines: &mut Vec<Line<'static>>) {
    // Build topological ranks for nodes
    let node_map: HashMap<String, &MermaidNode> = diagram.nodes.iter().map(|n| (n.id.clone(), n)).collect();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    for n in &diagram.nodes {
        in_degree.entry(n.id.clone()).or_insert(0);
    }
    for e in &diagram.edges {
        *in_degree.entry(e.to.clone()).or_insert(0) += 1;
    }

    let mut layers: Vec<Vec<&MermaidNode>> = Vec::new();
    let mut visited: HashMap<String, bool> = HashMap::new();

    let mut current_layer: Vec<&MermaidNode> = diagram
        .nodes
        .iter()
        .filter(|n| in_degree.get(&n.id).copied().unwrap_or(0) == 0)
        .collect();

    if current_layer.is_empty() {
        current_layer = diagram.nodes.iter().collect();
    }

    while !current_layer.is_empty() {
        for n in &current_layer {
            visited.insert(n.id.clone(), true);
        }
        layers.push(current_layer.clone());

        let mut next_layer: Vec<&MermaidNode> = Vec::new();
        for n in &current_layer {
            for e in &diagram.edges {
                if e.from == n.id {
                    if let Some(target) = node_map.get(&e.to) {
                        if !visited.contains_key(&target.id) && !next_layer.iter().any(|x| x.id == target.id) {
                            next_layer.push(target);
                        }
                    }
                }
            }
        }
        current_layer = next_layer;
    }

    // Render each layer and connecting arrows
    for (i, layer) in layers.iter().enumerate() {
        render_node_row(layer, theme, lines);

        if i < layers.len() - 1 {
            // Find edges from current layer to next layer
            let mut edge_labels = Vec::new();
            for n in layer {
                for e in &diagram.edges {
                    if e.from == n.id {
                        if let Some(lbl) = &e.label {
                            edge_labels.push(lbl.clone());
                        }
                    }
                }
            }

            let label_str = if !edge_labels.is_empty() {
                format!(" [{}] ", edge_labels.join(", "))
            } else {
                "".to_string()
            };

            lines.push(Line::from(vec![
                Span::styled("   │ ", Style::default().fg(theme.primary)),
                Span::styled(label_str, Style::default().fg(theme.accent)),
            ]));
            lines.push(Line::from(vec![Span::styled(
                "   ▼ ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )]));
        }
    }
}

fn render_left_right(diagram: &MermaidDiagram, theme: &Theme, lines: &mut Vec<Line<'static>>) {
    // Render side-by-side or horizontally connected chain
    let mut spans: Vec<Span<'static>> = Vec::new();

    for (i, node) in diagram.nodes.iter().enumerate() {
        let (box_start, box_end) = match node.shape {
            NodeShape::Rectangle => ("[ ", " ]"),
            NodeShape::Rounded => ("( ", " )"),
            NodeShape::Diamond => ("◇ ", " ◇"),
            NodeShape::Database => ("[( ", " )]"),
        };

        spans.push(Span::styled(box_start, Style::default().fg(theme.border)));
        spans.push(Span::styled(
            node.label.clone(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(box_end, Style::default().fg(theme.border)));

        if i < diagram.nodes.len() - 1 {
            let mut label = None;
            for e in &diagram.edges {
                if e.from == node.id {
                    label = e.label.clone();
                    break;
                }
            }

            let arrow = if let Some(lbl) = label {
                format!(" ──[{}]──► ", lbl)
            } else {
                " ──────► ".to_string()
            };

            spans.push(Span::styled(
                arrow,
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ));
        }
    }

    lines.push(Line::from(spans));
}

fn render_node_row(row: &[&MermaidNode], theme: &Theme, lines: &mut Vec<Line<'static>>) {
    for node in row {
        let (top_border, bot_border, side) = match node.shape {
            NodeShape::Rectangle => ("┌─────────────┐", "└─────────────┘", "│"),
            NodeShape::Rounded => ("╭─────────────╮", "╰─────────────╯", "│"),
            NodeShape::Diamond => (r"/────────────\", r"\────────────/", "│"),
            NodeShape::Database => ("(============)", "(============)", "│"),
        };

        lines.push(Line::from(vec![Span::styled(
            format!("   {}", top_border),
            Style::default().fg(theme.border),
        )]));

        let padded_label = format!("{:^13}", node.label);
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(side, Style::default().fg(theme.border)),
            Span::styled(
                padded_label,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(side, Style::default().fg(theme.border)),
        ]));

        lines.push(Line::from(vec![Span::styled(
            format!("   {}", bot_border),
            Style::default().fg(theme.border),
        )]));
    }
}
