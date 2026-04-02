use serde_json::{json, Value};

pub struct ToolParamsFix {
    pub params: Value,
    pub warning: Option<String>,
}

/// Schema-aware tool parameters completion for truncated JSON
/// Returns the fixed params along with a warning message if any fields were auto-filled
pub fn complete_tool_params(tool_name: &str, json_str: &str) -> Option<ToolParamsFix> {
    let trimmed = json_str.trim();
    if trimmed.is_empty() {
        return None;
    }

    let fixed = complete_json_braces(trimmed)?;
    let obj = serde_json::from_str::<serde_json::Map<String, Value>>(&fixed).ok()?;

    let mut warnings = Vec::new();

    match tool_name {
        "write" => {
            let mut result = obj;
            if !result.contains_key("path") {
                result.insert("path".to_string(), json!(""));
                warnings.push("`path`");
            }
            if !result.contains_key("content") {
                result.insert("content".to_string(), json!(""));
                warnings.push("`content`");
            }
            let warning = if warnings.is_empty() {
                None
            } else {
                Some(format!(
                    "Tool '{}' arguments were truncated. Auto-filled missing required fields: {}",
                    tool_name,
                    warnings.join(", ")
                ))
            };
            Some(ToolParamsFix {
                params: Value::Object(result),
                warning,
            })
        }
        "edit" => {
            let mut result = obj;
            if !result.contains_key("path") {
                result.insert("path".to_string(), json!(""));
                warnings.push("`path`");
            }
            if !result.contains_key("before") {
                result.insert("before".to_string(), json!(""));
                warnings.push("`before`");
            }
            if !result.contains_key("after") {
                result.insert("after".to_string(), json!(""));
                warnings.push("`after`");
            }
            let warning = if warnings.is_empty() {
                None
            } else {
                Some(format!(
                    "Tool '{}' arguments were truncated. Auto-filled missing required fields: {}",
                    tool_name,
                    warnings.join(", ")
                ))
            };
            Some(ToolParamsFix {
                params: Value::Object(result),
                warning,
            })
        }
        "shell" => {
            let mut result = obj;
            if !result.contains_key("command") {
                result.insert("command".to_string(), json!(""));
                warnings.push("`command`");
            }
            let warning = if warnings.is_empty() {
                None
            } else {
                Some(format!(
                    "Tool '{}' arguments were truncated. Auto-filled missing required fields: {}",
                    tool_name,
                    warnings.join(", ")
                ))
            };
            Some(ToolParamsFix {
                params: Value::Object(result),
                warning,
            })
        }
        "read" => {
            let mut result = obj;
            if !result.contains_key("path") {
                result.insert("path".to_string(), json!(""));
                warnings.push("`path`");
            }
            let warning = if warnings.is_empty() {
                None
            } else {
                Some(format!(
                    "Tool '{}' arguments were truncated. Auto-filled missing required fields: {}",
                    tool_name,
                    warnings.join(", ")
                ))
            };
            Some(ToolParamsFix {
                params: Value::Object(result),
                warning,
            })
        }
        "tree" => {
            let mut result = obj;
            if !result.contains_key("path") {
                result.insert("path".to_string(), json!(""));
                warnings.push("`path`");
            }
            let warning = if warnings.is_empty() {
                None
            } else {
                Some(format!(
                    "Tool '{}' arguments were truncated. Auto-filled missing required fields: {}",
                    tool_name,
                    warnings.join(", ")
                ))
            };
            Some(ToolParamsFix {
                params: Value::Object(result),
                warning,
            })
        }
        _ => Some(ToolParamsFix {
            params: Value::Object(obj),
            warning: None,
        }),
    }
}

/// Complete missing closing braces/brackets in JSON string, and handle truncated strings
pub fn complete_json_braces(json_str: &str) -> Option<String> {
    let trimmed = json_str.trim();

    if serde_json::from_str::<Value>(trimmed).is_ok() {
        return Some(trimmed.to_string());
    }

    let mut open_braces: i32 = 0;
    let mut open_brackets: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut last_unclosed_string_quote: Option<usize> = None;

    for (i, ch) in trimmed.chars().enumerate() {
        match ch {
            '\\' => {
                escaped = true;
            }
            '"' if !escaped => {
                in_string = !in_string;
                if in_string {
                    last_unclosed_string_quote = Some(i);
                }
            }
            '{' if !in_string => open_braces += 1,
            '}' if !in_string => open_braces = open_braces.saturating_sub(1),
            '[' if !in_string => open_brackets += 1,
            ']' if !in_string => open_brackets = open_brackets.saturating_sub(1),
            _ => {}
        }
        if escaped {
            escaped = false;
        }
    }

    let mut fixed = trimmed.to_string();

    if in_string {
        if let Some(quote_pos) = last_unclosed_string_quote {
            if quote_pos > 0 {
                let prefix: String = trimmed.chars().take(quote_pos).collect();
                if !prefix.ends_with('\\') {
                    fixed.push('"');
                }
            }
        }
    }

    for _ in 0..open_brackets {
        fixed.push(']');
    }
    for _ in 0..open_braces {
        fixed.push('}');
    }

    if serde_json::from_str::<Value>(&fixed).is_ok() {
        return Some(fixed);
    }

    None
}

/// Parses tool arguments from a JSON string, with auto-fix recovery on parse failure.
/// Returns the parsed JSON value, or falls back to empty `{}` if auto-fix also fails.
#[inline]
pub fn parse_tool_arguments(tool_name: &str, arguments: &str) -> Value {
    if arguments.is_empty() {
        return json!({});
    }
    match serde_json::from_str(arguments) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                "Tool {} arguments parsing failed ({} bytes), attempting auto-fix: {}",
                tool_name,
                arguments.len(),
                e
            );
            complete_tool_params(tool_name, arguments)
                .map(|fix| {
                    if let Some(w) = &fix.warning {
                        tracing::warn!("{}", w);
                    }
                    fix.params
                })
                .unwrap_or_else(|| {
                    tracing::warn!(
                        "Failed to auto-fix JSON for tool {}, using empty params",
                        tool_name
                    );
                    json!({})
                })
        }
    }
}
