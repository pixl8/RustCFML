//! CFML Tag Parser - Converts CFML tag syntax to script syntax
//!
//! This module preprocesses CFML tag-based code into equivalent CFScript code,
//! allowing the existing script parser to handle everything uniformly.
//!
//! Supported tags:
//! - <cfset variable = value>
//! - <cfoutput>...</cfoutput>
//! - <cfif condition>...</cfif>
//! - <cfelseif condition>
//! - <cfelse>
//! - <cfloop> (index, condition, array, list, query)
//! - <cfscript>...</cfscript>
//! - <cffunction name="..." ...>...</cffunction>
//! - <cfargument name="..." ...>
//! - <cfreturn expression>
//! - <cfinclude template="path">
//! - <cfdump var="#expression#">
//! - <cfthrow message="...">
//! - <cftry>...</cftry>
//! - <cfcatch type="...">...</cfcatch>
//! - <cfabort>
//! - <cfparam name="..." default="...">
//! - <cfcomponent>...</cfcomponent>
//! - <cfproperty name="..." ...>
//! - <cfhttp url="..." method="..." result="...">
//! - <cfquery name="..." datasource="...">SQL</cfquery>

/// Check if source contains CFML tags
pub fn has_cfml_tags(source: &str) -> bool {
    let lower = source.to_lowercase();
    lower.contains("<cf") || lower.contains("</cf")
}

/// Convert CFML tag-based source code to equivalent CFScript
pub fn tags_to_script(source: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i < len - 1 && chars[i] == '<' && is_cf_tag_start(&chars, i, len) {
            let (script, consumed) = parse_cf_tag(&chars, i, len);
            result.push_str(&script);
            i += consumed;
        } else if chars[i] == '#' && i + 1 < len && chars[i + 1] != '#' {
            // Hash expression inside text: #expr# -> writeOutput(expr);
            // But only if we're in a text context (not inside a tag attribute)
            // Check if there's a matching closing #
            if let Some(end) = find_closing_hash(&chars, i + 1, len) {
                let expr: String = chars[i + 1..end].iter().collect();
                result.push_str(&format!("writeOutput({});", expr));
                i = end + 1;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else if chars[i] == '#' && i + 1 < len && chars[i + 1] == '#' {
            // Escaped hash ## -> literal #
            result.push_str("writeOutput(\"##\");");
            i += 2;
        } else {
            // Plain text - collect until we hit a tag or hash expression
            let start = i;
            while i < len && !(chars[i] == '<' && is_cf_tag_start(&chars, i, len))
                && !(chars[i] == '#' && i + 1 < len)
            {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            if !text.is_empty() && text.trim().len() > 0 {
                // Output plain text
                let escaped = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");
                result.push_str(&format!("writeOutput(\"{}\");", escaped));
            }
        }
    }

    result
}

fn is_cf_tag_start(chars: &[char], pos: usize, len: usize) -> bool {
    if pos + 3 >= len {
        return false;
    }
    let next_two: String = chars[pos + 1..pos + 3].iter().collect();
    let next_lower = next_two.to_lowercase();
    next_lower == "cf" || (chars[pos + 1] == '/' && pos + 4 < len && {
        let after_slash: String = chars[pos + 2..pos + 4].iter().collect();
        after_slash.to_lowercase() == "cf"
    })
}

fn find_closing_hash(chars: &[char], start: usize, len: usize) -> Option<usize> {
    let mut i = start;
    let mut depth = 0;
    while i < len {
        if chars[i] == '#' && depth == 0 {
            return Some(i);
        }
        if chars[i] == '(' {
            depth += 1;
        }
        if chars[i] == ')' && depth > 0 {
            depth -= 1;
        }
        i += 1;
    }
    None
}

fn parse_cf_tag(chars: &[char], start: usize, len: usize) -> (String, usize) {
    // Determine if closing tag
    let is_closing = chars.get(start + 1) == Some(&'/');

    // Extract tag name
    let name_start = if is_closing { start + 2 } else { start + 1 };
    let mut name_end = name_start;
    while name_end < len && chars[name_end].is_alphanumeric() {
        name_end += 1;
    }
    let tag_name: String = chars[name_start..name_end].iter().collect();
    let tag_lower = tag_name.to_lowercase();

    // For closing tags, just skip them (the opening tag handler manages scope)
    if is_closing {
        let close_end = find_tag_end(chars, name_end, len);
        // Return empty and consumed count
        match tag_lower.as_str() {
            "cfif" => return ("}\n".to_string(), close_end - start),
            "cfloop" => return ("}\n".to_string(), close_end - start),
            "cfoutput" => return (String::new(), close_end - start),
            "cffunction" => return ("}\n".to_string(), close_end - start),
            "cfcomponent" => return ("}\n".to_string(), close_end - start),
            "cftry" => return (String::new(), close_end - start), // try block closed by catch
            "cfcatch" => return ("}\n".to_string(), close_end - start),
            "cfscript" => return (String::new(), close_end - start),
            _ => return (String::new(), close_end - start),
        }
    }

    // Parse attributes
    let (attrs, tag_end) = parse_tag_attributes(chars, name_end, len);

    match tag_lower.as_str() {
        "cfset" => {
            // <cfset expression>
            // The "expression" is everything after cfset until >
            let expr: String = chars[name_end..tag_end - 1]
                .iter()
                .collect::<String>()
                .trim()
                .to_string();
            // Handle #...# in expressions
            let expr = strip_hashes(&expr);
            (format!("{};\n", expr), tag_end - start)
        }
        "cfoutput" => {
            // <cfoutput> just marks a region where # expressions are evaluated
            // The content between cfoutput tags is handled by the main loop
            (String::new(), tag_end - start)
        }
        "cfif" => {
            let condition = get_attr_or_body(&attrs, chars, name_end, tag_end);
            let condition = strip_hashes(&condition);
            (format!("if ({}) {{\n", condition), tag_end - start)
        }
        "cfelseif" => {
            let condition = get_attr_or_body(&attrs, chars, name_end, tag_end);
            let condition = strip_hashes(&condition);
            (format!("}} else if ({}) {{\n", condition), tag_end - start)
        }
        "cfelse" => {
            ("} else {\n".to_string(), tag_end - start)
        }
        "cfloop" => {
            parse_cfloop_tag(&attrs, tag_end - start)
        }
        "cfscript" => {
            // Everything between <cfscript> and </cfscript> is raw script
            // Find the closing </cfscript>
            if let Some(end_tag_pos) = find_closing_tag(chars, tag_end, len, "cfscript") {
                let script: String = chars[tag_end..end_tag_pos].iter().collect();
                let close_end = find_tag_end(chars, end_tag_pos, len);
                (script, close_end - start)
            } else {
                (String::new(), tag_end - start)
            }
        }
        "cffunction" => {
            let name = attrs.get("name").cloned().unwrap_or_default();
            let access = attrs.get("access").cloned().unwrap_or("public".to_string());
            let return_type = attrs.get("returntype").cloned().unwrap_or_default();

            // Scan ahead for <cfargument> tags to extract parameter names
            let param_names = scan_cfargument_tags(chars, tag_end, len);

            let mut sig = String::new();
            if !access.is_empty() {
                sig.push_str(&access);
                sig.push(' ');
            }
            if !return_type.is_empty() {
                sig.push_str(&return_type);
                sig.push(' ');
            }
            sig.push_str(&format!("function {}({}) {{\n", name, param_names.join(", ")));
            (sig, tag_end - start)
        }
        "cfargument" => {
            let name = attrs.get("name").cloned().unwrap_or_default();
            let default = attrs.get("default").cloned();
            if let Some(def) = default {
                let def = strip_hashes(&def);
                // Quote the default if it's not already a number, boolean, or quoted
                let def_val = quote_if_needed(&def);
                (
                    format!("if (isNull(arguments.{})) {{ arguments.{} = {}; }}\n", name, name, def_val),
                    tag_end - start,
                )
            } else {
                (String::new(), tag_end - start)
            }
        }
        "cfreturn" => {
            let expr = get_attr_or_body(&attrs, chars, name_end, tag_end);
            let expr = strip_hashes(&expr);
            (format!("return {};\n", expr), tag_end - start)
        }
        "cfinclude" => {
            let template = attrs.get("template").cloned().unwrap_or_default();
            (format!("include \"{}\";\n", template), tag_end - start)
        }
        "cfdump" => {
            let var = attrs.get("var").cloned().unwrap_or("\"\"".to_string());
            let var = strip_hashes(&var);
            (format!("writeDump({});\n", var), tag_end - start)
        }
        "cfthrow" => {
            let message = attrs.get("message").cloned().unwrap_or("An error occurred".to_string());
            let message = strip_hashes(&message);
            let message_val = quote_if_needed(&message);
            (format!("throw({});\n", message_val), tag_end - start)
        }
        "cftry" => {
            ("try {\n".to_string(), tag_end - start)
        }
        "cfcatch" => {
            let catch_type = attrs.get("type").cloned().unwrap_or("any".to_string());
            (format!("}} catch ({} e) {{\n", catch_type), tag_end - start)
        }
        "cfabort" => {
            ("abort;\n".to_string(), tag_end - start)
        }
        "cfparam" => {
            let name = attrs.get("name").cloned().unwrap_or_default();
            let default = attrs.get("default").cloned().unwrap_or("\"\"".to_string());
            let default = strip_hashes(&default);
            // Clean name - remove scope prefix quotes
            let clean_name = name.replace('"', "").replace('\'', "");
            (
                format!("if (isNull({})) {{ {} = {}; }}\n", clean_name, clean_name, default),
                tag_end - start,
            )
        }
        "cfcomponent" => {
            let name = attrs.get("name").cloned().unwrap_or("Component".to_string());
            let extends = attrs.get("extends").cloned();
            let mut decl = format!("component {} ", name);
            if let Some(ext) = extends {
                decl.push_str(&format!("extends {} ", ext));
            }
            // Pass through extra attributes as metadata key="value" pairs
            for (k, v) in &attrs {
                if k != "name" && k != "extends" {
                    decl.push_str(&format!("{}=\"{}\" ", k, v));
                }
            }
            decl.push_str("{\n");
            (decl, tag_end - start)
        }
        "cfproperty" => {
            let name = attrs.get("name").cloned().unwrap_or_default();
            let default = attrs.get("default").cloned();
            if let Some(def) = default {
                (format!("this.{} = {};\n", name, strip_hashes(&def)), tag_end - start)
            } else {
                (format!("this.{} = \"\";\n", name), tag_end - start)
            }
        }
        "cfhttp" => {
            let url = attrs.get("url").cloned().unwrap_or_default();
            let method = attrs.get("method").cloned().unwrap_or("GET".to_string());
            let result_var = attrs.get("result").cloned().unwrap_or("cfhttp".to_string());
            let timeout = attrs.get("timeout").cloned();
            let charset = attrs.get("charset").cloned();

            let mut opts = Vec::new();
            opts.push(format!("url: \"{}\"", url));
            opts.push(format!("method: \"{}\"", method));
            if let Some(t) = timeout {
                opts.push(format!("timeout: {}", t));
            }
            if let Some(c) = charset {
                opts.push(format!("charset: \"{}\"", c));
            }

            (format!("{} = cfhttp({{ {} }});\n", result_var, opts.join(", ")), tag_end - start)
        }
        "cfquery" => {
            let name = attrs.get("name").cloned().unwrap_or("queryResult".to_string());
            let datasource = attrs.get("datasource").cloned();
            let return_type = attrs.get("returntype").cloned();

            // Everything between <cfquery> and </cfquery> is the SQL
            if let Some(end_tag_pos) = find_closing_tag(chars, tag_end, len, "cfquery") {
                let sql_raw: String = chars[tag_end..end_tag_pos].iter().collect();
                let sql = sql_raw.trim().replace('"', "\\\"").replace('\n', " ").replace('\r', "");
                let close_end = find_tag_end(chars, end_tag_pos, len);

                let mut opts_parts = Vec::new();
                if let Some(ds) = datasource {
                    opts_parts.push(format!("datasource: \"{}\"", ds));
                }
                if let Some(rt) = return_type {
                    opts_parts.push(format!("returnType: \"{}\"", rt));
                }

                let opts_str = if opts_parts.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{ {} }}", opts_parts.join(", "))
                };

                (format!("{} = queryExecute(\"{}\", [], {});\n", name, sql, opts_str), close_end - start)
            } else {
                (String::new(), tag_end - start)
            }
        }
        _ => {
            // Unknown tag, skip it
            (String::new(), tag_end - start)
        }
    }
}

fn find_tag_end(chars: &[char], start: usize, len: usize) -> usize {
    let mut i = start;
    let mut in_string = false;
    let mut string_char = '"';
    while i < len {
        if !in_string && (chars[i] == '"' || chars[i] == '\'') {
            in_string = true;
            string_char = chars[i];
        } else if in_string && chars[i] == string_char {
            in_string = false;
        } else if !in_string && chars[i] == '>' {
            return i + 1;
        }
        i += 1;
    }
    len
}

fn parse_tag_attributes(
    chars: &[char],
    start: usize,
    len: usize,
) -> (std::collections::HashMap<String, String>, usize) {
    let mut attrs = std::collections::HashMap::new();
    let mut i = start;

    // Skip whitespace
    while i < len && chars[i].is_whitespace() {
        i += 1;
    }

    while i < len && chars[i] != '>' && !(chars[i] == '/' && i + 1 < len && chars[i + 1] == '>') {
        // Parse attribute name
        let attr_start = i;
        while i < len && chars[i] != '=' && chars[i] != '>' && !chars[i].is_whitespace() {
            i += 1;
        }
        let attr_name: String = chars[attr_start..i].iter().collect();

        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        if i < len && chars[i] == '=' {
            i += 1; // skip =
            // Skip whitespace
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            // Parse attribute value
            if i < len && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                i += 1;
                let val_start = i;
                while i < len && chars[i] != quote {
                    i += 1;
                }
                let val: String = chars[val_start..i].iter().collect();
                if i < len {
                    i += 1; // skip closing quote
                }
                attrs.insert(attr_name.to_lowercase(), val);
            } else {
                // Unquoted value
                let val_start = i;
                while i < len && !chars[i].is_whitespace() && chars[i] != '>' {
                    i += 1;
                }
                let val: String = chars[val_start..i].iter().collect();
                attrs.insert(attr_name.to_lowercase(), val);
            }
        } else if !attr_name.is_empty() {
            attrs.insert(attr_name.to_lowercase(), String::new());
        }

        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }
    }

    // Find the actual end of the tag
    let tag_end = find_tag_end(chars, i, len);
    (attrs, tag_end)
}

fn get_attr_or_body(
    attrs: &std::collections::HashMap<String, String>,
    chars: &[char],
    name_end: usize,
    tag_end: usize,
) -> String {
    // For tags like <cfif condition> where the condition is not in an attribute
    // It's everything between the tag name and >
    if attrs.is_empty() {
        return chars[name_end..tag_end - 1]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
    }
    // If there are "real" attributes, return the body part
    chars[name_end..tag_end - 1]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

/// Quote a string value if it's not already a number, boolean, expression, or quoted
fn quote_if_needed(s: &str) -> String {
    let s = s.trim();
    // Already quoted
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return s.to_string();
    }
    // Number
    if s.parse::<f64>().is_ok() {
        return s.to_string();
    }
    // Boolean/null keywords
    let lower = s.to_lowercase();
    if lower == "true" || lower == "false" || lower == "null" || lower == "yes" || lower == "no" {
        return s.to_string();
    }
    // Contains operators or function calls - looks like an expression
    if s.contains('(') || s.contains('+') || s.contains('-') || s.contains('*')
        || s.contains('/') || s.contains('&') || s.contains('.') || s.contains('[')
    {
        return s.to_string();
    }
    // Otherwise, quote it
    format!("\"{}\"", s.replace('"', "\\\""))
}

fn strip_hashes(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('#') && s.ends_with('#') && s.len() > 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn find_closing_tag(chars: &[char], start: usize, len: usize, tag_name: &str) -> Option<usize> {
    let target = format!("</{}", tag_name);
    let target_lower = target.to_lowercase();
    let mut i = start;
    while i < len {
        if chars[i] == '<' && chars.get(i + 1) == Some(&'/') {
            let remaining: String = chars[i..].iter().take(target.len() + 1).collect();
            if remaining.to_lowercase().starts_with(&target_lower) {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Scan ahead from current position to find <cfargument> tags and extract their names
fn scan_cfargument_tags(chars: &[char], start: usize, len: usize) -> Vec<String> {
    let mut names = Vec::new();
    let mut i = start;

    while i < len {
        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }
        // Check if we hit a <cfargument
        if i + 12 < len && chars[i] == '<' {
            let tag: String = chars[i..i + 12].iter().collect();
            if tag.to_lowercase() == "<cfargument " || tag.to_lowercase() == "<cfargument>" {
                // Parse the tag's attributes
                let name_start = i + 1; // skip <
                let mut j = name_start;
                while j < len && chars[j].is_alphanumeric() {
                    j += 1;
                }
                let (tag_attrs, _) = parse_tag_attributes(chars, j, len);
                if let Some(name) = tag_attrs.get("name") {
                    names.push(name.clone());
                }
                // Skip to end of tag
                while i < len && chars[i] != '>' {
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                continue;
            }
            // If we hit any other CF tag (like <cfreturn>, <cfset>, etc.) or closing </cffunction>, stop scanning
            let next_chars: String = chars[i..std::cmp::min(i + 15, len)].iter().collect();
            let next_lower = next_chars.to_lowercase();
            if next_lower.starts_with("</cffunction") || next_lower.starts_with("<cfreturn")
                || next_lower.starts_with("<cfset") || next_lower.starts_with("<cfif")
                || next_lower.starts_with("<cfloop") || next_lower.starts_with("<cfoutput")
                || next_lower.starts_with("<cftry")
            {
                break;
            }
        }
        i += 1;
    }

    names
}

fn parse_cfloop_tag(
    attrs: &std::collections::HashMap<String, String>,
    consumed: usize,
) -> (String, usize) {
    // Different loop types based on attributes
    if let (Some(from), Some(to), Some(index)) = (
        attrs.get("from"),
        attrs.get("to"),
        attrs.get("index"),
    ) {
        let step = attrs.get("step").cloned().unwrap_or("1".to_string());
        let from = strip_hashes(from);
        let to = strip_hashes(to);
        let step = strip_hashes(&step);
        (
            format!(
                "for (var {} = {}; {} <= {}; {} = {} + {}) {{\n",
                index, from, index, to, index, index, step
            ),
            consumed,
        )
    } else if let Some(condition) = attrs.get("condition") {
        let condition = strip_hashes(condition);
        (format!("while ({}) {{\n", condition), consumed)
    } else if let (Some(array), Some(index)) = (attrs.get("array"), attrs.get("index")) {
        let array = strip_hashes(array);
        let index = strip_hashes(index);
        (format!("for (var {} in {}) {{\n", index, array), consumed)
    } else if let (Some(list), Some(index)) = (attrs.get("list"), attrs.get("index")) {
        let list = strip_hashes(list);
        let index = strip_hashes(index);
        let delimiters = attrs
            .get("delimiters")
            .cloned()
            .unwrap_or(",".to_string());
        (
            format!(
                "for (var {} in listToArray({}, \"{}\")) {{\n",
                index, list, delimiters
            ),
            consumed,
        )
    } else if let (Some(query), Some(index)) = (attrs.get("query"), attrs.get("index").or(attrs.get("item"))) {
        let query = strip_hashes(query);
        (
            format!("for (var {} in {}) {{\n", index, query),
            consumed,
        )
    } else if let Some(collection) = attrs.get("collection") {
        let collection = strip_hashes(collection);
        let item = attrs.get("item").cloned().unwrap_or("item".to_string());
        let key = attrs.get("key");
        if let Some(key) = key {
            (
                format!("for (var {} in structKeyArray({})) {{ var {} = {}[{}];\n", key, collection, item, collection, key),
                consumed,
            )
        } else {
            (format!("for (var {} in {}) {{\n", item, collection), consumed)
        }
    } else {
        // Infinite loop? Just use while(true)
        ("while (true) {\n".to_string(), consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfset() {
        let input = "<cfset x = 5>";
        assert!(has_cfml_tags(input));
        let result = tags_to_script(input);
        assert!(result.contains("x = 5"));
    }

    #[test]
    fn test_cfif() {
        let input = "<cfif x GT 5>yes</cfif>";
        let result = tags_to_script(input);
        assert!(result.contains("if (x GT 5)"));
    }

    #[test]
    fn test_cfoutput_hash() {
        let input = "<cfoutput>#name#</cfoutput>";
        let result = tags_to_script(input);
        assert!(result.contains("writeOutput(name)"));
    }

    #[test]
    fn test_cfloop_index() {
        let input = r#"<cfloop from="1" to="10" index="i">body</cfloop>"#;
        let result = tags_to_script(input);
        assert!(result.contains("for (var i = 1; i <= 10; i = i + 1)"));
    }
}
