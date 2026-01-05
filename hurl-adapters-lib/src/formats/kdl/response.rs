//! Response translation from KDL to Hurl AST.

use hurl_core::ast::{
    Assert, Capture, CookiePath, KeyValue, Predicate, PredicateFuncValue, PredicateValue, Query,
    QueryValue, Response, Section, SectionValue, Status, StatusValue, Version, VersionValue,
};
use kdl::{KdlNode, KdlValue};

use crate::formats::kdl::error::{Result, TranslationError};
use crate::writer::helpers::{
    dummy_source_info, empty_whitespace, simple_line_terminator, simple_template, space,
    template_with_placeholders,
};

/// Translates an expect node to a hurl Response.
pub fn translate_response(node: &KdlNode, step_name: Option<&str>) -> Result<Response> {
    let mut status = Status {
        value: StatusValue::Any,
        source_info: dummy_source_info(),
    };
    let mut version = Version {
        value: VersionValue::VersionAny,
        source_info: dummy_source_info(),
    };
    let mut headers = Vec::new();
    let mut sections = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            match child.name().value() {
                "status" => {
                    status = translate_status(child)?;
                }
                "version" => {
                    version = translate_version(child)?;
                }
                "headers" => {
                    headers.extend(translate_response_headers(child)?);
                }
                "captures" => {
                    sections.push(translate_captures_section(child, step_name)?);
                }
                "asserts" => {
                    sections.push(translate_asserts_section(child)?);
                }
                other => {
                    return Err(TranslationError::InvalidStructure(format!(
                        "unknown expect section: {}",
                        other
                    )));
                }
            }
        }
    }

    Ok(Response {
        line_terminators: vec![],
        version,
        space0: space(),
        status,
        space1: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        headers,
        sections,
        body: None,
        source_info: dummy_source_info(),
    })
}

/// Translates a status node.
fn translate_status(node: &KdlNode) -> Result<Status> {
    let value = node
        .entries()
        .first()
        .map(|e| e.value())
        .ok_or_else(|| TranslationError::MissingRequiredField {
            node: "status".to_string(),
            field: "value".to_string(),
        })?;

    let status_value = match value {
        KdlValue::Integer(i) => {
            match (*i).try_into() {
                Ok(u) => StatusValue::Specific(u),
                Err(_) => {
                    return Err(TranslationError::InvalidStructure(format!(
                        "status code must be a non-negative integer between 0 and {}, got: {}",
                        u64::MAX,
                        i
                    )));
                }
            }
        }
        KdlValue::String(s) if s == "*" => StatusValue::Any,
        _ => StatusValue::Any,
    };

    Ok(Status {
        value: status_value,
        source_info: dummy_source_info(),
    })
}

/// Translates a version node.
fn translate_version(node: &KdlNode) -> Result<Version> {
    let value = node
        .entries()
        .first()
        .and_then(|e| e.value().as_string())
        .unwrap_or("*");

    let version_value = match value {
        "HTTP/1.0" => VersionValue::Version1,
        "HTTP/1.1" => VersionValue::Version11,
        "HTTP/2" => VersionValue::Version2,
        "HTTP/3" => VersionValue::Version3,
        "*" | _ => VersionValue::VersionAny,
    };

    Ok(Version {
        value: version_value,
        source_info: dummy_source_info(),
    })
}

/// Translates response headers.
fn translate_response_headers(node: &KdlNode) -> Result<Vec<KeyValue>> {
    let mut headers = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let key = child.name().value();
            let value = child
                .entries()
                .first()
                .map(|e| kdl_value_to_string(e.value()))
                .unwrap_or_default();

            headers.push(KeyValue {
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

    Ok(headers)
}

/// Translates a captures section.
fn translate_captures_section(node: &KdlNode, step_name: Option<&str>) -> Result<Section> {
    let mut captures = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let capture = translate_capture(child, step_name)?;
            captures.push(capture);
        }
    }

    Ok(Section {
        line_terminators: vec![],
        space0: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        value: SectionValue::Captures(captures),
        source_info: dummy_source_info(),
    })
}

/// Translates a single capture.
fn translate_capture(node: &KdlNode, step_name: Option<&str>) -> Result<Capture> {
    let var_name = node.name().value();

    // Prefix variable name with step name if provided
    let full_name = if let Some(step) = step_name {
        format!("{}.{}", step, var_name)
    } else {
        var_name.to_string()
    };

    let entries: Vec<_> = node.entries().iter().collect();
    let query_type = entries
        .first()
        .and_then(|e| e.value().as_string())
        .ok_or_else(|| TranslationError::MissingRequiredField {
            node: format!("capture.{}", var_name),
            field: "query_type".to_string(),
        })?;

    let query = translate_query(query_type, &entries[1..])?;

    Ok(Capture {
        line_terminators: vec![],
        space0: empty_whitespace(),
        name: simple_template(&full_name),
        space1: empty_whitespace(),
        space2: space(),
        query,
        filters: vec![],
        space3: empty_whitespace(),
        redact: false,
        line_terminator0: simple_line_terminator(),
    })
}

/// Translates a query type with arguments.
fn translate_query(query_type: &str, args: &[&kdl::KdlEntry]) -> Result<Query> {
    let query_value = match query_type {
        "jsonpath" => {
            let expr = args
                .first()
                .and_then(|e| e.value().as_string())
                .ok_or_else(|| TranslationError::MissingRequiredField {
                    node: "capture".to_string(),
                    field: "jsonpath expression".to_string(),
                })?;
            QueryValue::Jsonpath {
                space0: space(),
                expr: simple_template(expr),
            }
        }
        "xpath" => {
            let expr = args
                .first()
                .and_then(|e| e.value().as_string())
                .ok_or_else(|| TranslationError::MissingRequiredField {
                    node: "capture".to_string(),
                    field: "xpath expression".to_string(),
                })?;
            QueryValue::Xpath {
                space0: space(),
                expr: simple_template(expr),
            }
        }
        "header" => {
            let name = args
                .first()
                .and_then(|e| e.value().as_string())
                .ok_or_else(|| TranslationError::MissingRequiredField {
                    node: "capture".to_string(),
                    field: "header name".to_string(),
                })?;
            QueryValue::Header {
                space0: space(),
                name: simple_template(name),
            }
        }
        "cookie" => {
            let name = args
                .first()
                .and_then(|e| e.value().as_string())
                .ok_or_else(|| TranslationError::MissingRequiredField {
                    node: "capture".to_string(),
                    field: "cookie name".to_string(),
                })?;
            QueryValue::Cookie {
                space0: space(),
                expr: CookiePath {
                    name: simple_template(name),
                    attribute: None,
                },
            }
        }
        "regex" => {
            let pattern = args
                .first()
                .and_then(|e| e.value().as_string())
                .ok_or_else(|| TranslationError::MissingRequiredField {
                    node: "capture".to_string(),
                    field: "regex pattern".to_string(),
                })?;
            QueryValue::Regex {
                space0: space(),
                value: hurl_core::ast::RegexValue::Template(simple_template(pattern)),
            }
        }
        "body" => QueryValue::Body,
        "status" => QueryValue::Status,
        "url" => QueryValue::Url,
        "duration" => QueryValue::Duration,
        "version" => QueryValue::Version,
        _ => {
            return Err(TranslationError::InvalidStructure(format!(
                "unknown query type: {}",
                query_type
            )))
        }
    };

    Ok(Query {
        source_info: dummy_source_info(),
        value: query_value,
    })
}

/// Translates an asserts section.
fn translate_asserts_section(node: &KdlNode) -> Result<Section> {
    let mut asserts = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let assert = translate_assert(child)?;
            asserts.push(assert);
        }
    }

    Ok(Section {
        line_terminators: vec![],
        space0: empty_whitespace(),
        line_terminator0: simple_line_terminator(),
        value: SectionValue::Asserts(asserts),
        source_info: dummy_source_info(),
    })
}

/// Translates a single assert.
fn translate_assert(node: &KdlNode) -> Result<Assert> {
    let query_type = node.name().value();
    let entries: Vec<_> = node.entries().iter().collect();

    // Find the predicate operator and value
    let (query, predicate) = if query_type == "status" || query_type == "duration" || query_type == "version" {
        let query = translate_query(query_type, &[])?;
        let predicate = translate_predicate_from_entries(&entries)?;
        (query, predicate)
    } else {
        let query_args = if entries.is_empty() { &[] } else { &entries[0..1] };
        let predicate_entries = if entries.len() > 1 { &entries[1..] } else { &[] };

        let query = translate_query(query_type, query_args)?;
        let predicate = translate_predicate_from_entries(predicate_entries)?;
        (query, predicate)
    };

    Ok(Assert {
        line_terminators: vec![],
        space0: empty_whitespace(),
        query,
        filters: vec![],
        space1: space(),
        predicate,
        line_terminator0: simple_line_terminator(),
    })
}

/// Translates predicate from KDL entries.
fn translate_predicate_from_entries(entries: &[&kdl::KdlEntry]) -> Result<Predicate> {
    let mut not = false;
    let mut op: Option<String> = None;
    let mut value: Option<&KdlValue> = None;

    for entry in entries {
        if let Some(name) = entry.name() {
            let name_str = name.value();
            if name_str == "not" {
                not = true;
            } else {
                op = Some(name_str.to_string());
                value = Some(entry.value());
            }
        } else {
            if let Some(s) = entry.value().as_string() {
                if is_operator(s) || is_predicate_name(s) {
                    op = Some(s.to_string());
                } else {
                    value = Some(entry.value());
                }
            } else {
                value = Some(entry.value());
            }
        }
    }

    let predicate_func = if let Some(operator) = op {
        translate_predicate_func(&operator, value)?
    } else {
        PredicateFuncValue::Exist
    };

    Ok(Predicate {
        not,
        space0: empty_whitespace(),
        predicate_func: hurl_core::ast::PredicateFunc {
            source_info: dummy_source_info(),
            value: predicate_func,
        },
    })
}

fn is_operator(s: &str) -> bool {
    matches!(
        s,
        "==" | "!=" | ">" | ">=" | "<" | "<=" | "startsWith" | "endsWith" | "contains" | "matches"
    )
}

fn is_predicate_name(s: &str) -> bool {
    matches!(
        s,
        "exists"
            | "isBoolean"
            | "isEmpty"
            | "isFloat"
            | "isInteger"
            | "isNumber"
            | "isString"
            | "isCollection"
            | "isDate"
            | "isIsoDate"
    )
}

fn translate_predicate_func(op: &str, value: Option<&KdlValue>) -> Result<PredicateFuncValue> {
    match op {
        "==" | "equals" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("equals requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::Equal {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        "!=" | "notEquals" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("notEquals requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::NotEqual {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        ">" | "greaterThan" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("greaterThan requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::GreaterThan {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        ">=" | "greaterThanOrEquals" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("greaterThanOrEquals requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::GreaterThanOrEqual {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        "<" | "lessThan" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("lessThan requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::LessThan {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        "<=" | "lessThanOrEquals" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("lessThanOrEquals requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::LessThanOrEqual {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        "startsWith" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("startsWith requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::StartWith {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        "endsWith" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("endsWith requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::EndWith {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        "contains" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("contains requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::Contain {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        "matches" => {
            let val = value.ok_or_else(|| {
                TranslationError::InvalidPredicate("matches requires a value".to_string())
            })?;
            Ok(PredicateFuncValue::Match {
                space0: space(),
                value: kdl_to_predicate_value(val)?,
            })
        }
        "exists" => Ok(PredicateFuncValue::Exist),
        "isBoolean" => Ok(PredicateFuncValue::IsBoolean),
        "isEmpty" => Ok(PredicateFuncValue::IsEmpty),
        "isFloat" => Ok(PredicateFuncValue::IsFloat),
        "isInteger" => Ok(PredicateFuncValue::IsInteger),
        "isNumber" => Ok(PredicateFuncValue::IsNumber),
        "isString" => Ok(PredicateFuncValue::IsString),
        "isCollection" => Ok(PredicateFuncValue::IsCollection),
        "isDate" => Ok(PredicateFuncValue::IsDate),
        "isIsoDate" => Ok(PredicateFuncValue::IsIsoDate),
        _ => Err(TranslationError::InvalidPredicate(format!(
            "unknown predicate: {}",
            op
        ))),
    }
}

fn kdl_to_predicate_value(value: &KdlValue) -> Result<PredicateValue> {
    use hurl_core::typing::ToSource;
    match value {
        KdlValue::String(s) => {
            if s.contains("{{") {
                Ok(PredicateValue::String(template_with_placeholders(s)))
            } else {
                Ok(PredicateValue::String(simple_template(s)))
            }
        }
        KdlValue::Integer(i) => {
            let i64_val = i64::try_from(*i).map_err(|_| {
                TranslationError::InvalidPredicate(format!(
                    "integer value {} is outside the valid range for i64 ({} to {})",
                    i,
                    i64::MIN,
                    i64::MAX
                ))
            })?;
            Ok(PredicateValue::Number(hurl_core::ast::Number::Integer(
                hurl_core::ast::I64::new(i64_val, i.to_string().to_source()),
            )))
        }
        KdlValue::Float(f) => Ok(PredicateValue::Number(hurl_core::ast::Number::Float(
            hurl_core::ast::Float::new(*f, f.to_string().to_source()),
        ))),
        KdlValue::Bool(b) => Ok(PredicateValue::Bool(*b)),
        KdlValue::Null => Ok(PredicateValue::Null),
    }
}

fn kdl_value_to_string(value: &KdlValue) -> String {
    match value {
        KdlValue::String(s) => s.clone(),
        KdlValue::Integer(i) => i.to_string(),
        KdlValue::Float(f) => f.to_string(),
        KdlValue::Bool(b) => b.to_string(),
        KdlValue::Null => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kdl::KdlDocument;

    #[test]
    fn test_simple_expect() {
        let kdl: KdlDocument = r#"
            expect {
                status 200
            }
        "#
        .parse()
        .unwrap();

        let node = kdl.nodes().first().unwrap();
        let response = translate_response(node, None).unwrap();

        assert!(matches!(response.status.value, StatusValue::Specific(200)));
    }

    #[test]
    fn test_captures_with_step_name() {
        let kdl: KdlDocument = r#"
            expect {
                status 200
                captures {
                    token jsonpath "$.token"
                }
            }
        "#
        .parse()
        .unwrap();

        let node = kdl.nodes().first().unwrap();
        let response = translate_response(node, Some("login")).unwrap();

        let captures_section = response
            .sections
            .iter()
            .find(|s| matches!(s.value, SectionValue::Captures(_)));
        assert!(captures_section.is_some());
    }
}
