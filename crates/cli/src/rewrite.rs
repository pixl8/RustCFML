//! Tuckey-compatible urlrewrite.xml parser and rewrite engine.
//!
//! Parses `urlrewrite.xml` files and applies URL rewrite rules to incoming
//! requests in `--serve` mode. Supports regex and wildcard matching, conditions
//! on method/port/headers, and forward/redirect/permanent-redirect actions.

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum MatchType {
    Regex,
    Wildcard,
}

#[derive(Debug, Clone)]
pub enum RewriteType {
    Forward,
    Redirect,
    PermanentRedirect,
}

#[derive(Debug, Clone)]
enum ConditionType {
    Method,
    Port,
    Header(String),
}

#[derive(Debug, Clone)]
enum ConditionOp {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

#[derive(Debug, Clone)]
struct RewriteCondition {
    cond_type: ConditionType,
    operator: ConditionOp,
    value: String,
    case_sensitive: bool,
}

#[derive(Debug, Clone)]
pub struct RewriteRule {
    #[allow(dead_code)]
    name: Option<String>,
    enabled: bool,
    match_type: MatchType,
    case_sensitive: bool,
    from: String,
    to: Option<String>,
    to_type: RewriteType,
    to_last: bool,
    conditions: Vec<RewriteCondition>,
}

pub struct RewriteResult {
    pub new_path: String,
    pub rewrite_type: RewriteType,
}

// ---------------------------------------------------------------------------
// XML parser
// ---------------------------------------------------------------------------

/// Parse a urlrewrite.xml file into a list of rewrite rules.
pub fn parse_urlrewrite_xml(path: &Path) -> Vec<RewriteRule> {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Could not read urlrewrite.xml: {}", e);
            return Vec::new();
        }
    };

    let mut rules = Vec::new();
    let mut pos = 0;
    let bytes = content.as_bytes();

    while pos < bytes.len() {
        // Skip to next '<'
        match content[pos..].find('<') {
            Some(i) => pos += i,
            None => break,
        }

        // Skip XML comments
        if content[pos..].starts_with("<!--") {
            match content[pos..].find("-->") {
                Some(i) => {
                    pos += i + 3;
                    continue;
                }
                None => break,
            }
        }

        // Skip processing instructions
        if content[pos..].starts_with("<?") {
            match content[pos..].find("?>") {
                Some(i) => {
                    pos += i + 2;
                    continue;
                }
                None => break,
            }
        }

        // Look for <rule> opening tag
        if content[pos..].starts_with("<rule") {
            let rule_start = pos;
            // Find closing </rule>
            match content[pos..].find("</rule>") {
                Some(i) => {
                    let rule_end = pos + i + 7;
                    let rule_block = &content[rule_start..rule_end];
                    if let Some(rule) = parse_rule_block(rule_block) {
                        rules.push(rule);
                    }
                    pos = rule_end;
                }
                None => {
                    pos += 1;
                }
            }
        } else {
            pos += 1;
        }
    }

    rules
}

/// Parse a single <rule>...</rule> block.
fn parse_rule_block(block: &str) -> Option<RewriteRule> {
    // Extract <rule> tag attributes
    let rule_tag_end = block.find('>')?;
    let rule_tag = &block[..rule_tag_end + 1];
    let enabled = get_xml_attr(rule_tag, "enabled").map_or(true, |v| v != "false");
    let match_type = match get_xml_attr(rule_tag, "match-type").as_deref() {
        Some("wildcard") => MatchType::Wildcard,
        _ => MatchType::Regex,
    };

    let name = extract_element_text(block, "name");
    let from = match extract_element_text(block, "from") {
        Some(f) => f,
        None => return None, // <from> is required
    };

    // Parse <to> element with attributes
    let (to_text, to_type, to_last, case_sensitive) = parse_to_element(block);

    // Parse <condition> elements
    let conditions = parse_conditions(block);

    Some(RewriteRule {
        name,
        enabled,
        match_type,
        case_sensitive,
        from,
        to: to_text,
        to_type,
        to_last,
        conditions,
    })
}

/// Extract an XML attribute value from an opening tag string.
fn get_xml_attr(tag: &str, attr_name: &str) -> Option<String> {
    let pattern_dq = format!("{}=\"", attr_name);
    let pattern_sq = format!("{}='", attr_name);

    if let Some(start) = tag.find(&pattern_dq) {
        let value_start = start + pattern_dq.len();
        if let Some(end) = tag[value_start..].find('"') {
            return Some(tag[value_start..value_start + end].to_string());
        }
    } else if let Some(start) = tag.find(&pattern_sq) {
        let value_start = start + pattern_sq.len();
        if let Some(end) = tag[value_start..].find('\'') {
            return Some(tag[value_start..value_start + end].to_string());
        }
    }
    None
}

/// Extract text content of a simple XML element like `<name>text</name>`.
fn extract_element_text(block: &str, element: &str) -> Option<String> {
    let open = format!("<{}", element);
    let close = format!("</{}>", element);

    let start = block.find(&open)?;
    let tag_end = block[start..].find('>')? + start + 1;
    let end = block[tag_end..].find(&close)? + tag_end;
    Some(block[tag_end..end].trim().to_string())
}

/// Parse the `<to>` element, returning (text, type, last, casesensitive).
fn parse_to_element(block: &str) -> (Option<String>, RewriteType, bool, bool) {
    let open = "<to";
    let close = "</to>";

    let start = match block.find(open) {
        Some(s) => s,
        None => return (None, RewriteType::Forward, true, false),
    };

    let tag_end = match block[start..].find('>') {
        Some(e) => start + e,
        None => return (None, RewriteType::Forward, true, false),
    };
    let tag = &block[start..tag_end + 1];

    let is_self_closing = tag.ends_with("/>");

    let to_type = match get_xml_attr(tag, "type").as_deref() {
        Some("redirect") => RewriteType::Redirect,
        Some("permanent-redirect") => RewriteType::PermanentRedirect,
        Some("temporary-redirect") => RewriteType::Redirect,
        _ => RewriteType::Forward,
    };

    let to_last = get_xml_attr(tag, "last").map_or(true, |v| v != "false");
    let case_sensitive = get_xml_attr(tag, "casesensitive").map_or(false, |v| v == "true");

    if is_self_closing {
        return (None, to_type, to_last, case_sensitive);
    }

    let text_start = tag_end + 1;
    let text_end = match block[text_start..].find(close) {
        Some(e) => text_start + e,
        None => return (None, to_type, to_last, case_sensitive),
    };

    let text = block[text_start..text_end].trim().to_string();
    let text = if text.is_empty() { None } else { Some(text) };

    (text, to_type, to_last, case_sensitive)
}

/// Parse all `<condition>` elements in a rule block.
fn parse_conditions(block: &str) -> Vec<RewriteCondition> {
    let mut conditions = Vec::new();
    let mut search_from = 0;

    while let Some(start) = block[search_from..].find("<condition") {
        let abs_start = search_from + start;
        let tag_end = match block[abs_start..].find('>') {
            Some(e) => abs_start + e,
            None => break,
        };
        let tag = &block[abs_start..tag_end + 1];

        let cond_type = match get_xml_attr(tag, "type").as_deref() {
            Some("method") => ConditionType::Method,
            Some("port") => ConditionType::Port,
            Some("header") => {
                let header_name = get_xml_attr(tag, "name").unwrap_or_default();
                ConditionType::Header(header_name)
            }
            _ => {
                search_from = tag_end + 1;
                continue;
            }
        };

        let operator = match get_xml_attr(tag, "operator").as_deref() {
            Some("equal") => ConditionOp::Equal,
            Some("notequal") => ConditionOp::NotEqual,
            Some("greater") => ConditionOp::Greater,
            Some("less") => ConditionOp::Less,
            Some("greaterorequal") => ConditionOp::GreaterOrEqual,
            Some("lessorequal") => ConditionOp::LessOrEqual,
            _ => ConditionOp::Equal,
        };

        let case_sensitive =
            get_xml_attr(tag, "casesensitive").map_or(false, |v| v == "true");

        let is_self_closing = tag.ends_with("/>");
        let value = if is_self_closing {
            get_xml_attr(tag, "value").unwrap_or_default()
        } else {
            match block[tag_end + 1..].find("</condition>") {
                Some(e) => block[tag_end + 1..tag_end + 1 + e].trim().to_string(),
                None => String::new(),
            }
        };

        conditions.push(RewriteCondition {
            cond_type,
            operator,
            value,
            case_sensitive,
        });

        search_from = tag_end + 1;
    }

    conditions
}

// ---------------------------------------------------------------------------
// Wildcard-to-regex converter
// ---------------------------------------------------------------------------

/// Convert a wildcard pattern to a regex string.
/// `*` matches a single path segment, `**` matches across segments.
fn wildcard_to_regex(pattern: &str) -> String {
    let mut result = String::from("^");
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            result.push_str("(.*)");
            i += 2;
        } else if chars[i] == '*' {
            result.push_str("([^/]*)");
            i += 1;
        } else {
            let c = chars[i];
            if ".+?^${}()|[]\\".contains(c) {
                result.push('\\');
            }
            result.push(c);
            i += 1;
        }
    }

    result.push('$');
    result
}

// ---------------------------------------------------------------------------
// Condition evaluator
// ---------------------------------------------------------------------------

fn check_condition(
    cond: &RewriteCondition,
    method: &str,
    port: u16,
    headers: &HashMap<String, String>,
) -> bool {
    let actual = match &cond.cond_type {
        ConditionType::Method => method.to_string(),
        ConditionType::Port => port.to_string(),
        ConditionType::Header(name) => {
            let name_lower = name.to_lowercase();
            headers
                .iter()
                .find(|(k, _)| k.to_lowercase() == name_lower)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        }
    };

    let (actual_cmp, value_cmp) = if cond.case_sensitive {
        (actual.clone(), cond.value.clone())
    } else {
        (actual.to_lowercase(), cond.value.to_lowercase())
    };

    match cond.operator {
        ConditionOp::Equal => actual_cmp == value_cmp,
        ConditionOp::NotEqual => actual_cmp != value_cmp,
        ConditionOp::Greater => actual_cmp > value_cmp,
        ConditionOp::Less => actual_cmp < value_cmp,
        ConditionOp::GreaterOrEqual => actual_cmp >= value_cmp,
        ConditionOp::LessOrEqual => actual_cmp <= value_cmp,
    }
}

// ---------------------------------------------------------------------------
// Rewrite engine
// ---------------------------------------------------------------------------

/// Apply rewrite rules to a URL path. Returns a `RewriteResult` if any rule matched.
pub fn apply_rewrite_rules(
    rules: &[RewriteRule],
    url_path: &str,
    method: &str,
    port: u16,
    headers: &HashMap<String, String>,
) -> Option<RewriteResult> {
    let mut current_path = url_path.to_string();
    let mut last_result: Option<RewriteResult> = None;

    for rule in rules {
        if !rule.enabled {
            continue;
        }

        // All conditions must pass
        let conditions_pass = rule
            .conditions
            .iter()
            .all(|c| check_condition(c, method, port, headers));
        if !conditions_pass {
            continue;
        }

        // Build regex pattern
        let pattern_str = match rule.match_type {
            MatchType::Wildcard => wildcard_to_regex(&rule.from),
            MatchType::Regex => rule.from.clone(),
        };

        let regex = if rule.case_sensitive {
            Regex::new(&pattern_str)
        } else {
            Regex::new(&format!("(?i){}", pattern_str))
        };

        let regex = match regex {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "Warning: Invalid rewrite pattern '{}': {}",
                    rule.from, e
                );
                continue;
            }
        };

        if let Some(captures) = regex.captures(&current_path) {
            if let Some(ref to) = rule.to {
                // Substitute backreferences $1, $2, etc.
                let mut new_path = to.clone();
                for i in 1..captures.len() {
                    if let Some(m) = captures.get(i) {
                        new_path = new_path.replace(&format!("${}", i), m.as_str());
                    }
                }

                last_result = Some(RewriteResult {
                    new_path: new_path.clone(),
                    rewrite_type: rule.to_type.clone(),
                });
                current_path = new_path;
            } else {
                // No <to> — matched but pass-through
                last_result = Some(RewriteResult {
                    new_path: current_path.clone(),
                    rewrite_type: RewriteType::Forward,
                });
            }

            if rule.to_last {
                break;
            }
        }
    }

    last_result
}
