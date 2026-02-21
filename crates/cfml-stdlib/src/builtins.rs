//! CFML Built-in Functions - Standard Library
//!
//! Implements the core CFML built-in function library including:
//! - String functions
//! - Array functions
//! - Struct functions
//! - Math functions
//! - Date/Time functions
//! - Type checking functions
//! - Conversion functions
//! - List functions
//! - JSON functions
//! - Output functions
//! - Query functions
//! - System functions

use cfml_common::dynamic::{CfmlAccess, CfmlClosureBody, CfmlFunction, CfmlQuery, CfmlValue};
use cfml_common::vm::{CfmlError, CfmlResult};
use std::collections::HashMap;
use regex::Regex;

pub type BuiltinFunction = fn(Vec<CfmlValue>) -> CfmlResult;

/// Returns all builtin functions as CfmlValue::Function references for the globals table
pub fn get_builtins() -> HashMap<String, CfmlValue> {
    let mut builtins = HashMap::new();
    for (name, _) in get_builtin_functions() {
        builtins.insert(name.clone(), create_builtin_func(name.as_str()));
    }
    builtins
}

/// Returns all builtin function implementations
pub fn get_builtin_functions() -> HashMap<String, BuiltinFunction> {
    let mut f: HashMap<String, BuiltinFunction> = HashMap::new();

    // ---- Output functions ----
    f.insert("writeOutput".into(), write_output);
    f.insert("writeDump".into(), write_dump);
    f.insert("dump".into(), write_dump);

    // ---- String functions ----
    f.insert("len".into(), fn_len);
    f.insert("ucase".into(), fn_ucase);
    f.insert("lcase".into(), fn_lcase);
    f.insert("trim".into(), fn_trim);
    f.insert("ltrim".into(), fn_ltrim);
    f.insert("rtrim".into(), fn_rtrim);
    f.insert("replace".into(), fn_replace);
    f.insert("replaceNoCase".into(), fn_replace_no_case);
    f.insert("find".into(), fn_find);
    f.insert("findNoCase".into(), fn_find_no_case);
    f.insert("findOneOf".into(), fn_find_one_of);
    f.insert("mid".into(), fn_mid);
    f.insert("left".into(), fn_left);
    f.insert("right".into(), fn_right);
    f.insert("reverse".into(), fn_reverse);
    f.insert("repeatString".into(), fn_repeat_string);
    f.insert("insert".into(), fn_insert);
    f.insert("removeChars".into(), fn_remove_chars);
    f.insert("spanIncluding".into(), fn_span_including);
    f.insert("spanExcluding".into(), fn_span_excluding);
    f.insert("compare".into(), fn_compare);
    f.insert("compareNoCase".into(), fn_compare_no_case);
    f.insert("asc".into(), fn_asc);
    f.insert("chr".into(), fn_chr);
    f.insert("reFind".into(), fn_re_find);
    f.insert("reFindNoCase".into(), fn_re_find_no_case);
    f.insert("reReplace".into(), fn_re_replace);
    f.insert("reReplaceNoCase".into(), fn_re_replace_no_case);
    f.insert("reMatch".into(), fn_re_match);
    f.insert("reMatchNoCase".into(), fn_re_match_no_case);
    f.insert("wrap".into(), fn_wrap);
    f.insert("stripCr".into(), fn_strip_cr);
    f.insert("toBase64".into(), fn_to_base64);
    f.insert("toBinary".into(), fn_to_binary);
    f.insert("urlEncodedFormat".into(), fn_url_encode);
    f.insert("urlDecode".into(), fn_url_decode);
    f.insert("htmlEditFormat".into(), fn_html_edit_format);
    f.insert("htmlCodeFormat".into(), fn_html_code_format);
    f.insert("encodeForHTML".into(), fn_html_edit_format);
    f.insert("lJustify".into(), fn_ljustify);
    f.insert("rJustify".into(), fn_rjustify);
    f.insert("numberFormat".into(), fn_number_format);
    f.insert("decimalFormat".into(), fn_decimal_format);
    f.insert("formatBaseN".into(), fn_format_base_n);
    f.insert("inputBaseN".into(), fn_input_base_n);

    // ---- Array functions ----
    f.insert("arrayNew".into(), fn_array_new);
    f.insert("arrayLen".into(), fn_array_len);
    f.insert("arrayAppend".into(), fn_array_append);
    f.insert("arrayPrepend".into(), fn_array_prepend);
    f.insert("arrayDeleteAt".into(), fn_array_delete_at);
    f.insert("arrayInsertAt".into(), fn_array_insert_at);
    f.insert("arrayContains".into(), fn_array_contains);
    f.insert("arrayContainsNoCase".into(), fn_array_contains_no_case);
    f.insert("arrayFind".into(), fn_array_find);
    f.insert("arrayFindNoCase".into(), fn_array_find_no_case);
    f.insert("arraySort".into(), fn_array_sort);
    f.insert("arrayReverse".into(), fn_array_reverse);
    f.insert("arraySlice".into(), fn_array_slice);
    f.insert("arrayToList".into(), fn_array_to_list);
    f.insert("arrayMerge".into(), fn_array_merge);
    f.insert("arrayClear".into(), fn_array_clear);
    f.insert("arrayIsDefined".into(), fn_array_is_defined);
    f.insert("arraySet".into(), fn_array_set);
    f.insert("arraySwap".into(), fn_array_swap);
    f.insert("arrayMin".into(), fn_array_min);
    f.insert("arrayMax".into(), fn_array_max);
    f.insert("arrayAvg".into(), fn_array_avg);
    f.insert("arraySum".into(), fn_array_sum);
    f.insert("arrayMap".into(), fn_array_map);
    f.insert("arrayFilter".into(), fn_array_filter);
    f.insert("arrayReduce".into(), fn_array_reduce);
    f.insert("arrayEach".into(), fn_array_each);
    f.insert("isArray".into(), fn_is_array);

    // ---- Struct functions ----
    f.insert("structNew".into(), fn_struct_new);
    f.insert("structCount".into(), fn_struct_count);
    f.insert("structKeyExists".into(), fn_struct_key_exists);
    f.insert("structKeyList".into(), fn_struct_key_list);
    f.insert("structKeyArray".into(), fn_struct_key_array);
    f.insert("structDelete".into(), fn_struct_delete);
    f.insert("structInsert".into(), fn_struct_insert);
    f.insert("structUpdate".into(), fn_struct_update);
    f.insert("structFind".into(), fn_struct_find);
    f.insert("structFindKey".into(), fn_struct_find_key);
    f.insert("structFindValue".into(), fn_struct_find_value);
    f.insert("structClear".into(), fn_struct_clear);
    f.insert("structCopy".into(), fn_struct_copy);
    f.insert("structAppend".into(), fn_struct_append);
    f.insert("structIsEmpty".into(), fn_struct_is_empty);
    f.insert("structSort".into(), fn_struct_sort);
    f.insert("structEach".into(), fn_struct_each);
    f.insert("structMap".into(), fn_struct_map);
    f.insert("structFilter".into(), fn_struct_filter);
    f.insert("isStruct".into(), fn_is_struct);

    // ---- Type checking functions ----
    f.insert("isNull".into(), fn_is_null);
    f.insert("isDefined".into(), fn_is_defined);
    f.insert("isSimpleValue".into(), fn_is_simple_value);
    f.insert("isNumeric".into(), fn_is_numeric);
    f.insert("isBoolean".into(), fn_is_boolean);
    f.insert("isDate".into(), fn_is_date);
    f.insert("isQuery".into(), fn_is_query);
    f.insert("isObject".into(), fn_is_object);
    f.insert("isBinary".into(), fn_is_binary);
    f.insert("isCustomFunction".into(), fn_is_custom_function);
    f.insert("isClosure".into(), fn_is_closure);
    f.insert("isValid".into(), fn_is_valid);

    // ---- Conversion functions ----
    f.insert("toString".into(), fn_to_string);
    f.insert("toNumeric".into(), fn_to_numeric);
    f.insert("toBoolean".into(), fn_to_boolean);
    f.insert("val".into(), fn_val);
    f.insert("int".into(), fn_int);
    f.insert("javacast".into(), fn_java_cast);

    // ---- Math functions ----
    f.insert("abs".into(), fn_abs);
    f.insert("ceiling".into(), fn_ceiling);
    f.insert("floor".into(), fn_floor);
    f.insert("round".into(), fn_round);
    f.insert("rand".into(), fn_rand);
    f.insert("randRange".into(), fn_rand_range);
    f.insert("randomize".into(), fn_randomize);
    f.insert("max".into(), fn_max);
    f.insert("min".into(), fn_min);
    f.insert("sqr".into(), fn_sqr);
    f.insert("sqrt".into(), fn_sqr);
    f.insert("exp".into(), fn_exp);
    f.insert("log".into(), fn_log);
    f.insert("log10".into(), fn_log10);
    f.insert("sin".into(), fn_sin);
    f.insert("cos".into(), fn_cos);
    f.insert("tan".into(), fn_tan);
    f.insert("asin".into(), fn_asin);
    f.insert("acos".into(), fn_acos);
    f.insert("atan".into(), fn_atan);
    f.insert("pi".into(), fn_pi);
    f.insert("sgn".into(), fn_sgn);
    f.insert("fix".into(), fn_fix);
    f.insert("pow".into(), fn_pow);
    f.insert("bitAnd".into(), fn_bit_and);
    f.insert("bitOr".into(), fn_bit_or);
    f.insert("bitXor".into(), fn_bit_xor);
    f.insert("bitNot".into(), fn_bit_not);
    f.insert("bitSHLN".into(), fn_bit_shln);
    f.insert("bitSHRN".into(), fn_bit_shrn);

    // ---- Date/Time functions ----
    f.insert("now".into(), fn_now);
    f.insert("createDate".into(), fn_create_date);
    f.insert("createDateTime".into(), fn_create_date_time);
    f.insert("createODBCDate".into(), fn_create_odbc_date);
    f.insert("createODBCDateTime".into(), fn_create_odbc_date_time);
    f.insert("dateAdd".into(), fn_date_add);
    f.insert("dateDiff".into(), fn_date_diff);
    f.insert("dateFormat".into(), fn_date_format);
    f.insert("timeFormat".into(), fn_time_format);
    f.insert("dateTimeFormat".into(), fn_date_time_format);
    f.insert("year".into(), fn_year);
    f.insert("month".into(), fn_month);
    f.insert("day".into(), fn_day);
    f.insert("hour".into(), fn_hour);
    f.insert("minute".into(), fn_minute);
    f.insert("second".into(), fn_second);
    f.insert("dayOfWeek".into(), fn_day_of_week);
    f.insert("dayOfWeekAsString".into(), fn_day_of_week_as_string);
    f.insert("dayOfYear".into(), fn_day_of_year);
    f.insert("daysInMonth".into(), fn_days_in_month);
    f.insert("daysInYear".into(), fn_days_in_year);
    f.insert("firstDayOfMonth".into(), fn_first_day_of_month);
    f.insert("isLeapYear".into(), fn_is_leap_year);
    f.insert("monthAsString".into(), fn_month_as_string);
    f.insert("quarter".into(), fn_quarter);
    f.insert("week".into(), fn_week);
    f.insert("getTickCount".into(), fn_get_tick_count);

    // ---- List functions ----
    f.insert("listNew".into(), fn_list_new);
    f.insert("listLen".into(), fn_list_len);
    f.insert("listAppend".into(), fn_list_append);
    f.insert("listPrepend".into(), fn_list_prepend);
    f.insert("listGetAt".into(), fn_list_get_at);
    f.insert("listSetAt".into(), fn_list_set_at);
    f.insert("listInsertAt".into(), fn_list_insert_at);
    f.insert("listDeleteAt".into(), fn_list_delete_at);
    f.insert("listFind".into(), fn_list_find);
    f.insert("listFindNoCase".into(), fn_list_find_no_case);
    f.insert("listContains".into(), fn_list_contains);
    f.insert("listContainsNoCase".into(), fn_list_contains_no_case);
    f.insert("listSort".into(), fn_list_sort);
    f.insert("listToArray".into(), fn_list_to_array);
    f.insert("listFirst".into(), fn_list_first);
    f.insert("listLast".into(), fn_list_last);
    f.insert("listRest".into(), fn_list_rest);
    f.insert("listRemoveDuplicates".into(), fn_list_remove_duplicates);
    f.insert("listValueCount".into(), fn_list_value_count);
    f.insert("listValueCountNoCase".into(), fn_list_value_count_no_case);
    f.insert("listChangeDelims".into(), fn_list_change_delims);
    f.insert("listQualify".into(), fn_list_qualify);
    f.insert("listCompact".into(), fn_list_compact);

    // ---- JSON functions ----
    f.insert("serializeJSON".into(), fn_serialize_json);
    f.insert("deserializeJSON".into(), fn_deserialize_json);
    f.insert("isJSON".into(), fn_is_json);

    // ---- Query functions ----
    f.insert("queryNew".into(), fn_query_new);
    f.insert("queryAddRow".into(), fn_query_add_row);
    f.insert("querySetCell".into(), fn_query_set_cell);
    f.insert("queryAddColumn".into(), fn_query_add_column);

    // ---- Utility functions ----
    f.insert("evaluate".into(), fn_evaluate);
    f.insert("iif".into(), fn_iif);
    f.insert("duplicate".into(), fn_duplicate);
    f.insert("sleep".into(), fn_sleep);
    f.insert("getMetadata".into(), fn_get_metadata);
    f.insert("createUUID".into(), fn_create_uuid);
    f.insert("createGUID".into(), fn_create_uuid);
    f.insert("hash".into(), fn_hash);
    f.insert("lsParseNumber".into(), fn_ls_parse_number);

    // ---- System functions ----
    f.insert("getTickCount".into(), fn_get_tick_count);

    // ---- File I/O functions ----
    f.insert("fileRead".into(), fn_file_read);
    f.insert("fileWrite".into(), fn_file_write);
    f.insert("fileAppend".into(), fn_file_append);
    f.insert("fileExists".into(), fn_file_exists);
    f.insert("fileDelete".into(), fn_file_delete);
    f.insert("fileMove".into(), fn_file_move);
    f.insert("fileCopy".into(), fn_file_copy);
    f.insert("directoryCreate".into(), fn_directory_create);
    f.insert("directoryExists".into(), fn_directory_exists);
    f.insert("directoryDelete".into(), fn_directory_delete);
    f.insert("directoryList".into(), fn_directory_list);
    f.insert("getTempDirectory".into(), fn_get_temp_directory);
    f.insert("getTempFile".into(), fn_get_temp_file);
    f.insert("getFileInfo".into(), fn_get_file_info);
    f.insert("expandPath".into(), fn_expand_path);

    // ---- Additional builtins ----
    f.insert("encodeForURL".into(), fn_encode_for_url);
    f.insert("encodeForCSS".into(), fn_encode_for_css);
    f.insert("encodeForJavaScript".into(), fn_encode_for_javascript);
    f.insert("listReduce".into(), fn_list_reduce);
    f.insert("arrayPop".into(), fn_array_pop);
    f.insert("arrayShift".into(), fn_array_shift);

    // ---- HTTP functions ----
    #[cfg(feature = "http")]
    f.insert("cfhttp".into(), fn_cfhttp);

    // ---- Database functions ----
    #[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db"))]
    f.insert("queryExecute".into(), fn_query_execute);

    f
}

fn create_builtin_func(name: &str) -> CfmlValue {
    CfmlValue::Function(CfmlFunction {
        name: name.to_string(),
        params: Vec::new(),
        body: CfmlClosureBody::Expression(Box::new(CfmlValue::Null)),
        return_type: None,
        access: CfmlAccess::Public,
    })
}

// ---- Helper functions ----

fn get_arg(args: &[CfmlValue], idx: usize) -> &CfmlValue {
    args.get(idx).unwrap_or(&CfmlValue::Null)
}

fn get_str(args: &[CfmlValue], idx: usize) -> String {
    args.get(idx).map(|v| v.as_string()).unwrap_or_default()
}

fn get_int(args: &[CfmlValue], idx: usize) -> i64 {
    match args.get(idx) {
        Some(CfmlValue::Int(i)) => *i,
        Some(CfmlValue::Double(d)) => *d as i64,
        Some(CfmlValue::String(s)) => s.parse().unwrap_or(0),
        Some(CfmlValue::Bool(b)) => if *b { 1 } else { 0 },
        _ => 0,
    }
}

fn get_float(args: &[CfmlValue], idx: usize) -> f64 {
    match args.get(idx) {
        Some(CfmlValue::Int(i)) => *i as f64,
        Some(CfmlValue::Double(d)) => *d,
        Some(CfmlValue::String(s)) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn get_delimiter(args: &[CfmlValue], idx: usize) -> String {
    args.get(idx)
        .map(|v| v.as_string())
        .unwrap_or_else(|| ",".to_string())
}

fn pseudo_random() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Simple LCG
    let x = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) as u64;
    (x >> 11) as f64 / (1u64 << 53) as f64
}

// ===============================================
// OUTPUT FUNCTIONS
// ===============================================

fn write_output(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(val) = args.first() {
        print!("{}", val.as_string());
    }
    Ok(CfmlValue::Null)
}

fn write_dump(args: Vec<CfmlValue>) -> CfmlResult {
    for arg in &args {
        println!("{:?}", arg);
    }
    Ok(CfmlValue::Null)
}

// ===============================================
// STRING FUNCTIONS
// ===============================================

fn fn_len(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::String(s)) => Ok(CfmlValue::Int(s.len() as i64)),
        Some(CfmlValue::Array(a)) => Ok(CfmlValue::Int(a.len() as i64)),
        Some(CfmlValue::Struct(s)) => Ok(CfmlValue::Int(s.len() as i64)),
        _ => Ok(CfmlValue::Int(0)),
    }
}

fn fn_ucase(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(get_str(&args, 0).to_uppercase()))
}

fn fn_lcase(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(get_str(&args, 0).to_lowercase()))
}

fn fn_trim(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(get_str(&args, 0).trim().to_string()))
}

fn fn_ltrim(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(get_str(&args, 0).trim_start().to_string()))
}

fn fn_rtrim(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(get_str(&args, 0).trim_end().to_string()))
}

fn fn_replace(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        let string = get_str(&args, 0);
        let find = get_str(&args, 1);
        let replace_with = get_str(&args, 2);
        let scope = get_str(&args, 3).to_lowercase();
        if scope == "all" {
            Ok(CfmlValue::String(string.replace(&find, &replace_with)))
        } else {
            Ok(CfmlValue::String(string.replacen(&find, &replace_with, 1)))
        }
    } else {
        Ok(CfmlValue::String(get_str(&args, 0)))
    }
}

fn fn_replace_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        let string = get_str(&args, 0);
        let find = get_str(&args, 1).to_lowercase();
        let replace_with = get_str(&args, 2);
        let lower = string.to_lowercase();
        if let Some(pos) = lower.find(&find) {
            let mut result = String::new();
            result.push_str(&string[..pos]);
            result.push_str(&replace_with);
            result.push_str(&string[pos + find.len()..]);
            Ok(CfmlValue::String(result))
        } else {
            Ok(CfmlValue::String(string))
        }
    } else {
        Ok(CfmlValue::String(get_str(&args, 0)))
    }
}

fn fn_find(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        let substring = get_str(&args, 0);
        let string = get_str(&args, 1);
        let start = if args.len() >= 3 { get_int(&args, 2).max(1) as usize - 1 } else { 0 };
        if start < string.len() {
            if let Some(pos) = string[start..].find(&substring) {
                return Ok(CfmlValue::Int((pos + start + 1) as i64));
            }
        }
        Ok(CfmlValue::Int(0))
    } else {
        Ok(CfmlValue::Int(0))
    }
}

fn fn_find_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        let substring = get_str(&args, 0).to_lowercase();
        let string = get_str(&args, 1).to_lowercase();
        let start = if args.len() >= 3 { get_int(&args, 2).max(1) as usize - 1 } else { 0 };
        if start < string.len() {
            if let Some(pos) = string[start..].find(&substring) {
                return Ok(CfmlValue::Int((pos + start + 1) as i64));
            }
        }
        Ok(CfmlValue::Int(0))
    } else {
        Ok(CfmlValue::Int(0))
    }
}

fn fn_find_one_of(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        let chars = get_str(&args, 0);
        let string = get_str(&args, 1);
        for (i, c) in string.chars().enumerate() {
            if chars.contains(c) {
                return Ok(CfmlValue::Int((i + 1) as i64));
            }
        }
        Ok(CfmlValue::Int(0))
    } else {
        Ok(CfmlValue::Int(0))
    }
}

fn fn_mid(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        let string = get_str(&args, 0);
        let start = (get_int(&args, 1).max(1) as usize).saturating_sub(1);
        let length = get_int(&args, 2).max(0) as usize;
        let chars: Vec<char> = string.chars().collect();
        if start >= chars.len() {
            return Ok(CfmlValue::String(String::new()));
        }
        let end = (start + length).min(chars.len());
        Ok(CfmlValue::String(chars[start..end].iter().collect()))
    } else if args.len() >= 2 {
        let string = get_str(&args, 0);
        let start = (get_int(&args, 1).max(1) as usize).saturating_sub(1);
        let chars: Vec<char> = string.chars().collect();
        if start >= chars.len() {
            return Ok(CfmlValue::String(String::new()));
        }
        Ok(CfmlValue::String(chars[start..].iter().collect()))
    } else {
        Ok(CfmlValue::String(String::new()))
    }
}

fn fn_left(args: Vec<CfmlValue>) -> CfmlResult {
    let string = get_str(&args, 0);
    let count = get_int(&args, 1).max(0) as usize;
    let chars: Vec<char> = string.chars().collect();
    Ok(CfmlValue::String(chars[..count.min(chars.len())].iter().collect()))
}

fn fn_right(args: Vec<CfmlValue>) -> CfmlResult {
    let string = get_str(&args, 0);
    let count = get_int(&args, 1).max(0) as usize;
    let chars: Vec<char> = string.chars().collect();
    let start = chars.len().saturating_sub(count);
    Ok(CfmlValue::String(chars[start..].iter().collect()))
}

fn fn_reverse(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(get_str(&args, 0).chars().rev().collect()))
}

fn fn_repeat_string(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let count = get_int(&args, 1).max(0) as usize;
    Ok(CfmlValue::String(s.repeat(count)))
}

fn fn_insert(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        let substring = get_str(&args, 0);
        let string = get_str(&args, 1);
        let pos = get_int(&args, 2).max(0) as usize;
        let mut result = string.clone();
        if pos <= result.len() {
            result.insert_str(pos, &substring);
        }
        Ok(CfmlValue::String(result))
    } else {
        Ok(CfmlValue::String(get_str(&args, 0)))
    }
}

fn fn_remove_chars(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        let string = get_str(&args, 0);
        let start = (get_int(&args, 1).max(1) as usize).saturating_sub(1);
        let count = get_int(&args, 2).max(0) as usize;
        let mut chars: Vec<char> = string.chars().collect();
        let end = (start + count).min(chars.len());
        chars.drain(start..end);
        Ok(CfmlValue::String(chars.into_iter().collect()))
    } else {
        Ok(CfmlValue::String(get_str(&args, 0)))
    }
}

fn fn_span_including(args: Vec<CfmlValue>) -> CfmlResult {
    let string = get_str(&args, 0);
    let chars_set = get_str(&args, 1);
    let result: String = string.chars().take_while(|c| chars_set.contains(*c)).collect();
    Ok(CfmlValue::String(result))
}

fn fn_span_excluding(args: Vec<CfmlValue>) -> CfmlResult {
    let string = get_str(&args, 0);
    let chars_set = get_str(&args, 1);
    let result: String = string.chars().take_while(|c| !chars_set.contains(*c)).collect();
    Ok(CfmlValue::String(result))
}

fn fn_compare(args: Vec<CfmlValue>) -> CfmlResult {
    let a = get_str(&args, 0);
    let b = get_str(&args, 1);
    Ok(CfmlValue::Int(match a.cmp(&b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }))
}

fn fn_compare_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    let a = get_str(&args, 0).to_lowercase();
    let b = get_str(&args, 1).to_lowercase();
    Ok(CfmlValue::Int(match a.cmp(&b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }))
}

fn fn_asc(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    Ok(CfmlValue::Int(s.chars().next().map_or(0, |c| c as i64)))
}

fn fn_chr(args: Vec<CfmlValue>) -> CfmlResult {
    let code = get_int(&args, 0) as u32;
    Ok(CfmlValue::String(
        char::from_u32(code).map_or(String::new(), |c| c.to_string()),
    ))
}

fn fn_re_find(args: Vec<CfmlValue>) -> CfmlResult {
    re_find_impl(args, false)
}

fn fn_re_find_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    re_find_impl(args, true)
}

fn re_find_impl(args: Vec<CfmlValue>, case_insensitive: bool) -> CfmlResult {
    if args.len() < 2 {
        return Ok(CfmlValue::Int(0));
    }
    let pattern = get_str(&args, 0);
    let string = get_str(&args, 1);
    let start = if args.len() >= 3 { (get_int(&args, 2).max(1) as usize).saturating_sub(1) } else { 0 };
    let return_sub = if args.len() >= 4 { args[3].is_true() } else { false };

    let pat = if case_insensitive { format!("(?i){}", pattern) } else { pattern };
    let re = match Regex::new(&pat) {
        Ok(r) => r,
        Err(_) => return Ok(CfmlValue::Int(0)),
    };

    let search_str = if start < string.len() { &string[start..] } else { "" };

    if return_sub {
        if let Some(caps) = re.captures(search_str) {
            let mut pos_arr = Vec::new();
            let mut match_arr = Vec::new();
            let mut len_arr = Vec::new();
            for i in 0..caps.len() {
                if let Some(m) = caps.get(i) {
                    pos_arr.push(CfmlValue::Int((m.start() + start + 1) as i64));
                    match_arr.push(CfmlValue::String(m.as_str().to_string()));
                    len_arr.push(CfmlValue::Int(m.len() as i64));
                } else {
                    pos_arr.push(CfmlValue::Int(0));
                    match_arr.push(CfmlValue::String(String::new()));
                    len_arr.push(CfmlValue::Int(0));
                }
            }
            let mut result = HashMap::new();
            result.insert("POS".to_string(), CfmlValue::Array(pos_arr));
            result.insert("MATCH".to_string(), CfmlValue::Array(match_arr));
            result.insert("LEN".to_string(), CfmlValue::Array(len_arr));
            Ok(CfmlValue::Struct(result))
        } else {
            let mut result = HashMap::new();
            result.insert("POS".to_string(), CfmlValue::Array(vec![CfmlValue::Int(0)]));
            result.insert("MATCH".to_string(), CfmlValue::Array(vec![CfmlValue::String(String::new())]));
            result.insert("LEN".to_string(), CfmlValue::Array(vec![CfmlValue::Int(0)]));
            Ok(CfmlValue::Struct(result))
        }
    } else {
        match re.find(search_str) {
            Some(m) => Ok(CfmlValue::Int((m.start() + start + 1) as i64)),
            None => Ok(CfmlValue::Int(0)),
        }
    }
}

fn fn_re_replace(args: Vec<CfmlValue>) -> CfmlResult {
    re_replace_impl(args, false)
}

fn fn_re_replace_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    re_replace_impl(args, true)
}

fn re_replace_impl(args: Vec<CfmlValue>, case_insensitive: bool) -> CfmlResult {
    if args.len() < 3 {
        return Ok(CfmlValue::String(get_str(&args, 0)));
    }
    let string = get_str(&args, 0);
    let pattern = get_str(&args, 1);
    let replacement = get_str(&args, 2);
    let scope = get_str(&args, 3).to_lowercase();

    let pat = if case_insensitive { format!("(?i){}", pattern) } else { pattern };
    let re = match Regex::new(&pat) {
        Ok(r) => r,
        Err(_) => return Ok(CfmlValue::String(string)),
    };

    if scope == "all" {
        Ok(CfmlValue::String(re.replace_all(&string, replacement.as_str()).to_string()))
    } else {
        Ok(CfmlValue::String(re.replace(&string, replacement.as_str()).to_string()))
    }
}

fn fn_re_match(args: Vec<CfmlValue>) -> CfmlResult {
    re_match_impl(args, false)
}

fn fn_re_match_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    re_match_impl(args, true)
}

fn re_match_impl(args: Vec<CfmlValue>, case_insensitive: bool) -> CfmlResult {
    if args.len() < 2 {
        return Ok(CfmlValue::Array(Vec::new()));
    }
    // reMatch(regex, string) - regex is first arg
    let pattern = get_str(&args, 0);
    let string = get_str(&args, 1);

    let pat = if case_insensitive { format!("(?i){}", pattern) } else { pattern };
    let re = match Regex::new(&pat) {
        Ok(r) => r,
        Err(_) => return Ok(CfmlValue::Array(Vec::new())),
    };

    let matches: Vec<CfmlValue> = re.find_iter(&string)
        .map(|m| CfmlValue::String(m.as_str().to_string()))
        .collect();
    Ok(CfmlValue::Array(matches))
}

fn fn_wrap(args: Vec<CfmlValue>) -> CfmlResult {
    let string = get_str(&args, 0);
    let limit = get_int(&args, 1).max(1) as usize;
    let mut result = String::new();
    let mut col = 0;
    for word in string.split_whitespace() {
        if col + word.len() > limit && col > 0 {
            result.push('\n');
            col = 0;
        }
        if col > 0 {
            result.push(' ');
            col += 1;
        }
        result.push_str(word);
        col += word.len();
    }
    Ok(CfmlValue::String(result))
}

fn fn_strip_cr(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(get_str(&args, 0).replace('\r', "")))
}

fn fn_to_base64(args: Vec<CfmlValue>) -> CfmlResult {
    // Simple base64 encoding
    use std::fmt::Write;
    let input = get_str(&args, 0);
    let bytes = input.as_bytes();
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(alphabet[((n >> 18) & 63) as usize] as char);
        result.push(alphabet[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 { result.push(alphabet[((n >> 6) & 63) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(alphabet[(n & 63) as usize] as char); } else { result.push('='); }
    }
    Ok(CfmlValue::String(result))
}

fn fn_to_binary(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    Ok(CfmlValue::Binary(s.into_bytes()))
}

fn fn_url_encode(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push('+'),
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    Ok(CfmlValue::String(result))
}

fn fn_url_decode(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            }
            '+' => result.push(' '),
            _ => result.push(c),
        }
    }
    Ok(CfmlValue::String(result))
}

fn fn_html_edit_format(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    Ok(CfmlValue::String(
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;"),
    ))
}

fn fn_html_code_format(args: Vec<CfmlValue>) -> CfmlResult {
    let inner = fn_html_edit_format(args)?;
    Ok(CfmlValue::String(format!("<pre>{}</pre>", inner.as_string())))
}

fn fn_ljustify(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let length = get_int(&args, 1).max(0) as usize;
    Ok(CfmlValue::String(format!("{:<width$}", s, width = length)))
}

fn fn_rjustify(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let length = get_int(&args, 1).max(0) as usize;
    Ok(CfmlValue::String(format!("{:>width$}", s, width = length)))
}

fn fn_number_format(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_float(&args, 0);
    let mask = get_str(&args, 1);
    if mask.is_empty() {
        Ok(CfmlValue::String(format!("{:.0}", n)))
    } else {
        let decimals = mask.find('.').map(|p| mask.len() - p - 1).unwrap_or(0);
        Ok(CfmlValue::String(format!("{:.prec$}", n, prec = decimals)))
    }
}

fn fn_decimal_format(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_float(&args, 0);
    Ok(CfmlValue::String(format!("{:.2}", n)))
}

fn fn_format_base_n(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_int(&args, 0);
    let radix = get_int(&args, 1) as u32;
    match radix {
        2 => Ok(CfmlValue::String(format!("{:b}", n))),
        8 => Ok(CfmlValue::String(format!("{:o}", n))),
        16 => Ok(CfmlValue::String(format!("{:X}", n))),
        _ => Ok(CfmlValue::String(n.to_string())),
    }
}

fn fn_input_base_n(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let radix = get_int(&args, 1) as u32;
    Ok(CfmlValue::Int(
        i64::from_str_radix(&s, radix).unwrap_or(0),
    ))
}

// ===============================================
// ARRAY FUNCTIONS
// ===============================================

fn fn_array_new(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Array(Vec::new()))
}

fn fn_array_len(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Array(a)) => Ok(CfmlValue::Int(a.len() as i64)),
        _ => Ok(CfmlValue::Int(0)),
    }
}

fn fn_array_append(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        let mut arr = match &args[0] {
            CfmlValue::Array(a) => a.clone(),
            _ => Vec::new(),
        };
        arr.push(args[1].clone());
        Ok(CfmlValue::Array(arr))
    } else {
        Ok(args.into_iter().next().unwrap_or(CfmlValue::Array(Vec::new())))
    }
}

fn fn_array_prepend(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        let mut arr = match &args[0] {
            CfmlValue::Array(a) => a.clone(),
            _ => Vec::new(),
        };
        arr.insert(0, args[1].clone());
        Ok(CfmlValue::Array(arr))
    } else {
        Ok(args.into_iter().next().unwrap_or(CfmlValue::Array(Vec::new())))
    }
}

fn fn_array_delete_at(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        let mut arr = match &args[0] {
            CfmlValue::Array(a) => a.clone(),
            _ => return Ok(CfmlValue::Bool(false)),
        };
        let idx = (get_int(&args, 1) as usize).saturating_sub(1);
        if idx < arr.len() {
            arr.remove(idx);
            Ok(CfmlValue::Array(arr))
        } else {
            Ok(CfmlValue::Array(arr))
        }
    } else {
        Ok(CfmlValue::Bool(false))
    }
}

fn fn_array_insert_at(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        let mut arr = match &args[0] {
            CfmlValue::Array(a) => a.clone(),
            _ => return Ok(CfmlValue::Bool(false)),
        };
        let idx = (get_int(&args, 1) as usize).saturating_sub(1);
        if idx <= arr.len() {
            arr.insert(idx, args[2].clone());
        }
        Ok(CfmlValue::Array(arr))
    } else {
        Ok(CfmlValue::Bool(false))
    }
}

fn fn_array_contains(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Array(arr) = &args[0] {
            let needle = args[1].as_string();
            return Ok(CfmlValue::Bool(arr.iter().any(|v| v.as_string() == needle)));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_array_contains_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Array(arr) = &args[0] {
            let needle = args[1].as_string().to_lowercase();
            return Ok(CfmlValue::Bool(
                arr.iter().any(|v| v.as_string().to_lowercase() == needle),
            ));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_array_find(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Array(arr) = &args[0] {
            let needle = args[1].as_string();
            for (i, v) in arr.iter().enumerate() {
                if v.as_string() == needle {
                    return Ok(CfmlValue::Int((i + 1) as i64));
                }
            }
        }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_array_find_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Array(arr) = &args[0] {
            let needle = args[1].as_string().to_lowercase();
            for (i, v) in arr.iter().enumerate() {
                if v.as_string().to_lowercase() == needle {
                    return Ok(CfmlValue::Int((i + 1) as i64));
                }
            }
        }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_array_sort(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let mut sorted = arr.clone();
        let sort_type = get_str(&args, 1).to_lowercase();
        match sort_type.as_str() {
            "numeric" => {
                sorted.sort_by(|a, b| {
                    let fa = get_float(&[a.clone()], 0);
                    let fb = get_float(&[b.clone()], 0);
                    fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            _ => {
                sorted.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
            }
        }
        Ok(CfmlValue::Array(sorted))
    } else {
        Ok(CfmlValue::Array(Vec::new()))
    }
}

fn fn_array_reverse(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let mut reversed = arr.clone();
        reversed.reverse();
        Ok(CfmlValue::Array(reversed))
    } else {
        Ok(CfmlValue::Array(Vec::new()))
    }
}

fn fn_array_slice(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let offset = (get_int(&args, 1).max(1) as usize).saturating_sub(1);
        let length = if args.len() >= 3 {
            get_int(&args, 2).max(0) as usize
        } else {
            arr.len().saturating_sub(offset)
        };
        let end = (offset + length).min(arr.len());
        if offset < arr.len() {
            Ok(CfmlValue::Array(arr[offset..end].to_vec()))
        } else {
            Ok(CfmlValue::Array(Vec::new()))
        }
    } else {
        Ok(CfmlValue::Array(Vec::new()))
    }
}

fn fn_array_to_list(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let delimiter = get_delimiter(&args, 1);
        let items: Vec<String> = arr.iter().map(|v| v.as_string()).collect();
        Ok(CfmlValue::String(items.join(&delimiter)))
    } else {
        Ok(CfmlValue::String(String::new()))
    }
}

fn fn_array_merge(args: Vec<CfmlValue>) -> CfmlResult {
    let mut result = Vec::new();
    for arg in &args {
        if let CfmlValue::Array(arr) = arg {
            result.extend(arr.clone());
        }
    }
    Ok(CfmlValue::Array(result))
}

fn fn_array_clear(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Array(Vec::new()))
}

fn fn_array_is_defined(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Array(arr) = &args[0] {
            let idx = get_int(&args, 1) as usize;
            return Ok(CfmlValue::Bool(idx >= 1 && idx <= arr.len()));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_array_set(args: Vec<CfmlValue>) -> CfmlResult {
    // arraySet(array, start, end, value)
    if args.len() >= 4 {
        if let CfmlValue::Array(arr) = &args[0] {
            let mut result = arr.clone();
            let start = (get_int(&args, 1) as usize).saturating_sub(1);
            let end = get_int(&args, 2) as usize;
            while result.len() < end {
                result.push(CfmlValue::Null);
            }
            for i in start..end.min(result.len()) {
                result[i] = args[3].clone();
            }
            return Ok(CfmlValue::Array(result));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_array_swap(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        if let CfmlValue::Array(arr) = &args[0] {
            let mut result = arr.clone();
            let i = (get_int(&args, 1) as usize).saturating_sub(1);
            let j = (get_int(&args, 2) as usize).saturating_sub(1);
            if i < result.len() && j < result.len() {
                result.swap(i, j);
            }
            return Ok(CfmlValue::Array(result));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_array_min(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let mut min = f64::INFINITY;
        for v in arr {
            let n = get_float(&[v.clone()], 0);
            if n < min { min = n; }
        }
        Ok(CfmlValue::Double(if min.is_infinite() { 0.0 } else { min }))
    } else {
        Ok(CfmlValue::Int(0))
    }
}

fn fn_array_max(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let mut max = f64::NEG_INFINITY;
        for v in arr {
            let n = get_float(&[v.clone()], 0);
            if n > max { max = n; }
        }
        Ok(CfmlValue::Double(if max.is_infinite() { 0.0 } else { max }))
    } else {
        Ok(CfmlValue::Int(0))
    }
}

fn fn_array_avg(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        if arr.is_empty() { return Ok(CfmlValue::Int(0)); }
        let sum: f64 = arr.iter().map(|v| get_float(&[v.clone()], 0)).sum();
        Ok(CfmlValue::Double(sum / arr.len() as f64))
    } else {
        Ok(CfmlValue::Int(0))
    }
}

fn fn_array_sum(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let sum: f64 = arr.iter().map(|v| get_float(&[v.clone()], 0)).sum();
        Ok(CfmlValue::Double(sum))
    } else {
        Ok(CfmlValue::Int(0))
    }
}

// Higher-order array functions (stubs - would need closure support in builtins)
fn fn_array_map(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(args.into_iter().next().unwrap_or(CfmlValue::Array(Vec::new())))
}
fn fn_array_filter(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(args.into_iter().next().unwrap_or(CfmlValue::Array(Vec::new())))
}
fn fn_array_reduce(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Null)
}
fn fn_array_each(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Null)
}

fn fn_is_array(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Array(_)))))
}

// ===============================================
// STRUCT FUNCTIONS
// ===============================================

fn fn_struct_new(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Struct(HashMap::new()))
}

fn fn_struct_count(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Struct(s)) => Ok(CfmlValue::Int(s.len() as i64)),
        _ => Ok(CfmlValue::Int(0)),
    }
}

fn fn_struct_key_exists(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Struct(s) = &args[0] {
            let key = args[1].as_string();
            return Ok(CfmlValue::Bool(
                s.contains_key(&key)
                    || s.contains_key(&key.to_uppercase())
                    || s.contains_key(&key.to_lowercase()),
            ));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_struct_key_list(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Struct(s)) = args.first() {
        let delimiter = get_delimiter(&args, 1);
        let keys: Vec<String> = s.keys().cloned().collect();
        Ok(CfmlValue::String(keys.join(&delimiter)))
    } else {
        Ok(CfmlValue::String(String::new()))
    }
}

fn fn_struct_key_array(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Struct(s)) = args.first() {
        let keys: Vec<CfmlValue> = s.keys().map(|k| CfmlValue::String(k.clone())).collect();
        Ok(CfmlValue::Array(keys))
    } else {
        Ok(CfmlValue::Array(Vec::new()))
    }
}

fn fn_struct_delete(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Struct(s) = &args[0] {
            let mut result = s.clone();
            let key = args[1].as_string();
            result.remove(&key);
            result.remove(&key.to_uppercase());
            return Ok(CfmlValue::Struct(result));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_struct_insert(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        if let CfmlValue::Struct(s) = &args[0] {
            let mut result = s.clone();
            result.insert(args[1].as_string(), args[2].clone());
            return Ok(CfmlValue::Struct(result));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_struct_update(args: Vec<CfmlValue>) -> CfmlResult {
    fn_struct_insert(args)
}

fn fn_struct_find(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Struct(s) = &args[0] {
            let key = args[1].as_string();
            return Ok(s.get(&key)
                .or_else(|| s.get(&key.to_uppercase()))
                .cloned()
                .unwrap_or(CfmlValue::Null));
        }
    }
    Ok(CfmlValue::Null)
}

fn fn_struct_find_key(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Array(Vec::new()))
}

fn fn_struct_find_value(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Array(Vec::new()))
}

fn fn_struct_clear(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Struct(HashMap::new()))
}

fn fn_struct_copy(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Struct(s)) => Ok(CfmlValue::Struct(s.clone())),
        _ => Ok(CfmlValue::Struct(HashMap::new())),
    }
}

fn fn_struct_append(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let (CfmlValue::Struct(a), CfmlValue::Struct(b)) = (&args[0], &args[1]) {
            let mut result = a.clone();
            for (k, v) in b {
                result.insert(k.clone(), v.clone());
            }
            return Ok(CfmlValue::Struct(result));
        }
    }
    Ok(args.into_iter().next().unwrap_or(CfmlValue::Struct(HashMap::new())))
}

fn fn_struct_is_empty(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Struct(s)) => Ok(CfmlValue::Bool(s.is_empty())),
        _ => Ok(CfmlValue::Bool(true)),
    }
}

fn fn_struct_sort(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Struct(s)) = args.first() {
        let mut keys: Vec<String> = s.keys().cloned().collect();
        keys.sort();
        Ok(CfmlValue::Array(keys.into_iter().map(CfmlValue::String).collect()))
    } else {
        Ok(CfmlValue::Array(Vec::new()))
    }
}

fn fn_struct_each(args: Vec<CfmlValue>) -> CfmlResult { Ok(CfmlValue::Null) }
fn fn_struct_map(args: Vec<CfmlValue>) -> CfmlResult { Ok(args.into_iter().next().unwrap_or(CfmlValue::Null)) }
fn fn_struct_filter(args: Vec<CfmlValue>) -> CfmlResult { Ok(args.into_iter().next().unwrap_or(CfmlValue::Null)) }

fn fn_is_struct(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Struct(_)))))
}

// ===============================================
// TYPE CHECKING FUNCTIONS
// ===============================================

fn fn_is_null(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), None | Some(CfmlValue::Null))))
}

fn fn_is_defined(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(!matches!(args.first(), None | Some(CfmlValue::Null))))
}

fn fn_is_simple_value(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(
        args.first(),
        Some(CfmlValue::Null | CfmlValue::Bool(_) | CfmlValue::Int(_) | CfmlValue::Double(_) | CfmlValue::String(_))
    )))
}

fn fn_is_numeric(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Int(_)) | Some(CfmlValue::Double(_)) => Ok(CfmlValue::Bool(true)),
        Some(CfmlValue::String(s)) => Ok(CfmlValue::Bool(s.trim().parse::<f64>().is_ok())),
        Some(CfmlValue::Bool(_)) => Ok(CfmlValue::Bool(true)),
        _ => Ok(CfmlValue::Bool(false)),
    }
}

fn fn_is_boolean(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Bool(_)) => Ok(CfmlValue::Bool(true)),
        Some(CfmlValue::String(s)) => {
            let lower = s.trim().to_lowercase();
            Ok(CfmlValue::Bool(matches!(lower.as_str(), "true" | "false" | "yes" | "no")))
        }
        Some(CfmlValue::Int(_)) | Some(CfmlValue::Double(_)) => Ok(CfmlValue::Bool(true)),
        _ => Ok(CfmlValue::Bool(false)),
    }
}

fn fn_is_date(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::String(s)) = args.first() {
        let result = !s.is_empty() && (s.contains('/') || s.contains('-'));
        Ok(CfmlValue::Bool(result))
    } else {
        Ok(CfmlValue::Bool(false))
    }
}

fn fn_is_query(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Query(_)))))
}

fn fn_is_object(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Component(_)))))
}

fn fn_is_binary(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Binary(_)))))
}

fn fn_is_custom_function(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Function(_)))))
}

fn fn_is_closure(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Closure(_)))))
}

fn fn_is_valid(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        let type_name = get_str(&args, 0).to_lowercase();
        let value = &args[1];
        match type_name.as_str() {
            "string" => Ok(CfmlValue::Bool(true)),
            "numeric" | "float" | "double" => fn_is_numeric(vec![value.clone()]),
            "integer" => {
                let s = value.as_string();
                Ok(CfmlValue::Bool(s.trim().parse::<i64>().is_ok()))
            }
            "boolean" => fn_is_boolean(vec![value.clone()]),
            "date" => fn_is_date(vec![value.clone()]),
            "array" => fn_is_array(vec![value.clone()]),
            "struct" => fn_is_struct(vec![value.clone()]),
            "email" => {
                let s = value.as_string();
                let re = Regex::new(r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$").unwrap();
                Ok(CfmlValue::Bool(re.is_match(&s)))
            }
            "url" => {
                let s = value.as_string().to_lowercase();
                Ok(CfmlValue::Bool(
                    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://")
                ))
            }
            "uuid" => {
                let s = value.as_string();
                let re = Regex::new(r"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}$").unwrap();
                Ok(CfmlValue::Bool(re.is_match(&s)))
            }
            "regex" => {
                let s = value.as_string();
                Ok(CfmlValue::Bool(Regex::new(&s).is_ok()))
            }
            "creditcard" => {
                let s: String = value.as_string().chars().filter(|c| c.is_ascii_digit()).collect();
                if s.len() < 13 || s.len() > 19 { return Ok(CfmlValue::Bool(false)); }
                // Luhn algorithm
                let mut sum = 0u32;
                let mut double = false;
                for c in s.chars().rev() {
                    let mut d = c.to_digit(10).unwrap_or(0);
                    if double { d *= 2; if d > 9 { d -= 9; } }
                    sum += d;
                    double = !double;
                }
                Ok(CfmlValue::Bool(sum % 10 == 0))
            }
            _ => Ok(CfmlValue::Bool(true)),
        }
    } else {
        Ok(CfmlValue::Bool(false))
    }
}

// ===============================================
// CONVERSION FUNCTIONS
// ===============================================

fn fn_to_string(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(get_str(&args, 0)))
}

fn fn_to_numeric(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Int(i)) => Ok(CfmlValue::Int(*i)),
        Some(CfmlValue::Double(d)) => Ok(CfmlValue::Double(*d)),
        Some(CfmlValue::Bool(b)) => Ok(CfmlValue::Int(if *b { 1 } else { 0 })),
        Some(CfmlValue::String(s)) => {
            if let Ok(i) = s.trim().parse::<i64>() {
                Ok(CfmlValue::Int(i))
            } else if let Ok(d) = s.trim().parse::<f64>() {
                Ok(CfmlValue::Double(d))
            } else {
                Ok(CfmlValue::Int(0))
            }
        }
        _ => Ok(CfmlValue::Int(0)),
    }
}

fn fn_to_boolean(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(args.first().map_or(false, |v| v.is_true())))
}

fn fn_val(args: Vec<CfmlValue>) -> CfmlResult {
    // val() extracts the leading numeric value from a string
    let s = get_str(&args, 0).trim().to_string();
    let mut num_str = String::new();
    let mut has_dot = false;
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else if c == '.' && !has_dot {
            has_dot = true;
            num_str.push(c);
        } else if c == '-' && i == 0 {
            num_str.push(c);
        } else {
            break;
        }
    }
    if num_str.is_empty() || num_str == "-" || num_str == "." {
        return Ok(CfmlValue::Int(0));
    }
    if has_dot {
        Ok(CfmlValue::Double(num_str.parse().unwrap_or(0.0)))
    } else {
        Ok(CfmlValue::Int(num_str.parse().unwrap_or(0)))
    }
}

fn fn_int(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_float(&args, 0);
    Ok(CfmlValue::Int(n as i64))
}

fn fn_java_cast(args: Vec<CfmlValue>) -> CfmlResult {
    // Simplified javacast
    if args.len() >= 2 {
        let type_name = get_str(&args, 0).to_lowercase();
        match type_name.as_str() {
            "string" => Ok(CfmlValue::String(args[1].as_string())),
            "int" | "integer" | "long" => Ok(CfmlValue::Int(get_int(&args, 1))),
            "double" | "float" => Ok(CfmlValue::Double(get_float(&args, 1))),
            "boolean" => Ok(CfmlValue::Bool(args[1].is_true())),
            "null" => Ok(CfmlValue::Null),
            _ => Ok(args[1].clone()),
        }
    } else {
        Ok(CfmlValue::Null)
    }
}

// ===============================================
// MATH FUNCTIONS
// ===============================================

fn fn_abs(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Int(i)) => Ok(CfmlValue::Int(i.abs())),
        Some(CfmlValue::Double(d)) => Ok(CfmlValue::Double(d.abs())),
        _ => Ok(CfmlValue::Double(get_float(&args, 0).abs())),
    }
}

fn fn_ceiling(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_float(&args, 0).ceil() as i64))
}

fn fn_floor(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_float(&args, 0).floor() as i64))
}

fn fn_round(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_float(&args, 0);
    if args.len() >= 2 {
        let precision = get_int(&args, 1);
        let factor = 10f64.powi(precision as i32);
        Ok(CfmlValue::Double((n * factor).round() / factor))
    } else {
        Ok(CfmlValue::Int(n.round() as i64))
    }
}

fn fn_rand(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(pseudo_random()))
}

fn fn_rand_range(args: Vec<CfmlValue>) -> CfmlResult {
    let min = get_float(&args, 0);
    let max = get_float(&args, 1);
    Ok(CfmlValue::Double(min + pseudo_random() * (max - min)))
}

fn fn_randomize(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(0)) // Seed not truly supported with simple PRNG
}

fn fn_max(args: Vec<CfmlValue>) -> CfmlResult {
    let a = get_float(&args, 0);
    let b = get_float(&args, 1);
    Ok(CfmlValue::Double(a.max(b)))
}

fn fn_min(args: Vec<CfmlValue>) -> CfmlResult {
    let a = get_float(&args, 0);
    let b = get_float(&args, 1);
    Ok(CfmlValue::Double(a.min(b)))
}

fn fn_sqr(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).sqrt()))
}

fn fn_exp(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).exp()))
}

fn fn_log(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).ln()))
}

fn fn_log10(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).log10()))
}

fn fn_sin(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).sin()))
}

fn fn_cos(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).cos()))
}

fn fn_tan(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).tan()))
}

fn fn_asin(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).asin()))
}

fn fn_acos(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).acos()))
}

fn fn_atan(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(get_float(&args, 0).atan()))
}

fn fn_pi(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Double(std::f64::consts::PI))
}

fn fn_sgn(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_float(&args, 0);
    Ok(CfmlValue::Int(if n > 0.0 { 1 } else if n < 0.0 { -1 } else { 0 }))
}

fn fn_fix(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_float(&args, 0);
    Ok(CfmlValue::Int(n.trunc() as i64))
}

fn fn_pow(args: Vec<CfmlValue>) -> CfmlResult {
    let base = get_float(&args, 0);
    let exp = get_float(&args, 1);
    Ok(CfmlValue::Double(base.powf(exp)))
}

fn fn_bit_and(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_int(&args, 0) & get_int(&args, 1)))
}

fn fn_bit_or(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_int(&args, 0) | get_int(&args, 1)))
}

fn fn_bit_xor(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_int(&args, 0) ^ get_int(&args, 1)))
}

fn fn_bit_not(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(!get_int(&args, 0)))
}

fn fn_bit_shln(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_int(&args, 0) << get_int(&args, 1)))
}

fn fn_bit_shrn(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_int(&args, 0) >> get_int(&args, 1)))
}

// ===============================================
// DATE/TIME FUNCTIONS
// ===============================================

fn fn_now(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
}

fn fn_create_date(args: Vec<CfmlValue>) -> CfmlResult {
    let year = get_int(&args, 0);
    let month = get_int(&args, 1);
    let day = get_int(&args, 2);
    Ok(CfmlValue::String(format!("{:04}-{:02}-{:02}", year, month, day)))
}

fn fn_create_date_time(args: Vec<CfmlValue>) -> CfmlResult {
    let year = get_int(&args, 0);
    let month = get_int(&args, 1);
    let day = get_int(&args, 2);
    let hour = get_int(&args, 3);
    let minute = get_int(&args, 4);
    let second = get_int(&args, 5);
    Ok(CfmlValue::String(format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second
    )))
}

fn fn_create_odbc_date(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    Ok(CfmlValue::String(format!("{{d '{}'}}", date)))
}

fn fn_create_odbc_date_time(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    Ok(CfmlValue::String(format!("{{ts '{}'}}", date)))
}

fn fn_date_add(_args: Vec<CfmlValue>) -> CfmlResult {
    // Would need proper date arithmetic
    Ok(CfmlValue::String(get_str(&_args, 2)))
}

fn fn_date_diff(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(0))
}

fn fn_date_format(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    let _mask = get_str(&args, 1);
    // Return date as-is for now (proper formatting would need date parsing)
    Ok(CfmlValue::String(date))
}

fn fn_time_format(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    Ok(CfmlValue::String(date))
}

fn fn_date_time_format(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    Ok(CfmlValue::String(date))
}

fn fn_year(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    let year: i64 = date.split(|c: char| !c.is_ascii_digit()).next().and_then(|s| s.parse().ok()).unwrap_or(0);
    Ok(CfmlValue::Int(year))
}

fn fn_month(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    let parts: Vec<&str> = date.split(|c: char| !c.is_ascii_digit()).collect();
    Ok(CfmlValue::Int(parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0)))
}

fn fn_day(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    let parts: Vec<&str> = date.split(|c: char| !c.is_ascii_digit()).collect();
    Ok(CfmlValue::Int(parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0)))
}

fn fn_hour(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    let parts: Vec<&str> = date.split(|c: char| !c.is_ascii_digit()).collect();
    Ok(CfmlValue::Int(parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0)))
}

fn fn_minute(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    let parts: Vec<&str> = date.split(|c: char| !c.is_ascii_digit()).collect();
    Ok(CfmlValue::Int(parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0)))
}

fn fn_second(args: Vec<CfmlValue>) -> CfmlResult {
    let date = get_str(&args, 0);
    let parts: Vec<&str> = date.split(|c: char| !c.is_ascii_digit()).collect();
    Ok(CfmlValue::Int(parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0)))
}

fn fn_day_of_week(_args: Vec<CfmlValue>) -> CfmlResult { Ok(CfmlValue::Int(1)) }
fn fn_day_of_week_as_string(args: Vec<CfmlValue>) -> CfmlResult {
    let dow = get_int(&args, 0);
    let names = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    Ok(CfmlValue::String(names.get((dow - 1) as usize).unwrap_or(&"").to_string()))
}
fn fn_day_of_year(_args: Vec<CfmlValue>) -> CfmlResult { Ok(CfmlValue::Int(1)) }
fn fn_days_in_month(_args: Vec<CfmlValue>) -> CfmlResult { Ok(CfmlValue::Int(30)) }
fn fn_days_in_year(_args: Vec<CfmlValue>) -> CfmlResult { Ok(CfmlValue::Int(365)) }
fn fn_first_day_of_month(_args: Vec<CfmlValue>) -> CfmlResult { Ok(CfmlValue::Int(1)) }
fn fn_is_leap_year(args: Vec<CfmlValue>) -> CfmlResult {
    let year = get_int(&args, 0);
    Ok(CfmlValue::Bool((year % 4 == 0 && year % 100 != 0) || year % 400 == 0))
}
fn fn_month_as_string(args: Vec<CfmlValue>) -> CfmlResult {
    let month = get_int(&args, 0);
    let names = ["January","February","March","April","May","June","July","August","September","October","November","December"];
    Ok(CfmlValue::String(names.get((month - 1) as usize).unwrap_or(&"").to_string()))
}
fn fn_quarter(args: Vec<CfmlValue>) -> CfmlResult {
    let month = get_int(&args, 0);
    Ok(CfmlValue::Int(((month - 1) / 3) + 1))
}
fn fn_week(_args: Vec<CfmlValue>) -> CfmlResult { Ok(CfmlValue::Int(1)) }

fn fn_get_tick_count(_args: Vec<CfmlValue>) -> CfmlResult {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
    Ok(CfmlValue::Int(ms))
}

// ===============================================
// LIST FUNCTIONS
// ===============================================

fn fn_list_new(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(String::new()))
}

fn fn_list_len(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    if list.is_empty() { return Ok(CfmlValue::Int(0)); }
    let delimiter = get_delimiter(&args, 1);
    Ok(CfmlValue::Int(list.split(&delimiter).count() as i64))
}

fn fn_list_append(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    if list.is_empty() {
        Ok(CfmlValue::String(value))
    } else {
        Ok(CfmlValue::String(format!("{}{}{}", list, delimiter, value)))
    }
}

fn fn_list_prepend(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    if list.is_empty() {
        Ok(CfmlValue::String(value))
    } else {
        Ok(CfmlValue::String(format!("{}{}{}", value, delimiter, list)))
    }
}

fn fn_list_get_at(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let index = (get_int(&args, 1) as usize).saturating_sub(1);
    let delimiter = get_delimiter(&args, 2);
    let items: Vec<&str> = list.split(&delimiter).collect();
    Ok(CfmlValue::String(items.get(index).unwrap_or(&"").trim().to_string()))
}

fn fn_list_set_at(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let index = (get_int(&args, 1) as usize).saturating_sub(1);
    let value = get_str(&args, 2);
    let delimiter = get_delimiter(&args, 3);
    let mut items: Vec<String> = list.split(&delimiter).map(|s| s.to_string()).collect();
    if index < items.len() {
        items[index] = value;
    }
    Ok(CfmlValue::String(items.join(&delimiter)))
}

fn fn_list_insert_at(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let index = (get_int(&args, 1) as usize).saturating_sub(1);
    let value = get_str(&args, 2);
    let delimiter = get_delimiter(&args, 3);
    let mut items: Vec<String> = list.split(&delimiter).map(|s| s.to_string()).collect();
    if index <= items.len() {
        items.insert(index, value);
    }
    Ok(CfmlValue::String(items.join(&delimiter)))
}

fn fn_list_delete_at(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let index = (get_int(&args, 1) as usize).saturating_sub(1);
    let delimiter = get_delimiter(&args, 2);
    let mut items: Vec<String> = list.split(&delimiter).map(|s| s.to_string()).collect();
    if index < items.len() {
        items.remove(index);
    }
    Ok(CfmlValue::String(items.join(&delimiter)))
}

fn fn_list_find(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    for (i, item) in list.split(&delimiter).enumerate() {
        if item.trim() == value { return Ok(CfmlValue::Int((i + 1) as i64)); }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_list_find_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1).to_lowercase();
    let delimiter = get_delimiter(&args, 2);
    for (i, item) in list.split(&delimiter).enumerate() {
        if item.trim().to_lowercase() == value { return Ok(CfmlValue::Int((i + 1) as i64)); }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_list_contains(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    for (i, item) in list.split(&delimiter).enumerate() {
        if item.trim().contains(&value) { return Ok(CfmlValue::Int((i + 1) as i64)); }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_list_contains_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1).to_lowercase();
    let delimiter = get_delimiter(&args, 2);
    for (i, item) in list.split(&delimiter).enumerate() {
        if item.trim().to_lowercase().contains(&value) { return Ok(CfmlValue::Int((i + 1) as i64)); }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_list_sort(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let sort_type = get_str(&args, 1).to_lowercase();
    let delimiter = get_delimiter(&args, 2);
    let mut items: Vec<String> = list.split(&delimiter).map(|s| s.trim().to_string()).collect();
    match sort_type.as_str() {
        "numeric" => {
            items.sort_by(|a, b| {
                let fa: f64 = a.parse().unwrap_or(0.0);
                let fb: f64 = b.parse().unwrap_or(0.0);
                fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        _ => items.sort(),
    }
    Ok(CfmlValue::String(items.join(&delimiter)))
}

fn fn_list_to_array(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    if list.is_empty() {
        return Ok(CfmlValue::Array(Vec::new()));
    }
    let items: Vec<CfmlValue> = list.split(&delimiter).map(|s| CfmlValue::String(s.trim().to_string())).collect();
    Ok(CfmlValue::Array(items))
}

fn fn_list_first(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    Ok(CfmlValue::String(list.split(&delimiter).next().unwrap_or("").trim().to_string()))
}

fn fn_list_last(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    Ok(CfmlValue::String(list.split(&delimiter).last().unwrap_or("").trim().to_string()))
}

fn fn_list_rest(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let items: Vec<&str> = list.split(&delimiter).collect();
    if items.len() > 1 {
        Ok(CfmlValue::String(items[1..].join(&delimiter)))
    } else {
        Ok(CfmlValue::String(String::new()))
    }
}

fn fn_list_remove_duplicates(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let mut seen = Vec::new();
    let mut result = Vec::new();
    for item in list.split(&delimiter) {
        let trimmed = item.trim().to_string();
        if !seen.contains(&trimmed) {
            seen.push(trimmed.clone());
            result.push(trimmed);
        }
    }
    Ok(CfmlValue::String(result.join(&delimiter)))
}

fn fn_list_value_count(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    let count = list.split(&delimiter).filter(|s| s.trim() == value).count();
    Ok(CfmlValue::Int(count as i64))
}

fn fn_list_value_count_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1).to_lowercase();
    let delimiter = get_delimiter(&args, 2);
    let count = list.split(&delimiter).filter(|s| s.trim().to_lowercase() == value).count();
    Ok(CfmlValue::Int(count as i64))
}

fn fn_list_change_delims(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let new_delim = get_str(&args, 1);
    let old_delim = get_delimiter(&args, 2);
    Ok(CfmlValue::String(list.split(&old_delim).collect::<Vec<_>>().join(&new_delim)))
}

fn fn_list_qualify(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let qualifier = get_str(&args, 2);
    let items: Vec<String> = list.split(&delimiter).map(|s| format!("{}{}{}", qualifier, s.trim(), qualifier)).collect();
    Ok(CfmlValue::String(items.join(&delimiter)))
}

fn fn_list_compact(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let items: Vec<&str> = list.split(&delimiter).filter(|s| !s.trim().is_empty()).collect();
    Ok(CfmlValue::String(items.join(&delimiter)))
}

// ===============================================
// JSON FUNCTIONS
// ===============================================

fn fn_serialize_json(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(serialize_value(args.first().unwrap_or(&CfmlValue::Null))))
}

fn serialize_value(val: &CfmlValue) -> String {
    match val {
        CfmlValue::Null => "null".to_string(),
        CfmlValue::Bool(b) => b.to_string(),
        CfmlValue::Int(i) => i.to_string(),
        CfmlValue::Double(d) => d.to_string(),
        CfmlValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")),
        CfmlValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(serialize_value).collect();
            format!("[{}]", items.join(","))
        }
        CfmlValue::Struct(s) => {
            let items: Vec<String> = s.iter().map(|(k, v)| format!("\"{}\":{}", k.replace('"', "\\\""), serialize_value(v))).collect();
            format!("{{{}}}", items.join(","))
        }
        _ => "null".to_string(),
    }
}

fn fn_deserialize_json(args: Vec<CfmlValue>) -> CfmlResult {
    let json = get_str(&args, 0).trim().to_string();
    Ok(parse_json(&json))
}

fn parse_json(s: &str) -> CfmlValue {
    let s = s.trim();
    if s == "null" { return CfmlValue::Null; }
    if s == "true" { return CfmlValue::Bool(true); }
    if s == "false" { return CfmlValue::Bool(false); }
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return CfmlValue::String(s[1..s.len()-1].replace("\\\"", "\"").replace("\\n", "\n").replace("\\\\", "\\"));
    }
    if let Ok(i) = s.parse::<i64>() { return CfmlValue::Int(i); }
    if let Ok(d) = s.parse::<f64>() { return CfmlValue::Double(d); }
    // Simplified: return as string for complex JSON
    CfmlValue::String(s.to_string())
}

fn fn_is_json(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0).trim().to_string();
    let valid = s.starts_with('{') || s.starts_with('[') || s.starts_with('"') || s == "null" || s == "true" || s == "false" || s.parse::<f64>().is_ok();
    Ok(CfmlValue::Bool(valid))
}

// ===============================================
// QUERY FUNCTIONS
// ===============================================

fn fn_query_new(args: Vec<CfmlValue>) -> CfmlResult {
    if args.is_empty() {
        return Ok(CfmlValue::Query(CfmlQuery::new(Vec::new())));
    }
    // queryNew("col1,col2") or queryNew(["col1","col2"])
    match &args[0] {
        CfmlValue::String(s) => {
            let columns: Vec<String> = s.split(',').map(|c| c.trim().to_string()).collect();
            Ok(CfmlValue::Query(CfmlQuery::new(columns)))
        }
        CfmlValue::Array(arr) => {
            let columns: Vec<String> = arr.iter().map(|v| v.as_string()).collect();
            Ok(CfmlValue::Query(CfmlQuery::new(columns)))
        }
        _ => Ok(CfmlValue::Query(CfmlQuery::new(Vec::new()))),
    }
}

fn fn_query_add_row(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Query(q)) = args.first() {
        let mut result = q.clone();
        let row = HashMap::new();
        result.rows.push(row);
        Ok(CfmlValue::Query(result))
    } else {
        Ok(CfmlValue::Null)
    }
}

fn fn_query_set_cell(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        if let CfmlValue::Query(q) = &args[0] {
            let mut result = q.clone();
            let column = args[1].as_string();
            let value = args[2].clone();
            let row_idx = if args.len() >= 4 {
                (get_int(&args, 3) as usize).saturating_sub(1)
            } else {
                result.rows.len().saturating_sub(1)
            };
            if row_idx < result.rows.len() {
                result.rows[row_idx].insert(column, value);
            }
            return Ok(CfmlValue::Query(result));
        }
    }
    Ok(CfmlValue::Null)
}

fn fn_query_add_column(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Query(q) = &args[0] {
            let mut result = q.clone();
            result.columns.push(args[1].as_string());
            return Ok(CfmlValue::Query(result));
        }
    }
    Ok(CfmlValue::Null)
}

// ===============================================
// UTILITY FUNCTIONS
// ===============================================

fn fn_evaluate(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Null) // Would need embedded parser
}

fn fn_iif(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        if args[0].is_true() { Ok(args[1].clone()) } else { Ok(args[2].clone()) }
    } else {
        Ok(CfmlValue::Null)
    }
}

fn fn_duplicate(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(args.into_iter().next().unwrap_or(CfmlValue::Null))
}

fn fn_sleep(args: Vec<CfmlValue>) -> CfmlResult {
    let ms = get_int(&args, 0).max(0) as u64;
    std::thread::sleep(std::time::Duration::from_millis(ms));
    Ok(CfmlValue::Null)
}

fn fn_get_metadata(args: Vec<CfmlValue>) -> CfmlResult {
    let mut meta = HashMap::new();
    if let Some(val) = args.first() {
        meta.insert("type".to_string(), CfmlValue::String(val.type_name().to_string()));
    }
    Ok(CfmlValue::Struct(meta))
}

fn fn_create_uuid(_args: Vec<CfmlValue>) -> CfmlResult {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let nanos = time.as_nanos();
    Ok(CfmlValue::String(format!(
        "{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
        (nanos >> 96) as u32,
        (nanos >> 80) as u16,
        ((nanos >> 64) as u16 & 0x0FFF) | 0x4000,
        ((nanos >> 48) as u16 & 0x3FFF) | 0x8000,
        nanos as u64 & 0xFFFFFFFFFFFF,
    )))
}

fn fn_hash(args: Vec<CfmlValue>) -> CfmlResult {
    use md5::Md5;
    use sha2::{Sha256, Sha384, Sha512, Digest};
    let input = get_str(&args, 0);
    let algorithm = if args.len() >= 2 { get_str(&args, 1).to_uppercase() } else { "MD5".to_string() };
    let hex = match algorithm.as_str() {
        "MD5" => {
            let mut hasher = Md5::new();
            hasher.update(input.as_bytes());
            format!("{:X}", hasher.finalize())
        }
        "SHA-256" | "SHA256" => {
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            format!("{:X}", hasher.finalize())
        }
        "SHA-384" | "SHA384" => {
            let mut hasher = Sha384::new();
            hasher.update(input.as_bytes());
            format!("{:X}", hasher.finalize())
        }
        "SHA-512" | "SHA512" => {
            let mut hasher = Sha512::new();
            hasher.update(input.as_bytes());
            format!("{:X}", hasher.finalize())
        }
        _ => {
            // Fallback to MD5
            let mut hasher = Md5::new();
            hasher.update(input.as_bytes());
            format!("{:X}", hasher.finalize())
        }
    };
    Ok(CfmlValue::String(hex))
}

fn fn_ls_parse_number(args: Vec<CfmlValue>) -> CfmlResult {
    fn_to_numeric(args)
}

// ===============================================
// FILE I/O FUNCTIONS
// ===============================================

fn fn_file_read(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(CfmlValue::String(contents)),
        Err(e) => Err(CfmlError::runtime(format!("fileRead: {}", e))),
    }
}

fn fn_file_write(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() < 2 {
        return Err(CfmlError::runtime("fileWrite requires path and data".to_string()));
    }
    let path = get_str(&args, 0);
    let data = get_str(&args, 1);
    match std::fs::write(&path, &data) {
        Ok(_) => Ok(CfmlValue::Null),
        Err(e) => Err(CfmlError::runtime(format!("fileWrite: {}", e))),
    }
}

fn fn_file_append(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() < 2 {
        return Err(CfmlError::runtime("fileAppend requires path and data".to_string()));
    }
    let path = get_str(&args, 0);
    let data = get_str(&args, 1);
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| CfmlError::runtime(format!("fileAppend: {}", e)))?;
    file.write_all(data.as_bytes())
        .map_err(|e| CfmlError::runtime(format!("fileAppend: {}", e)))?;
    Ok(CfmlValue::Null)
}

fn fn_file_exists(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    Ok(CfmlValue::Bool(std::path::Path::new(&path).exists()))
}

fn fn_file_delete(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    match std::fs::remove_file(&path) {
        Ok(_) => Ok(CfmlValue::Bool(true)),
        Err(e) => Err(CfmlError::runtime(format!("fileDelete: {}", e))),
    }
}

fn fn_file_move(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() < 2 {
        return Err(CfmlError::runtime("fileMove requires source and destination".to_string()));
    }
    let src = get_str(&args, 0);
    let dest = get_str(&args, 1);
    match std::fs::rename(&src, &dest) {
        Ok(_) => Ok(CfmlValue::Null),
        Err(e) => Err(CfmlError::runtime(format!("fileMove: {}", e))),
    }
}

fn fn_file_copy(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() < 2 {
        return Err(CfmlError::runtime("fileCopy requires source and destination".to_string()));
    }
    let src = get_str(&args, 0);
    let dest = get_str(&args, 1);
    match std::fs::copy(&src, &dest) {
        Ok(_) => Ok(CfmlValue::Null),
        Err(e) => Err(CfmlError::runtime(format!("fileCopy: {}", e))),
    }
}

fn fn_directory_create(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    match std::fs::create_dir_all(&path) {
        Ok(_) => Ok(CfmlValue::Null),
        Err(e) => Err(CfmlError::runtime(format!("directoryCreate: {}", e))),
    }
}

fn fn_directory_exists(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    Ok(CfmlValue::Bool(std::path::Path::new(&path).is_dir()))
}

fn fn_directory_delete(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    let recursive = if args.len() >= 2 { args[1].is_true() } else { false };
    let result = if recursive {
        std::fs::remove_dir_all(&path)
    } else {
        std::fs::remove_dir(&path)
    };
    match result {
        Ok(_) => Ok(CfmlValue::Null),
        Err(e) => Err(CfmlError::runtime(format!("directoryDelete: {}", e))),
    }
}

fn fn_directory_list(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    let recurse = if args.len() >= 2 { args[1].is_true() } else { false };
    let filter = if args.len() >= 3 { get_str(&args, 2) } else { String::new() };

    fn list_dir(path: &str, recurse: bool, filter: &str) -> Result<Vec<CfmlValue>, std::io::Error> {
        let mut results = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let name = entry_path.to_string_lossy().to_string();
            if filter.is_empty() || name.contains(filter) {
                results.push(CfmlValue::String(name.clone()));
            }
            if recurse && entry_path.is_dir() {
                results.extend(list_dir(&name, true, filter)?);
            }
        }
        Ok(results)
    }

    match list_dir(&path, recurse, &filter) {
        Ok(files) => Ok(CfmlValue::Array(files)),
        Err(e) => Err(CfmlError::runtime(format!("directoryList: {}", e))),
    }
}

fn fn_get_temp_directory(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(std::env::temp_dir().to_string_lossy().to_string()))
}

fn fn_get_temp_file(args: Vec<CfmlValue>) -> CfmlResult {
    let dir = if args.is_empty() {
        std::env::temp_dir().to_string_lossy().to_string()
    } else {
        get_str(&args, 0)
    };
    let prefix = if args.len() >= 2 { get_str(&args, 1) } else { "tmp".to_string() };
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::path::Path::new(&dir).join(format!("{}{}.tmp", prefix, ts));
    Ok(CfmlValue::String(path.to_string_lossy().to_string()))
}

fn fn_get_file_info(args: Vec<CfmlValue>) -> CfmlResult {
    let path_str = get_str(&args, 0);
    let path = std::path::Path::new(&path_str);
    let meta = std::fs::metadata(path)
        .map_err(|e| CfmlError::runtime(format!("getFileInfo: {}", e)))?;

    let mut info = HashMap::new();
    info.insert("name".to_string(), CfmlValue::String(
        path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
    ));
    info.insert("size".to_string(), CfmlValue::Int(meta.len() as i64));
    info.insert("type".to_string(), CfmlValue::String(
        if meta.is_dir() { "dir".to_string() } else { "file".to_string() }
    ));
    info.insert("canRead".to_string(), CfmlValue::Bool(!meta.permissions().readonly()));
    info.insert("canWrite".to_string(), CfmlValue::Bool(!meta.permissions().readonly()));
    if let Ok(modified) = meta.modified() {
        let secs = modified.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        info.insert("lastModified".to_string(), CfmlValue::Int(secs as i64));
    }
    Ok(CfmlValue::Struct(info))
}

fn fn_expand_path(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    match std::fs::canonicalize(&path) {
        Ok(abs) => Ok(CfmlValue::String(abs.to_string_lossy().to_string())),
        Err(_) => {
            let cwd = std::env::current_dir().unwrap_or_default();
            Ok(CfmlValue::String(cwd.join(&path).to_string_lossy().to_string()))
        }
    }
}

// ===============================================
// ADDITIONAL BUILTINS (Feature 3)
// ===============================================

fn fn_encode_for_url(args: Vec<CfmlValue>) -> CfmlResult {
    fn_url_encode(args)
}

fn fn_encode_for_css(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => result.push(c),
            _ => {
                result.push('\\');
                result.push_str(&format!("{:06X}", c as u32));
            }
        }
    }
    Ok(CfmlValue::String(result))
}

fn fn_encode_for_javascript(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '\'' => result.push_str("\\'"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '/' => result.push_str("\\/"),
            '<' => result.push_str("\\u003C"),
            '>' => result.push_str("\\u003E"),
            _ => result.push(c),
        }
    }
    Ok(CfmlValue::String(result))
}

fn fn_list_reduce(args: Vec<CfmlValue>) -> CfmlResult {
    // listReduce is a higher-order function; without closure support in builtins,
    // return the list as-is (handled by VM for closures)
    Ok(args.into_iter().next().unwrap_or(CfmlValue::Null))
}

fn fn_array_pop(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let mut result = arr.clone();
        if let Some(last) = result.pop() {
            Ok(last)
        } else {
            Err(CfmlError::runtime("Cannot pop from empty array".to_string()))
        }
    } else {
        Err(CfmlError::runtime("arrayPop requires an array".to_string()))
    }
}

fn fn_array_shift(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Array(arr)) = args.first() {
        let mut result = arr.clone();
        if !result.is_empty() {
            Ok(result.remove(0))
        } else {
            Err(CfmlError::runtime("Cannot shift from empty array".to_string()))
        }
    } else {
        Err(CfmlError::runtime("arrayShift requires an array".to_string()))
    }
}

// ===============================================
// HTTP CLIENT (cfhttp)
// ===============================================

#[cfg(feature = "http")]
fn fn_cfhttp(args: Vec<CfmlValue>) -> CfmlResult {
    use std::collections::HashMap;

    let arg = args.into_iter().next().unwrap_or(CfmlValue::Null);

    // Parse arguments: either a URL string or an options struct
    let (url, method, headers, body, timeout_secs) = match &arg {
        CfmlValue::String(url) => (url.clone(), "GET".to_string(), HashMap::new(), None, 30u64),
        CfmlValue::Struct(opts) => {
            let url = opts.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("url"))
                .map(|(_, v)| v.as_string())
                .unwrap_or_default();
            let method = opts.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("method"))
                .map(|(_, v)| v.as_string().to_uppercase())
                .unwrap_or_else(|| "GET".to_string());
            let hdrs: HashMap<String, String> = opts.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("headers"))
                .and_then(|(_,v)| if let CfmlValue::Struct(h) = v {
                    Some(h.iter().map(|(k, v)| (k.clone(), v.as_string())).collect())
                } else { None })
                .unwrap_or_default();
            let body = opts.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("body"))
                .and_then(|(_, v)| if matches!(v, CfmlValue::Null) { None } else { Some(v.as_string()) });
            let timeout = opts.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("timeout"))
                .map(|(_, v)| match v { CfmlValue::Int(i) => *i as u64, CfmlValue::Double(d) => *d as u64, CfmlValue::String(s) => s.parse().unwrap_or(30), _ => 30 })
                .unwrap_or(30);
            (url, method, hdrs, body, timeout)
        }
        _ => return Err(CfmlError::runtime("cfhttp requires a URL string or options struct".to_string())),
    };

    if url.is_empty() {
        return Err(CfmlError::runtime("cfhttp: url is required".to_string()));
    }

    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build();

    let mut request = match method.as_str() {
        "GET" => agent.get(&url),
        "POST" => agent.post(&url),
        "PUT" => agent.put(&url),
        "DELETE" => agent.delete(&url),
        "PATCH" => agent.request("PATCH", &url),
        "HEAD" => agent.head(&url),
        "OPTIONS" => agent.request("OPTIONS", &url),
        _ => agent.get(&url),
    };

    for (k, v) in &headers {
        request = request.set(k, v);
    }

    let response = if let Some(body_str) = &body {
        request.send_string(body_str)
    } else {
        request.call()
    };

    let mut result_struct: HashMap<String, CfmlValue> = HashMap::new();

    match response {
        Ok(resp) => {
            let status = resp.status();
            let status_text = resp.status_text().to_string();
            let http_version = resp.http_version().to_string();
            let content_type = resp.content_type().to_string();

            let mut resp_headers: HashMap<String, CfmlValue> = HashMap::new();
            for name in resp.headers_names() {
                if let Some(val) = resp.header(&name) {
                    resp_headers.insert(name, CfmlValue::String(val.to_string()));
                }
            }

            let body_text = resp.into_string().unwrap_or_default();

            let (mime, charset) = parse_content_type(&content_type);

            result_struct.insert("statusCode".to_string(), CfmlValue::String(format!("{} {}", status, status_text)));
            result_struct.insert("status_code".to_string(), CfmlValue::Int(status as i64));
            result_struct.insert("statusText".to_string(), CfmlValue::String(status_text.clone()));
            result_struct.insert("status_text".to_string(), CfmlValue::String(status_text));
            result_struct.insert("fileContent".to_string(), CfmlValue::String(body_text));
            result_struct.insert("mimeType".to_string(), CfmlValue::String(mime));
            result_struct.insert("charset".to_string(), CfmlValue::String(charset));
            result_struct.insert("responseHeader".to_string(), CfmlValue::Struct(resp_headers));
            result_struct.insert("errorDetail".to_string(), CfmlValue::String(String::new()));
            result_struct.insert("HTTP_Version".to_string(), CfmlValue::String(http_version));
        }
        Err(ureq::Error::Status(code, resp)) => {
            let status_text = resp.status_text().to_string();
            let http_version = resp.http_version().to_string();
            let content_type = resp.content_type().to_string();

            let mut resp_headers: HashMap<String, CfmlValue> = HashMap::new();
            for name in resp.headers_names() {
                if let Some(val) = resp.header(&name) {
                    resp_headers.insert(name, CfmlValue::String(val.to_string()));
                }
            }

            let body_text = resp.into_string().unwrap_or_default();
            let (mime, charset) = parse_content_type(&content_type);

            result_struct.insert("statusCode".to_string(), CfmlValue::String(format!("{} {}", code, status_text)));
            result_struct.insert("status_code".to_string(), CfmlValue::Int(code as i64));
            result_struct.insert("statusText".to_string(), CfmlValue::String(status_text.clone()));
            result_struct.insert("status_text".to_string(), CfmlValue::String(status_text));
            result_struct.insert("fileContent".to_string(), CfmlValue::String(body_text));
            result_struct.insert("mimeType".to_string(), CfmlValue::String(mime));
            result_struct.insert("charset".to_string(), CfmlValue::String(charset));
            result_struct.insert("responseHeader".to_string(), CfmlValue::Struct(resp_headers));
            result_struct.insert("errorDetail".to_string(), CfmlValue::String(String::new()));
            result_struct.insert("HTTP_Version".to_string(), CfmlValue::String(http_version));
        }
        Err(ureq::Error::Transport(e)) => {
            result_struct.insert("statusCode".to_string(), CfmlValue::String("0".to_string()));
            result_struct.insert("status_code".to_string(), CfmlValue::Int(0));
            result_struct.insert("statusText".to_string(), CfmlValue::String(String::new()));
            result_struct.insert("status_text".to_string(), CfmlValue::String(String::new()));
            result_struct.insert("fileContent".to_string(), CfmlValue::String(String::new()));
            result_struct.insert("mimeType".to_string(), CfmlValue::String(String::new()));
            result_struct.insert("charset".to_string(), CfmlValue::String("UTF-8".to_string()));
            result_struct.insert("responseHeader".to_string(), CfmlValue::Struct(HashMap::new()));
            result_struct.insert("errorDetail".to_string(), CfmlValue::String(e.to_string()));
            result_struct.insert("HTTP_Version".to_string(), CfmlValue::String(String::new()));
        }
    }

    Ok(CfmlValue::Struct(result_struct))
}

#[cfg(feature = "http")]
fn parse_content_type(ct: &str) -> (String, String) {
    let parts: Vec<&str> = ct.splitn(2, ';').collect();
    let mime = parts[0].trim().to_string();
    let charset = if parts.len() > 1 {
        parts[1]
            .split('=')
            .nth(1)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "UTF-8".to_string())
    } else {
        "UTF-8".to_string()
    };
    (mime, charset)
}

// ===============================================
// DATABASE (queryExecute)
// ===============================================

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db"))]
enum DbDriver {
    Sqlite(String),
    Mysql(String),
    Postgres(String),
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db"))]
fn parse_datasource(ds: &str) -> DbDriver {
    if ds.starts_with("mysql://") {
        DbDriver::Mysql(ds.to_string())
    } else if ds.starts_with("postgresql://") || ds.starts_with("postgres://") {
        DbDriver::Postgres(ds.to_string())
    } else if ds.starts_with("sqlite://") {
        DbDriver::Sqlite(ds[9..].to_string())
    } else {
        DbDriver::Sqlite(ds.to_string()) // :memory: or file path
    }
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db"))]
fn is_select_query(sql: &str) -> bool {
    let trimmed = sql.trim_start();
    trimmed.len() >= 6 && trimmed[..6].eq_ignore_ascii_case("SELECT")
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db"))]
fn fn_query_execute(args: Vec<CfmlValue>) -> CfmlResult {
    let sql = get_str(&args, 0);
    if sql.is_empty() {
        return Err(CfmlError::runtime("queryExecute: SQL string is required".to_string()));
    }

    let params_arg = args.get(1).cloned().unwrap_or(CfmlValue::Null);
    let options_arg = args.get(2).cloned().unwrap_or(CfmlValue::Null);

    // Extract datasource from options
    let datasource = match &options_arg {
        CfmlValue::Struct(opts) => opts.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("datasource"))
            .map(|(_, v)| v.as_string())
            .unwrap_or_else(|| ":memory:".to_string()),
        _ => ":memory:".to_string(),
    };

    // Extract returnType from options
    let return_type = match &options_arg {
        CfmlValue::Struct(opts) => opts.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("returntype") || k.eq_ignore_ascii_case("returnType"))
            .map(|(_, v)| v.as_string().to_lowercase())
            .unwrap_or_else(|| "query".to_string()),
        _ => "query".to_string(),
    };

    match parse_datasource(&datasource) {
        #[cfg(feature = "sqlite")]
        DbDriver::Sqlite(path) => execute_sqlite(&path, &sql, &params_arg, &return_type),
        #[cfg(feature = "mysql_db")]
        DbDriver::Mysql(url) => execute_mysql(&url, &sql, &params_arg, &return_type),
        #[cfg(feature = "postgres_db")]
        DbDriver::Postgres(url) => execute_postgres(&url, &sql, &params_arg, &return_type),
        #[allow(unreachable_patterns)]
        _ => Err(CfmlError::runtime(format!(
            "queryExecute: database driver not available for datasource '{}'. Enable the appropriate feature (sqlite, mysql_db, postgres_db).",
            datasource
        ))),
    }
}

// -----------------------------------------------
// SQLite driver
// -----------------------------------------------

#[cfg(feature = "sqlite")]
fn execute_sqlite(path: &str, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    use rusqlite::{Connection, types::Value as SqlValue};

    let conn = Connection::open(path)
        .map_err(|e| CfmlError::runtime(format!("queryExecute: failed to open database '{}': {}", path, e)))?;

    let bound_params = build_sqlite_params(params_arg, sql)?;

    if is_select_query(sql) {
        let mut stmt = conn.prepare(sql)
            .map_err(|e| CfmlError::runtime(format!("queryExecute: SQL error: {}", e)))?;

        let column_count = stmt.column_count();
        let columns: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
            .collect();

        let rows_result: Result<Vec<HashMap<String, CfmlValue>>, _> = stmt
            .query_map(rusqlite::params_from_iter(bound_params.iter()), |row| {
                let mut row_map = HashMap::new();
                for (i, col) in columns.iter().enumerate() {
                    let val: SqlValue = row.get_unwrap(i);
                    row_map.insert(col.clone(), sqlite_to_cfml(val));
                }
                Ok(row_map)
            })
            .map_err(|e| CfmlError::runtime(format!("queryExecute: query error: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CfmlError::runtime(format!("queryExecute: row error: {}", e)));

        let rows = rows_result?;
        build_query_result(columns, rows, sql, return_type)
    } else {
        let affected = conn.execute(sql, rusqlite::params_from_iter(bound_params.iter()))
            .map_err(|e| CfmlError::runtime(format!("queryExecute: SQL error: {}", e)))?;

        let last_id = conn.last_insert_rowid();
        build_mutation_result(affected as i64, last_id)
    }
}

#[cfg(feature = "sqlite")]
fn build_sqlite_params(params_arg: &CfmlValue, sql: &str) -> Result<Vec<rusqlite::types::Value>, CfmlError> {
    match params_arg {
        CfmlValue::Null => Ok(vec![]),
        CfmlValue::Array(arr) => {
            arr.iter().map(|v| Ok(cfml_to_sqlite(v))).collect()
        }
        CfmlValue::Struct(map) => {
            let mut result = Vec::new();
            let bytes = sql.as_bytes();
            let len = bytes.len();
            let mut i = 0;
            while i < len {
                if bytes[i] == b':' && (i == 0 || !bytes[i-1].is_ascii_alphanumeric()) {
                    let start = i + 1;
                    let mut end = start;
                    while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                        end += 1;
                    }
                    if end > start {
                        let param_name: String = String::from_utf8_lossy(&bytes[start..end]).to_string();
                        let val = map.iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case(&param_name))
                            .map(|(_, v)| v)
                            .unwrap_or(&CfmlValue::Null);
                        result.push(cfml_to_sqlite(val));
                        i = end;
                        continue;
                    }
                }
                if bytes[i] == b'\'' {
                    i += 1;
                    while i < len && bytes[i] != b'\'' {
                        i += 1;
                    }
                }
                i += 1;
            }
            Ok(result)
        }
        _ => Ok(vec![]),
    }
}

#[cfg(feature = "sqlite")]
fn cfml_to_sqlite(val: &CfmlValue) -> rusqlite::types::Value {
    use rusqlite::types::Value as SqlValue;
    match val {
        CfmlValue::Null => SqlValue::Null,
        CfmlValue::Bool(b) => SqlValue::Integer(if *b { 1 } else { 0 }),
        CfmlValue::Int(i) => SqlValue::Integer(*i),
        CfmlValue::Double(d) => SqlValue::Real(*d),
        CfmlValue::String(s) => SqlValue::Text(s.clone()),
        CfmlValue::Binary(b) => SqlValue::Blob(b.clone()),
        _ => SqlValue::Text(val.as_string()),
    }
}

#[cfg(feature = "sqlite")]
fn sqlite_to_cfml(val: rusqlite::types::Value) -> CfmlValue {
    use rusqlite::types::Value as SqlValue;
    match val {
        SqlValue::Null => CfmlValue::Null,
        SqlValue::Integer(i) => CfmlValue::Int(i),
        SqlValue::Real(d) => CfmlValue::Double(d),
        SqlValue::Text(s) => CfmlValue::String(s),
        SqlValue::Blob(b) => CfmlValue::Binary(b),
    }
}

// -----------------------------------------------
// MySQL driver
// -----------------------------------------------

#[cfg(feature = "mysql_db")]
fn execute_mysql(url: &str, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    use mysql::*;
    use mysql::prelude::*;

    let pool = Pool::new(url)
        .map_err(|e| CfmlError::runtime(format!("queryExecute: MySQL connection error: {}", e)))?;
    let mut conn = pool.get_conn()
        .map_err(|e| CfmlError::runtime(format!("queryExecute: MySQL connection error: {}", e)))?;

    let params = match params_arg {
        CfmlValue::Array(arr) => {
            let vals: Vec<mysql::Value> = arr.iter().map(cfml_to_mysql_value).collect();
            Params::Positional(vals)
        }
        CfmlValue::Struct(map) => {
            let mut named: HashMap<Vec<u8>, mysql::Value> = HashMap::new();
            for (k, v) in map.iter() {
                named.insert(k.as_bytes().to_vec(), cfml_to_mysql_value(v));
            }
            Params::Named(named)
        }
        _ => Params::Empty,
    };

    if is_select_query(sql) {
        let result: Vec<Row> = conn.exec(sql, &params)
            .map_err(|e| CfmlError::runtime(format!("queryExecute: MySQL query error: {}", e)))?;

        // Extract column names from result set
        let columns: Vec<String> = if let Some(first_row) = result.first() {
            first_row.columns_ref().iter()
                .map(|c| c.name_str().to_string())
                .collect()
        } else {
            // No rows - try to get columns from prepared statement
            vec![]
        };

        let mut rows: Vec<HashMap<String, CfmlValue>> = Vec::with_capacity(result.len());
        for row in &result {
            let mut row_map = HashMap::new();
            for (i, col) in columns.iter().enumerate() {
                let val: mysql::Value = row.get(i).unwrap_or(mysql::Value::NULL);
                row_map.insert(col.clone(), mysql_value_to_cfml(val));
            }
            rows.push(row_map);
        }

        build_query_result(columns, rows, sql, return_type)
    } else {
        conn.exec_drop(sql, &params)
            .map_err(|e| CfmlError::runtime(format!("queryExecute: MySQL error: {}", e)))?;

        let affected = conn.affected_rows() as i64;
        let last_id = conn.last_insert_id() as i64;
        build_mutation_result(affected, last_id)
    }
}

#[cfg(feature = "mysql_db")]
fn cfml_to_mysql_value(val: &CfmlValue) -> mysql::Value {
    match val {
        CfmlValue::Null => mysql::Value::NULL,
        CfmlValue::Bool(b) => mysql::Value::from(*b),
        CfmlValue::Int(i) => mysql::Value::from(*i),
        CfmlValue::Double(d) => mysql::Value::from(*d),
        CfmlValue::String(s) => mysql::Value::from(s.as_str()),
        CfmlValue::Binary(b) => mysql::Value::Bytes(b.clone()),
        _ => mysql::Value::from(val.as_string()),
    }
}

#[cfg(feature = "mysql_db")]
fn mysql_value_to_cfml(val: mysql::Value) -> CfmlValue {
    match val {
        mysql::Value::NULL => CfmlValue::Null,
        mysql::Value::Int(i) => CfmlValue::Int(i),
        mysql::Value::UInt(u) => CfmlValue::Int(u as i64),
        mysql::Value::Float(f) => CfmlValue::Double(f as f64),
        mysql::Value::Double(d) => CfmlValue::Double(d),
        mysql::Value::Bytes(b) => {
            match String::from_utf8(b.clone()) {
                Ok(s) => CfmlValue::String(s),
                Err(_) => CfmlValue::Binary(b),
            }
        }
        mysql::Value::Date(..) | mysql::Value::Time(..) => {
            // Format dates/times as strings
            CfmlValue::String(format!("{:?}", val))
        }
    }
}

// -----------------------------------------------
// PostgreSQL driver
// -----------------------------------------------

#[cfg(feature = "postgres_db")]
fn execute_postgres(url: &str, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    use postgres::{Client, NoTls};

    let mut client = Client::connect(url, NoTls)
        .map_err(|e| CfmlError::runtime(format!("queryExecute: PostgreSQL connection error: {}", e)))?;

    // Rewrite :name params to $1,$2,... for PostgreSQL
    let (rewritten_sql, ordered_params) = rewrite_params_for_postgres(sql, params_arg)?;

    // Build param references for the postgres crate
    let param_refs: Vec<&(dyn postgres::types::ToSql + Sync)> = ordered_params.iter()
        .map(|v| v as &(dyn postgres::types::ToSql + Sync))
        .collect();

    if is_select_query(sql) {
        let rows = client.query(&rewritten_sql, &param_refs)
            .map_err(|e| CfmlError::runtime(format!("queryExecute: PostgreSQL query error: {}", e)))?;

        let columns: Vec<String> = if let Some(first_row) = rows.first() {
            first_row.columns().iter()
                .map(|c| c.name().to_string())
                .collect()
        } else {
            vec![]
        };

        let mut result_rows: Vec<HashMap<String, CfmlValue>> = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut row_map = HashMap::new();
            for (i, col) in columns.iter().enumerate() {
                let val = postgres_row_to_cfml(row, i);
                row_map.insert(col.clone(), val);
            }
            result_rows.push(row_map);
        }

        build_query_result(columns, result_rows, sql, return_type)
    } else {
        let affected = client.execute(&rewritten_sql, &param_refs)
            .map_err(|e| CfmlError::runtime(format!("queryExecute: PostgreSQL error: {}", e)))?;

        build_mutation_result(affected as i64, 0) // PG uses RETURNING, not last_insert_id
    }
}

#[derive(Debug)]
#[cfg(feature = "postgres_db")]
enum PgParam {
    Null,
    Bool(bool),
    Int(i64),
    Double(f64),
    Text(String),
    Bytes(Vec<u8>),
}

#[cfg(feature = "postgres_db")]
impl postgres::types::ToSql for PgParam {
    fn to_sql(&self, ty: &postgres::types::Type, out: &mut postgres::types::private::BytesMut) -> Result<postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            PgParam::Null => Ok(postgres::types::IsNull::Yes),
            PgParam::Bool(b) => b.to_sql(ty, out),
            PgParam::Int(i) => i.to_sql(ty, out),
            PgParam::Double(d) => d.to_sql(ty, out),
            PgParam::Text(s) => s.to_sql(ty, out),
            PgParam::Bytes(b) => b.to_sql(ty, out),
        }
    }

    fn accepts(_ty: &postgres::types::Type) -> bool {
        true
    }

    postgres::types::to_sql_checked!();
}

#[cfg(feature = "postgres_db")]
fn cfml_to_pg_param(val: &CfmlValue) -> PgParam {
    match val {
        CfmlValue::Null => PgParam::Null,
        CfmlValue::Bool(b) => PgParam::Bool(*b),
        CfmlValue::Int(i) => PgParam::Int(*i),
        CfmlValue::Double(d) => PgParam::Double(*d),
        CfmlValue::String(s) => PgParam::Text(s.clone()),
        CfmlValue::Binary(b) => PgParam::Bytes(b.clone()),
        _ => PgParam::Text(val.as_string()),
    }
}

#[cfg(feature = "postgres_db")]
fn rewrite_params_for_postgres(sql: &str, params_arg: &CfmlValue) -> Result<(String, Vec<PgParam>), CfmlError> {
    match params_arg {
        CfmlValue::Null => Ok((sql.to_string(), vec![])),
        CfmlValue::Array(arr) => {
            // Positional: replace ? with $1, $2, ...
            let mut result = String::with_capacity(sql.len());
            let mut idx = 1;
            let bytes = sql.as_bytes();
            let len = bytes.len();
            let mut i = 0;
            while i < len {
                if bytes[i] == b'?' {
                    result.push('$');
                    result.push_str(&idx.to_string());
                    idx += 1;
                } else if bytes[i] == b'\'' {
                    result.push('\'');
                    i += 1;
                    while i < len && bytes[i] != b'\'' {
                        result.push(bytes[i] as char);
                        i += 1;
                    }
                    if i < len { result.push('\''); }
                } else {
                    result.push(bytes[i] as char);
                }
                i += 1;
            }
            let params: Vec<PgParam> = arr.iter().map(cfml_to_pg_param).collect();
            Ok((result, params))
        }
        CfmlValue::Struct(map) => {
            // Named: replace :name with $1, $2, ... tracking seen names
            let mut result = String::with_capacity(sql.len());
            let mut param_order: Vec<String> = Vec::new();
            let bytes = sql.as_bytes();
            let len = bytes.len();
            let mut i = 0;
            while i < len {
                if bytes[i] == b':' && (i == 0 || !bytes[i-1].is_ascii_alphanumeric()) {
                    let start = i + 1;
                    let mut end = start;
                    while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                        end += 1;
                    }
                    if end > start {
                        let param_name = String::from_utf8_lossy(&bytes[start..end]).to_string();
                        // Check if we've seen this param before
                        let idx = if let Some(pos) = param_order.iter().position(|n| n.eq_ignore_ascii_case(&param_name)) {
                            pos + 1
                        } else {
                            param_order.push(param_name.clone());
                            param_order.len()
                        };
                        result.push('$');
                        result.push_str(&idx.to_string());
                        i = end;
                        continue;
                    }
                }
                if bytes[i] == b'\'' {
                    result.push('\'');
                    i += 1;
                    while i < len && bytes[i] != b'\'' {
                        result.push(bytes[i] as char);
                        i += 1;
                    }
                    if i < len { result.push('\''); }
                } else {
                    result.push(bytes[i] as char);
                }
                i += 1;
            }
            // Build ordered params vec
            let params: Vec<PgParam> = param_order.iter().map(|name| {
                let val = map.iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(name))
                    .map(|(_, v)| v)
                    .unwrap_or(&CfmlValue::Null);
                cfml_to_pg_param(val)
            }).collect();
            Ok((result, params))
        }
        _ => Ok((sql.to_string(), vec![])),
    }
}

#[cfg(feature = "postgres_db")]
fn postgres_row_to_cfml(row: &postgres::Row, col_idx: usize) -> CfmlValue {
    use postgres::types::Type;
    let col_type = row.columns()[col_idx].type_();

    match *col_type {
        Type::BOOL => row.get::<_, Option<bool>>(col_idx)
            .map(|b| CfmlValue::Bool(b))
            .unwrap_or(CfmlValue::Null),
        Type::INT2 => row.get::<_, Option<i16>>(col_idx)
            .map(|i| CfmlValue::Int(i as i64))
            .unwrap_or(CfmlValue::Null),
        Type::INT4 => row.get::<_, Option<i32>>(col_idx)
            .map(|i| CfmlValue::Int(i as i64))
            .unwrap_or(CfmlValue::Null),
        Type::INT8 => row.get::<_, Option<i64>>(col_idx)
            .map(|i| CfmlValue::Int(i))
            .unwrap_or(CfmlValue::Null),
        Type::FLOAT4 => row.get::<_, Option<f32>>(col_idx)
            .map(|f| CfmlValue::Double(f as f64))
            .unwrap_or(CfmlValue::Null),
        Type::FLOAT8 => row.get::<_, Option<f64>>(col_idx)
            .map(|f| CfmlValue::Double(f))
            .unwrap_or(CfmlValue::Null),
        Type::BYTEA => row.get::<_, Option<Vec<u8>>>(col_idx)
            .map(|b| CfmlValue::Binary(b))
            .unwrap_or(CfmlValue::Null),
        _ => {
            // Default: try to get as string (works for VARCHAR, TEXT, DATE, TIMESTAMP, etc.)
            row.get::<_, Option<String>>(col_idx)
                .map(|s| CfmlValue::String(s))
                .unwrap_or(CfmlValue::Null)
        }
    }
}

// -----------------------------------------------
// Shared result builders
// -----------------------------------------------

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db"))]
fn build_query_result(columns: Vec<String>, rows: Vec<HashMap<String, CfmlValue>>, sql: &str, return_type: &str) -> CfmlResult {
    if return_type == "array" {
        let arr: Vec<CfmlValue> = rows.into_iter()
            .map(|r| CfmlValue::Struct(r))
            .collect();
        Ok(CfmlValue::Array(arr))
    } else {
        let query = CfmlQuery {
            columns,
            rows,
            sql: Some(sql.to_string()),
        };
        Ok(CfmlValue::Query(query))
    }
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db"))]
fn build_mutation_result(affected: i64, last_id: i64) -> CfmlResult {
    let mut result = HashMap::new();
    result.insert("recordCount".to_string(), CfmlValue::Int(affected));
    result.insert("generatedKey".to_string(), CfmlValue::Int(last_id));
    Ok(CfmlValue::Struct(result))
}
