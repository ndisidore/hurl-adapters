//! Request translation from KDL to Hurl AST.

use hurl_core::ast::{
    Cookie, KeyValue, Method, MultipartParam, Request, Section, SectionValue,
};
use kdl::KdlNode;

use crate::formats::kdl::body::translate_body;
use crate::formats::kdl::error::{Result, TranslationError};
use crate::formats::kdl::VALID_HTTP_METHODS;
use crate::writer::helpers::{
    dummy_source_info, empty_whitespace, simple_line_terminator, simple_template, space,
    template_with_placeholders,
};

/// Translates a KDL request node to a hurl Request.
///
/// Expected KDL structure:
/// ```kdl
/// GET "https://example.com" name="step" {
///     headers { ... }
///     query { ... }
///     form { ... }
///     cookies { ... }
///     basic-auth { ... }
///     options { ... }
///     body json { ... }
/// }
/// ```
///
/// # Errors
///
/// Returns an error if the request structure is invalid.
pub fn translate_request(node: &KdlNode) -> Result<Request> {
    let method_str = node.name().value();

    // Validate method
    if !VALID_HTTP_METHODS.contains(&method_str) {
        return Err(TranslationError::InvalidMethod(method_str.to_string()));
    }

    // Get URL from first argument
    let url = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| TranslationError::MissingRequiredField {
            node: method_str.to_string(),
            field: "url".to_string(),
        })?;

    // Process children for headers, sections, and body
    let mut headers = Vec::new();
    let mut sections = Vec::new();
    let mut body = None;

    if let Some(children) = node.children() {
        for child in children.nodes() {
            match child.name().value() {
                "headers" => {
                    headers.extend(translate_headers(child)?);
                }
                "query" => {
                    sections.push(translate_query_section(child));
                }
                "form" => {
                    sections.push(translate_form_section(child));
                }
                "multipart" => {
                    sections.push(translate_multipart_section(child));
                }
                "cookies" => {
                    sections.push(translate_cookies_section(child));
                }
                "basic-auth" => {
                    sections.push(translate_basic_auth_section(child));
                }
                "options" => {
                    sections.push(translate_options_section(child));
                }
                "body" => {
                    body = Some(translate_body(child)?);
                }
                "expect" => {
                    // Response expectations are handled separately
                }
                other => {
                    return Err(TranslationError::InvalidStructure(format!(
                        "unknown request section: {other}"
                    )));
                }
            }
        }
    }

    Ok(Request {
        line_terminators: vec![],
        space0: empty_whitespace(),
        method: Method::new(method_str),
        space1: space(),
        url: template_with_placeholders(url),
        line_terminator0: simple_line_terminator(),
        headers,
        sections,
        body,
        source_info: dummy_source_info(),
    })
}

/// Translates headers from a KDL node.
fn translate_headers(node: &KdlNode) -> Result<Vec<KeyValue>> {
    let mut headers = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let key = child.name().value();
            let value = child
                .entries()
                .first()
                .and_then(|e| e.value().as_string())
                .ok_or_else(|| TranslationError::MissingRequiredField {
                    node: format!("headers.{key}"),
                    field: "value".to_string(),
                })?;

            headers.push(KeyValue {
                line_terminators: vec![],
                space0: empty_whitespace(),
                key: simple_template(key),
                space1: empty_whitespace(),
                space2: space(),
                value: template_with_placeholders(value),
                line_terminator0: simple_line_terminator(),
            });
        }
    }

    Ok(headers)
}

/// Translates a query section.
fn translate_query_section(node: &KdlNode) -> Section {
    let params = translate_key_value_children(node);
    Section {
        line_terminators: vec![],
        space0: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        value: SectionValue::QueryParams(params, true), // Use short syntax [Query]
        source_info: dummy_source_info(),
    }
}

/// Translates a form section.
fn translate_form_section(node: &KdlNode) -> Section {
    let params = translate_key_value_children(node);
    Section {
        line_terminators: vec![],
        space0: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        value: SectionValue::FormParams(params, true), // Use short syntax [Form]
        source_info: dummy_source_info(),
    }
}

/// Translates a multipart section.
fn translate_multipart_section(node: &KdlNode) -> Section {
    let mut params = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();

            // Check if it's a file param
            if name == "file" {
                // TODO: Handle file params
                continue;
            }

            // Regular param
            let value = child
                .entries()
                .first()
                .map(|e| kdl_value_to_string(e.value()))
                .unwrap_or_default();

            params.push(MultipartParam::Param(KeyValue {
                line_terminators: vec![],
                space0: empty_whitespace(),
                key: simple_template(name),
                space1: empty_whitespace(),
                space2: space(),
                value: template_with_placeholders(&value),
                line_terminator0: simple_line_terminator(),
            }));
        }
    }

    Section {
        line_terminators: vec![],
        space0: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        value: SectionValue::MultipartFormData(params, true), // Use short syntax [Multipart]
        source_info: dummy_source_info(),
    }
}

/// Translates a cookies section.
fn translate_cookies_section(node: &KdlNode) -> Section {
    let mut cookies = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            let value = child
                .entries()
                .first()
                .map(|e| kdl_value_to_string(e.value()))
                .unwrap_or_default();

            cookies.push(Cookie {
                line_terminators: vec![],
                space0: empty_whitespace(),
                name: simple_template(name),
                space1: empty_whitespace(),
                space2: space(),
                value: template_with_placeholders(&value),
                line_terminator0: simple_line_terminator(),
            });
        }
    }

    Section {
        line_terminators: vec![],
        space0: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        value: SectionValue::Cookies(cookies),
        source_info: dummy_source_info(),
    }
}

/// Translates a basic-auth section.
fn translate_basic_auth_section(node: &KdlNode) -> Section {
    let kv = if let Some(children) = node.children() {
        if let Some(child) = children.nodes().first() {
            let username = child.name().value();
            let password = child
                .entries()
                .first()
                .map(|e| kdl_value_to_string(e.value()))
                .unwrap_or_default();

            Some(KeyValue {
                line_terminators: vec![],
                space0: empty_whitespace(),
                key: simple_template(username),
                space1: empty_whitespace(),
                space2: space(),
                value: template_with_placeholders(&password),
                line_terminator0: simple_line_terminator(),
            })
        } else {
            None
        }
    } else {
        None
    };

    Section {
        line_terminators: vec![],
        space0: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        value: SectionValue::BasicAuth(kv),
        source_info: dummy_source_info(),
    }
}

/// Translates an options section.
fn translate_options_section(_node: &KdlNode) -> Section {
    // TODO: Implement full options translation
    Section {
        line_terminators: vec![],
        space0: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        value: SectionValue::Options(vec![]),
        source_info: dummy_source_info(),
    }
}

/// Translates children nodes to key-value pairs.
fn translate_key_value_children(node: &KdlNode) -> Vec<KeyValue> {
    let mut params = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let key = child.name().value();
            let value = child
                .entries()
                .first()
                .map(|e| kdl_value_to_string(e.value()))
                .unwrap_or_default();

            params.push(KeyValue {
                line_terminators: vec![],
                space0: empty_whitespace(),
                key: simple_template(key),
                space1: empty_whitespace(),
                space2: space(),
                value: template_with_placeholders(&value),
                line_terminator0: simple_line_terminator(),
            });
        }
    }

    params
}

/// Converts a KDL value to a string representation.
fn kdl_value_to_string(value: &kdl::KdlValue) -> String {
    match value {
        kdl::KdlValue::String(s) => s.clone(),
        kdl::KdlValue::Integer(i) => i.to_string(),
        kdl::KdlValue::Float(f) => f.to_string(),
        kdl::KdlValue::Bool(b) => b.to_string(),
        kdl::KdlValue::Null => "null".to_string(),
    }
}

/// Extracts the step name from a request node (from name="..." property).
#[must_use]
pub fn get_step_name(node: &KdlNode) -> Option<String> {
    node.entries()
        .iter()
        .find(|e| e.name().map(kdl::KdlIdentifier::value) == Some("name"))
        .and_then(|e| e.value().as_string())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hurl_core::typing::ToSource;
    use kdl::KdlDocument;

    #[test]
    fn test_simple_get_request() {
        let kdl: KdlDocument = r#"GET "https://example.com""#.parse().unwrap();
        let node = kdl.nodes().first().unwrap();
        let request = translate_request(node).unwrap();

        assert_eq!(request.method.to_string(), "GET");
        assert!(request.url.to_source().as_str().contains("example.com"));
    }

    #[test]
    fn test_post_with_headers() {
        let kdl: KdlDocument = r#"
            POST "https://api.example.com" {
                headers {
                    Content-Type "application/json"
                    Authorization "Bearer {{token}}"
                }
            }
        "#
        .parse()
        .unwrap();

        let node = kdl.nodes().first().unwrap();
        let request = translate_request(node).unwrap();

        assert_eq!(request.method.to_string(), "POST");
        assert_eq!(request.headers.len(), 2);
    }

    #[test]
    fn test_get_step_name() {
        let kdl: KdlDocument = r#"GET "https://example.com" name="login""#.parse().unwrap();
        let node = kdl.nodes().first().unwrap();
        let name = get_step_name(node);

        assert_eq!(name, Some("login".to_string()));
    }
}
