use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramDirection {
    TopDown,
    LeftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeShape {
    Rectangle,
    Rounded,
    Diamond,
    Database,
}

#[derive(Debug, Clone)]
pub struct MermaidNode {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
}

#[derive(Debug, Clone)]
pub struct MermaidEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: String, // "arrow", "line", "dotted", "thick"
}

#[derive(Debug, Clone)]
pub struct MermaidDiagram {
    pub direction: DiagramDirection,
    pub nodes: Vec<MermaidNode>,
    pub edges: Vec<MermaidEdge>,
}

static EDGE_REGEX: OnceLock<Regex> = OnceLock::new();
static NODE_DEF_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_edge_regex() -> &'static Regex {
    EDGE_REGEX.get_or_init(|| {
        Regex::new(r#"(?x)
            ^ \s*
            ([A-Za-z0-9_]+)
            (?:\s* (?:\[|\(|\{\|\(\[) (.*?) (?:\]|\)|\}|\)\]) )?
            \s*
            (-->|---|==>|-\.-|--\s*.*?\s*-->|-->\|.*?\|)
            \s*
            ([A-Za-z0-9_]+)
            (?:\s* (?:\[|\(|\{\|\(\[) (.*?) (?:\]|\)|\}|\)\]) )?
            \s* $
        "#).unwrap()
    })
}

fn get_node_def_regex() -> &'static Regex {
    NODE_DEF_REGEX.get_or_init(|| {
        Regex::new(r#"^([A-Za-z0-9_]+)\s*(\[|\(|\{\|\(\[)(.*?)(\]|\)|\}|\)\])$"#).unwrap()
    })
}

pub fn parse_mermaid(input: &str) -> Option<MermaidDiagram> {
    let mut direction = DiagramDirection::TopDown;
    let mut node_map: HashMap<String, MermaidNode> = HashMap::new();
    let mut node_order: Vec<String> = Vec::new();
    let mut edges: Vec<MermaidEdge> = Vec::new();

    let mut lines = input.lines();
    
    // Find header
    let mut header_found = false;
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }
        if trimmed.starts_with("graph") || trimmed.starts_with("flowchart") {
            header_found = true;
            if trimmed.contains("LR") {
                direction = DiagramDirection::LeftRight;
            } else {
                direction = DiagramDirection::TopDown;
            }
            break;
        }
    }

    if !header_found {
        return None;
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // Try parsing edge first
        if let Some(edge) = parse_line_as_edge(trimmed, &mut node_map, &mut node_order) {
            edges.push(edge);
            continue;
        }

        // Try parsing single node definition
        if let Some(node) = parse_node_def(trimmed) {
            if !node_map.contains_key(&node.id) {
                node_order.push(node.id.clone());
            }
            node_map.insert(node.id.clone(), node);
        }
    }

    let nodes = node_order
        .into_iter()
        .filter_map(|id| node_map.remove(&id))
        .collect();

    Some(MermaidDiagram {
        direction,
        nodes,
        edges,
    })
}

fn parse_node_def(text: &str) -> Option<MermaidNode> {
    let text = text.trim();
    if text.contains('[') && text.contains(']') {
        let id = text.split('[').next()?.trim().to_string();
        let label = text.split('[').nth(1)?.split(']').next()?.trim().to_string();
        return Some(MermaidNode {
            id,
            label,
            shape: NodeShape::Rectangle,
        });
    } else if text.contains('{') && text.contains('}') {
        let id = text.split('{').next()?.trim().to_string();
        let label = text.split('{').nth(1)?.split('}').next()?.trim().to_string();
        return Some(MermaidNode {
            id,
            label,
            shape: NodeShape::Diamond,
        });
    } else if text.contains('(') && text.contains(')') {
        let id = text.split('(').next()?.trim().to_string();
        let label = text.split('(').nth(1)?.split(')').next()?.trim().to_string();
        return Some(MermaidNode {
            id,
            label,
            shape: NodeShape::Rounded,
        });
    } else if !text.contains(' ') && !text.contains('-') && !text.contains('>') {
        return Some(MermaidNode {
            id: text.to_string(),
            label: text.to_string(),
            shape: NodeShape::Rectangle,
        });
    }
    None
}

fn parse_line_as_edge(
    line: &str,
    node_map: &mut HashMap<String, MermaidNode>,
    node_order: &mut Vec<String>,
) -> Option<MermaidEdge> {
    // Splits by edge operator e.g. -->, ---, ==>, -.-
    let ops = ["-->", "---", "==>", "-.-"];
    let mut matched_op = None;
    let mut label = None;

    for op in ops {
        if line.contains(op) {
            matched_op = Some(op);
            break;
        }
    }

    let op = matched_op?;
    let parts: Vec<&str> = line.split(op).collect();
    if parts.len() < 2 {
        return None;
    }

    let mut left_str = parts[0].trim();
    let mut right_str = parts[1].trim();

    // Check if label is in pipe format: A -->|label| B
    if right_str.starts_with('|') {
        if let Some(pipe_end) = right_str[1..].find('|') {
            label = Some(right_str[1..1 + pipe_end].trim().to_string());
            right_str = right_str[1 + pipe_end + 1..].trim();
        }
    }

    let from_node = parse_node_token(left_str, node_map, node_order)?;
    let to_node = parse_node_token(right_str, node_map, node_order)?;

    Some(MermaidEdge {
        from: from_node.id,
        to: to_node.id,
        label,
        style: match op {
            "==>" => "thick".to_string(),
            "-.-" => "dotted".to_string(),
            "---" => "line".to_string(),
            _ => "arrow".to_string(),
        },
    })
}

fn parse_node_token(
    token: &str,
    node_map: &mut HashMap<String, MermaidNode>,
    node_order: &mut Vec<String>,
) -> Option<MermaidNode> {
    if let Some(node) = parse_node_def(token) {
        if !node_map.contains_key(&node.id) {
            node_order.push(node.id.clone());
        }
        node_map.insert(node.id.clone(), node.clone());
        Some(node)
    } else {
        let id = token.trim().to_string();
        if id.is_empty() {
            return None;
        }
        if let Some(existing) = node_map.get(&id) {
            Some(existing.clone())
        } else {
            let node = MermaidNode {
                id: id.clone(),
                label: id.clone(),
                shape: NodeShape::Rectangle,
            };
            node_order.push(id.clone());
            node_map.insert(id, node.clone());
            Some(node)
        }
    }
}
