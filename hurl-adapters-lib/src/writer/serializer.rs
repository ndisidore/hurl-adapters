//! Hurl AST to string serialization.

use std::fmt::Write;

use hurl_core::ast::{
    Bytes, HurlFile, MultipartParam, Predicate, PredicateFuncValue, PredicateValue, Query,
    QueryValue, RegexValue, Response, Section, SectionValue, StatusValue, VersionValue,
};
use hurl_core::typing::ToSource;

/// Converts a `HurlFile` AST to a string.
#[must_use]
pub fn hurl_file_to_string(hurl_file: &HurlFile) -> String {
    let mut output = String::new();

    for entry in &hurl_file.entries {
        // Request line
        output.push_str(&entry.request.method.to_string());
        output.push(' ');
        output.push_str(entry.request.url.to_source().as_str());
        output.push('\n');

        // Headers
        for header in &entry.request.headers {
            output.push_str(header.key.to_source().as_str());
            output.push(':');
            output.push(' ');
            output.push_str(header.value.to_source().as_str());
            output.push('\n');
        }

        // Sections
        for section in &entry.request.sections {
            output.push_str(&format_section(section));
        }

        // Body
        if let Some(body) = &entry.request.body {
            output.push_str(&format_bytes(&body.value));
            output.push('\n');
        }

        // Response
        if let Some(response) = &entry.response {
            output.push_str(&format_response(response));
        }

        output.push('\n');
    }

    output
}

/// Formats a body bytes to hurl string.
#[must_use]
pub fn format_bytes(bytes: &Bytes) -> String {
    match bytes {
        Bytes::Json(json) => json.to_source().to_string(),
        Bytes::Xml(xml) => xml.clone(),
        Bytes::MultilineString(m) => m.to_source().to_string(),
        Bytes::OnelineString(s) => format!("`{}`", s.to_source()),
        Bytes::Base64(b) => format!("base64,{};", b.source),
        Bytes::Hex(h) => format!("hex,{};", h.source),
        Bytes::File(f) => format!("file,{};", f.filename.to_source()),
    }
}

/// Formats a section to hurl string.
#[must_use]
pub fn format_section(section: &Section) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "[{}]", section.identifier());

    match &section.value {
        SectionValue::QueryParams(params, _) | SectionValue::FormParams(params, _) => {
            for param in params {
                output.push_str(param.key.to_source().as_str());
                output.push(':');
                output.push(' ');
                output.push_str(param.value.to_source().as_str());
                output.push('\n');
            }
        }
        SectionValue::Cookies(cookies) => {
            for cookie in cookies {
                output.push_str(cookie.name.to_source().as_str());
                output.push(':');
                output.push(' ');
                output.push_str(cookie.value.to_source().as_str());
                output.push('\n');
            }
        }
        SectionValue::Captures(captures) => {
            for capture in captures {
                output.push_str(capture.name.to_source().as_str());
                output.push(':');
                output.push(' ');
                output.push_str(&format_query(&capture.query));
                output.push('\n');
            }
        }
        SectionValue::Asserts(asserts) => {
            for assert in asserts {
                output.push_str(&format_query(&assert.query));
                output.push(' ');
                output.push_str(&format_predicate(&assert.predicate));
                output.push('\n');
            }
        }
        SectionValue::BasicAuth(kv) => {
            if let Some(kv) = kv {
                output.push_str(kv.key.to_source().as_str());
                output.push(':');
                output.push(' ');
                output.push_str(kv.value.to_source().as_str());
                output.push('\n');
            }
        }
        SectionValue::MultipartFormData(params, _) => {
            for param in params {
                match param {
                    MultipartParam::Param(kv) => {
                        output.push_str(kv.key.to_source().as_str());
                        output.push(':');
                        output.push(' ');
                        output.push_str(kv.value.to_source().as_str());
                        output.push('\n');
                    }
                    MultipartParam::FileParam(fp) => {
                        output.push_str(fp.key.to_source().as_str());
                        output.push(':');
                        output.push_str(" file,");
                        output.push_str(fp.value.filename.to_source().as_str());
                        output.push_str(";\n");
                    }
                }
            }
        }
        SectionValue::Options(options) => {
            // TODO: Implement proper options formatting matching the expected output schema.
            // SectionValue::Options serialization is not yet implemented in format_section().
            // Options are currently dropped during serialization. Future work should:
            // 1. Determine the expected Hurl format for options sections
            // 2. Format each option in the options vector appropriately
            // 3. Add the formatted options to the output string
            eprintln!(
                "Warning: SectionValue::Options serialization not implemented in format_section(); \
                 {} option(s) dropped",
                options.len()
            );
        }
    }

    output
}

/// Formats a query to hurl string.
#[must_use]
pub fn format_query(query: &Query) -> String {
    match &query.value {
        QueryValue::Status => "status".to_string(),
        QueryValue::Version => "version".to_string(),
        QueryValue::Url => "url".to_string(),
        QueryValue::Header { name, .. } => {
            format!("header \"{}\"", name.to_source())
        }
        QueryValue::Cookie { expr, .. } => {
            format!("cookie \"{}\"", expr.name.to_source())
        }
        QueryValue::Body => "body".to_string(),
        QueryValue::Xpath { expr, .. } => {
            format!("xpath \"{}\"", expr.to_source())
        }
        QueryValue::Jsonpath { expr, .. } => {
            format!("jsonpath \"{}\"", expr.to_source())
        }
        QueryValue::Regex { value, .. } => {
            let pattern = match value {
                RegexValue::Template(t) => t.to_source().to_string(),
                RegexValue::Regex(r) => r.inner.to_string(),
            };
            format!("regex \"{pattern}\"")
        }
        QueryValue::Variable { name, .. } => {
            format!("variable \"{}\"", name.to_source())
        }
        QueryValue::Duration => "duration".to_string(),
        QueryValue::Bytes => "bytes".to_string(),
        QueryValue::Sha256 => "sha256".to_string(),
        QueryValue::Md5 => "md5".to_string(),
        QueryValue::Certificate { .. } => "certificate".to_string(),
        QueryValue::Ip => "ip".to_string(),
    }
}

/// Formats a predicate to hurl string.
#[must_use]
pub fn format_predicate(predicate: &Predicate) -> String {
    let mut output = String::new();

    if predicate.not {
        output.push_str("not ");
    }

    let pred_str = match &predicate.predicate_func.value {
        PredicateFuncValue::Equal { value, .. } => {
            format!("== {}", format_predicate_value(value))
        }
        PredicateFuncValue::NotEqual { value, .. } => {
            format!("!= {}", format_predicate_value(value))
        }
        PredicateFuncValue::GreaterThan { value, .. } => {
            format!("> {}", format_predicate_value(value))
        }
        PredicateFuncValue::GreaterThanOrEqual { value, .. } => {
            format!(">= {}", format_predicate_value(value))
        }
        PredicateFuncValue::LessThan { value, .. } => {
            format!("< {}", format_predicate_value(value))
        }
        PredicateFuncValue::LessThanOrEqual { value, .. } => {
            format!("<= {}", format_predicate_value(value))
        }
        PredicateFuncValue::StartWith { value, .. } => {
            format!("startsWith {}", format_predicate_value(value))
        }
        PredicateFuncValue::EndWith { value, .. } => {
            format!("endsWith {}", format_predicate_value(value))
        }
        PredicateFuncValue::Contain { value, .. } => {
            format!("contains {}", format_predicate_value(value))
        }
        PredicateFuncValue::Include { value, .. } => {
            format!("includes {}", format_predicate_value(value))
        }
        PredicateFuncValue::Match { value, .. } => {
            format!("matches {}", format_predicate_value(value))
        }
        PredicateFuncValue::Exist => "exists".to_string(),
        PredicateFuncValue::IsBoolean => "isBoolean".to_string(),
        PredicateFuncValue::IsCollection => "isCollection".to_string(),
        PredicateFuncValue::IsDate => "isDate".to_string(),
        PredicateFuncValue::IsEmpty => "isEmpty".to_string(),
        PredicateFuncValue::IsFloat => "isFloat".to_string(),
        PredicateFuncValue::IsInteger => "isInteger".to_string(),
        PredicateFuncValue::IsIsoDate => "isIsoDate".to_string(),
        PredicateFuncValue::IsNumber => "isNumber".to_string(),
        PredicateFuncValue::IsString => "isString".to_string(),
        PredicateFuncValue::IsIpv4 => "isIpv4".to_string(),
        PredicateFuncValue::IsIpv6 => "isIpv6".to_string(),
    };

    output.push_str(&pred_str);
    output
}

/// Formats a predicate value to hurl string.
#[must_use]
pub fn format_predicate_value(value: &PredicateValue) -> String {
    match value {
        PredicateValue::String(t) => {
            let s = t.to_source().to_string();
            if s.starts_with("{{") {
                s
            } else {
                format!("\"{s}\"")
            }
        }
        PredicateValue::Number(n) => n.to_string(),
        PredicateValue::Bool(b) => b.to_string(),
        PredicateValue::Null => "null".to_string(),
        PredicateValue::Hex(h) => format!("hex,{};", h.source),
        PredicateValue::Base64(b) => format!("base64,{};", b.source),
        PredicateValue::Placeholder(p) => format!("{{{{{}}}}}", p.expr),
        PredicateValue::Regex(r) => format!("/{}/", r.inner),
        PredicateValue::MultilineString(m) => m.to_source().to_string(),
        PredicateValue::File(f) => {
            format!("file,{};", f.filename.to_source())
        }
    }
}

/// Formats a response to hurl string.
#[must_use]
pub fn format_response(response: &Response) -> String {
    let mut output = String::new();

    let version = match response.version.value {
        VersionValue::Version1 => "HTTP/1.0",
        VersionValue::Version11 => "HTTP/1.1",
        VersionValue::Version2 => "HTTP/2",
        VersionValue::Version3 => "HTTP/3",
        VersionValue::VersionAny => "HTTP",
    };

    let status = match response.status.value {
        StatusValue::Any => "*".to_string(),
        StatusValue::Specific(code) => code.to_string(),
    };

    let _ = writeln!(output, "{version} {status}");

    for header in &response.headers {
        output.push_str(header.key.to_source().as_str());
        output.push(':');
        output.push(' ');
        output.push_str(header.value.to_source().as_str());
        output.push('\n');
    }

    for section in &response.sections {
        output.push_str(&format_section(section));
    }

    output
}
