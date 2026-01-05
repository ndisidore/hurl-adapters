//! Main translator module for KDL to Hurl conversion.

use std::collections::HashSet;

use hurl_core::ast::{Entry, HurlFile};
use kdl::{KdlDocument, KdlNode};

use crate::formats::kdl::error::{Result, TranslationError};
use crate::formats::kdl::request::{get_step_name, translate_request};
use crate::formats::kdl::response::translate_response;
use crate::formats::kdl::VALID_HTTP_METHODS;
use crate::writer::hurl_file_to_string;

/// Translates a KDL document to a Hurl file AST.
pub fn translate(doc: &KdlDocument) -> Result<HurlFile> {
    let mut entries = Vec::new();
    let mut step_names = HashSet::new();

    for node in doc.nodes() {
        let node_name = node.name().value();

        if VALID_HTTP_METHODS.contains(&node_name) {
            let entry = translate_entry(node, &mut step_names)?;
            entries.push(entry);
        }
    }

    Ok(HurlFile {
        entries,
        line_terminators: vec![],
    })
}

/// Translates a KDL document to a Hurl format string.
pub fn translate_to_string(doc: &KdlDocument) -> Result<String> {
    let hurl_file = translate(doc)?;
    Ok(hurl_file_to_string(&hurl_file))
}

/// Translates a single KDL entry (request + optional response).
fn translate_entry(node: &KdlNode, step_names: &mut HashSet<String>) -> Result<Entry> {
    let step_name = get_step_name(node);

    if let Some(ref name) = step_name {
        if step_names.contains(name) {
            return Err(TranslationError::DuplicateStepName(name.clone()));
        }
        step_names.insert(name.clone());
    }

    let request = translate_request(node)?;

    let response = if let Some(children) = node.children() {
        children
            .nodes()
            .iter()
            .find(|n| n.name().value() == "expect")
            .map(|n| translate_response(n, step_name.as_deref()))
            .transpose()?
    } else {
        None
    };

    Ok(Entry { request, response })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_get() {
        let kdl: KdlDocument = r#"GET "https://example.com""#.parse().unwrap();
        let hurl = translate_to_string(&kdl).unwrap();
        assert!(hurl.contains("GET https://example.com"));
    }

    #[test]
    fn test_get_with_expect() {
        let kdl: KdlDocument = r#"
            GET "https://example.com" {
                expect {
                    status 200
                }
            }
        "#
        .parse()
        .unwrap();

        let hurl = translate_to_string(&kdl).unwrap();
        assert!(hurl.contains("GET https://example.com"));
        assert!(hurl.contains("HTTP 200"));
    }

    #[test]
    fn test_post_with_headers_and_body() {
        let kdl: KdlDocument = r#"
            POST "https://api.example.com/users" {
                headers {
                    Content-Type "application/json"
                }
                body json {
                    name "John"
                }
                expect {
                    status 201
                }
            }
        "#
        .parse()
        .unwrap();

        let hurl = translate_to_string(&kdl).unwrap();
        assert!(hurl.contains("POST https://api.example.com/users"));
        assert!(hurl.contains("Content-Type: application/json"));
        assert!(hurl.contains("HTTP 201"));
    }

    #[test]
    fn test_chained_requests() {
        let kdl: KdlDocument = r#"
            POST "https://api.example.com/login" name="login" {
                body json {
                    username "test"
                    password "secret"
                }
                expect {
                    status 200
                    captures {
                        token jsonpath "$.token"
                    }
                }
            }

            GET "https://api.example.com/profile" {
                headers {
                    Authorization "Bearer {{login.token}}"
                }
                expect {
                    status 200
                }
            }
        "#
        .parse()
        .unwrap();

        let hurl = translate_to_string(&kdl).unwrap();
        assert!(hurl.contains("POST https://api.example.com/login"));
        assert!(hurl.contains("login.token"));
        assert!(hurl.contains("Bearer {{login.token}}"));
    }

    #[test]
    fn test_duplicate_step_name_error() {
        let kdl: KdlDocument = r#"
            GET "https://example.com/1" name="step1" {
                expect { status 200 }
            }
            GET "https://example.com/2" name="step1" {
                expect { status 200 }
            }
        "#
        .parse()
        .unwrap();

        let result = translate(&kdl);
        assert!(matches!(result, Err(TranslationError::DuplicateStepName(_))));
    }
}
