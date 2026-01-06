//! Helpers for constructing hurl AST nodes with minimal whitespace.

use hurl_core::ast::{LineTerminator, SourceInfo, Template, TemplateElement, Whitespace};
use hurl_core::reader::Pos;
use hurl_core::typing::ToSource;

/// Creates a dummy source info pointing to line 1, column 1.
#[must_use]
pub fn dummy_source_info() -> SourceInfo {
    SourceInfo {
        start: Pos { line: 1, column: 1 },
        end: Pos { line: 1, column: 1 },
    }
}

/// Creates whitespace with a single space.
#[must_use]
pub fn space() -> Whitespace {
    Whitespace {
        value: " ".to_string(),
        source_info: dummy_source_info(),
    }
}

/// Creates empty whitespace.
#[must_use]
pub fn empty_whitespace() -> Whitespace {
    Whitespace {
        value: String::new(),
        source_info: dummy_source_info(),
    }
}

/// Creates a newline whitespace.
#[must_use]
pub fn newline_whitespace() -> Whitespace {
    Whitespace {
        value: "\n".to_string(),
        source_info: dummy_source_info(),
    }
}

/// Creates a simple line terminator (newline without comment).
#[must_use]
pub fn simple_line_terminator() -> LineTerminator {
    LineTerminator {
        space0: empty_whitespace(),
        comment: None,
        newline: newline_whitespace(),
    }
}

/// Creates a template from a simple string (no placeholders).
#[must_use]
pub fn simple_template(value: &str) -> Template {
    Template {
        delimiter: None,
        elements: vec![TemplateElement::String {
            value: value.to_string(),
            source: value.to_source(),
        }],
        source_info: dummy_source_info(),
    }
}

/// Creates a template that may contain placeholders like `{{variable}}`.
/// This parses the string and converts `{{...}}` into Expression elements.
#[must_use]
#[allow(clippy::indexing_slicing)]
pub fn template_with_placeholders(value: &str) -> Template {
    let mut elements = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = value.chars().collect();

    while current_pos < chars.len() {
        // Look for {{
        if current_pos + 1 < chars.len()
            && chars[current_pos] == '{'
            && chars[current_pos + 1] == '{'
        {
            // Find closing }}
            let start = current_pos + 2;
            let mut end = start;
            while end + 1 < chars.len() && !(chars[end] == '}' && chars[end + 1] == '}') {
                end += 1;
            }

            if end + 1 < chars.len() {
                // Found a placeholder
                let var_name: String = chars[start..end].iter().collect();
                elements.push(TemplateElement::Placeholder(hurl_core::ast::Placeholder {
                    space0: empty_whitespace(),
                    expr: hurl_core::ast::Expr {
                        kind: hurl_core::ast::ExprKind::Variable(hurl_core::ast::Variable {
                            name: var_name,
                            source_info: dummy_source_info(),
                        }),
                        source_info: dummy_source_info(),
                    },
                    space1: empty_whitespace(),
                }));
                current_pos = end + 2;
            } else {
                // No closing }}, treat as literal
                let ch = chars[current_pos].to_string();
                elements.push(TemplateElement::String {
                    source: ch.to_source(),
                    value: ch,
                });
                current_pos += 1;
            }
        } else {
            // Regular character, accumulate into string
            let start = current_pos;
            while current_pos < chars.len()
                && !(current_pos + 1 < chars.len()
                    && chars[current_pos] == '{'
                    && chars[current_pos + 1] == '{')
            {
                current_pos += 1;
            }
            let text: String = chars[start..current_pos].iter().collect();
            if !text.is_empty() {
                elements.push(TemplateElement::String {
                    source: text.to_source(),
                    value: text,
                });
            }
        }
    }

    Template {
        delimiter: None,
        elements,
        source_info: dummy_source_info(),
    }
}

/// Creates a template from a string that may contain `{{placeholder}}` syntax,
/// wrapped in quotes for use in JSON string values.
#[must_use]
pub fn quoted_template_with_placeholders(value: &str) -> Template {
    let mut template = template_with_placeholders(value);
    template.delimiter = Some('"');
    template
}

#[cfg(test)]
mod tests {
    use super::*;
    use hurl_core::typing::ToSource;

    #[test]
    fn test_simple_template() {
        let t = simple_template("hello");
        assert_eq!(t.to_source().as_str(), "hello");
    }

    #[test]
    fn test_template_with_placeholder() {
        let t = template_with_placeholders("Bearer {{token}}");
        assert_eq!(t.to_source().as_str(), "Bearer {{token}}");
    }

    #[test]
    fn test_template_with_multiple_placeholders() {
        let t = template_with_placeholders("{{a}} and {{b}}");
        assert_eq!(t.to_source().as_str(), "{{a}} and {{b}}");
    }

    #[test]
    fn test_quoted_template_with_placeholders() {
        let t = quoted_template_with_placeholders("Hello {{name}}");
        assert_eq!(t.to_source().as_str(), r#""Hello {{name}}""#);
        assert_eq!(t.delimiter, Some('"'));
    }
}
