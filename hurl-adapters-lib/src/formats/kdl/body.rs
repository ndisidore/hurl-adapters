//! Body handling for KDL to Hurl translation.

use hurl_core::ast::{
    Base64, Body, Bytes, File, GraphQl, GraphQlVariables, Hex, JsonListElement, JsonObjectElement,
    JsonValue, MultilineString, MultilineStringKind, Template,
};
use hurl_core::typing::ToSource;
use kdl::{KdlNode, KdlValue};

use crate::formats::kdl::error::{Result, TranslationError};
use crate::writer::helpers::{
    dummy_source_info, empty_whitespace, newline_whitespace, quoted_template_with_placeholders,
    simple_line_terminator, space, template_with_placeholders,
};

/// Translates a KDL body node to a hurl Body.
///
/// # Errors
///
/// Returns an error if the body type is invalid or missing.
pub fn translate_body(node: &KdlNode) -> Result<Body> {
    let body_type = node
        .entries()
        .iter()
        .find_map(|e| e.name().map(kdl::KdlIdentifier::value))
        .or_else(|| node.entries().first().and_then(|e| e.value().as_string()))
        .ok_or_else(|| TranslationError::InvalidBody {
            reason: "body node must specify type (json, xml, text, file, base64, hex)".to_string(),
        })?;

    let bytes = match body_type {
        "json" => translate_json_body(node)?,
        "xml" => translate_xml_body(node)?,
        "graphql" => translate_graphql_body(node)?,
        "text" => translate_text_body(node)?,
        "file" => translate_file_body(node)?,
        "base64" => translate_base64_body(node)?,
        "hex" => translate_hex_body(node)?,
        other => {
            return Err(TranslationError::InvalidBody {
                reason: format!("unknown body type: {other}"),
            });
        }
    };

    Ok(Body {
        line_terminators: vec![],
        space0: empty_whitespace(),
        value: bytes,
        line_terminator0: simple_line_terminator(),
    })
}

/// Translates a JSON body from KDL children to hurl `JsonValue`.
fn translate_json_body(node: &KdlNode) -> Result<Bytes> {
    let children = node
        .children()
        .ok_or_else(|| TranslationError::InvalidBody {
            reason: "json body requires children nodes".to_string(),
        })?;

    let json = kdl_to_json_object(children.nodes())?;
    Ok(Bytes::Json(json))
}

/// Converts KDL nodes to a JSON object.
fn kdl_to_json_object(nodes: &[KdlNode]) -> Result<JsonValue> {
    let mut elements = Vec::new();
    let count = nodes.len();

    for (i, node) in nodes.iter().enumerate() {
        let key = node.name().value();
        let value = kdl_node_to_json_value(node)?;
        let is_last = i == count - 1;

        elements.push(JsonObjectElement {
            space0: "\n    ".to_string(),
            name: quoted_json_template(key),
            space1: String::new(),
            space2: " ".to_string(),
            value,
            // Add newline after last element for proper closing brace placement
            space3: if is_last {
                "\n".to_string()
            } else {
                String::new()
            },
        });
    }

    Ok(JsonValue::Object {
        space0: String::new(),
        elements,
    })
}

/// Converts a single KDL node to a JSON value.
fn kdl_node_to_json_value(node: &KdlNode) -> Result<JsonValue> {
    // If node has children, it's an object
    if let Some(children) = node.children() {
        return kdl_to_json_object(children.nodes());
    }

    // Check for array syntax: node [val1, val2, ...]
    let entries: Vec<_> = node.entries().iter().collect();

    // If first entry is an array-like structure (multiple values without names)
    if entries.len() > 1 && entries.iter().all(|e| e.name().is_none()) {
        let elements: Vec<_> = entries
            .iter()
            .map(|e| {
                let val = kdl_value_to_json(e.value());
                JsonListElement {
                    space0: String::new(),
                    value: val,
                    space1: String::new(),
                }
            })
            .collect();
        return Ok(JsonValue::List {
            space0: String::new(),
            elements,
        });
    }

    // Single value
    if let Some(entry) = entries.first() {
        return Ok(kdl_value_to_json(entry.value()));
    }

    // No value means null
    Ok(JsonValue::Null)
}

/// Converts a KDL value to a JSON value.
fn kdl_value_to_json(value: &KdlValue) -> JsonValue {
    match value {
        KdlValue::String(s) => {
            // Check if it contains placeholders
            if s.contains("{{") {
                JsonValue::String(quoted_template_with_placeholders(s))
            } else {
                JsonValue::String(quoted_json_template(s))
            }
        }
        KdlValue::Integer(i) => JsonValue::Number(i.to_string()),
        KdlValue::Float(f) => JsonValue::Number(f.to_string()),
        KdlValue::Bool(b) => JsonValue::Boolean(*b),
        KdlValue::Null => JsonValue::Null,
    }
}

/// Creates a quoted template for JSON string values.
fn quoted_json_template(value: &str) -> Template {
    Template {
        delimiter: Some('"'),
        elements: vec![hurl_core::ast::TemplateElement::String {
            value: value.to_string(),
            source: value.to_source(),
        }],
        source_info: dummy_source_info(),
    }
}

/// Translates an XML body.
fn translate_xml_body(node: &KdlNode) -> Result<Bytes> {
    let xml_content = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| TranslationError::InvalidBody {
            reason: "xml body requires string content".to_string(),
        })?;

    Ok(Bytes::MultilineString(MultilineString {
        attributes: vec![],
        space: empty_whitespace(),
        newline: newline_whitespace(),
        kind: MultilineStringKind::Xml(template_with_placeholders(xml_content)),
    }))
}

/// Translates a GraphQL body.
fn translate_graphql_body(node: &KdlNode) -> Result<Bytes> {
    let query = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| TranslationError::InvalidBody {
            reason: "graphql body requires query string".to_string(),
        })?;

    // Check for variables in children
    let variables = if let Some(children) = node.children() {
        let vars_node = children
            .nodes()
            .iter()
            .find(|n| n.name().value() == "variables");
        if let Some(vars) = vars_node {
            if let Some(vars_children) = vars.children() {
                Some(GraphQlVariables {
                    space: space(),
                    value: kdl_to_json_object(vars_children.nodes())?,
                    whitespace: newline_whitespace(),
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(Bytes::MultilineString(MultilineString {
        attributes: vec![],
        space: empty_whitespace(),
        newline: newline_whitespace(),
        kind: MultilineStringKind::GraphQl(GraphQl {
            value: template_with_placeholders(query),
            variables,
        }),
    }))
}

/// Translates a text body.
fn translate_text_body(node: &KdlNode) -> Result<Bytes> {
    let text = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| TranslationError::InvalidBody {
            reason: "text body requires string content".to_string(),
        })?;

    Ok(Bytes::MultilineString(MultilineString {
        attributes: vec![],
        space: empty_whitespace(),
        newline: newline_whitespace(),
        kind: MultilineStringKind::Text(template_with_placeholders(text)),
    }))
}

/// Translates a file body.
fn translate_file_body(node: &KdlNode) -> Result<Bytes> {
    let filename = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| TranslationError::InvalidBody {
            reason: "file body requires filename".to_string(),
        })?;

    Ok(Bytes::File(File {
        space0: empty_whitespace(),
        filename: simple_template(filename),
        space1: empty_whitespace(),
    }))
}

/// Translates a base64 body.
fn translate_base64_body(node: &KdlNode) -> Result<Bytes> {
    let encoded = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| TranslationError::InvalidBody {
            reason: "base64 body requires encoded string".to_string(),
        })?;

    Ok(Bytes::Base64(Base64 {
        space0: empty_whitespace(),
        value: encoded.as_bytes().to_vec(),
        source: encoded.to_source(),
        space1: empty_whitespace(),
    }))
}

/// Translates a hex body.
fn translate_hex_body(node: &KdlNode) -> Result<Bytes> {
    let hex_string = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| TranslationError::InvalidBody {
            reason: "hex body requires hex string".to_string(),
        })?;

    // Validate hex string has even length
    if hex_string.len() % 2 != 0 {
        return Err(TranslationError::InvalidHex {
            reason: format!(
                "hex string has odd length ({}), must be even",
                hex_string.len()
            ),
        });
    }

    // Decode hex to bytes with proper error handling
    let bytes: Result<Vec<u8>> = hex_string
        .as_bytes()
        .chunks(2)
        .enumerate()
        .map(|(idx, chunk)| {
            let s = std::str::from_utf8(chunk).map_err(|e| TranslationError::InvalidHex {
                reason: format!("invalid UTF-8 in hex string at position {}: {e}", idx * 2),
            })?;

            u8::from_str_radix(s, 16).map_err(|e| TranslationError::InvalidHex {
                reason: format!(
                    "invalid hex character(s) '{s}' at position {}: {e}",
                    idx * 2
                ),
            })
        })
        .collect();

    Ok(Bytes::Hex(Hex {
        space0: empty_whitespace(),
        value: bytes?,
        source: hex_string.to_source(),
        space1: empty_whitespace(),
    }))
}

fn simple_template(value: &str) -> Template {
    crate::writer::helpers::simple_template(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kdl::KdlDocument;

    #[test]
    fn test_json_body() {
        let kdl: KdlDocument = r#"body json { username "test"; count 42 }"#.parse().unwrap();
        let node = kdl.nodes().first().unwrap();
        let body = translate_body(node).unwrap();

        // Check that we got a JSON body
        if let hurl_core::ast::Bytes::Json(json) = body.value {
            let source = json.to_source();
            assert!(source.as_str().contains("username"));
            assert!(source.as_str().contains("test"));
        } else {
            panic!("Expected JSON body");
        }
    }

    #[test]
    fn test_json_body_with_placeholder() {
        let kdl: KdlDocument = r#"body json { password "{{PASSWORD}}" }"#.parse().unwrap();
        let node = kdl.nodes().first().unwrap();
        let body = translate_body(node).unwrap();

        if let hurl_core::ast::Bytes::Json(json) = body.value {
            let source = json.to_source();
            // Placeholder values must be quoted in JSON output
            assert!(
                source.as_str().contains(r#""{{PASSWORD}}""#),
                "Expected quoted placeholder, got: {}",
                source.as_str()
            );
        } else {
            panic!("Expected JSON body");
        }
    }
}
