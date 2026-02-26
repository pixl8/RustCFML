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
//! - <cfheader statuscode="..." statustext="..." name="..." value="...">
//! - <cfcontent reset="..." type="..." variable="...">
//! - <cflocation url="..." statuscode="..." addtoken="...">
//! - <cfdirectory action="..." directory="..." name="..." filter="..." recurse="...">
//! - <cfinvoke component="..." method="..." returnvariable="...">

/// Check if source contains CFML tags or CFML comments
pub fn has_cfml_tags(source: &str) -> bool {
    let lower = source.to_lowercase();
    lower.contains("<cf") || lower.contains("</cf") || source.contains("<!---")
}

/// Convert CFML tag-based source code to equivalent CFScript
pub fn tags_to_script(source: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Strip CFML comments: <!--- ... --->
        if i + 4 < len && chars[i] == '<' && chars[i + 1] == '!' && chars[i + 2] == '-' && chars[i + 3] == '-' && chars[i + 4] == '-' {
            // Find closing --->
            let mut j = i + 5;
            while j + 2 < len {
                if chars[j] == '-' && chars[j + 1] == '-' && chars[j + 2] == '>' {
                    j += 3;
                    break;
                }
                j += 1;
            }
            if j + 2 >= len && !(j >= 3 && chars[j - 1] == '>' && chars[j - 2] == '-' && chars[j - 3] == '-') {
                j = len; // unclosed comment, skip to end
            }
            i = j;
            continue;
        }
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
            // Plain text - collect until we hit a tag, hash expression, or CFML comment
            let start = i;
            while i < len && !(chars[i] == '<' && is_cf_tag_start(&chars, i, len))
                && !(chars[i] == '#' && i + 1 < len)
                && !(i + 4 < len && chars[i] == '<' && chars[i + 1] == '!' && chars[i + 2] == '-' && chars[i + 3] == '-' && chars[i + 4] == '-')
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
            "cfinterface" => return ("}\n".to_string(), close_end - start),
            "cftry" => return (String::new(), close_end - start), // try block closed by catch
            "cfcatch" => return ("}\n".to_string(), close_end - start),
            "cfscript" => return (String::new(), close_end - start),
            "cfsavecontent" => return (String::new(), close_end - start),
            "cftransaction" => return (String::new(), close_end - start),
            _ => return (String::new(), close_end - start),
        }
    }

    // Tags with freeform expression bodies (not key=value attributes) —
    // use find_tag_end directly to avoid misparsing expressions containing quotes/equals
    match tag_lower.as_str() {
        "cfset" | "cfif" | "cfelseif" | "cfreturn" => {
            let tag_end = find_tag_end(chars, name_end, len);
            let raw: String = chars[name_end..tag_end - 1].iter().collect();
            let body = raw.trim();
            let body = if body.ends_with('/') {
                body[..body.len() - 1].trim()
            } else {
                body
            };
            let body = strip_hashes(body);
            // CFML tags don't use backslash escaping, but the script parser does.
            // Escape backslashes in string literals so they survive script parsing.
            let body = escape_backslashes_in_tag_strings(&body);
            let result = match tag_lower.as_str() {
                "cfset" => format!("{};\n", body),
                "cfif" => format!("if ({}) {{\n", body),
                "cfelseif" => format!("}} else if ({}) {{\n", body),
                "cfreturn" => format!("return {};\n", body),
                _ => unreachable!(),
            };
            return (result, tag_end - start);
        }
        _ => {}
    }

    // Parse attributes for all other tags
    let (attrs, tag_end) = parse_tag_attributes(chars, name_end, len);

    match tag_lower.as_str() {
        "cfoutput" => {
            // <cfoutput> just marks a region where # expressions are evaluated
            // The content between cfoutput tags is handled by the main loop
            (String::new(), tag_end - start)
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
            // message is always a string — quote it directly (escape internal quotes)
            let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
            (format!("throw(\"{}\");\n", escaped), tag_end - start)
        }
        "cftry" => {
            ("try {\n".to_string(), tag_end - start)
        }
        "cfcatch" => {
            let catch_type = attrs.get("type").cloned().unwrap_or("any".to_string());
            (format!("}} catch ({} e) {{\n", catch_type), tag_end - start)
        }
        "cfabort" => {
            ("__cfabort();\n".to_string(), tag_end - start)
        }
        "cfparam" => {
            let name = attrs.get("name").cloned().unwrap_or_default();
            let default = attrs.get("default").cloned().unwrap_or("\"\"".to_string());
            let default = strip_hashes(&default);
            // Clean name - remove scope prefix quotes and strip hash expressions
            let clean_name = strip_hashes(&name.replace('"', "").replace('\'', ""));
            (
                format!("if (isNull({})) {{ {} = {}; }}\n", clean_name, clean_name, default),
                tag_end - start,
            )
        }
        "cfcomponent" => {
            let name = attrs.get("name").cloned();
            let extends = attrs.get("extends").cloned();
            let implements = attrs.get("implements").cloned();
            let mut decl = if let Some(ref n) = name {
                format!("component {} ", n)
            } else {
                "component ".to_string()
            };
            if let Some(ext) = extends {
                decl.push_str(&format!("extends {} ", ext));
            }
            if let Some(imp) = implements {
                decl.push_str(&format!("implements {} ", imp));
            }
            // Pass through extra attributes as metadata key="value" pairs
            for (k, v) in &attrs {
                if k != "name" && k != "extends" && k != "implements" {
                    decl.push_str(&format!("{}=\"{}\" ", k, v));
                }
            }
            decl.push_str("{\n");
            (decl, tag_end - start)
        }
        "cfinterface" => {
            let name = attrs.get("name").cloned();
            let extends = attrs.get("extends").cloned();
            let mut decl = if let Some(ref n) = name {
                format!("interface {} ", n)
            } else {
                "interface ".to_string()
            };
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
                let close_end = find_tag_end(chars, end_tag_pos, len);

                // Scan for <cfqueryparam> tags — replace with ? and collect params
                let (cleaned_sql, query_params) = scan_cfqueryparam_tags(&sql_raw);

                // Process remaining hash expressions in SQL for string interpolation
                let sql = process_sql_hashes(&cleaned_sql);

                let mut opts_parts = Vec::new();
                if let Some(ds) = &datasource {
                    let ds_val = strip_hashes(ds);
                    if ds != &ds_val {
                        // Dynamic datasource — emit as variable reference
                        opts_parts.push(format!("datasource: {}", ds_val));
                    } else {
                        opts_parts.push(format!("datasource: \"{}\"", ds));
                    }
                }
                if let Some(rt) = return_type {
                    opts_parts.push(format!("returnType: \"{}\"", rt));
                }

                let opts_str = if opts_parts.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{ {} }}", opts_parts.join(", "))
                };

                let params_str = if query_params.is_empty() {
                    "[]".to_string()
                } else {
                    let param_strs: Vec<String> = query_params.iter().map(|p| {
                        let mut parts = Vec::new();
                        parts.push(format!("value: {}", p.value_expr));
                        parts.push(format!("cfsqltype: \"{}\"", p.cfsqltype));
                        if p.null {
                            parts.push("null: true".to_string());
                        }
                        if p.list {
                            parts.push("list: true".to_string());
                            if p.separator != "," {
                                parts.push(format!("separator: \"{}\"", p.separator));
                            }
                        }
                        format!("{{ {} }}", parts.join(", "))
                    }).collect();
                    format!("[{}]", param_strs.join(", "))
                };

                (format!("{} = queryExecute({}, {}, {});\n", name, sql, params_str, opts_str), close_end - start)
            } else {
                (String::new(), tag_end - start)
            }
        }
        "cfheader" => {
            // <cfheader statuscode="200" statustext="OK">
            // → __cfheader({statuscode: 200, statustext: "OK"});
            let mut parts = Vec::new();
            for (k, v) in &attrs {
                let raw = v.trim();
                if raw.starts_with('#') && raw.ends_with('#') && raw.len() > 2 {
                    // Dynamic expression: strip hashes, emit bare
                    parts.push(format!("{}: {}", k, strip_hashes(raw)));
                } else if raw.parse::<f64>().is_ok() {
                    parts.push(format!("{}: {}", k, raw));
                } else {
                    parts.push(format!("{}: \"{}\"", k, raw.replace('"', "\\\"")));
                }
            }
            (format!("__cfheader({{ {} }});\n", parts.join(", ")), tag_end - start)
        }
        "cfcontent" => {
            // <cfcontent reset="true" type="application/json">
            // → __cfcontent({reset: true, type: "application/json"});
            let mut parts = Vec::new();
            for (k, v) in &attrs {
                let val = strip_hashes(&v);
                if k == "reset" {
                    let lower = val.to_lowercase();
                    if lower == "true" || lower == "yes" {
                        parts.push(format!("{}: true", k));
                    } else {
                        parts.push(format!("{}: false", k));
                    }
                } else if k == "variable" {
                    parts.push(format!("{}: {}", k, val));
                } else {
                    parts.push(format!("{}: \"{}\"", k, val.replace('"', "\\\"")));
                }
            }
            (format!("__cfcontent({{ {} }});\n", parts.join(", ")), tag_end - start)
        }
        "cflocation" => {
            // <cflocation url="/path" statuscode="302" addtoken="false">
            // → __cflocation({url: "/path", statuscode: 302, addtoken: false});
            let mut parts = Vec::new();
            for (k, v) in &attrs {
                let raw = v.trim();
                if raw.starts_with('#') && raw.ends_with('#') && raw.len() > 2 {
                    parts.push(format!("{}: {}", k, strip_hashes(raw)));
                } else if raw.parse::<f64>().is_ok() {
                    parts.push(format!("{}: {}", k, raw));
                } else {
                    let lower = raw.to_lowercase();
                    if lower == "true" || lower == "yes" {
                        parts.push(format!("{}: true", k));
                    } else if lower == "false" || lower == "no" {
                        parts.push(format!("{}: false", k));
                    } else {
                        parts.push(format!("{}: \"{}\"", k, raw.replace('"', "\\\"")));
                    }
                }
            }
            (format!("__cflocation({{ {} }});\n", parts.join(", ")), tag_end - start)
        }
        "cfdirectory" => {
            // <cfdirectory action="list" directory="." name="qDir" recurse="true">
            // → qDir = cfdirectory({action: "list", directory: ".", recurse: true});
            let name = attrs.get("name").cloned();
            let mut parts = Vec::new();
            for (k, v) in &attrs {
                if k == "name" {
                    continue;
                }
                let raw = v.trim();
                if raw.starts_with('#') && raw.ends_with('#') && raw.len() > 2 {
                    parts.push(format!("{}: {}", k, strip_hashes(raw)));
                } else {
                    let lower = raw.to_lowercase();
                    if lower == "true" || lower == "yes" {
                        parts.push(format!("{}: true", k));
                    } else if lower == "false" || lower == "no" {
                        parts.push(format!("{}: false", k));
                    } else if raw.parse::<f64>().is_ok() {
                        parts.push(format!("{}: {}", k, raw));
                    } else {
                        parts.push(format!("{}: \"{}\"", k, raw.replace('"', "\\\"")));
                    }
                }
            }
            let call = format!("cfdirectory({{ {} }})", parts.join(", "));
            if let Some(n) = name {
                (format!("{} = {};\n", n, call), tag_end - start)
            } else {
                (format!("{};\n", call), tag_end - start)
            }
        }
        "cfsavecontent" => {
            let variable = attrs.get("variable").cloned().unwrap_or("__savecontent_result".to_string());
            // Find closing tag — body between is processed by main loop
            if let Some(end_tag_pos) = find_closing_tag(chars, tag_end, len, "cfsavecontent") {
                let body: String = chars[tag_end..end_tag_pos].iter().collect();
                let close_end = find_tag_end(chars, end_tag_pos, len);
                // Process body through main loop (handles hash expressions, nested tags, text)
                let body_script = tags_to_script(&body);
                (format!("__cfsavecontent_start();\n{}{} = __cfsavecontent_end();\n", body_script, variable), close_end - start)
            } else {
                (format!("__cfsavecontent_start();\n"), tag_end - start)
            }
        }
        "cftransaction" => {
            let action = attrs.get("action").cloned().unwrap_or_else(|| "begin".to_string());
            let isolation = attrs.get("isolation").cloned();
            let datasource = attrs.get("datasource").cloned();

            match action.to_lowercase().as_str() {
                "commit" => {
                    (format!("__cftransaction_commit();\n"), tag_end - start)
                }
                "rollback" => {
                    (format!("__cftransaction_rollback();\n"), tag_end - start)
                }
                _ => {
                    // "begin" (default) — wraps body in try/catch
                    if let Some(end_tag_pos) = find_closing_tag(chars, tag_end, len, "cftransaction") {
                        let body: String = chars[tag_end..end_tag_pos].iter().collect();
                        let close_end = find_tag_end(chars, end_tag_pos, len);
                        let body_script = tags_to_script(&body);

                        // Build args for __cftransaction_start
                        let mut txn_args = vec!["\"begin\"".to_string()];
                        if let Some(ref iso) = isolation {
                            txn_args.push(format!("\"{}\"", iso));
                        }
                        if let Some(ref ds) = datasource {
                            let ds_val = strip_hashes(ds);
                            if ds != &ds_val {
                                txn_args.push(ds_val);
                            } else {
                                txn_args.push(format!("\"{}\"", ds));
                            }
                        } else {
                            // Try to extract datasource from the first cfquery inside
                            let ds_from_body = extract_datasource_from_body(&body);
                            if let Some(ds) = ds_from_body {
                                txn_args.push(format!("\"{}\"", ds));
                            }
                        }

                        (format!(
                            "__cftransaction_start({});\ntry {{\n{}\n__cftransaction_commit();\n}} catch(any __txn_e) {{\n__cftransaction_rollback();\nthrow __txn_e;\n}}\n",
                            txn_args.join(", "), body_script
                        ), close_end - start)
                    } else {
                        (format!("__cftransaction_start(\"begin\");\n"), tag_end - start)
                    }
                }
            }
        }
        "cfinvoke" => {
            // <cfinvoke component="MyComp" method="greet" name="World" returnvariable="msg">
            // → msg = __cfinvoke(MyComp, "greet", {name: "World"});
            let component = attrs.get("component").cloned().unwrap_or_default();
            let method = attrs.get("method").cloned().unwrap_or_default();
            let return_var = attrs.get("returnvariable").cloned();
            let arg_collection = attrs.get("argumentcollection").cloned();

            // Component: strip hashes for dynamic (#var#), quote for static name
            let comp_expr = if component.starts_with('#') && component.ends_with('#') && component.len() > 2 {
                strip_hashes(&component)
            } else {
                format!("\"{}\"", component)
            };

            // Method: always quoted
            let method_expr = format!("\"{}\"", method);

            // Third argument: argumentcollection or struct of remaining attrs
            let third_arg = if let Some(ac) = arg_collection {
                let ac = strip_hashes(&ac);
                ac
            } else {
                let reserved = ["component", "method", "returnvariable", "argumentcollection"];
                let mut extra_parts = Vec::new();
                for (k, v) in &attrs {
                    if reserved.contains(&k.as_str()) {
                        continue;
                    }
                    let val = strip_hashes(&v);
                    extra_parts.push(format!("{}: {}", k, quote_if_needed(&val)));
                }
                format!("{{ {} }}", extra_parts.join(", "))
            };

            let call = format!("__cfinvoke({}, {}, {})", comp_expr, method_expr, third_arg);
            if let Some(rv) = return_var {
                (format!("{} = {};\n", rv, call), tag_end - start)
            } else {
                (format!("{};\n", call), tag_end - start)
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

/// Escape backslashes inside string literals in tag body expressions.
/// CFML tag-based code doesn't use backslash escaping, but the script parser does.
/// This converts `\` to `\\` inside string literals so the script parser
/// correctly interprets them as literal backslashes.
fn escape_backslashes_in_tag_strings(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut result = String::new();
    let mut i = 0;

    while i < len {
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            result.push(quote); // opening quote
            i += 1;
            while i < len {
                if chars[i] == quote {
                    // Check for doubled quote (CFML escape: "" or '')
                    if i + 1 < len && chars[i + 1] == quote {
                        result.push(quote);
                        result.push(quote);
                        i += 2;
                    } else {
                        // End of string
                        break;
                    }
                } else if chars[i] == '\\' {
                    result.push('\\');
                    result.push('\\');
                    i += 1;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            if i < len {
                result.push(chars[i]); // closing quote
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn strip_hashes(s: &str) -> String {
    let s = s.trim();
    // If the entire string is wrapped in #...#, just strip outer hashes
    if s.starts_with('#') && s.ends_with('#') && s.len() > 2 && s[1..s.len()-1].find('#').is_none() {
        return s[1..s.len() - 1].to_string();
    }
    // Handle embedded #expr# within larger expressions
    // Replace #expr# with just expr (strip the hash delimiters)
    if !s.contains('#') {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut result = String::new();
    let mut i = 0;
    while i < len {
        if chars[i] == '#' {
            // Look for closing #
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '#') {
                let end = i + 1 + end;
                // Extract expression between hashes
                let expr: String = chars[i + 1..end].iter().collect();
                result.push_str(&expr);
                i = end + 1;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
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
    } else if let Some(query) = attrs.get("query") {
        let query = strip_hashes(query);
        if let Some(index) = attrs.get("index").or(attrs.get("item")) {
            (
                format!("for (var {} in {}) {{\n", index, query),
                consumed,
            )
        } else {
            // <cfloop query="q"> without index — CFML query row loop
            // q.column resolves to the current row's column value
            (
                format!("for (var __qrow in {}) {{ {} = __qrow;\n", query, query),
                consumed,
            )
        }
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

/// Extract datasource from the first <cfquery> tag in a body string
fn extract_datasource_from_body(body: &str) -> Option<String> {
    let lower = body.to_lowercase();
    if let Some(pos) = lower.find("<cfquery") {
        let chars: Vec<char> = body.chars().collect();
        let len = chars.len();
        // Skip tag name
        let mut i = pos + 8; // past "<cfquery"
        while i < len && chars[i].is_alphanumeric() {
            i += 1;
        }
        let (attrs, _) = parse_tag_attributes(&chars, i, len);
        return attrs.get("datasource").cloned();
    }
    None
}

// -----------------------------------------------
// cfqueryparam scanning
// -----------------------------------------------

struct CfQueryParam {
    value_expr: String,  // The value expression (script-ready: variable ref or string literal)
    cfsqltype: String,
    null: bool,
    list: bool,
    separator: String,
}

/// Scan SQL body for <cfqueryparam> tags, replace them with ? placeholders,
/// and collect structured parameter info.
fn scan_cfqueryparam_tags(sql_body: &str) -> (String, Vec<CfQueryParam>) {
    let mut result = String::with_capacity(sql_body.len());
    let mut params = Vec::new();
    let chars: Vec<char> = sql_body.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Look for <cfqueryparam
        if i + 14 < len && chars[i] == '<' {
            let ahead: String = chars[i..std::cmp::min(i + 14, len)].iter().collect();
            if ahead.to_lowercase().starts_with("<cfqueryparam") {
                // Check if followed by space or > (not a different tag)
                let next_after = chars.get(i + 13);
                if next_after == Some(&' ') || next_after == Some(&'>') || next_after == Some(&'/') || next_after == Some(&'\t') || next_after == Some(&'\n') {
                    // Parse the tag attributes
                    let name_end = i + 13; // after "cfqueryparam"
                    let (tag_attrs, _) = parse_tag_attributes(&chars, name_end, len);

                    // Extract cfqueryparam attributes
                    let value_raw = tag_attrs.get("value").cloned().unwrap_or_default();
                    let cfsqltype = tag_attrs.get("cfsqltype").cloned()
                        .unwrap_or_else(|| "cf_sql_varchar".to_string());
                    let null = tag_attrs.get("null")
                        .map(|v| v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
                        .unwrap_or(false);
                    let list = tag_attrs.get("list")
                        .map(|v| v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
                        .unwrap_or(false);
                    let separator = tag_attrs.get("separator").cloned().unwrap_or_else(|| ",".to_string());

                    // Convert value to script expression
                    let value_expr = if null {
                        "\"\"".to_string()
                    } else {
                        let stripped = strip_hashes(&value_raw);
                        if stripped != value_raw {
                            // Had hashes — it's a variable reference
                            stripped
                        } else if value_raw.is_empty() {
                            "\"\"".to_string()
                        } else {
                            // Literal string value
                            format!("\"{}\"", value_raw.replace('"', "\\\""))
                        }
                    };

                    params.push(CfQueryParam {
                        value_expr,
                        cfsqltype,
                        null,
                        list,
                        separator,
                    });

                    // Replace with ? placeholder
                    result.push('?');

                    // Skip to end of <cfqueryparam> tag
                    while i < len && chars[i] != '>' {
                        i += 1;
                    }
                    if i < len {
                        i += 1; // skip >
                    }
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    (result, params)
}

/// Process hash expressions in SQL for cfquery.
/// Converts #var# to string concatenation: `"..." & var & "..."`
/// Returns a script expression that builds the final SQL string.
fn process_sql_hashes(sql: &str) -> String {
    let sql = sql.trim().replace('\n', " ").replace('\r', "");

    if !sql.contains('#') {
        // No hash expressions — simple string literal
        return format!("\"{}\"", sql.replace('"', "\\\""));
    }

    // Split on hash pairs and build concatenation
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut parts: Vec<String> = Vec::new();
    let mut current_text = String::new();
    let mut i = 0;

    while i < len {
        if chars[i] == '#' {
            // Look for closing #
            if let Some(end_offset) = chars[i + 1..].iter().position(|&c| c == '#') {
                let end = i + 1 + end_offset;
                // Flush current text
                if !current_text.is_empty() {
                    parts.push(format!("\"{}\"", current_text.replace('"', "\\\"")));
                    current_text = String::new();
                }
                // Extract expression
                let expr: String = chars[i + 1..end].iter().collect();
                parts.push(expr);
                i = end + 1;
                continue;
            }
        }
        current_text.push(chars[i]);
        i += 1;
    }

    // Flush remaining text
    if !current_text.is_empty() {
        parts.push(format!("\"{}\"", current_text.replace('"', "\\\"")));
    }

    if parts.len() == 1 {
        parts.into_iter().next().unwrap()
    } else {
        parts.join(" & ")
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
