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
use serde_json;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, Datelike, Timelike};

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
    f.insert("encodeForHTML".into(), fn_encode_for_html);
    f.insert("lJustify".into(), fn_ljustify);
    f.insert("rJustify".into(), fn_rjustify);
    f.insert("numberFormat".into(), fn_number_format);
    f.insert("decimalFormat".into(), fn_decimal_format);
    f.insert("formatBaseN".into(), fn_format_base_n);
    f.insert("inputBaseN".into(), fn_input_base_n);
    f.insert("replaceList".into(), fn_replace_list);
    f.insert("replaceListNoCase".into(), fn_replace_list_no_case);
    f.insert("xmlFormat".into(), fn_xml_format);
    f.insert("paragraphFormat".into(), fn_paragraph_format);
    f.insert("cJustify".into(), fn_cjustify);

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
    f.insert("arraySome".into(), fn_array_each);  // VM intercepts
    f.insert("arrayEvery".into(), fn_array_each);  // VM intercepts
    f.insert("isArray".into(), fn_is_array);
    f.insert("arrayIsEmpty".into(), fn_array_is_empty);
    f.insert("arrayDelete".into(), fn_array_delete);
    f.insert("arrayFindAll".into(), fn_array_find_all);
    f.insert("arrayFindAllNoCase".into(), fn_array_find_all_no_case);
    f.insert("arrayFirst".into(), fn_array_first);
    f.insert("arrayLast".into(), fn_array_last);

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
    f.insert("structReduce".into(), fn_struct_each);  // VM intercepts
    f.insert("structSome".into(), fn_struct_each);  // VM intercepts
    f.insert("structEvery".into(), fn_struct_each);  // VM intercepts
    f.insert("isStruct".into(), fn_is_struct);
    f.insert("structGet".into(), fn_struct_get);
    f.insert("structValueArray".into(), fn_struct_value_array);
    f.insert("structEquals".into(), fn_struct_equals);
    f.insert("structKeyTranslate".into(), fn_struct_key_translate);

    // ---- General utility functions ----
    f.insert("isEmpty".into(), fn_is_empty);

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
    f.insert("createTime".into(), fn_create_time);
    f.insert("createODBCDate".into(), fn_create_odbc_date);
    f.insert("createODBCDateTime".into(), fn_create_odbc_date_time);
    f.insert("createODBCTime".into(), fn_create_odbc_time);
    f.insert("dateAdd".into(), fn_date_add);
    f.insert("dateDiff".into(), fn_date_diff);
    f.insert("dateFormat".into(), fn_date_format);
    f.insert("timeFormat".into(), fn_time_format);
    f.insert("dateTimeFormat".into(), fn_date_time_format);
    f.insert("parseDateTime".into(), fn_parse_date_time);
    f.insert("datePart".into(), fn_date_part);
    f.insert("dateCompare".into(), fn_date_compare);
    f.insert("year".into(), fn_year);
    f.insert("month".into(), fn_month);
    f.insert("day".into(), fn_day);
    f.insert("hour".into(), fn_hour);
    f.insert("minute".into(), fn_minute);
    f.insert("second".into(), fn_second);
    f.insert("dayOfWeek".into(), fn_day_of_week);
    f.insert("dayOfWeekAsString".into(), fn_day_of_week_as_string);
    f.insert("dayOfWeekShortAsString".into(), fn_day_of_week_short_as_string);
    f.insert("dayOfYear".into(), fn_day_of_year);
    f.insert("daysInMonth".into(), fn_days_in_month);
    f.insert("daysInYear".into(), fn_days_in_year);
    f.insert("firstDayOfMonth".into(), fn_first_day_of_month);
    f.insert("isLeapYear".into(), fn_is_leap_year);
    f.insert("monthAsString".into(), fn_month_as_string);
    f.insert("monthShortAsString".into(), fn_month_short_as_string);
    f.insert("quarter".into(), fn_quarter);
    f.insert("week".into(), fn_week);
    f.insert("getTickCount".into(), fn_get_tick_count);
    f.insert("getFunctionList".into(), fn_get_function_list);
    f.insert("getContextRoot".into(), fn_get_context_root);
    f.insert("GetContextRoot".into(), fn_get_context_root);
    f.insert("getPageContext".into(), fn_get_page_context);

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
    f.insert("listEach".into(), fn_list_each);
    f.insert("listMap".into(), fn_list_map);
    f.insert("listFilter".into(), fn_list_filter);

    // ---- JSON functions ----
    f.insert("serializeJSON".into(), fn_serialize_json);
    f.insert("deserializeJSON".into(), fn_deserialize_json);
    f.insert("isJSON".into(), fn_is_json);

    // ---- Query functions ----
    f.insert("queryNew".into(), fn_query_new);
    f.insert("queryAddRow".into(), fn_query_add_row);
    f.insert("querySetCell".into(), fn_query_set_cell);
    f.insert("queryAddColumn".into(), fn_query_add_column);
    f.insert("queryGetRow".into(), fn_query_get_row as BuiltinFunction);
    f.insert("queryGetCell".into(), fn_query_get_cell as BuiltinFunction);
    f.insert("queryRecordCount".into(), fn_query_record_count as BuiltinFunction);
    f.insert("queryColumnCount".into(), fn_query_column_count as BuiltinFunction);
    f.insert("queryColumnList".into(), fn_query_column_list as BuiltinFunction);
    f.insert("queryDeleteRow".into(), fn_query_delete_row as BuiltinFunction);
    f.insert("queryDeleteColumn".into(), fn_query_delete_column as BuiltinFunction);
    // Higher-order query functions (VM-intercepted stubs)
    f.insert("queryEach".into(), fn_query_ho_stub as BuiltinFunction);
    f.insert("queryMap".into(), fn_query_ho_stub as BuiltinFunction);
    f.insert("queryFilter".into(), fn_query_ho_stub as BuiltinFunction);
    f.insert("queryReduce".into(), fn_query_ho_stub as BuiltinFunction);
    f.insert("querySort".into(), fn_query_ho_stub as BuiltinFunction);
    f.insert("querySome".into(), fn_query_ho_stub as BuiltinFunction);
    f.insert("queryEvery".into(), fn_query_ho_stub as BuiltinFunction);

    // ---- Utility functions ----
    f.insert("evaluate".into(), fn_evaluate);
    f.insert("iif".into(), fn_iif);
    f.insert("duplicate".into(), fn_duplicate);
    f.insert("sleep".into(), fn_sleep);
    f.insert("getMetadata".into(), fn_get_metadata);
    f.insert("isInstanceOf".into(), fn_is_instance_of);
    f.insert("createObject".into(), fn_create_object);
    f.insert("getDirectoryFromPath".into(), fn_get_directory_from_path);
    f.insert("getComponentMetadata".into(), fn_get_component_metadata);
    f.insert("createUUID".into(), fn_create_uuid);
    f.insert("createGUID".into(), fn_create_guid);
    f.insert("hash".into(), fn_hash);
    f.insert("lsParseNumber".into(), fn_ls_parse_number);

    // ---- System functions ----
    f.insert("getTickCount".into(), fn_get_tick_count);
    f.insert("getFunctionList".into(), fn_get_function_list);
    f.insert("getCurrentTemplatePath".into(), fn_get_current_template_path);
    f.insert("getBaseTemplatePath".into(), fn_get_base_template_path);
    f.insert("getTimeZone".into(), fn_get_time_zone);
    f.insert("getContextRoot".into(), fn_get_context_root);
    f.insert("GetContextRoot".into(), fn_get_context_root);
    f.insert("getPageContext".into(), fn_get_page_context);

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

    // ---- HTTP/Tag infrastructure (VM-intercepted) ----
    f.insert("__cfheader".into(), fn_cfheader_stub);
    f.insert("__cfcontent".into(), fn_cfcontent_stub);
    f.insert("__cflocation".into(), fn_cflocation_stub);
    f.insert("getHTTPRequestData".into(), fn_get_http_request_data_stub);
    f.insert("__cfinvoke".into(), fn_cfinvoke_stub);
    f.insert("__cfsavecontent_start".into(), fn_cfsavecontent_start_stub);
    f.insert("__cfsavecontent_end".into(), fn_cfsavecontent_end_stub);
    f.insert("__cfabort".into(), fn_cfabort_stub);
    f.insert("invoke".into(), fn_invoke_stub);
    f.insert("__cftransaction_start".into(), fn_cftransaction_start_stub);
    f.insert("__cftransaction_commit".into(), fn_cftransaction_commit_stub);
    f.insert("__cftransaction_rollback".into(), fn_cftransaction_rollback_stub);
    f.insert("cfdirectory".into(), fn_cfdirectory);

    // ---- HTTP functions ----
    #[cfg(feature = "http")]
    f.insert("cfhttp".into(), fn_cfhttp);

    // ---- Database functions ----
    #[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
    f.insert("queryExecute".into(), fn_query_execute);

    // ---- Security functions ----
    f.insert("hmac".into(), fn_hmac);
    #[cfg(feature = "security")]
    f.insert("generateSecretKey".into(), fn_generate_secret_key);
    f.insert("encrypt".into(), fn_encrypt);
    f.insert("decrypt".into(), fn_decrypt);

    // ---- XML functions ----
    #[cfg(feature = "xml")]
    {
        f.insert("xmlParse".into(), fn_xml_parse);
        f.insert("xmlSearch".into(), fn_xml_search);
        f.insert("isXML".into(), fn_is_xml);
        f.insert("xmlTransform".into(), fn_xml_transform_stub);
        f.insert("xmlValidate".into(), fn_xml_validate_stub);
    }

    f
}

fn create_builtin_func(name: &str) -> CfmlValue {
    CfmlValue::Function(CfmlFunction {
        name: name.to_string(),
        params: Vec::new(),
        body: CfmlClosureBody::Expression(Box::new(CfmlValue::Null)),
        return_type: None,
        access: CfmlAccess::Public,
        captured_scope: None,
    })
}

// ---- Helper functions ----

#[allow(dead_code)]
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

/// Case-insensitive key lookup for CFML structs. Returns the actual key in the HashMap.
fn struct_find_key_ci<'a>(s: &'a HashMap<String, CfmlValue>, key: &str) -> Option<&'a str> {
    if s.contains_key(key) {
        return Some(s.keys().find(|k| *k == key).unwrap());
    }
    let key_lower = key.to_lowercase();
    s.keys().find(|k| k.to_lowercase() == key_lower).map(|k| k.as_str())
}

/// CFML list splitting: each character in `delimiters` is a separate delimiter.
/// Empty elements are excluded (CFML default behavior).
fn cfml_list_split<'a>(list: &'a str, delimiters: &str) -> Vec<&'a str> {
    if list.is_empty() {
        return Vec::new();
    }
    list.split(|c: char| delimiters.contains(c))
        .filter(|s| !s.is_empty())
        .collect()
}

/// CFML list splitting that keeps empty elements (for includeEmptyValues=true).
fn cfml_list_split_keep_empty<'a>(list: &'a str, delimiters: &str) -> Vec<&'a str> {
    if list.is_empty() {
        return Vec::new();
    }
    list.split(|c: char| delimiters.contains(c)).collect()
}

// Thread-local xorshift64 PRNG state for deterministic randomize()/rand() support
thread_local! {
    static PRNG_STATE: std::cell::Cell<u64> = std::cell::Cell::new(0);
    static PRNG_SEEDED: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

fn xorshift64(state: u64) -> u64 {
    let mut x = state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

fn cfml_random() -> f64 {
    PRNG_SEEDED.with(|seeded| {
        if seeded.get() {
            PRNG_STATE.with(|state| {
                let next = xorshift64(state.get());
                state.set(next);
                (next >> 11) as f64 / (1u64 << 53) as f64
            })
        } else {
            // Fallback: time-based pseudo-random (non-deterministic)
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let x = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) as u64;
            (x >> 11) as f64 / (1u64 << 53) as f64
        }
    })
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
        let scope = if args.len() >= 4 { get_str(&args, 3).to_lowercase() } else { "one".to_string() };
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
        let find = get_str(&args, 1);
        let replace_with = get_str(&args, 2);
        let scope = if args.len() >= 4 { get_str(&args, 3).to_lowercase() } else { "one".to_string() };
        let find_lower = find.to_lowercase();

        if scope == "all" {
            let mut result = String::new();
            let lower = string.to_lowercase();
            let mut start = 0;
            while let Some(pos) = lower[start..].find(&find_lower) {
                result.push_str(&string[start..start + pos]);
                result.push_str(&replace_with);
                start += pos + find.len();
            }
            result.push_str(&string[start..]);
            Ok(CfmlValue::String(result))
        } else {
            let lower = string.to_lowercase();
            if let Some(pos) = lower.find(&find_lower) {
                let mut result = String::new();
                result.push_str(&string[..pos]);
                result.push_str(&replace_with);
                result.push_str(&string[pos + find.len()..]);
                Ok(CfmlValue::String(result))
            } else {
                Ok(CfmlValue::String(string))
            }
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
        let start = if args.len() >= 3 { (get_int(&args, 2) as usize).saturating_sub(1) } else { 0 };
        for (i, c) in string.chars().enumerate().skip(start) {
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
    let strip = args.get(2).map(|v| v.is_true()).unwrap_or(false);
    let input = if strip { string.replace('\n', " ").replace('\r', " ") } else { string };
    let mut result = String::new();
    let mut col = 0;
    for word in input.split_whitespace() {
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
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bytes = Vec::new();
    let chars: Vec<u8> = s.bytes().filter(|&b| b != b'\n' && b != b'\r' && b != b' ').collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 >= chars.len() { break; }
        let b0 = table.iter().position(|&c| c == chars[i]).unwrap_or(0) as u32;
        let b1 = table.iter().position(|&c| c == chars[i + 1]).unwrap_or(0) as u32;
        let b2 = if i + 2 < chars.len() && chars[i + 2] != b'=' { table.iter().position(|&c| c == chars[i + 2]).unwrap_or(0) as u32 } else { 0 };
        let b3 = if i + 3 < chars.len() && chars[i + 3] != b'=' { table.iter().position(|&c| c == chars[i + 3]).unwrap_or(0) as u32 } else { 0 };
        let triple = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;
        bytes.push(((triple >> 16) & 0xFF) as u8);
        if i + 2 < chars.len() && chars[i + 2] != b'=' { bytes.push(((triple >> 8) & 0xFF) as u8); }
        if i + 3 < chars.len() && chars[i + 3] != b'=' { bytes.push((triple & 0xFF) as u8); }
        i += 4;
    }
    Ok(CfmlValue::Binary(bytes))
}

fn fn_url_encode(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '*' => result.push(c),
            ' ' => result.push_str("%20"),
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
    let mut bytes = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    bytes.push(byte);
                }
                if chars.peek() != Some(&'%') {
                    if let Ok(decoded) = String::from_utf8(bytes.clone()) {
                        result.push_str(&decoded);
                    } else {
                        for b in &bytes { result.push(*b as char); }
                    }
                    bytes.clear();
                }
            }
            '+' => {
                if !bytes.is_empty() {
                    if let Ok(decoded) = String::from_utf8(bytes.clone()) {
                        result.push_str(&decoded);
                    }
                    bytes.clear();
                }
                result.push(' ');
            }
            _ => {
                if !bytes.is_empty() {
                    if let Ok(decoded) = String::from_utf8(bytes.clone()) {
                        result.push_str(&decoded);
                    }
                    bytes.clear();
                }
                result.push(c);
            }
        }
    }
    if !bytes.is_empty() {
        if let Ok(decoded) = String::from_utf8(bytes.clone()) {
            result.push_str(&decoded);
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

fn fn_encode_for_html(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    Ok(CfmlValue::String(
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2f;"),
    ))
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

fn add_thousands_separator(s: &str) -> String {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len <= 3 { return s.to_string(); }
    let mut result = String::new();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

fn fn_number_format(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_float(&args, 0);
    let mask = get_str(&args, 1);
    if mask.is_empty() {
        let rounded = n.round() as i64;
        let s = rounded.to_string();
        let negative = rounded < 0;
        let digits = if negative { &s[1..] } else { &s };
        let formatted = add_thousands_separator(digits);
        if negative {
            return Ok(CfmlValue::String(format!("-{}", formatted)));
        }
        return Ok(CfmlValue::String(formatted));
    }

    let has_dollar = mask.contains('$');
    let has_parens = mask.contains('(') && mask.contains(')');
    let has_plus = mask.contains('+');
    let has_comma = mask.contains(',');

    let decimals = if let Some(dot_pos) = mask.find('.') {
        mask[dot_pos + 1..].chars().filter(|c| *c == '9' || *c == '0' || *c == '_').count()
    } else {
        0
    };

    let formatted_num = format!("{:.prec$}", n.abs(), prec = decimals);
    let parts: Vec<&str> = formatted_num.split('.').collect();
    let int_part = parts[0];
    let dec_part = if parts.len() > 1 { parts[1] } else { "" };

    let int_formatted = if has_comma {
        add_thousands_separator(int_part)
    } else {
        int_part.to_string()
    };

    let mut result = if decimals > 0 {
        format!("{}.{}", int_formatted, dec_part)
    } else {
        int_formatted
    };

    if n < 0.0 {
        if has_parens {
            result = format!("({})", result);
        } else {
            result = format!("-{}", result);
        }
    } else if has_plus {
        result = format!("+{}", result);
    }

    if has_dollar {
        if result.starts_with('-') || result.starts_with('(') {
            let sign = result.chars().next().unwrap();
            result = format!("{}${}", sign, &result[1..]);
        } else {
            result = format!("${}", result);
        }
    }

    Ok(CfmlValue::String(result))
}

fn fn_decimal_format(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_float(&args, 0);
    let formatted = format!("{:.2}", n.abs());
    let parts: Vec<&str> = formatted.split('.').collect();
    let int_with_commas = add_thousands_separator(parts[0]);
    let result = format!("{}.{}", int_with_commas, parts.get(1).unwrap_or(&"00"));
    if n < 0.0 {
        Ok(CfmlValue::String(format!("-{}", result)))
    } else {
        Ok(CfmlValue::String(result))
    }
}

fn fn_format_base_n(args: Vec<CfmlValue>) -> CfmlResult {
    let n = get_int(&args, 0) as i32;
    let radix = get_int(&args, 1) as u32;
    if radix < 2 || radix > 36 {
        return Err(CfmlError::runtime("formatBaseN: radix must be between 2 and 36".to_string()));
    }
    let is_negative = n < 0;
    let abs_n = if is_negative { (n as i64).unsigned_abs() } else { n as u64 };
    if abs_n == 0 { return Ok(CfmlValue::String("0".to_string())); }
    let digits = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result = String::new();
    let mut val = abs_n;
    while val > 0 {
        let d = (val % radix as u64) as usize;
        result.push(digits.as_bytes()[d] as char);
        val /= radix as u64;
    }
    if is_negative { result.push('-'); }
    Ok(CfmlValue::String(result.chars().rev().collect()))
}

fn fn_input_base_n(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let radix = get_int(&args, 1) as u32;
    Ok(CfmlValue::Int(
        i64::from_str_radix(&s, radix).unwrap_or(0),
    ))
}

fn fn_replace_list(args: Vec<CfmlValue>) -> CfmlResult {
    let mut string = get_str(&args, 0);
    let list1 = get_str(&args, 1);
    let list2 = get_str(&args, 2);
    let delimiter = get_delimiter(&args, 3);
    let items1: Vec<&str> = list1.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
    let items2: Vec<&str> = list2.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
    for (i, find) in items1.iter().enumerate() {
        let replace_with = items2.get(i).unwrap_or(&"");
        string = string.replace(find, replace_with);
    }
    Ok(CfmlValue::String(string))
}

fn fn_replace_list_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    let mut string = get_str(&args, 0);
    let list1 = get_str(&args, 1);
    let list2 = get_str(&args, 2);
    let delimiter = get_delimiter(&args, 3);
    let items1: Vec<&str> = list1.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
    let items2: Vec<&str> = list2.split(|c: char| delimiter.contains(c)).filter(|s| !s.is_empty()).collect();
    for (i, find) in items1.iter().enumerate() {
        let replace_with = items2.get(i).unwrap_or(&"");
        let lower = string.to_lowercase();
        let find_lower = find.to_lowercase();
        let mut result = String::new();
        let mut start = 0;
        while let Some(pos) = lower[start..].find(&find_lower) {
            result.push_str(&string[start..start + pos]);
            result.push_str(replace_with);
            start += pos + find.len();
        }
        result.push_str(&string[start..]);
        string = result;
    }
    Ok(CfmlValue::String(string))
}

fn fn_xml_format(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    Ok(CfmlValue::String(
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;"),
    ))
}

fn fn_paragraph_format(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let result = s.replace("\r\n", "\n")
        .split('\n')
        .map(|line| if line.trim().is_empty() { "<p>".to_string() } else { format!("{}<br>", line) })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(CfmlValue::String(result))
}

fn fn_cjustify(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    let length = get_int(&args, 1) as usize;
    if s.len() >= length {
        return Ok(CfmlValue::String(s));
    }
    let padding = length - s.len();
    let left_pad = padding / 2;
    let right_pad = padding - left_pad;
    Ok(CfmlValue::String(format!("{}{}{}", " ".repeat(left_pad), s, " ".repeat(right_pad))))
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
        let mut result = arr.clone();
        let sort_type = if args.len() > 1 { get_str(&args, 1).to_lowercase() } else { "text".to_string() };
        let sort_order = if args.len() > 2 { get_str(&args, 2).to_lowercase() } else { "asc".to_string() };
        match sort_type.as_str() {
            "numeric" => {
                result.sort_by(|a, b| {
                    let fa = a.as_string().parse::<f64>().unwrap_or(0.0);
                    let fb = b.as_string().parse::<f64>().unwrap_or(0.0);
                    fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            "textnocase" => {
                result.sort_by(|a, b| a.as_string().to_lowercase().cmp(&b.as_string().to_lowercase()));
            }
            _ => {
                result.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
            }
        }
        if sort_order == "desc" { result.reverse(); }
        Ok(CfmlValue::Array(result))
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
        let offset = get_int(&args, 1);
        let length = if args.len() >= 3 { Some(get_int(&args, 2) as usize) } else { None };

        let start = if offset >= 0 {
            (offset as usize).saturating_sub(1) // 1-based to 0-based
        } else {
            // Negative: count from end
            let from_end = (-offset) as usize;
            if from_end > arr.len() { 0 } else { arr.len() - from_end }
        };

        if start >= arr.len() {
            return Ok(CfmlValue::Array(Vec::new()));
        }

        let end = match length {
            Some(len) => (start + len).min(arr.len()),
            None => arr.len(),
        };

        Ok(CfmlValue::Array(arr[start..end].to_vec()))
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
    if args.len() >= 2 {
        if let (CfmlValue::Array(a), CfmlValue::Array(b)) = (&args[0], &args[1]) {
            let leave_index = args.get(2).map(|v| v.is_true()).unwrap_or(false);
            if leave_index {
                let mut result = a.clone();
                for (i, item) in b.iter().enumerate() {
                    if i < result.len() {
                        result[i] = item.clone();
                    } else {
                        result.push(item.clone());
                    }
                }
                return Ok(CfmlValue::Array(result));
            } else {
                let mut result = a.clone();
                result.extend(b.clone());
                return Ok(CfmlValue::Array(result));
            }
        }
    }
    Ok(CfmlValue::Array(Vec::new()))
}

fn fn_array_clear(_args: Vec<CfmlValue>) -> CfmlResult {
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
fn fn_array_reduce(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Null)
}
fn fn_array_each(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Null)
}

fn fn_is_array(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Array(_)))))
}

fn fn_array_is_empty(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Array(arr)) => Ok(CfmlValue::Bool(arr.is_empty())),
        _ => Ok(CfmlValue::Bool(true)),
    }
}

fn fn_array_delete(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Array(arr) = &args[0] {
            let value_str = args[1].as_string().to_lowercase();
            let mut result = arr.clone();
            if let Some(pos) = result.iter().position(|v| v.as_string().to_lowercase() == value_str) {
                result.remove(pos);
            }
            return Ok(CfmlValue::Array(result));
        }
    }
    Ok(CfmlValue::Array(Vec::new()))
}

fn fn_array_find_all(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Array(arr) = &args[0] {
            let value = args[1].as_string();
            let indices: Vec<CfmlValue> = arr.iter().enumerate()
                .filter(|(_, v)| v.as_string() == value)
                .map(|(i, _)| CfmlValue::Int((i + 1) as i64))
                .collect();
            return Ok(CfmlValue::Array(indices));
        }
    }
    Ok(CfmlValue::Array(Vec::new()))
}

fn fn_array_find_all_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Array(arr) = &args[0] {
            let value = args[1].as_string().to_lowercase();
            let indices: Vec<CfmlValue> = arr.iter().enumerate()
                .filter(|(_, v)| v.as_string().to_lowercase() == value)
                .map(|(i, _)| CfmlValue::Int((i + 1) as i64))
                .collect();
            return Ok(CfmlValue::Array(indices));
        }
    }
    Ok(CfmlValue::Array(Vec::new()))
}

fn fn_array_first(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Array(arr)) => Ok(arr.first().cloned().unwrap_or(CfmlValue::Null)),
        _ => Err(CfmlError::runtime("arrayFirst: argument must be an array".to_string())),
    }
}

fn fn_array_last(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Array(arr)) => Ok(arr.last().cloned().unwrap_or(CfmlValue::Null)),
        _ => Err(CfmlError::runtime("arrayLast: argument must be an array".to_string())),
    }
}

fn fn_is_empty(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::String(s)) => Ok(CfmlValue::Bool(s.is_empty())),
        Some(CfmlValue::Array(arr)) => Ok(CfmlValue::Bool(arr.is_empty())),
        Some(CfmlValue::Struct(s)) => Ok(CfmlValue::Bool(s.is_empty())),
        Some(CfmlValue::Null) => Ok(CfmlValue::Bool(true)),
        _ => Ok(CfmlValue::Bool(false)),
    }
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
            return Ok(CfmlValue::Bool(struct_find_key_ci(s, &key).is_some()));
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
            if let Some(actual_key) = struct_find_key_ci(&result, &key) {
                let owned_key = actual_key.to_string();
                result.remove(&owned_key);
            }
            return Ok(CfmlValue::Struct(result));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_struct_insert(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 3 {
        if let CfmlValue::Struct(s) = &args[0] {
            let mut result = s.clone();
            let key = args[1].as_string();
            let allow_overwrite = if args.len() >= 4 { args[3].is_true() } else { true };
            if !allow_overwrite {
                if struct_find_key_ci(&result, &key).is_some() {
                    return Err(CfmlError::runtime(format!("Key '{}' already exists in struct. Use allowOverwrite=true to overwrite.", key)));
                }
            }
            if let Some(actual_key) = struct_find_key_ci(&result, &key) {
                if actual_key != key {
                    let owned_key = actual_key.to_string();
                    result.remove(&owned_key);
                }
            }
            result.insert(key, args[2].clone());
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
            if let Some(actual_key) = struct_find_key_ci(s, &key) {
                return Ok(s.get(actual_key).cloned().unwrap_or(CfmlValue::Null));
            }
            return Ok(CfmlValue::Null);
        }
    }
    Ok(CfmlValue::Null)
}

fn fn_struct_find_key(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Struct(s) = &args[0] {
            let key = get_str(&args, 1);
            let scope = if args.len() >= 3 { get_str(&args, 2).to_lowercase() } else { "one".to_string() };
            let mut results = Vec::new();
            struct_find_key_recursive(s, &key, "", &scope, &mut results);
            return Ok(CfmlValue::Array(results));
        }
    }
    Ok(CfmlValue::Array(Vec::new()))
}

fn struct_find_key_recursive(
    s: &HashMap<String, CfmlValue>,
    search_key: &str,
    path: &str,
    scope: &str,
    results: &mut Vec<CfmlValue>,
) {
    let search_lower = search_key.to_lowercase();
    for (k, v) in s {
        let current_path = if path.is_empty() { k.clone() } else { format!("{}.{}", path, k) };
        if k.to_lowercase() == search_lower {
            let mut result_struct = HashMap::new();
            result_struct.insert("owner".to_string(), CfmlValue::Struct(s.clone()));
            result_struct.insert("path".to_string(), CfmlValue::String(current_path.clone()));
            result_struct.insert("value".to_string(), v.clone());
            results.push(CfmlValue::Struct(result_struct));
            if scope == "one" { return; }
        }
        if let CfmlValue::Struct(nested) = v {
            struct_find_key_recursive(nested, search_key, &current_path, scope, results);
            if scope == "one" && !results.is_empty() { return; }
        }
        if let CfmlValue::Array(arr) = v {
            for (i, item) in arr.iter().enumerate() {
                if let CfmlValue::Struct(nested) = item {
                    let arr_path = format!("{}[{}]", current_path, i + 1);
                    struct_find_key_recursive(nested, search_key, &arr_path, scope, results);
                    if scope == "one" && !results.is_empty() { return; }
                }
            }
        }
    }
}

fn fn_struct_find_value(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Struct(s) = &args[0] {
            let search_value = get_str(&args, 1);
            let scope = if args.len() >= 3 { get_str(&args, 2).to_lowercase() } else { "one".to_string() };
            let mut results = Vec::new();
            struct_find_value_recursive(s, &search_value, "", &scope, &mut results);
            return Ok(CfmlValue::Array(results));
        }
    }
    Ok(CfmlValue::Array(Vec::new()))
}

fn struct_find_value_recursive(
    s: &HashMap<String, CfmlValue>,
    search_value: &str,
    path: &str,
    scope: &str,
    results: &mut Vec<CfmlValue>,
) {
    let search_lower = search_value.to_lowercase();
    for (k, v) in s {
        let current_path = if path.is_empty() { k.clone() } else { format!("{}.{}", path, k) };
        if v.as_string().to_lowercase() == search_lower {
            let mut result_struct = HashMap::new();
            result_struct.insert("owner".to_string(), CfmlValue::Struct(s.clone()));
            result_struct.insert("path".to_string(), CfmlValue::String(current_path.clone()));
            result_struct.insert("key".to_string(), CfmlValue::String(k.clone()));
            results.push(CfmlValue::Struct(result_struct));
            if scope == "one" { return; }
        }
        if let CfmlValue::Struct(nested) = v {
            struct_find_value_recursive(nested, search_value, &current_path, scope, results);
            if scope == "one" && !results.is_empty() { return; }
        }
        if let CfmlValue::Array(arr) = v {
            for (i, item) in arr.iter().enumerate() {
                if let CfmlValue::Struct(nested) = item {
                    let arr_path = format!("{}[{}]", current_path, i + 1);
                    struct_find_value_recursive(nested, search_value, &arr_path, scope, results);
                    if scope == "one" && !results.is_empty() { return; }
                }
            }
        }
    }
}

fn fn_struct_clear(_args: Vec<CfmlValue>) -> CfmlResult {
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
            let overwrite = if args.len() >= 3 { args[2].is_true() } else { true };
            let mut result = a.clone();
            for (k, v) in b {
                if overwrite || struct_find_key_ci(&result, k).is_none() {
                    result.insert(k.clone(), v.clone());
                }
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
        let sort_type = if args.len() > 1 { get_str(&args, 1).to_lowercase() } else { "text".to_string() };
        let sort_order = if args.len() > 2 { get_str(&args, 2).to_lowercase() } else { "asc".to_string() };
        let mut keys: Vec<String> = s.keys().cloned().collect();
        match sort_type.as_str() {
            "numeric" => {
                keys.sort_by(|a, b| {
                    let va = s.get(a).map(|v| v.as_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
                    let vb = s.get(b).map(|v| v.as_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
                    va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            "textnocase" => {
                keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            }
            _ => keys.sort(),
        }
        if sort_order == "desc" { keys.reverse(); }
        Ok(CfmlValue::Array(keys.into_iter().map(CfmlValue::String).collect()))
    } else {
        Ok(CfmlValue::Array(Vec::new()))
    }
}

fn fn_struct_each(_args: Vec<CfmlValue>) -> CfmlResult { Ok(CfmlValue::Null) }
fn fn_struct_map(args: Vec<CfmlValue>) -> CfmlResult { Ok(args.into_iter().next().unwrap_or(CfmlValue::Null)) }
fn fn_struct_filter(args: Vec<CfmlValue>) -> CfmlResult { Ok(args.into_iter().next().unwrap_or(CfmlValue::Null)) }

fn fn_is_struct(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Bool(matches!(args.first(), Some(CfmlValue::Struct(_)))))
}

fn fn_struct_get(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = CfmlValue::Struct(HashMap::new());
    for part in parts.iter().rev() {
        let mut s = HashMap::new();
        s.insert(part.to_string(), current);
        current = CfmlValue::Struct(s);
    }
    let mut result = &current;
    for part in &parts {
        if let CfmlValue::Struct(s) = result {
            if let Some(v) = s.get(*part) {
                result = v;
            }
        }
    }
    Ok(result.clone())
}

fn fn_struct_value_array(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Struct(s)) = args.first() {
        let values: Vec<CfmlValue> = s.values().cloned().collect();
        Ok(CfmlValue::Array(values))
    } else {
        Ok(CfmlValue::Array(Vec::new()))
    }
}

fn fn_struct_equals(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let (CfmlValue::Struct(a), CfmlValue::Struct(b)) = (&args[0], &args[1]) {
            if a.len() != b.len() { return Ok(CfmlValue::Bool(false)); }
            for (k, v) in a {
                match b.get(k) {
                    Some(bv) => {
                        if v.as_string() != bv.as_string() {
                            return Ok(CfmlValue::Bool(false));
                        }
                    }
                    None => {
                        match struct_find_key_ci(b, k) {
                            Some(actual) => {
                                if v.as_string() != b.get(actual).unwrap().as_string() {
                                    return Ok(CfmlValue::Bool(false));
                                }
                            }
                            None => return Ok(CfmlValue::Bool(false)),
                        }
                    }
                }
            }
            return Ok(CfmlValue::Bool(true));
        }
    }
    Ok(CfmlValue::Bool(false))
}

fn fn_struct_key_translate(args: Vec<CfmlValue>) -> CfmlResult {
    if let Some(CfmlValue::Struct(s)) = args.first() {
        let retain = args.get(1).map(|v| v.is_true()).unwrap_or(false);
        let mut result = HashMap::new();
        for (k, v) in s {
            let new_key = if retain { k.clone() } else { k.to_lowercase() };
            result.insert(new_key, v.clone());
        }
        return Ok(CfmlValue::Struct(result));
    }
    Err(CfmlError::runtime("structKeyTranslate requires a struct argument".into()))
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
        Some(CfmlValue::Bool(_) | CfmlValue::Int(_) | CfmlValue::Double(_) | CfmlValue::String(_))
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
        Some(CfmlValue::Int(_)) | Some(CfmlValue::Double(_)) => Ok(CfmlValue::Bool(true)),
        Some(CfmlValue::String(s)) => {
            let lower = s.trim().to_lowercase();
            Ok(CfmlValue::Bool(
                matches!(lower.as_str(), "true" | "false" | "yes" | "no")
                || s.trim().parse::<f64>().is_ok()
            ))
        }
        _ => Ok(CfmlValue::Bool(false)),
    }
}

fn fn_is_date(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    Ok(CfmlValue::Bool(parse_cfml_date(&s).is_some()))
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
            "query" => fn_is_query(vec![value.clone()]),
            "uuid" => {
                // CFML UUID format: 8-4-4-16 (35 chars total)
                let s = value.as_string();
                let re = Regex::new(r"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{16}$").unwrap();
                Ok(CfmlValue::Bool(re.is_match(&s)))
            }
            "guid" => {
                // Standard GUID format: 8-4-4-4-12
                let s = value.as_string();
                let re = Regex::new(r"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}$").unwrap();
                Ok(CfmlValue::Bool(re.is_match(&s)))
            }
            "range" => {
                // isValid("range", value, min, max)
                if args.len() >= 4 {
                    let n = value.as_string().parse::<f64>().unwrap_or(f64::NAN);
                    if n.is_nan() { return Ok(CfmlValue::Bool(false)); }
                    let min_val = get_float(&args, 2);
                    let max_val = get_float(&args, 3);
                    Ok(CfmlValue::Bool(n >= min_val && n <= max_val))
                } else {
                    Ok(CfmlValue::Bool(false))
                }
            }
            "regex" => {
                let s = value.as_string();
                Ok(CfmlValue::Bool(Regex::new(&s).is_ok()))
            }
            "creditcard" => {
                let s: String = value.as_string().chars().filter(|c| c.is_ascii_digit()).collect();
                if s.len() < 13 || s.len() > 19 { return Ok(CfmlValue::Bool(false)); }
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
            "zipcode" => {
                let s = value.as_string();
                let re = Regex::new(r"^\d{5}(-\d{4})?$").unwrap();
                Ok(CfmlValue::Bool(re.is_match(&s)))
            }
            "telephone" | "phone" => {
                let digits: String = value.as_string().chars().filter(|c| c.is_ascii_digit()).collect();
                Ok(CfmlValue::Bool(digits.len() >= 10 && digits.len() <= 15))
            }
            "ssn" | "social_security_number" => {
                let s = value.as_string();
                let re = Regex::new(r"^\d{3}-\d{2}-\d{4}$").unwrap();
                Ok(CfmlValue::Bool(re.is_match(&s)))
            }
            _ => Ok(CfmlValue::Bool(false)),
        }
    } else {
        Ok(CfmlValue::Bool(false))
    }
}

// ===============================================
// CONVERSION FUNCTIONS
// ===============================================

fn fn_to_string(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Binary(bytes)) => {
            Ok(CfmlValue::String(String::from_utf8_lossy(bytes).to_string()))
        }
        _ => Ok(CfmlValue::String(get_str(&args, 0))),
    }
}

fn fn_to_numeric(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Int(i)) => Ok(CfmlValue::Int(*i)),
        Some(CfmlValue::Double(d)) => Ok(CfmlValue::Double(*d)),
        Some(CfmlValue::Bool(b)) => Ok(CfmlValue::Int(if *b { 1 } else { 0 })),
        Some(CfmlValue::String(s)) => {
            let trimmed = s.trim();
            if let Ok(i) = trimmed.parse::<i64>() {
                Ok(CfmlValue::Int(i))
            } else if let Ok(d) = trimmed.parse::<f64>() {
                Ok(CfmlValue::Double(d))
            } else {
                Err(CfmlError::runtime(format!("Cannot convert '{}' to a number", s)))
            }
        }
        _ => Err(CfmlError::runtime("Cannot convert value to a number".to_string())),
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
        } else if (c == '-' || c == '+') && i == 0 {
            if c == '-' { num_str.push(c); }
            // Skip '+' sign (don't push it, but continue parsing)
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
    Ok(CfmlValue::Int(n.floor() as i64))
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
    Ok(CfmlValue::Double(cfml_random()))
}

fn fn_rand_range(args: Vec<CfmlValue>) -> CfmlResult {
    let min = get_int(&args, 0);
    let max = get_int(&args, 1);
    let range = (max - min + 1) as f64;
    let result = min + (cfml_random() * range).floor() as i64;
    Ok(CfmlValue::Int(result.min(max)))
}

fn fn_randomize(args: Vec<CfmlValue>) -> CfmlResult {
    let seed = get_float(&args, 0);
    // Seed the thread-local PRNG for deterministic rand() output
    let seed_bits = (seed.to_bits()).max(1); // ensure non-zero
    PRNG_STATE.with(|state| state.set(seed_bits));
    PRNG_SEEDED.with(|seeded| seeded.set(true));
    Ok(CfmlValue::Double(0.0))
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
    let n = get_int(&args, 0) as i32;
    Ok(CfmlValue::Int((!n) as i64))
}

fn fn_bit_shln(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_int(&args, 0) << get_int(&args, 1)))
}

fn fn_bit_shrn(args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::Int(get_int(&args, 0) >> get_int(&args, 1)))
}

// ===============================================
// DATE/TIME HELPERS
// ===============================================

/// Convert 2-digit year to 4-digit: 0-29 → 2000-2029, 30-99 → 1930-1999
fn short_year(y: i64) -> i64 {
    if y >= 0 && y <= 29 { 2000 + y }
    else if y >= 30 && y <= 99 { 1900 + y }
    else { y }
}

/// Days in a given month/year
fn days_in_month_calc(year: i32, month: u32) -> u32 {
    match month {
        1 => 31,
        2 => if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 29 } else { 28 },
        3 => 31, 4 => 30, 5 => 31, 6 => 30,
        7 => 31, 8 => 31, 9 => 30, 10 => 31, 11 => 30, 12 => 31,
        _ => 30,
    }
}

/// Add months to a NaiveDateTime, clamping day to valid range
fn add_months(dt: &NaiveDateTime, months: i64) -> Option<NaiveDateTime> {
    let total = (dt.year() as i64) * 12 + (dt.month0() as i64) + months;
    let new_year = total.div_euclid(12) as i32;
    let new_month = (total.rem_euclid(12) as u32) + 1;
    let max_day = days_in_month_calc(new_year, new_month);
    let new_day = dt.day().min(max_day);
    NaiveDate::from_ymd_opt(new_year, new_month, new_day)
        .and_then(|d| d.and_hms_opt(dt.hour(), dt.minute(), dt.second()))
}

/// Parse ODBC literal: {d '...'}, {t '...'}, {ts '...'}
fn parse_odbc_literal(s: &str) -> Option<NaiveDateTime> {
    let start = s.find('\'')?;
    let end = s.rfind('\'')?;
    if start >= end { return None; }
    let inner = &s[start+1..end];
    let lower = s.to_lowercase();
    if lower.starts_with("{ts ") {
        NaiveDateTime::parse_from_str(inner, "%Y-%m-%d %H:%M:%S").ok()
    } else if lower.starts_with("{d ") {
        NaiveDate::parse_from_str(inner, "%Y-%m-%d").ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
    } else if lower.starts_with("{t ") {
        NaiveTime::parse_from_str(inner, "%H:%M:%S").ok()
            .and_then(|t| NaiveDate::from_ymd_opt(2000, 1, 1).map(|d| d.and_time(t)))
    } else {
        None
    }
}

/// Central date parser: tries ODBC, ISO 8601, common US/EU formats, time-only, date serial
fn parse_cfml_date(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    if s.is_empty() { return None; }

    // ODBC literals
    if s.starts_with('{') {
        return parse_odbc_literal(s);
    }

    // DateTime formats (most specific first)
    for fmt in &[
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y %I:%M:%S %p",
        "%m/%d/%Y %H:%M",
        "%d %b %Y %H:%M:%S",
        "%b %d, %Y %H:%M:%S",
        "%B %d, %Y %H:%M:%S",
        "%d-%b-%Y %H:%M:%S",
    ] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
    }

    // Date-only formats → midnight
    for fmt in &[
        "%Y-%m-%d",
        "%m/%d/%Y",
        "%m-%d-%Y",
        "%d %b %Y",
        "%b %d, %Y",
        "%B %d, %Y",
        "%d-%b-%Y",
    ] {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return d.and_hms_opt(0, 0, 0);
        }
    }

    // Time-only → base date 2000-01-01
    for fmt in &["%H:%M:%S", "%I:%M:%S %p", "%H:%M"] {
        if let Ok(t) = NaiveTime::parse_from_str(s, fmt) {
            return NaiveDate::from_ymd_opt(2000, 1, 1).map(|d| d.and_time(t));
        }
    }

    // Date serial number (days since 1899-12-30, OLE Automation date)
    if let Ok(n) = s.parse::<f64>() {
        if n.is_finite() {
            let base = NaiveDate::from_ymd_opt(1899, 12, 30)?;
            let days = n.floor() as i64;
            let frac = n - n.floor();
            let secs = (frac * 86400.0) as u32;
            return base.and_hms_opt(0, 0, 0)
                .and_then(|dt| dt.checked_add_signed(chrono::Duration::days(days)))
                .and_then(|dt| dt.checked_add_signed(chrono::Duration::seconds(secs as i64)));
        }
    }

    None
}

/// Determines whether `m`/`mm` means month or minute
#[derive(Clone, Copy)]
enum FormatMode { Date, Time, DateTime }

fn month_name_full(m: u32) -> &'static str {
    match m {
        1 => "January", 2 => "February", 3 => "March", 4 => "April",
        5 => "May", 6 => "June", 7 => "July", 8 => "August",
        9 => "September", 10 => "October", 11 => "November", 12 => "December",
        _ => "",
    }
}
fn month_name_short(m: u32) -> &'static str {
    match m {
        1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
        5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
        9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
        _ => "",
    }
}
fn day_name_full(w: chrono::Weekday) -> &'static str {
    match w {
        chrono::Weekday::Mon => "Monday", chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday", chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday", chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    }
}
fn day_name_short(w: chrono::Weekday) -> &'static str {
    match w {
        chrono::Weekday::Mon => "Mon", chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed", chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri", chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    }
}
fn hour12(h: u32) -> u32 {
    match h % 12 { 0 => 12, other => other }
}

/// Resolve preset mask names into actual mask patterns
fn resolve_preset(mask: &str, mode: &FormatMode) -> String {
    let lower = mask.to_lowercase();
    match mode {
        FormatMode::Date => match lower.as_str() {
            "" => "dd-mmm-yy".into(),
            "short" => "m/d/yy".into(),
            "medium" => "mmm d, yyyy".into(),
            "long" => "mmmm d, yyyy".into(),
            "full" => "dddd, mmmm d, yyyy".into(),
            _ => mask.into(),
        },
        FormatMode::Time => match lower.as_str() {
            "" => "hh:mm tt".into(),
            "short" => "h:mm tt".into(),
            "medium" => "h:mm:ss tt".into(),
            "long" | "full" => "h:mm:ss tt".into(),
            _ => mask.into(),
        },
        FormatMode::DateTime => match lower.as_str() {
            "" => "dd-mmm-yyyy HH:nn:ss".into(),
            "short" => "m/d/yy h:nn tt".into(),
            "medium" => "mmm d, yyyy h:nn:ss tt".into(),
            "long" => "mmmm d, yyyy h:nn:ss tt".into(),
            "full" => "dddd, mmmm d, yyyy h:nn:ss tt".into(),
            _ => mask.into(),
        },
    }
}

/// Match a format token at position `pos` in the mask character array
fn match_format_token(chars: &[char], pos: usize, dt: &NaiveDateTime, mode: FormatMode) -> Option<(usize, String)> {
    let remaining = chars.len() - pos;
    // 4-char tokens
    if remaining >= 4 {
        let four: String = chars[pos..pos+4].iter().collect();
        match four.to_lowercase().as_str() {
            "dddd" => return Some((4, day_name_full(dt.weekday()).into())),
            "mmmm" => return Some((4, match mode {
                FormatMode::Time => format!("{:02}", dt.minute()),
                _ => month_name_full(dt.month()).into(),
            })),
            "yyyy" => return Some((4, format!("{:04}", dt.year()))),
            _ => {}
        }
    }
    // 3-char tokens
    if remaining >= 3 {
        let three: String = chars[pos..pos+3].iter().collect();
        match three.to_lowercase().as_str() {
            "ddd" => return Some((3, day_name_short(dt.weekday()).into())),
            "mmm" => return Some((3, match mode {
                FormatMode::Time => format!("{:02}", dt.minute()),
                _ => month_name_short(dt.month()).into(),
            })),
            _ => {}
        }
    }
    // 2-char tokens
    if remaining >= 2 {
        let two: String = chars[pos..pos+2].iter().collect();
        match two.as_str() {
            "dd" | "DD" => return Some((2, format!("{:02}", dt.day()))),
            "mm" | "MM" => return Some((2, match mode {
                FormatMode::Time => format!("{:02}", dt.minute()),
                _ => format!("{:02}", dt.month()),
            })),
            "yy" | "YY" => return Some((2, format!("{:02}", dt.year() % 100))),
            "HH" => return Some((2, format!("{:02}", dt.hour()))),
            "hh" => return Some((2, format!("{:02}", hour12(dt.hour())))),
            "nn" | "NN" => return Some((2, format!("{:02}", dt.minute()))),
            "ss" | "SS" => return Some((2, format!("{:02}", dt.second()))),
            "tt" | "TT" => return Some((2, if dt.hour() < 12 { "AM".into() } else { "PM".into() })),
            _ => {}
        }
    }
    // 1-char tokens
    if remaining >= 1 {
        match chars[pos] {
            'd' | 'D' => return Some((1, format!("{}", dt.day()))),
            'm' | 'M' => return Some((1, match mode {
                FormatMode::Time => format!("{}", dt.minute()),
                _ => format!("{}", dt.month()),
            })),
            'y' | 'Y' => return Some((1, format!("{:02}", dt.year() % 100))),
            'H' => return Some((1, format!("{}", dt.hour()))),
            'h' => return Some((1, format!("{}", hour12(dt.hour())))),
            'n' | 'N' => return Some((1, format!("{}", dt.minute()))),
            's' | 'S' => return Some((1, format!("{}", dt.second()))),
            't' | 'T' => return Some((1, if dt.hour() < 12 { "A".into() } else { "P".into() })),
            'l' | 'L' => return Some((1, "000".into())),
            _ => {}
        }
    }
    None
}

/// Format a NaiveDateTime using a CFML mask string
fn format_cfml_date(dt: &NaiveDateTime, mask: &str, mode: FormatMode) -> String {
    let resolved = match mask.to_lowercase().as_str() {
        "" | "short" | "medium" | "long" | "full" => resolve_preset(mask, &mode),
        _ => mask.to_string(),
    };
    let chars: Vec<char> = resolved.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    while i < chars.len() {
        if let Some((len, replacement)) = match_format_token(&chars, i, dt, mode) {
            result.push_str(&replacement);
            i += len;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

// ===============================================
// DATE/TIME FUNCTIONS
// ===============================================

fn fn_now(_args: Vec<CfmlValue>) -> CfmlResult {
    Ok(CfmlValue::String(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
}

fn fn_create_date(args: Vec<CfmlValue>) -> CfmlResult {
    let year = short_year(get_int(&args, 0));
    let month = get_int(&args, 1);
    let day = get_int(&args, 2);
    Ok(CfmlValue::String(format!("{:04}-{:02}-{:02}", year, month, day)))
}

fn fn_create_date_time(args: Vec<CfmlValue>) -> CfmlResult {
    let year = short_year(get_int(&args, 0));
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

fn fn_create_time(args: Vec<CfmlValue>) -> CfmlResult {
    let hour = get_int(&args, 0);
    let minute = get_int(&args, 1);
    let second = get_int(&args, 2);
    Ok(CfmlValue::String(format!("{:02}:{:02}:{:02}", hour, minute, second)))
}

fn fn_create_odbc_date(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    if let Some(dt) = parse_cfml_date(&s) {
        Ok(CfmlValue::String(format!("{{d '{}'}}", dt.format("%Y-%m-%d"))))
    } else {
        Ok(CfmlValue::String(format!("{{d '{}'}}", s)))
    }
}

fn fn_create_odbc_date_time(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    if let Some(dt) = parse_cfml_date(&s) {
        Ok(CfmlValue::String(format!("{{ts '{}'}}", dt.format("%Y-%m-%d %H:%M:%S"))))
    } else {
        Ok(CfmlValue::String(format!("{{ts '{}'}}", s)))
    }
}

fn fn_create_odbc_time(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    if let Some(dt) = parse_cfml_date(&s) {
        Ok(CfmlValue::String(format!("{{t '{}'}}", dt.format("%H:%M:%S"))))
    } else {
        Ok(CfmlValue::String(format!("{{t '{}'}}", s)))
    }
}

fn fn_date_add(args: Vec<CfmlValue>) -> CfmlResult {
    let part = get_str(&args, 0).to_lowercase();
    let number = get_int(&args, 1);
    let date_str = get_str(&args, 2);
    let dt = parse_cfml_date(&date_str)
        .ok_or_else(|| CfmlError::runtime(format!("Invalid date: {}", date_str)))?;

    let result: Option<NaiveDateTime> = match part.as_str() {
        "yyyy" => add_months(&dt, number * 12),
        "q" => add_months(&dt, number * 3),
        "m" => add_months(&dt, number),
        "y" | "d" => dt.checked_add_signed(chrono::Duration::days(number)),
        "w" => dt.checked_add_signed(chrono::Duration::days(number)),
        "ww" => dt.checked_add_signed(chrono::Duration::weeks(number)),
        "h" => dt.checked_add_signed(chrono::Duration::hours(number)),
        "n" => dt.checked_add_signed(chrono::Duration::minutes(number)),
        "s" => dt.checked_add_signed(chrono::Duration::seconds(number)),
        "l" => dt.checked_add_signed(chrono::Duration::milliseconds(number)),
        _ => Some(dt),
    };

    match result {
        Some(r) => Ok(CfmlValue::String(r.format("%Y-%m-%d %H:%M:%S").to_string())),
        None => Err(CfmlError::runtime("Date arithmetic overflow".into())),
    }
}

fn fn_date_diff(args: Vec<CfmlValue>) -> CfmlResult {
    let part = get_str(&args, 0).to_lowercase();
    let date1 = parse_cfml_date(&get_str(&args, 1))
        .ok_or_else(|| CfmlError::runtime("Invalid date1".into()))?;
    let date2 = parse_cfml_date(&get_str(&args, 2))
        .ok_or_else(|| CfmlError::runtime("Invalid date2".into()))?;

    let diff = match part.as_str() {
        "yyyy" => date2.year() as i64 - date1.year() as i64,
        "q" => {
            let q1 = (date1.year() as i64) * 4 + ((date1.month() as i64 - 1) / 3);
            let q2 = (date2.year() as i64) * 4 + ((date2.month() as i64 - 1) / 3);
            q2 - q1
        }
        "m" => {
            (date2.year() as i64 - date1.year() as i64) * 12
                + date2.month() as i64 - date1.month() as i64
        }
        "y" | "d" => (date2 - date1).num_days(),
        "w" => (date2 - date1).num_days() / 7,
        "ww" => (date2 - date1).num_days() / 7,
        "h" => (date2 - date1).num_hours(),
        "n" => (date2 - date1).num_minutes(),
        "s" => (date2 - date1).num_seconds(),
        "l" => (date2 - date1).num_milliseconds(),
        _ => 0,
    };
    Ok(CfmlValue::Int(diff))
}

fn fn_date_format(args: Vec<CfmlValue>) -> CfmlResult {
    let date_str = get_str(&args, 0);
    let mask = if args.len() > 1 { get_str(&args, 1) } else { String::new() };
    let dt = parse_cfml_date(&date_str)
        .ok_or_else(|| CfmlError::runtime(format!("Invalid date: {}", date_str)))?;
    Ok(CfmlValue::String(format_cfml_date(&dt, &mask, FormatMode::Date)))
}

fn fn_time_format(args: Vec<CfmlValue>) -> CfmlResult {
    let date_str = get_str(&args, 0);
    let mask = if args.len() > 1 { get_str(&args, 1) } else { String::new() };
    let dt = parse_cfml_date(&date_str)
        .ok_or_else(|| CfmlError::runtime(format!("Invalid date: {}", date_str)))?;
    Ok(CfmlValue::String(format_cfml_date(&dt, &mask, FormatMode::Time)))
}

fn fn_date_time_format(args: Vec<CfmlValue>) -> CfmlResult {
    let date_str = get_str(&args, 0);
    let mask = if args.len() > 1 { get_str(&args, 1) } else { String::new() };
    let dt = parse_cfml_date(&date_str)
        .ok_or_else(|| CfmlError::runtime(format!("Invalid date: {}", date_str)))?;
    Ok(CfmlValue::String(format_cfml_date(&dt, &mask, FormatMode::DateTime)))
}

fn fn_parse_date_time(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    match parse_cfml_date(&s) {
        Some(dt) => Ok(CfmlValue::String(dt.format("%Y-%m-%d %H:%M:%S").to_string())),
        None => Err(CfmlError::runtime(format!("Cannot parse date: {}", s))),
    }
}

fn fn_year(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(dt.year() as i64))
}

fn fn_month(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(dt.month() as i64))
}

fn fn_day(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(dt.day() as i64))
}

fn fn_hour(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(dt.hour() as i64))
}

fn fn_minute(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(dt.minute() as i64))
}

fn fn_second(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(dt.second() as i64))
}

/// CFML dayOfWeek: 1=Sunday, 2=Monday, ..., 7=Saturday
fn fn_day_of_week(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(dt.weekday().number_from_sunday() as i64))
}

fn fn_day_of_week_as_string(args: Vec<CfmlValue>) -> CfmlResult {
    let input = get_str(&args, 0);
    // Accept either a day number (1-7) or a date string
    let dow = if let Ok(n) = input.parse::<i64>() {
        n
    } else if let Some(dt) = parse_cfml_date(&input) {
        dt.weekday().number_from_sunday() as i64
    } else {
        return Ok(CfmlValue::String(String::new()));
    };
    let names = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    Ok(CfmlValue::String(names.get((dow - 1) as usize).unwrap_or(&"").to_string()))
}

fn fn_day_of_week_short_as_string(args: Vec<CfmlValue>) -> CfmlResult {
    let input = get_str(&args, 0);
    let dow = if let Ok(n) = input.parse::<i64>() {
        n
    } else if let Some(dt) = parse_cfml_date(&input) {
        dt.weekday().number_from_sunday() as i64
    } else {
        return Ok(CfmlValue::String(String::new()));
    };
    let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    Ok(CfmlValue::String(names.get((dow - 1) as usize).unwrap_or(&"").to_string()))
}

fn fn_day_of_year(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(dt.ordinal() as i64))
}

fn fn_days_in_month(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(days_in_month_calc(dt.year(), dt.month()) as i64))
}

fn fn_days_in_year(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    let y = dt.year();
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    Ok(CfmlValue::Int(if leap { 366 } else { 365 }))
}

/// Returns the day-of-year for the first day of the date's month
fn fn_first_day_of_month(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    let first = NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1)
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    Ok(CfmlValue::Int(first.ordinal() as i64))
}

fn fn_is_leap_year(args: Vec<CfmlValue>) -> CfmlResult {
    let input = get_str(&args, 0);
    // Accept either a year number or a date string
    let year = if let Ok(y) = input.parse::<i64>() {
        y
    } else if let Some(dt) = parse_cfml_date(&input) {
        dt.year() as i64
    } else {
        return Ok(CfmlValue::Bool(false));
    };
    Ok(CfmlValue::Bool((year % 4 == 0 && year % 100 != 0) || year % 400 == 0))
}

fn fn_month_as_string(args: Vec<CfmlValue>) -> CfmlResult {
    let input = get_str(&args, 0);
    let month = if let Ok(m) = input.parse::<i64>() {
        m
    } else if let Some(dt) = parse_cfml_date(&input) {
        dt.month() as i64
    } else {
        return Ok(CfmlValue::String(String::new()));
    };
    Ok(CfmlValue::String(month_name_full(month as u32).to_string()))
}

fn fn_month_short_as_string(args: Vec<CfmlValue>) -> CfmlResult {
    let input = get_str(&args, 0);
    let month = if let Ok(m) = input.parse::<i64>() {
        m
    } else if let Some(dt) = parse_cfml_date(&input) {
        dt.month() as i64
    } else {
        return Ok(CfmlValue::String(String::new()));
    };
    Ok(CfmlValue::String(month_name_short(month as u32).to_string()))
}

/// quarter(date) - returns 1-4 based on the month of the date
fn fn_quarter(args: Vec<CfmlValue>) -> CfmlResult {
    let input = get_str(&args, 0);
    let month = if let Ok(m) = input.parse::<i64>() {
        // Legacy: accept a raw month number
        m
    } else if let Some(dt) = parse_cfml_date(&input) {
        dt.month() as i64
    } else {
        return Ok(CfmlValue::Int(0));
    };
    Ok(CfmlValue::Int(((month - 1) / 3) + 1))
}

fn fn_week(args: Vec<CfmlValue>) -> CfmlResult {
    let dt = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    // CFML week: ISO week number
    Ok(CfmlValue::Int(dt.iso_week().week() as i64))
}

/// datePart(datepart, date) - extracts the specified part from a date
fn fn_date_part(args: Vec<CfmlValue>) -> CfmlResult {
    let part = get_str(&args, 0).to_lowercase();
    let dt = parse_cfml_date(&get_str(&args, 1))
        .ok_or_else(|| CfmlError::runtime("Invalid date".into()))?;
    let val = match part.as_str() {
        "yyyy" => dt.year() as i64,
        "q" => ((dt.month() as i64 - 1) / 3) + 1,
        "m" => dt.month() as i64,
        "y" => dt.ordinal() as i64,
        "d" => dt.day() as i64,
        "w" => dt.weekday().number_from_sunday() as i64,
        "ww" => dt.iso_week().week() as i64,
        "h" => dt.hour() as i64,
        "n" => dt.minute() as i64,
        "s" => dt.second() as i64,
        "l" => 0, // milliseconds not tracked
        _ => return Err(CfmlError::runtime(format!("Invalid datepart: {}", part))),
    };
    Ok(CfmlValue::Int(val))
}

/// dateCompare(date1, date2 [, datePart]) - returns -1, 0, or 1
fn fn_date_compare(args: Vec<CfmlValue>) -> CfmlResult {
    let dt1 = parse_cfml_date(&get_str(&args, 0))
        .ok_or_else(|| CfmlError::runtime("Invalid date1".into()))?;
    let dt2 = parse_cfml_date(&get_str(&args, 1))
        .ok_or_else(|| CfmlError::runtime("Invalid date2".into()))?;
    let part = if args.len() > 2 { get_str(&args, 2).to_lowercase() } else { "s".into() };

    // Truncate precision based on datepart
    let (v1, v2) = match part.as_str() {
        "yyyy" => (
            NaiveDate::from_ymd_opt(dt1.year(), 1, 1).unwrap().and_hms_opt(0,0,0).unwrap(),
            NaiveDate::from_ymd_opt(dt2.year(), 1, 1).unwrap().and_hms_opt(0,0,0).unwrap(),
        ),
        "m" => (
            NaiveDate::from_ymd_opt(dt1.year(), dt1.month(), 1).unwrap().and_hms_opt(0,0,0).unwrap(),
            NaiveDate::from_ymd_opt(dt2.year(), dt2.month(), 1).unwrap().and_hms_opt(0,0,0).unwrap(),
        ),
        "d" => (
            dt1.date().and_hms_opt(0,0,0).unwrap(),
            dt2.date().and_hms_opt(0,0,0).unwrap(),
        ),
        "h" => (
            dt1.date().and_hms_opt(dt1.hour(), 0, 0).unwrap(),
            dt2.date().and_hms_opt(dt2.hour(), 0, 0).unwrap(),
        ),
        "n" => (
            dt1.date().and_hms_opt(dt1.hour(), dt1.minute(), 0).unwrap(),
            dt2.date().and_hms_opt(dt2.hour(), dt2.minute(), 0).unwrap(),
        ),
        _ => (dt1, dt2), // "s" or default: full precision
    };

    let cmp = if v1 < v2 { -1i64 } else if v1 > v2 { 1 } else { 0 };
    Ok(CfmlValue::Int(cmp))
}

fn fn_get_tick_count(_args: Vec<CfmlValue>) -> CfmlResult {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
    Ok(CfmlValue::Int(ms))
}

fn fn_get_function_list(_args: Vec<CfmlValue>) -> CfmlResult {
    // Return a struct of all registered builtin function names
    // Keys are function names, values are empty strings (matching CFML behavior)
    let mut result = std::collections::HashMap::new();
    for (name, _) in get_builtin_functions() {
        result.insert(name, CfmlValue::String(String::new()));
    }
    Ok(CfmlValue::Struct(result))
}

fn fn_get_context_root(_args: Vec<CfmlValue>) -> CfmlResult {
    // In a servlet context, returns the context root. For RustCFML, always "".
    Ok(CfmlValue::String(String::new()))
}

fn fn_get_page_context(_args: Vec<CfmlValue>) -> CfmlResult {
    // Stub: getPageContext() returns a Java PageContext in ACF/Lucee.
    // We return a struct with common methods as stubs.
    let mut ctx = std::collections::HashMap::new();
    ctx.insert("getRequest".to_string(), CfmlValue::Null);
    ctx.insert("getResponse".to_string(), CfmlValue::Null);
    Ok(CfmlValue::Struct(ctx))
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
    Ok(CfmlValue::Int(cfml_list_split(&list, &delimiter).len() as i64))
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
    let items = cfml_list_split(&list, &delimiter);
    Ok(CfmlValue::String(items.get(index).unwrap_or(&"").to_string()))
}

fn fn_list_set_at(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let index = (get_int(&args, 1) as usize).saturating_sub(1);
    let value = get_str(&args, 2);
    let delimiter = get_delimiter(&args, 3);
    let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
    let mut items: Vec<String> = cfml_list_split(&list, &delimiter).iter().map(|s| s.to_string()).collect();
    if index < items.len() {
        items[index] = value;
    }
    Ok(CfmlValue::String(items.join(&first_delim)))
}

fn fn_list_insert_at(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let index = (get_int(&args, 1) as usize).saturating_sub(1);
    let value = get_str(&args, 2);
    let delimiter = get_delimiter(&args, 3);
    let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
    let mut items: Vec<String> = cfml_list_split(&list, &delimiter).iter().map(|s| s.to_string()).collect();
    if index <= items.len() {
        items.insert(index, value);
    }
    Ok(CfmlValue::String(items.join(&first_delim)))
}

fn fn_list_delete_at(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let index = (get_int(&args, 1) as usize).saturating_sub(1);
    let delimiter = get_delimiter(&args, 2);
    let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
    let mut items: Vec<String> = cfml_list_split(&list, &delimiter).iter().map(|s| s.to_string()).collect();
    if index < items.len() {
        items.remove(index);
    }
    Ok(CfmlValue::String(items.join(&first_delim)))
}

fn fn_list_find(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    for (i, item) in cfml_list_split(&list, &delimiter).iter().enumerate() {
        if item.trim() == value { return Ok(CfmlValue::Int((i + 1) as i64)); }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_list_find_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1).to_lowercase();
    let delimiter = get_delimiter(&args, 2);
    for (i, item) in cfml_list_split(&list, &delimiter).iter().enumerate() {
        if item.trim().to_lowercase() == value { return Ok(CfmlValue::Int((i + 1) as i64)); }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_list_contains(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    for (i, item) in cfml_list_split(&list, &delimiter).iter().enumerate() {
        if item.trim().contains(&value) { return Ok(CfmlValue::Int((i + 1) as i64)); }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_list_contains_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1).to_lowercase();
    let delimiter = get_delimiter(&args, 2);
    for (i, item) in cfml_list_split(&list, &delimiter).iter().enumerate() {
        if item.trim().to_lowercase().contains(&value) { return Ok(CfmlValue::Int((i + 1) as i64)); }
    }
    Ok(CfmlValue::Int(0))
}

fn fn_list_sort(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let sort_type = if args.len() > 1 { get_str(&args, 1).to_lowercase() } else { "text".to_string() };
    let sort_order = if args.len() > 2 { get_str(&args, 2).to_lowercase() } else { "asc".to_string() };
    let delimiter = if args.len() > 3 { get_str(&args, 3) } else { ",".to_string() };
    let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
    let mut items: Vec<String> = cfml_list_split(&list, &delimiter).iter().map(|s| s.trim().to_string()).collect();
    match sort_type.as_str() {
        "numeric" => {
            items.sort_by(|a, b| {
                let fa: f64 = a.parse().unwrap_or(0.0);
                let fb: f64 = b.parse().unwrap_or(0.0);
                fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        "textnocase" => {
            items.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        }
        _ => items.sort(), // "text" - case-sensitive
    }
    if sort_order == "desc" {
        items.reverse();
    }
    Ok(CfmlValue::String(items.join(&first_delim)))
}

fn fn_list_to_array(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let include_empty = args.get(2).map(|v| v.is_true()).unwrap_or(false);
    if list.is_empty() {
        return Ok(CfmlValue::Array(Vec::new()));
    }
    let items: Vec<CfmlValue> = if include_empty {
        cfml_list_split_keep_empty(&list, &delimiter).iter().map(|s| CfmlValue::String(s.to_string())).collect()
    } else {
        cfml_list_split(&list, &delimiter).iter().map(|s| CfmlValue::String(s.to_string())).collect()
    };
    Ok(CfmlValue::Array(items))
}

fn fn_list_first(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let items = cfml_list_split(&list, &delimiter);
    Ok(CfmlValue::String(items.first().unwrap_or(&"").to_string()))
}

fn fn_list_last(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let items = cfml_list_split(&list, &delimiter);
    Ok(CfmlValue::String(items.last().unwrap_or(&"").to_string()))
}

fn fn_list_rest(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
    let items = cfml_list_split(&list, &delimiter);
    if items.len() > 1 {
        Ok(CfmlValue::String(items[1..].join(&first_delim)))
    } else {
        Ok(CfmlValue::String(String::new()))
    }
}

fn fn_list_remove_duplicates(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let ignore_case = args.get(2).map(|v| v.is_true()).unwrap_or(false);
    let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
    let mut seen = Vec::new();
    let mut result = Vec::new();
    for item in cfml_list_split(&list, &delimiter) {
        let compare_key = if ignore_case { item.to_lowercase() } else { item.to_string() };
        if !seen.contains(&compare_key) {
            seen.push(compare_key);
            result.push(item.to_string());
        }
    }
    Ok(CfmlValue::String(result.join(&first_delim)))
}

fn fn_list_value_count(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    let count = cfml_list_split(&list, &delimiter).iter().filter(|s| s.trim() == value).count();
    Ok(CfmlValue::Int(count as i64))
}

fn fn_list_value_count_no_case(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let value = get_str(&args, 1).to_lowercase();
    let delimiter = get_delimiter(&args, 2);
    let count = cfml_list_split(&list, &delimiter).iter().filter(|s| s.trim().to_lowercase() == value).count();
    Ok(CfmlValue::Int(count as i64))
}

fn fn_list_change_delims(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let new_delim = get_str(&args, 1);
    let old_delim = get_delimiter(&args, 2);
    Ok(CfmlValue::String(cfml_list_split(&list, &old_delim).join(&new_delim)))
}

fn fn_list_qualify(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let qualifier = get_str(&args, 1);
    let delimiter = get_delimiter(&args, 2);
    let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
    let items: Vec<String> = cfml_list_split(&list, &delimiter).iter().map(|s| format!("{}{}{}", qualifier, s.trim(), qualifier)).collect();
    Ok(CfmlValue::String(items.join(&first_delim)))
}

fn fn_list_compact(args: Vec<CfmlValue>) -> CfmlResult {
    let list = get_str(&args, 0);
    let delimiter = get_delimiter(&args, 1);
    let first_delim = delimiter.chars().next().unwrap_or(',').to_string();
    let items: Vec<&str> = cfml_list_split(&list, &delimiter);
    Ok(CfmlValue::String(items.join(&first_delim)))
}

fn fn_list_each(_args: Vec<CfmlValue>) -> CfmlResult {
    // Needs VM closure support
    Err(CfmlError::runtime("listEach() requires VM-level closure support".to_string()))
}

fn fn_list_map(_args: Vec<CfmlValue>) -> CfmlResult {
    // Needs VM closure support
    Err(CfmlError::runtime("listMap() requires VM-level closure support".to_string()))
}

fn fn_list_filter(_args: Vec<CfmlValue>) -> CfmlResult {
    // Needs VM closure support
    Err(CfmlError::runtime("listFilter() requires VM-level closure support".to_string()))
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
        CfmlValue::Query(q) => {
            let rows: Vec<String> = q.rows.iter().map(|row| {
                let fields: Vec<String> = q.columns.iter().map(|col| {
                    let val = row.get(col).unwrap_or(&CfmlValue::Null);
                    format!("\"{}\":{}", col.replace('"', "\\\""), serialize_value(val))
                }).collect();
                format!("{{{}}}", fields.join(","))
            }).collect();
            format!("[{}]", rows.join(","))
        }
        _ => "null".to_string(),
    }
}

fn fn_deserialize_json(args: Vec<CfmlValue>) -> CfmlResult {
    let json = get_str(&args, 0);
    match serde_json::from_str::<serde_json::Value>(&json) {
        Ok(value) => Ok(serde_json_to_cfml(value)),
        Err(e) => Err(CfmlError::runtime(format!("Invalid JSON: {}", e))),
    }
}

fn serde_json_to_cfml(value: serde_json::Value) -> CfmlValue {
    match value {
        serde_json::Value::Null => CfmlValue::Null,
        serde_json::Value::Bool(b) => CfmlValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                CfmlValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                CfmlValue::Double(f)
            } else {
                CfmlValue::Int(0)
            }
        }
        serde_json::Value::String(s) => CfmlValue::String(s),
        serde_json::Value::Array(arr) => {
            CfmlValue::Array(arr.into_iter().map(serde_json_to_cfml).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, serde_json_to_cfml(v));
            }
            CfmlValue::Struct(map)
        }
    }
}

fn fn_is_json(args: Vec<CfmlValue>) -> CfmlResult {
    let s = get_str(&args, 0);
    Ok(CfmlValue::Bool(serde_json::from_str::<serde_json::Value>(&s).is_ok()))
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
        let num_rows = if args.len() >= 2 {
            match &args[1] {
                CfmlValue::Int(n) => *n as usize,
                CfmlValue::Struct(data) => {
                    let mut row = HashMap::new();
                    for (k, v) in data {
                        row.insert(k.clone(), v.clone());
                    }
                    result.rows.push(row);
                    return Ok(CfmlValue::Query(result));
                }
                CfmlValue::Array(rows) => {
                    for item in rows {
                        if let CfmlValue::Struct(data) = item {
                            result.rows.push(data.clone());
                        } else {
                            result.rows.push(HashMap::new());
                        }
                    }
                    return Ok(CfmlValue::Query(result));
                }
                _ => 1,
            }
        } else {
            1
        };
        for _ in 0..num_rows {
            result.rows.push(HashMap::new());
        }
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
            let col_name = args[1].as_string();
            result.columns.push(col_name.clone());
            if let Some(CfmlValue::Array(values)) = args.get(2) {
                for (i, val) in values.iter().enumerate() {
                    if i < result.rows.len() {
                        result.rows[i].insert(col_name.clone(), val.clone());
                    }
                }
            }
            return Ok(CfmlValue::Query(result));
        }
    }
    Ok(CfmlValue::Null)
}

fn fn_query_get_row(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Query(q) = &args[0] {
            let row_idx = (get_int(&args, 1) as usize).saturating_sub(1);
            if row_idx < q.rows.len() {
                return Ok(CfmlValue::Struct(q.rows[row_idx].clone()));
            }
            return Err(CfmlError::runtime(format!("queryGetRow: row {} is out of range (query has {} rows)", row_idx + 1, q.rows.len())));
        }
    }
    Err(CfmlError::runtime("queryGetRow requires a query and row number".to_string()))
}

fn fn_query_get_cell(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Query(q) = &args[0] {
            let column = args[1].as_string();
            let row_idx = if args.len() >= 3 {
                (get_int(&args, 2) as usize).saturating_sub(1)
            } else {
                0
            };
            if row_idx < q.rows.len() {
                let col_lower = column.to_lowercase();
                for (k, v) in &q.rows[row_idx] {
                    if k.to_lowercase() == col_lower {
                        return Ok(v.clone());
                    }
                }
                return Ok(CfmlValue::Null);
            }
            return Err(CfmlError::runtime(format!("queryGetCell: row {} is out of range", row_idx + 1)));
        }
    }
    Err(CfmlError::runtime("queryGetCell requires a query and column name".to_string()))
}

fn fn_query_record_count(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Query(q)) => Ok(CfmlValue::Int(q.rows.len() as i64)),
        _ => Ok(CfmlValue::Int(0)),
    }
}

fn fn_query_column_count(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Query(q)) => Ok(CfmlValue::Int(q.columns.len() as i64)),
        _ => Ok(CfmlValue::Int(0)),
    }
}

fn fn_query_column_list(args: Vec<CfmlValue>) -> CfmlResult {
    match args.first() {
        Some(CfmlValue::Query(q)) => Ok(CfmlValue::String(q.columns.join(","))),
        _ => Ok(CfmlValue::String(String::new())),
    }
}

fn fn_query_delete_row(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Query(q) = &args[0] {
            let mut result = q.clone();
            let row_idx = (get_int(&args, 1) as usize).saturating_sub(1);
            if row_idx < result.rows.len() {
                result.rows.remove(row_idx);
                return Ok(CfmlValue::Query(result));
            }
            return Err(CfmlError::runtime(format!("queryDeleteRow: row {} is out of range", row_idx + 1)));
        }
    }
    Ok(CfmlValue::Null)
}

fn fn_query_delete_column(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() >= 2 {
        if let CfmlValue::Query(q) = &args[0] {
            let mut result = q.clone();
            let col_name = args[1].as_string().to_lowercase();
            result.columns.retain(|c| c.to_lowercase() != col_name);
            for row in &mut result.rows {
                let key_to_remove: Option<String> = row.keys().find(|k| k.to_lowercase() == col_name).cloned();
                if let Some(key) = key_to_remove {
                    row.remove(&key);
                }
            }
            return Ok(CfmlValue::Query(result));
        }
    }
    Ok(CfmlValue::Null)
}

fn fn_query_ho_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    // Stub — VM intercepts these before reaching here
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
        match val {
            CfmlValue::Struct(s) => {
                // Extract __name
                if let Some(name) = s.get("__name") {
                    meta.insert("name".to_string(), name.clone());
                }
                // Type
                meta.insert("type".to_string(), CfmlValue::String("component".to_string()));

                // Extract __extends info
                if let Some(CfmlValue::Array(chain)) = s.get("__extends_chain") {
                    if let Some(first) = chain.first() {
                        let mut extends_meta = HashMap::new();
                        extends_meta.insert("name".to_string(), first.clone());
                        meta.insert("extends".to_string(), CfmlValue::Struct(extends_meta));
                    }
                    meta.insert("fullExtends".to_string(), CfmlValue::Array(chain.clone()));
                }

                // Extract __metadata (custom attributes)
                // In CFML, custom attributes appear as top-level keys in getMetadata()
                if let Some(CfmlValue::Struct(md)) = s.get("__metadata") {
                    for (mk, mv) in md {
                        meta.insert(mk.clone(), mv.clone());
                    }
                    meta.insert("metadata".to_string(), CfmlValue::Struct(md.clone()));
                }

                // Enumerate functions
                let mut functions = Vec::new();
                for (k, v) in s {
                    if k.starts_with("__") { continue; }
                    if let CfmlValue::Function(f) = v {
                        let mut func_meta = HashMap::new();
                        func_meta.insert("name".to_string(), CfmlValue::String(k.clone()));
                        func_meta.insert("access".to_string(), CfmlValue::String(
                            match f.access {
                                CfmlAccess::Public => "public",
                                CfmlAccess::Private => "private",
                                CfmlAccess::Package => "package",
                                CfmlAccess::Remote => "remote",
                            }.to_string()
                        ));
                        if let Some(ref rt) = f.return_type {
                            func_meta.insert("returnType".to_string(), CfmlValue::String(rt.clone()));
                        }
                        // Parameter details
                        let params: Vec<CfmlValue> = f.params.iter().map(|p| {
                            let mut pm = HashMap::new();
                            pm.insert("name".to_string(), CfmlValue::String(p.name.clone()));
                            if let Some(ref t) = p.param_type {
                                pm.insert("type".to_string(), CfmlValue::String(t.clone()));
                            }
                            pm.insert("required".to_string(), CfmlValue::Bool(p.required));
                            if let Some(ref d) = p.default {
                                pm.insert("default".to_string(), d.clone());
                            }
                            CfmlValue::Struct(pm)
                        }).collect();
                        func_meta.insert("parameters".to_string(), CfmlValue::Array(params));
                        // Check for function metadata (__funcmeta_<name>)
                        let meta_key = format!("__funcmeta_{}", k);
                        if let Some(CfmlValue::Struct(fm)) = s.get(&meta_key) {
                            func_meta.insert("metadata".to_string(), CfmlValue::Struct(fm.clone()));
                        }
                        functions.push(CfmlValue::Struct(func_meta));
                    }
                }
                meta.insert("functions".to_string(), CfmlValue::Array(functions));

                // Enumerate properties (non-function, non-internal keys)
                let mut properties = Vec::new();
                for (k, v) in s {
                    if k.starts_with("__") { continue; }
                    if matches!(v, CfmlValue::Function(_)) { continue; }
                    let mut prop_meta = HashMap::new();
                    prop_meta.insert("name".to_string(), CfmlValue::String(k.clone()));
                    prop_meta.insert("type".to_string(), CfmlValue::String(v.type_name().to_string()));
                    properties.push(CfmlValue::Struct(prop_meta));
                }
                meta.insert("properties".to_string(), CfmlValue::Array(properties));
            }
            CfmlValue::Function(f) => {
                meta.insert("name".to_string(), CfmlValue::String(f.name.clone()));
                meta.insert("access".to_string(), CfmlValue::String(
                    match f.access {
                        CfmlAccess::Public => "public",
                        CfmlAccess::Private => "private",
                        CfmlAccess::Package => "package",
                        CfmlAccess::Remote => "remote",
                    }.to_string()
                ));
                if let Some(ref rt) = f.return_type {
                    meta.insert("returnType".to_string(), CfmlValue::String(rt.clone()));
                }
                let params: Vec<CfmlValue> = f.params.iter().map(|p| {
                    let mut pm = HashMap::new();
                    pm.insert("name".to_string(), CfmlValue::String(p.name.clone()));
                    if let Some(ref t) = p.param_type {
                        pm.insert("type".to_string(), CfmlValue::String(t.clone()));
                    }
                    pm.insert("required".to_string(), CfmlValue::Bool(p.required));
                    if let Some(ref d) = p.default {
                        pm.insert("default".to_string(), d.clone());
                    }
                    CfmlValue::Struct(pm)
                }).collect();
                meta.insert("parameters".to_string(), CfmlValue::Array(params));
            }
            _ => {
                meta.insert("type".to_string(), CfmlValue::String(val.type_name().to_string()));
            }
        }
    }
    Ok(CfmlValue::Struct(meta))
}

fn fn_is_instance_of(args: Vec<CfmlValue>) -> CfmlResult {
    if args.len() < 2 {
        return Ok(CfmlValue::Bool(false));
    }
    let obj = &args[0];
    let type_name = args[1].as_string();
    let type_lower = type_name.to_lowercase();

    if let CfmlValue::Struct(s) = obj {
        // Check direct name match
        if let Some(CfmlValue::String(name)) = s.get("__name") {
            if name.to_lowercase() == type_lower {
                return Ok(CfmlValue::Bool(true));
            }
            // Also check last segment (e.g., "resource" matches "taffy.core.resource")
            if let Some(last) = name.split('.').last() {
                if last.to_lowercase() == type_lower {
                    return Ok(CfmlValue::Bool(true));
                }
            }
        }

        // Walk extends chain
        if let Some(CfmlValue::Array(chain)) = s.get("__extends_chain") {
            for item in chain {
                let item_str = item.as_string();
                if item_str.to_lowercase() == type_lower {
                    return Ok(CfmlValue::Bool(true));
                }
                // Check last segment
                if let Some(last) = item_str.split('.').last() {
                    if last.to_lowercase() == type_lower {
                        return Ok(CfmlValue::Bool(true));
                    }
                }
            }
        }

        // Check direct interfaces (__implements)
        if let Some(CfmlValue::Array(ifaces)) = s.get("__implements") {
            for item in ifaces {
                let item_str = item.as_string();
                if item_str.to_lowercase() == type_lower {
                    return Ok(CfmlValue::Bool(true));
                }
                if let Some(last) = item_str.split('.').last() {
                    if last.to_lowercase() == type_lower {
                        return Ok(CfmlValue::Bool(true));
                    }
                }
            }
        }

        // Check inherited interfaces (__implements_chain)
        if let Some(CfmlValue::Array(ifaces)) = s.get("__implements_chain") {
            for item in ifaces {
                let item_str = item.as_string();
                if item_str.to_lowercase() == type_lower {
                    return Ok(CfmlValue::Bool(true));
                }
                if let Some(last) = item_str.split('.').last() {
                    if last.to_lowercase() == type_lower {
                        return Ok(CfmlValue::Bool(true));
                    }
                }
            }
        }
    }

    Ok(CfmlValue::Bool(false))
}

fn fn_create_object(args: Vec<CfmlValue>) -> CfmlResult {
    // Stub - VM intercepts this call before it reaches here
    // If we get here, just return a struct with a marker
    if args.len() >= 2 {
        let obj_type = args[0].as_string().to_lowercase();
        if obj_type == "component" {
            let mut s = HashMap::new();
            s.insert("__createObject".to_string(), CfmlValue::String(args[1].as_string()));
            return Ok(CfmlValue::Struct(s));
        }
    }
    Ok(CfmlValue::Null)
}

fn fn_create_uuid(_args: Vec<CfmlValue>) -> CfmlResult {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let nanos = time.as_nanos() as u64;
    let random_bits = ((cfml_random() * u32::MAX as f64) as u64) << 32
                    | (cfml_random() * u32::MAX as f64) as u64;
    let mixed = nanos ^ random_bits;
    // CFML UUID format: 8-4-4-16
    Ok(CfmlValue::String(format!(
        "{:08X}-{:04X}-{:04X}-{:016X}",
        ((mixed >> 32) as u32),
        ((mixed >> 16) as u16),
        (mixed as u16),
        (nanos.wrapping_mul(6364136223846793005).wrapping_add(random_bits)),
    )))
}

fn fn_create_guid(_args: Vec<CfmlValue>) -> CfmlResult {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let nanos = time.as_nanos() as u64;
    let random_bits = ((cfml_random() * u32::MAX as f64) as u64) << 32
                    | (cfml_random() * u32::MAX as f64) as u64;
    let mixed = nanos ^ random_bits;
    let extra = nanos.wrapping_mul(6364136223846793005).wrapping_add(random_bits);
    // Standard GUID format: 8-4-4-4-12
    Ok(CfmlValue::String(format!(
        "{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
        (mixed >> 32) as u32,
        (mixed >> 16) as u16,
        ((mixed as u16) & 0x0FFF) | 0x4000,
        ((extra >> 48) as u16 & 0x3FFF) | 0x8000,
        extra & 0xFFFFFFFFFFFF,
    )))
}

fn fn_hash(args: Vec<CfmlValue>) -> CfmlResult {
    use md5::Md5;
    use sha2::{Sha256, Sha384, Sha512, Digest};
    use sha1::Sha1;
    let input = get_str(&args, 0);
    let algorithm = if args.len() >= 2 { get_str(&args, 1).to_uppercase() } else { "MD5".to_string() };
    let hex = match algorithm.as_str() {
        "MD5" => {
            let mut hasher = Md5::new();
            hasher.update(input.as_bytes());
            format!("{:X}", hasher.finalize())
        }
        "SHA-1" | "SHA1" => {
            let mut hasher = Sha1::new();
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
    // directoryList(path [, recurse [, listInfo [, filter]]])
    let path = get_str(&args, 0);
    let recurse = if args.len() >= 2 { args[1].is_true() } else { false };
    let list_info = if args.len() >= 3 { get_str(&args, 2).to_lowercase() } else { "path".to_string() };
    let filter = if args.len() >= 4 { get_str(&args, 3) } else { String::new() };

    fn matches_filter(filename: &str, filter: &str) -> bool {
        if filter.is_empty() { return true; }
        // Support glob patterns like "*.cfc", "*.cfm"
        if filter.starts_with("*.") {
            let ext = &filter[1..]; // ".cfc"
            filename.to_lowercase().ends_with(&ext.to_lowercase())
        } else if filter.contains('*') || filter.contains('?') {
            // Simple glob: convert to check
            filename.to_lowercase().contains(&filter.replace("*", "").to_lowercase())
        } else {
            filename.to_lowercase().contains(&filter.to_lowercase())
        }
    }

    fn list_dir(path: &str, recurse: bool, filter: &str, list_info: &str) -> Result<Vec<CfmlValue>, std::io::Error> {
        let mut results = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let full_path = entry_path.to_string_lossy().to_string();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if entry_path.is_file() && matches_filter(&file_name, filter) {
                let value = match list_info {
                    "name" => file_name.clone(),
                    _ => full_path.clone(), // "path" is default
                };
                results.push(CfmlValue::String(value));
            }
            if recurse && entry_path.is_dir() {
                results.extend(list_dir(&full_path, true, filter, list_info)?);
            }
        }
        Ok(results)
    }

    match list_dir(&path, recurse, &filter, &list_info) {
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

fn fn_get_directory_from_path(args: Vec<CfmlValue>) -> CfmlResult {
    let path = get_str(&args, 0);
    if path.is_empty() {
        return Ok(CfmlValue::String(String::new()));
    }
    let p = std::path::Path::new(&path);
    match p.parent() {
        Some(parent) => {
            let mut dir = parent.to_string_lossy().to_string();
            if !dir.is_empty() && !dir.ends_with(std::path::MAIN_SEPARATOR) {
                dir.push(std::path::MAIN_SEPARATOR);
            }
            Ok(CfmlValue::String(dir))
        }
        None => Ok(CfmlValue::String(path)),
    }
}

fn fn_get_current_template_path(_args: Vec<CfmlValue>) -> CfmlResult {
    // Stub — VM intercepts this call to return the actual template path
    Ok(CfmlValue::String(String::new()))
}

fn fn_get_component_metadata(_args: Vec<CfmlValue>) -> CfmlResult {
    // Stub — VM intercepts this call to resolve component metadata
    Ok(CfmlValue::Struct(HashMap::new()))
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

fn fn_list_reduce(_args: Vec<CfmlValue>) -> CfmlResult {
    // Needs VM closure support - stub
    Err(CfmlError::runtime("listReduce() requires VM-level closure support".to_string()))
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

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
enum DbDriver {
    Sqlite(String),
    Mysql(String),
    Postgres(String),
    Mssql(String),
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn parse_datasource(ds: &str) -> DbDriver {
    if ds.starts_with("mysql://") {
        DbDriver::Mysql(ds.to_string())
    } else if ds.starts_with("postgresql://") || ds.starts_with("postgres://") {
        DbDriver::Postgres(ds.to_string())
    } else if ds.starts_with("mssql://") || ds.starts_with("sqlserver://") {
        DbDriver::Mssql(ds.to_string())
    } else if ds.starts_with("sqlite://") {
        DbDriver::Sqlite(ds[9..].to_string())
    } else {
        DbDriver::Sqlite(ds.to_string()) // :memory: or file path
    }
}

// -----------------------------------------------
// Connection Pool Manager
// -----------------------------------------------

use std::sync::{Mutex, OnceLock};

/// Global pool manager — maps datasource URL → pool instance (type-erased)
static POOL_MANAGER: OnceLock<Mutex<HashMap<String, Box<dyn std::any::Any + Send>>>> = OnceLock::new();

fn get_pool_manager() -> &'static Mutex<HashMap<String, Box<dyn std::any::Any + Send>>> {
    POOL_MANAGER.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(feature = "sqlite")]
struct SqliteConnectionManager {
    path: String,
}

#[cfg(feature = "sqlite")]
impl r2d2::ManageConnection for SqliteConnectionManager {
    type Connection = rusqlite::Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        rusqlite::Connection::open(&self.path)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.execute_batch("SELECT 1").map_err(Into::into)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[cfg(feature = "sqlite")]
fn get_sqlite_pool(path: &str) -> Result<r2d2::Pool<SqliteConnectionManager>, CfmlError> {
    let mut manager = get_pool_manager().lock().unwrap();
    let key = format!("sqlite:{}", path);
    if let Some(pool_any) = manager.get(&key) {
        if let Some(pool) = pool_any.downcast_ref::<r2d2::Pool<SqliteConnectionManager>>() {
            return Ok(pool.clone());
        }
    }
    let mgr = SqliteConnectionManager { path: path.to_string() };
    let pool = r2d2::Pool::builder()
        .max_size(10)
        .min_idle(Some(1))
        .connection_timeout(std::time::Duration::from_secs(30))
        .build(mgr)
        .map_err(|e| CfmlError::runtime(format!("queryExecute: failed to create SQLite pool: {}", e)))?;
    manager.insert(key, Box::new(pool.clone()));
    Ok(pool)
}

#[cfg(feature = "mysql_db")]
fn get_mysql_pool(url: &str) -> Result<mysql::Pool, CfmlError> {
    let mut manager = get_pool_manager().lock().unwrap();
    let key = format!("mysql:{}", url);
    if let Some(pool_any) = manager.get(&key) {
        if let Some(pool) = pool_any.downcast_ref::<mysql::Pool>() {
            return Ok(pool.clone());
        }
    }
    let pool = mysql::Pool::new(url)
        .map_err(|e| CfmlError::runtime(format!("queryExecute: MySQL pool creation error: {}", e)))?;
    manager.insert(key, Box::new(pool.clone()));
    Ok(pool)
}

#[cfg(feature = "postgres_db")]
struct PostgresConnectionManager {
    url: String,
}

#[cfg(feature = "postgres_db")]
impl r2d2::ManageConnection for PostgresConnectionManager {
    type Connection = postgres::Client;
    type Error = postgres::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        postgres::Client::connect(&self.url, postgres::NoTls)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.simple_query("SELECT 1").map(|_| ())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[cfg(feature = "postgres_db")]
fn get_postgres_pool(url: &str) -> Result<r2d2::Pool<PostgresConnectionManager>, CfmlError> {
    let mut manager = get_pool_manager().lock().unwrap();
    let key = format!("postgres:{}", url);
    if let Some(pool_any) = manager.get(&key) {
        if let Some(pool) = pool_any.downcast_ref::<r2d2::Pool<PostgresConnectionManager>>() {
            return Ok(pool.clone());
        }
    }
    let mgr = PostgresConnectionManager { url: url.to_string() };
    let pool = r2d2::Pool::builder()
        .max_size(10)
        .min_idle(Some(1))
        .connection_timeout(std::time::Duration::from_secs(30))
        .build(mgr)
        .map_err(|e| CfmlError::runtime(format!("queryExecute: failed to create PostgreSQL pool: {}", e)))?;
    manager.insert(key, Box::new(pool.clone()));
    Ok(pool)
}

// -----------------------------------------------
// Structured query parameter normalization
// -----------------------------------------------

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn normalize_query_params(params_arg: &CfmlValue) -> (Vec<CfmlValue>, Vec<String>) {
    // If params is an array of structs with "value" key, extract typed values
    // Returns (effective_values, type_hints)
    match params_arg {
        CfmlValue::Array(arr) if !arr.is_empty() => {
            // Check if first element is a struct with "value" key (cfqueryparam style)
            if let Some(CfmlValue::Struct(first)) = arr.first() {
                let has_value_key = first.iter().any(|(k, _)| k.eq_ignore_ascii_case("value"));
                if has_value_key {
                    let mut values = Vec::with_capacity(arr.len());
                    let mut type_hints = Vec::with_capacity(arr.len());
                    for item in arr {
                        if let CfmlValue::Struct(s) = item {
                            let value = s.iter()
                                .find(|(k, _)| k.eq_ignore_ascii_case("value"))
                                .map(|(_, v)| v.clone())
                                .unwrap_or(CfmlValue::Null);

                            let is_null = s.iter()
                                .find(|(k, _)| k.eq_ignore_ascii_case("null"))
                                .map(|(_, v)| {
                                    match v {
                                        CfmlValue::Bool(b) => *b,
                                        CfmlValue::String(s) => s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes"),
                                        _ => false,
                                    }
                                })
                                .unwrap_or(false);

                            let cfsqltype = s.iter()
                                .find(|(k, _)| k.eq_ignore_ascii_case("cfsqltype"))
                                .map(|(_, v)| v.as_string().to_lowercase())
                                .unwrap_or_else(|| "cf_sql_varchar".to_string());

                            let is_list = s.iter()
                                .find(|(k, _)| k.eq_ignore_ascii_case("list"))
                                .map(|(_, v)| {
                                    match v {
                                        CfmlValue::Bool(b) => *b,
                                        CfmlValue::String(s) => s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes"),
                                        _ => false,
                                    }
                                })
                                .unwrap_or(false);

                            let separator = s.iter()
                                .find(|(k, _)| k.eq_ignore_ascii_case("separator"))
                                .map(|(_, v)| v.as_string())
                                .unwrap_or_else(|| ",".to_string());

                            if is_null {
                                values.push(CfmlValue::Null);
                                type_hints.push(cfsqltype);
                            } else if is_list {
                                // Expand list value into multiple params
                                let val_str = value.as_string();
                                for part in val_str.split(&*separator) {
                                    let trimmed = part.trim();
                                    values.push(coerce_by_sqltype(trimmed, &cfsqltype));
                                    type_hints.push(cfsqltype.clone());
                                }
                            } else {
                                let coerced = coerce_by_sqltype_value(&value, &cfsqltype);
                                values.push(coerced);
                                type_hints.push(cfsqltype);
                            }
                        } else {
                            values.push(item.clone());
                            type_hints.push("cf_sql_varchar".to_string());
                        }
                    }
                    return (values, type_hints);
                }
            }
            // Plain array — pass through
            (arr.clone(), vec!["cf_sql_varchar".to_string(); arr.len()])
        }
        _ => (vec![], vec![]),
    }
}

/// Get how many placeholder values each structured param generates (1 for normal, N for list)
#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn get_list_placeholder_counts(params_arg: &CfmlValue) -> Vec<usize> {
    match params_arg {
        CfmlValue::Array(arr) => {
            arr.iter().map(|item| {
                if let CfmlValue::Struct(s) = item {
                    let is_list = s.iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case("list"))
                        .map(|(_, v)| {
                            match v {
                                CfmlValue::Bool(b) => *b,
                                CfmlValue::String(s) => s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes"),
                                _ => false,
                            }
                        })
                        .unwrap_or(false);
                    if is_list {
                        let separator = s.iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case("separator"))
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_else(|| ",".to_string());
                        let value = s.iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case("value"))
                            .map(|(_, v)| v.as_string())
                            .unwrap_or_default();
                        value.split(&*separator).filter(|s| !s.trim().is_empty()).count().max(1)
                    } else {
                        1
                    }
                } else {
                    1
                }
            }).collect()
        }
        _ => vec![],
    }
}

/// Expand SQL ? placeholders for list params: if param N generates 3 values, replace its ? with ?,?,?
#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn expand_sql_placeholders(sql: &str, counts: &[usize]) -> String {
    let mut result = String::with_capacity(sql.len() + counts.len() * 4);
    let mut param_idx = 0;
    let bytes = sql.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'?' && param_idx < counts.len() {
            let count = counts[param_idx];
            for j in 0..count {
                if j > 0 { result.push(','); }
                result.push('?');
            }
            param_idx += 1;
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
    result
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn coerce_by_sqltype(val_str: &str, sqltype: &str) -> CfmlValue {
    match sqltype {
        s if s.contains("integer") || s.contains("bigint") || s.contains("smallint") || s.contains("tinyint") => {
            val_str.parse::<i64>().map(CfmlValue::Int).unwrap_or(CfmlValue::String(val_str.to_string()))
        }
        s if s.contains("float") || s.contains("double") || s.contains("decimal") || s.contains("numeric") || s.contains("real") || s.contains("money") => {
            val_str.parse::<f64>().map(CfmlValue::Double).unwrap_or(CfmlValue::String(val_str.to_string()))
        }
        s if s.contains("bit") || s.contains("boolean") => {
            let lower = val_str.to_lowercase();
            CfmlValue::Bool(lower == "true" || lower == "yes" || lower == "1")
        }
        _ => CfmlValue::String(val_str.to_string()),
    }
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn coerce_by_sqltype_value(val: &CfmlValue, sqltype: &str) -> CfmlValue {
    match sqltype {
        s if s.contains("integer") || s.contains("bigint") || s.contains("smallint") || s.contains("tinyint") => {
            match val {
                CfmlValue::Int(_) => val.clone(),
                CfmlValue::Double(d) => CfmlValue::Int(*d as i64),
                CfmlValue::String(s) => s.parse::<i64>().map(CfmlValue::Int).unwrap_or(val.clone()),
                CfmlValue::Bool(b) => CfmlValue::Int(if *b { 1 } else { 0 }),
                _ => val.clone(),
            }
        }
        s if s.contains("float") || s.contains("double") || s.contains("decimal") || s.contains("numeric") || s.contains("real") || s.contains("money") => {
            match val {
                CfmlValue::Double(_) => val.clone(),
                CfmlValue::Int(i) => CfmlValue::Double(*i as f64),
                CfmlValue::String(s) => s.parse::<f64>().map(CfmlValue::Double).unwrap_or(val.clone()),
                _ => val.clone(),
            }
        }
        s if s.contains("bit") || s.contains("boolean") => {
            match val {
                CfmlValue::Bool(_) => val.clone(),
                CfmlValue::Int(i) => CfmlValue::Bool(*i != 0),
                CfmlValue::String(s) => {
                    let lower = s.to_lowercase();
                    CfmlValue::Bool(lower == "true" || lower == "yes" || lower == "1")
                }
                _ => val.clone(),
            }
        }
        s if s == "cf_sql_null" => CfmlValue::Null,
        _ => val.clone(),
    }
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn is_select_query(sql: &str) -> bool {
    let trimmed = sql.trim_start();
    trimmed.len() >= 6 && trimmed[..6].eq_ignore_ascii_case("SELECT")
}

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
pub fn fn_query_execute(args: Vec<CfmlValue>) -> CfmlResult {
    let sql = get_str(&args, 0);
    if sql.is_empty() {
        return Err(CfmlError::runtime("queryExecute: SQL string is required".to_string()));
    }

    let raw_params = args.get(1).cloned().unwrap_or(CfmlValue::Null);
    let options_arg = args.get(2).cloned().unwrap_or(CfmlValue::Null);

    // Normalize structured params (cfqueryparam-style array of structs) to plain values
    // Also expand SQL for list params (single ? → multiple ?,?,?)
    let (sql, params_arg) = match &raw_params {
        CfmlValue::Array(arr) if !arr.is_empty() => {
            if let Some(CfmlValue::Struct(first)) = arr.first() {
                if first.iter().any(|(k, _)| k.eq_ignore_ascii_case("value")) {
                    let (values, _hints) = normalize_query_params(&raw_params);
                    // Check if any list params require SQL expansion
                    let placeholder_counts = get_list_placeholder_counts(&raw_params);
                    let expanded_sql = if placeholder_counts.iter().any(|&c| c > 1) {
                        expand_sql_placeholders(&sql, &placeholder_counts)
                    } else {
                        sql
                    };
                    (expanded_sql, CfmlValue::Array(values))
                } else {
                    (sql, raw_params)
                }
            } else {
                (sql, raw_params)
            }
        }
        _ => (sql, raw_params),
    };

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
        #[cfg(feature = "mssql_db")]
        DbDriver::Mssql(url) => execute_mssql(&url, &sql, &params_arg, &return_type),
        #[allow(unreachable_patterns)]
        _ => Err(CfmlError::runtime(format!(
            "queryExecute: database driver not available for datasource '{}'. Enable the appropriate feature (sqlite, mysql_db, postgres_db, mssql_db).",
            datasource
        ))),
    }
}

// -----------------------------------------------
// SQLite driver
// -----------------------------------------------

#[cfg(feature = "sqlite")]
fn execute_sqlite(path: &str, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    use rusqlite::types::Value as SqlValue;

    let pool = get_sqlite_pool(path)?;
    let conn = pool.get()
        .map_err(|e| CfmlError::runtime(format!("queryExecute: failed to get SQLite connection from pool: {}", e)))?;

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

    let pool = get_mysql_pool(url)?;
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
    let pool = get_postgres_pool(url)?;
    let mut client = pool.get()
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
// MSSQL driver (tiberius)
// -----------------------------------------------

#[cfg(feature = "mssql_db")]
fn execute_mssql(url: &str, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    use tiberius::{Client, Config, AuthMethod};
    use tokio::runtime::Builder;
    use tokio_util::compat::TokioAsyncWriteCompatExt;

    // Parse URL: mssql://user:pass@host:port/database or sqlserver://...
    let clean_url = url.replace("mssql://", "").replace("sqlserver://", "");
    // Format: user:pass@host:port/database
    let (auth_part, host_db) = clean_url.split_once('@')
        .ok_or_else(|| CfmlError::runtime("queryExecute: MSSQL URL must be mssql://user:pass@host:port/database".to_string()))?;
    let (user, pass) = auth_part.split_once(':')
        .ok_or_else(|| CfmlError::runtime("queryExecute: MSSQL URL must include user:password".to_string()))?;
    let (host_port, database) = host_db.split_once('/')
        .unwrap_or((host_db, "master"));
    let (host, port_str) = host_port.split_once(':')
        .unwrap_or((host_port, "1433"));
    let port: u16 = port_str.parse().unwrap_or(1433);

    let mut config = Config::new();
    config.host(host);
    config.port(port);
    config.database(database);
    config.authentication(AuthMethod::sql_server(user, pass));
    config.trust_cert();

    let addr = format!("{}:{}", host, port);

    // Normalize params
    let (effective_params, _type_hints) = normalize_query_params(params_arg);

    // Create single-threaded tokio runtime (safe inside spawn_blocking)
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CfmlError::runtime(format!("queryExecute: failed to create MSSQL runtime: {}", e)))?;

    rt.block_on(async {
        let tcp = tokio::net::TcpStream::connect(&addr).await
            .map_err(|e| CfmlError::runtime(format!("queryExecute: MSSQL connection error: {}", e)))?;
        let mut client = Client::connect(config, tcp.compat_write()).await
            .map_err(|e| CfmlError::runtime(format!("queryExecute: MSSQL connection error: {}", e)))?;

        if is_select_query(sql) {
            // Build query with inline params (tiberius simple query)
            let query_sql = mssql_inline_params(sql, &effective_params);
            let stream = client.simple_query(&query_sql).await
                .map_err(|e| CfmlError::runtime(format!("queryExecute: MSSQL query error: {}", e)))?;
            let result = stream.into_first_result().await
                .map_err(|e| CfmlError::runtime(format!("queryExecute: MSSQL result error: {}", e)))?;

            let columns: Vec<String> = if let Some(first_row) = result.first() {
                first_row.columns().iter()
                    .map(|c| c.name().to_string())
                    .collect()
            } else {
                vec![]
            };

            let mut rows: Vec<HashMap<String, CfmlValue>> = Vec::with_capacity(result.len());
            for row in &result {
                let mut row_map = HashMap::new();
                for (i, col) in columns.iter().enumerate() {
                    let val = mssql_column_to_cfml(row.get::<'_, &str, _>(i));
                    row_map.insert(col.clone(), val);
                }
                rows.push(row_map);
            }

            build_query_result(columns, rows, sql, return_type)
        } else {
            let query_sql = mssql_inline_params(sql, &effective_params);
            let result = client.simple_query(&query_sql).await
                .map_err(|e| CfmlError::runtime(format!("queryExecute: MSSQL error: {}", e)))?;
            let rows = result.into_first_result().await
                .map_err(|e| CfmlError::runtime(format!("queryExecute: MSSQL result error: {}", e)))?;
            build_mutation_result(rows.len() as i64, 0)
        }
    })
}

#[cfg(feature = "mssql_db")]
fn mssql_inline_params(sql: &str, params: &[CfmlValue]) -> String {
    if params.is_empty() {
        return sql.to_string();
    }
    let mut result = String::with_capacity(sql.len() + params.len() * 10);
    let mut param_idx = 0;
    let bytes = sql.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'?' && param_idx < params.len() {
            match &params[param_idx] {
                CfmlValue::Null => result.push_str("NULL"),
                CfmlValue::Int(n) => result.push_str(&n.to_string()),
                CfmlValue::Double(d) => result.push_str(&d.to_string()),
                CfmlValue::Bool(b) => result.push_str(if *b { "1" } else { "0" }),
                CfmlValue::String(s) => {
                    result.push('\'');
                    result.push_str(&s.replace('\'', "''"));
                    result.push('\'');
                }
                _ => {
                    let s = params[param_idx].as_string();
                    result.push('\'');
                    result.push_str(&s.replace('\'', "''"));
                    result.push('\'');
                }
            }
            param_idx += 1;
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
    result
}

#[cfg(feature = "mssql_db")]
fn mssql_column_to_cfml(val: Option<&str>) -> CfmlValue {
    match val {
        None => CfmlValue::Null,
        Some(s) => {
            // Try integer first, then float, else string
            if let Ok(i) = s.parse::<i64>() {
                CfmlValue::Int(i)
            } else if let Ok(f) = s.parse::<f64>() {
                CfmlValue::Double(f)
            } else {
                CfmlValue::String(s.to_string())
            }
        }
    }
}

// -----------------------------------------------
// Transaction support (public functions called by VM)
// -----------------------------------------------

/// Enum to hold driver-specific transaction connections
enum TransactionConn {
    #[cfg(feature = "sqlite")]
    Sqlite(r2d2::PooledConnection<SqliteConnectionManager>),
    #[cfg(feature = "mysql_db")]
    Mysql(mysql::PooledConn),
    #[cfg(feature = "postgres_db")]
    Postgres(r2d2::PooledConnection<PostgresConnectionManager>),
}

// ---- Public wrappers using Box<dyn Any> for VM interop ----

/// Begin a transaction — returns a type-erased connection in a Box<dyn Any>
#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
pub fn txn_begin_boxed(datasource: &str) -> Result<Box<dyn std::any::Any>, CfmlError> {
    let conn = transaction_begin(datasource)?;
    Ok(Box::new(conn))
}

/// Commit a transaction via type-erased connection
#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
pub fn txn_commit_boxed(conn: &mut Box<dyn std::any::Any>) -> Result<(), CfmlError> {
    if let Some(tc) = conn.downcast_mut::<TransactionConn>() {
        transaction_commit(tc)
    } else {
        Err(CfmlError::runtime("cftransaction: invalid transaction connection".to_string()))
    }
}

/// Rollback a transaction via type-erased connection
#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
pub fn txn_rollback_boxed(conn: &mut Box<dyn std::any::Any>) -> Result<(), CfmlError> {
    if let Some(tc) = conn.downcast_mut::<TransactionConn>() {
        transaction_rollback(tc)
    } else {
        Err(CfmlError::runtime("cftransaction: invalid transaction connection".to_string()))
    }
}

/// Execute a query within a transaction via type-erased connection
#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
pub fn txn_execute_boxed(conn: &mut Box<dyn std::any::Any>, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    if let Some(tc) = conn.downcast_mut::<TransactionConn>() {
        execute_with_transaction(tc, sql, params_arg, return_type)
    } else {
        Err(CfmlError::runtime("cftransaction: invalid transaction connection".to_string()))
    }
}

/// Begin a transaction — returns a held connection
#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn transaction_begin(datasource: &str) -> Result<TransactionConn, CfmlError> {
    match parse_datasource(datasource) {
        #[cfg(feature = "sqlite")]
        DbDriver::Sqlite(path) => {
            let pool = get_sqlite_pool(&path)?;
            let conn = pool.get()
                .map_err(|e| CfmlError::runtime(format!("cftransaction: SQLite pool error: {}", e)))?;
            conn.execute_batch("BEGIN")
                .map_err(|e| CfmlError::runtime(format!("cftransaction: BEGIN error: {}", e)))?;
            Ok(TransactionConn::Sqlite(conn))
        }
        #[cfg(feature = "mysql_db")]
        DbDriver::Mysql(url) => {
            let pool = get_mysql_pool(&url)?;
            let mut conn = pool.get_conn()
                .map_err(|e| CfmlError::runtime(format!("cftransaction: MySQL pool error: {}", e)))?;
            use mysql::prelude::Queryable;
            conn.query_drop("BEGIN")
                .map_err(|e| CfmlError::runtime(format!("cftransaction: BEGIN error: {}", e)))?;
            Ok(TransactionConn::Mysql(conn))
        }
        #[cfg(feature = "postgres_db")]
        DbDriver::Postgres(url) => {
            let pool = get_postgres_pool(&url)?;
            let mut conn = pool.get()
                .map_err(|e| CfmlError::runtime(format!("cftransaction: PostgreSQL pool error: {}", e)))?;
            conn.simple_query("BEGIN")
                .map_err(|e| CfmlError::runtime(format!("cftransaction: BEGIN error: {}", e)))?;
            Ok(TransactionConn::Postgres(conn))
        }
        #[allow(unreachable_patterns)]
        _ => Err(CfmlError::runtime(format!(
            "cftransaction: unsupported datasource '{}'", datasource
        ))),
    }
}

/// Commit a transaction
fn transaction_commit(conn: &mut TransactionConn) -> Result<(), CfmlError> {
    match conn {
        #[cfg(feature = "sqlite")]
        TransactionConn::Sqlite(c) => {
            c.execute_batch("COMMIT")
                .map_err(|e| CfmlError::runtime(format!("cftransaction: COMMIT error: {}", e)))
        }
        #[cfg(feature = "mysql_db")]
        TransactionConn::Mysql(c) => {
            use mysql::prelude::Queryable;
            c.query_drop("COMMIT")
                .map_err(|e| CfmlError::runtime(format!("cftransaction: COMMIT error: {}", e)))
        }
        #[cfg(feature = "postgres_db")]
        TransactionConn::Postgres(c) => {
            c.simple_query("COMMIT").map(|_| ())
                .map_err(|e| CfmlError::runtime(format!("cftransaction: COMMIT error: {}", e)))
        }
        #[allow(unreachable_patterns)]
        _ => Ok(()),
    }
}

/// Rollback a transaction
fn transaction_rollback(conn: &mut TransactionConn) -> Result<(), CfmlError> {
    match conn {
        #[cfg(feature = "sqlite")]
        TransactionConn::Sqlite(c) => {
            c.execute_batch("ROLLBACK")
                .map_err(|e| CfmlError::runtime(format!("cftransaction: ROLLBACK error: {}", e)))
        }
        #[cfg(feature = "mysql_db")]
        TransactionConn::Mysql(c) => {
            use mysql::prelude::Queryable;
            c.query_drop("ROLLBACK")
                .map_err(|e| CfmlError::runtime(format!("cftransaction: ROLLBACK error: {}", e)))
        }
        #[cfg(feature = "postgres_db")]
        TransactionConn::Postgres(c) => {
            c.simple_query("ROLLBACK").map(|_| ())
                .map_err(|e| CfmlError::runtime(format!("cftransaction: ROLLBACK error: {}", e)))
        }
        #[allow(unreachable_patterns)]
        _ => Ok(()),
    }
}

/// Execute a query using an existing transaction connection
fn execute_with_transaction(conn: &mut TransactionConn, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    match conn {
        #[cfg(feature = "sqlite")]
        TransactionConn::Sqlite(c) => {
            execute_sqlite_with_conn(c, sql, params_arg, return_type)
        }
        #[cfg(feature = "mysql_db")]
        TransactionConn::Mysql(c) => {
            execute_mysql_with_conn(c, sql, params_arg, return_type)
        }
        #[cfg(feature = "postgres_db")]
        TransactionConn::Postgres(c) => {
            execute_postgres_with_conn(c, sql, params_arg, return_type)
        }
        #[allow(unreachable_patterns)]
        _ => Err(CfmlError::runtime("Transaction: unsupported driver".to_string())),
    }
}

#[cfg(feature = "sqlite")]
fn execute_sqlite_with_conn(conn: &rusqlite::Connection, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    use rusqlite::types::Value as SqlValue;

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

#[cfg(feature = "mysql_db")]
fn execute_mysql_with_conn(conn: &mut mysql::PooledConn, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    use mysql::*;
    use mysql::prelude::*;

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
        let columns: Vec<String> = if let Some(first_row) = result.first() {
            first_row.columns_ref().iter()
                .map(|c| c.name_str().to_string())
                .collect()
        } else {
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

#[cfg(feature = "postgres_db")]
fn execute_postgres_with_conn(client: &mut postgres::Client, sql: &str, params_arg: &CfmlValue, return_type: &str) -> CfmlResult {
    let (rewritten_sql, ordered_params) = rewrite_params_for_postgres(sql, params_arg)?;
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
        build_mutation_result(affected as i64, 0)
    }
}

// -----------------------------------------------
// Shared result builders
// -----------------------------------------------

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
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

#[cfg(any(feature = "sqlite", feature = "mysql_db", feature = "postgres_db", feature = "mssql_db"))]
fn build_mutation_result(affected: i64, last_id: i64) -> CfmlResult {
    let mut result = HashMap::new();
    result.insert("recordCount".to_string(), CfmlValue::Int(affected));
    result.insert("generatedKey".to_string(), CfmlValue::Int(last_id));
    Ok(CfmlValue::Struct(result))
}

// -----------------------------------------------
// HTTP/Tag infrastructure stubs (VM-intercepted)
// -----------------------------------------------

fn fn_cfheader_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cfheader requires VM intercept".into()))
}

fn fn_cfcontent_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cfcontent requires VM intercept".into()))
}

fn fn_cflocation_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cflocation requires VM intercept".into()))
}

fn fn_get_http_request_data_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("getHTTPRequestData requires VM intercept".into()))
}

fn fn_cfinvoke_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cfinvoke requires VM intercept".into()))
}

fn fn_cfsavecontent_start_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cfsavecontent_start requires VM intercept".into()))
}

fn fn_cfsavecontent_end_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cfsavecontent_end requires VM intercept".into()))
}

fn fn_cfabort_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cfabort requires VM intercept".into()))
}

fn fn_invoke_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("invoke requires VM intercept".into()))
}

fn fn_cftransaction_start_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cftransaction_start requires VM intercept".into()))
}

fn fn_cftransaction_commit_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cftransaction_commit requires VM intercept".into()))
}

fn fn_cftransaction_rollback_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("__cftransaction_rollback requires VM intercept".into()))
}

// -----------------------------------------------
// cfdirectory - Full builtin implementation
// -----------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn fn_cfdirectory(args: Vec<CfmlValue>) -> CfmlResult {
    use std::fs;
    use std::path::Path;

    let opts = match args.first() {
        Some(CfmlValue::Struct(s)) => s,
        _ => return Err(CfmlError::runtime("cfdirectory requires a struct argument".into())),
    };

    // Case-insensitive key lookup helper
    fn get_ci<'a>(s: &'a HashMap<String, CfmlValue>, key: &str) -> Option<&'a CfmlValue> {
        let key_lower = key.to_lowercase();
        for (k, v) in s {
            if k.to_lowercase() == key_lower {
                return Some(v);
            }
        }
        None
    }

    let action = get_ci(opts, "action")
        .map(|v| v.as_string().to_lowercase())
        .unwrap_or_else(|| "list".into());

    let directory = get_ci(opts, "directory")
        .map(|v| v.as_string())
        .unwrap_or_default();

    match action.as_str() {
        "list" => {
            let filter = get_ci(opts, "filter")
                .map(|v| v.as_string())
                .unwrap_or_else(|| "*".into());
            let recurse = get_ci(opts, "recurse")
                .map(|v| match v {
                    CfmlValue::Bool(b) => *b,
                    CfmlValue::String(s) => {
                        let l = s.to_lowercase();
                        l == "true" || l == "yes"
                    }
                    _ => false,
                })
                .unwrap_or(false);

            let columns = vec![
                "name".to_string(),
                "directory".to_string(),
                "type".to_string(),
                "size".to_string(),
                "datelastmodified".to_string(),
            ];
            let mut rows: Vec<HashMap<String, CfmlValue>> = Vec::new();

            fn matches_glob(name: &str, pattern: &str) -> bool {
                if pattern == "*" {
                    return true;
                }
                if let Some(ext) = pattern.strip_prefix("*.") {
                    name.to_lowercase().ends_with(&format!(".{}", ext.to_lowercase()))
                } else {
                    name.to_lowercase() == pattern.to_lowercase()
                }
            }

            fn list_dir(
                dir: &Path,
                filter: &str,
                recurse: bool,
                rows: &mut Vec<HashMap<String, CfmlValue>>,
            ) -> Result<(), CfmlError> {
                let entries = fs::read_dir(dir).map_err(|e| {
                    CfmlError::runtime(format!("cfdirectory: cannot read directory: {}", e))
                })?;

                for entry in entries {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = metadata.is_dir();

                    if !is_dir && !matches_glob(&name, filter) {
                        if recurse && is_dir {
                            // still recurse into non-matching dirs
                        } else if !is_dir {
                            // skip non-matching files
                            if recurse && metadata.is_dir() {
                                // unreachable but safe
                            }
                            // actually skip
                        }
                    }

                    let file_type = if is_dir { "Dir" } else { "File" };
                    let size = if is_dir { 0i64 } else { metadata.len() as i64 };
                    let modified = metadata
                        .modified()
                        .ok()
                        .and_then(|t| {
                            let dt: chrono::DateTime<chrono::Local> = t.into();
                            Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        })
                        .unwrap_or_default();

                    let should_include = is_dir || matches_glob(&name, filter);

                    if should_include {
                        let mut row = HashMap::new();
                        row.insert("name".into(), CfmlValue::String(name.clone()));
                        row.insert(
                            "directory".into(),
                            CfmlValue::String(dir.to_string_lossy().to_string()),
                        );
                        row.insert("type".into(), CfmlValue::String(file_type.into()));
                        row.insert("size".into(), CfmlValue::Int(size));
                        row.insert("datelastmodified".into(), CfmlValue::String(modified));
                        rows.push(row);
                    }

                    if recurse && is_dir {
                        list_dir(&entry.path(), filter, recurse, rows)?;
                    }
                }
                Ok(())
            }

            list_dir(Path::new(&directory), &filter, recurse, &mut rows)?;

            let query = CfmlQuery {
                columns,
                rows,
                sql: None,
            };
            Ok(CfmlValue::Query(query))
        }
        "create" => {
            fs::create_dir_all(&directory).map_err(|e| {
                CfmlError::runtime(format!("cfdirectory create failed: {}", e))
            })?;
            Ok(CfmlValue::Null)
        }
        "delete" => {
            fs::remove_dir_all(&directory).map_err(|e| {
                CfmlError::runtime(format!("cfdirectory delete failed: {}", e))
            })?;
            Ok(CfmlValue::Null)
        }
        "rename" => {
            let new_dir = get_ci(opts, "newdirectory")
                .map(|v| v.as_string())
                .unwrap_or_default();
            if new_dir.is_empty() {
                return Err(CfmlError::runtime(
                    "cfdirectory rename requires 'newdirectory' attribute".into(),
                ));
            }
            fs::rename(&directory, &new_dir).map_err(|e| {
                CfmlError::runtime(format!("cfdirectory rename failed: {}", e))
            })?;
            Ok(CfmlValue::Null)
        }
        _ => Err(CfmlError::runtime(format!(
            "cfdirectory: unsupported action '{}'",
            action
        ))),
    }
}

#[cfg(target_arch = "wasm32")]
fn fn_cfdirectory(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("cfdirectory is not supported in wasm".into()))
}

// ==== ENCODING HELPERS ====

fn base64_encode_bytes(data: &[u8]) -> String {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(alphabet[((n >> 18) & 63) as usize] as char);
        result.push(alphabet[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            result.push(alphabet[((n >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(alphabet[(n & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode_bytes(s: &str) -> Vec<u8> {
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bytes = Vec::new();
    let chars: Vec<u8> = s.bytes().filter(|&b| b != b'\n' && b != b'\r' && b != b' ').collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 >= chars.len() { break; }
        let b0 = table.iter().position(|&c| c == chars[i]).unwrap_or(0) as u32;
        let b1 = table.iter().position(|&c| c == chars[i + 1]).unwrap_or(0) as u32;
        let b2 = if i + 2 < chars.len() && chars[i + 2] != b'=' {
            table.iter().position(|&c| c == chars[i + 2]).unwrap_or(0) as u32
        } else {
            0
        };
        let b3 = if i + 3 < chars.len() && chars[i + 3] != b'=' {
            table.iter().position(|&c| c == chars[i + 3]).unwrap_or(0) as u32
        } else {
            0
        };
        let triple = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;
        bytes.push(((triple >> 16) & 0xFF) as u8);
        if i + 2 < chars.len() && chars[i + 2] != b'=' {
            bytes.push(((triple >> 8) & 0xFF) as u8);
        }
        if i + 3 < chars.len() && chars[i + 3] != b'=' {
            bytes.push((triple & 0xFF) as u8);
        }
        i += 4;
    }
    bytes
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err("Invalid hex string length".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("Invalid hex: {}", e)))
        .collect()
}

fn uu_encode(data: &[u8]) -> String {
    let mut result = String::new();
    for chunk in data.chunks(45) {
        result.push((chunk.len() as u8 + 32) as char);
        for triple in chunk.chunks(3) {
            let b0 = triple[0] as u32;
            let b1 = triple.get(1).copied().unwrap_or(0) as u32;
            let b2 = triple.get(2).copied().unwrap_or(0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            result.push((((n >> 18) & 63) as u8).wrapping_add(32) as char);
            result.push((((n >> 12) & 63) as u8).wrapping_add(32) as char);
            result.push((((n >> 6) & 63) as u8).wrapping_add(32) as char);
            result.push(((n & 63) as u8).wrapping_add(32) as char);
        }
        result.push('\n');
    }
    result
}

fn uu_decode(s: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for line in s.lines() {
        if line.is_empty() { continue; }
        let line_bytes: Vec<u8> = line.bytes().collect();
        if line_bytes.is_empty() { continue; }
        let expected_len = (line_bytes[0].wrapping_sub(32) & 63) as usize;
        let mut i = 1;
        let mut decoded_in_line = Vec::new();
        while i + 3 < line_bytes.len() {
            let b0 = (line_bytes[i].wrapping_sub(32) & 63) as u32;
            let b1 = (line_bytes[i + 1].wrapping_sub(32) & 63) as u32;
            let b2 = (line_bytes[i + 2].wrapping_sub(32) & 63) as u32;
            let b3 = (line_bytes[i + 3].wrapping_sub(32) & 63) as u32;
            let n = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;
            decoded_in_line.push(((n >> 16) & 0xFF) as u8);
            decoded_in_line.push(((n >> 8) & 0xFF) as u8);
            decoded_in_line.push((n & 0xFF) as u8);
            i += 4;
        }
        decoded_in_line.truncate(expected_len);
        bytes.extend_from_slice(&decoded_in_line);
    }
    bytes
}

// ==== CIPHER HELPERS ====

fn cfmx_compat_encrypt(data: &[u8], key: &str) -> Vec<u8> {
    // CFMX_COMPAT is a simple XOR cipher with a key-derived seed
    let seed: u32 = key.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
    let mut rng = seed;
    data.iter().map(|&b| {
        rng = rng.wrapping_mul(214013).wrapping_add(2531011);
        let keystream = ((rng >> 16) & 0xFF) as u8;
        b ^ keystream
    }).collect()
}

fn cfmx_compat_decrypt(data: &[u8], key: &str) -> Vec<u8> {
    // XOR is symmetric, so decrypt = encrypt
    cfmx_compat_encrypt(data, key)
}

fn aes_cbc_encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    use aes::Aes128;
    use aes::Aes192;
    use aes::Aes256;
    use cbc::Encryptor;
    use cbc::cipher::BlockEncryptMut;
    use cbc::cipher::KeyIvInit;

    let iv = vec![0u8; 16]; // Zero IV (CFML default)

    match key.len() {
        16 => {
            let encryptor = Encryptor::<Aes128>::new_from_slices(key, &iv)
                .map_err(|e| format!("AES-128 init error: {}", e))?;
            Ok(encryptor.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(plaintext))
        }
        24 => {
            let encryptor = Encryptor::<Aes192>::new_from_slices(key, &iv)
                .map_err(|e| format!("AES-192 init error: {}", e))?;
            Ok(encryptor.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(plaintext))
        }
        32 => {
            let encryptor = Encryptor::<Aes256>::new_from_slices(key, &iv)
                .map_err(|e| format!("AES-256 init error: {}", e))?;
            Ok(encryptor.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(plaintext))
        }
        _ => Err(format!("Invalid AES key length: {} bytes (expected 16, 24, or 32)", key.len()))
    }
}

fn aes_cbc_decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    use aes::Aes128;
    use aes::Aes192;
    use aes::Aes256;
    use cbc::Decryptor;
    use cbc::cipher::BlockDecryptMut;
    use cbc::cipher::KeyIvInit;

    let iv = vec![0u8; 16];

    match key.len() {
        16 => {
            let decryptor = Decryptor::<Aes128>::new_from_slices(key, &iv)
                .map_err(|e| format!("AES-128 init error: {}", e))?;
            decryptor.decrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(ciphertext)
                .map_err(|e| format!("AES decryption error: {}", e))
        }
        24 => {
            let decryptor = Decryptor::<Aes192>::new_from_slices(key, &iv)
                .map_err(|e| format!("AES-192 init error: {}", e))?;
            decryptor.decrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(ciphertext)
                .map_err(|e| format!("AES decryption error: {}", e))
        }
        32 => {
            let decryptor = Decryptor::<Aes256>::new_from_slices(key, &iv)
                .map_err(|e| format!("AES-256 init error: {}", e))?;
            decryptor.decrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(ciphertext)
                .map_err(|e| format!("AES decryption error: {}", e))
        }
        _ => Err(format!("Invalid AES key length: {} bytes", key.len()))
    }
}

fn des_cbc_encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    use des::Des;
    use cbc::Encryptor;
    use cbc::cipher::BlockEncryptMut;
    use cbc::cipher::KeyIvInit;

    if key.len() != 8 {
        return Err(format!("DES key must be 8 bytes, got {}", key.len()));
    }
    let iv = [0u8; 8];
    let encryptor = Encryptor::<Des>::new_from_slices(key, &iv)
        .map_err(|e| format!("DES init error: {}", e))?;
    Ok(encryptor.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(plaintext))
}

fn des_cbc_decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    use des::Des;
    use cbc::Decryptor;
    use cbc::cipher::BlockDecryptMut;
    use cbc::cipher::KeyIvInit;

    if key.len() != 8 {
        return Err(format!("DES key must be 8 bytes, got {}", key.len()));
    }
    let iv = [0u8; 8];
    let decryptor = Decryptor::<Des>::new_from_slices(key, &iv)
        .map_err(|e| format!("DES init error: {}", e))?;
    decryptor.decrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(ciphertext)
        .map_err(|e| format!("DES decryption error: {}", e))
}

fn desede_cbc_encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    use des::TdesEde3;
    use cbc::Encryptor;
    use cbc::cipher::BlockEncryptMut;
    use cbc::cipher::KeyIvInit;

    if key.len() != 24 {
        return Err(format!("DESEDE key must be 24 bytes, got {}", key.len()));
    }
    let iv = [0u8; 8];
    let encryptor = Encryptor::<TdesEde3>::new_from_slices(key, &iv)
        .map_err(|e| format!("DESEDE init error: {}", e))?;
    Ok(encryptor.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(plaintext))
}

fn desede_cbc_decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    use des::TdesEde3;
    use cbc::Decryptor;
    use cbc::cipher::BlockDecryptMut;
    use cbc::cipher::KeyIvInit;

    if key.len() != 24 {
        return Err(format!("DESEDE key must be 24 bytes, got {}", key.len()));
    }
    let iv = [0u8; 8];
    let decryptor = Decryptor::<TdesEde3>::new_from_slices(key, &iv)
        .map_err(|e| format!("DESEDE init error: {}", e))?;
    decryptor.decrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(ciphertext)
        .map_err(|e| format!("DESEDE decryption error: {}", e))
}

fn blowfish_cbc_encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    use blowfish::Blowfish;
    use cbc::Encryptor;
    use cbc::cipher::BlockEncryptMut;
    use cbc::cipher::KeyIvInit;

    let iv = [0u8; 8];
    let encryptor = Encryptor::<Blowfish>::new_from_slices(key, &iv)
        .map_err(|e| format!("Blowfish init error: {}", e))?;
    Ok(encryptor.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(plaintext))
}

fn blowfish_cbc_decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    use blowfish::Blowfish;
    use cbc::Decryptor;
    use cbc::cipher::BlockDecryptMut;
    use cbc::cipher::KeyIvInit;

    let iv = [0u8; 8];
    let decryptor = Decryptor::<Blowfish>::new_from_slices(key, &iv)
        .map_err(|e| format!("Blowfish init error: {}", e))?;
    decryptor.decrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(ciphertext)
        .map_err(|e| format!("Blowfish decryption error: {}", e))
}

// ==== SECURITY BUILTIN FUNCTIONS ====

fn fn_hmac(args: Vec<CfmlValue>) -> CfmlResult {
    use hmac::{Hmac, Mac};
    use sha2::{Sha256, Sha384, Sha512};
    use sha1::Sha1;
    use md5::Md5;

    let message = get_str(&args, 0);
    let key = get_str(&args, 1);
    let algorithm = if args.len() >= 3 {
        get_str(&args, 2).to_uppercase()
    } else {
        "HMACSHA256".to_string()
    };
    // encoding param (4th) -- we always return hex, matching CFML default

    let hex_result = match algorithm.as_str() {
        "HMACMD5" | "HMAC-MD5" => {
            let mut mac = Hmac::<Md5>::new_from_slice(key.as_bytes())
                .map_err(|e| CfmlError::runtime(format!("HMAC init error: {}", e)))?;
            mac.update(message.as_bytes());
            hex_encode(&mac.finalize().into_bytes())
        }
        "HMACSHA1" | "HMAC-SHA1" => {
            let mut mac = Hmac::<Sha1>::new_from_slice(key.as_bytes())
                .map_err(|e| CfmlError::runtime(format!("HMAC init error: {}", e)))?;
            mac.update(message.as_bytes());
            hex_encode(&mac.finalize().into_bytes())
        }
        "HMACSHA256" | "HMAC-SHA256" | "" => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
                .map_err(|e| CfmlError::runtime(format!("HMAC init error: {}", e)))?;
            mac.update(message.as_bytes());
            hex_encode(&mac.finalize().into_bytes())
        }
        "HMACSHA384" | "HMAC-SHA384" => {
            let mut mac = Hmac::<Sha384>::new_from_slice(key.as_bytes())
                .map_err(|e| CfmlError::runtime(format!("HMAC init error: {}", e)))?;
            mac.update(message.as_bytes());
            hex_encode(&mac.finalize().into_bytes())
        }
        "HMACSHA512" | "HMAC-SHA512" => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key.as_bytes())
                .map_err(|e| CfmlError::runtime(format!("HMAC init error: {}", e)))?;
            mac.update(message.as_bytes());
            hex_encode(&mac.finalize().into_bytes())
        }
        _ => return Err(CfmlError::runtime(format!("Unsupported HMAC algorithm: {}", algorithm)))
    };

    Ok(CfmlValue::String(hex_result))
}

#[cfg(feature = "security")]
fn fn_generate_secret_key(args: Vec<CfmlValue>) -> CfmlResult {
    use rand::RngCore;

    let algorithm = if args.is_empty() {
        "AES".to_string()
    } else {
        get_str(&args, 0).to_uppercase()
    };
    let key_size = if args.len() >= 2 { get_int(&args, 1) as usize } else { 0 };

    let num_bytes = match algorithm.as_str() {
        "AES" => {
            let bits = if key_size > 0 { key_size } else { 128 };
            match bits {
                128 | 192 | 256 => bits / 8,
                _ => return Err(CfmlError::runtime(format!("Invalid AES key size: {}. Must be 128, 192, or 256", bits)))
            }
        }
        "DES" => 8,
        "DESEDE" | "DESEDE3" => 24,
        "BLOWFISH" => {
            let bits = if key_size > 0 { key_size } else { 128 };
            bits / 8
        }
        _ => return Err(CfmlError::runtime(format!("Unsupported algorithm: {}", algorithm)))
    };

    let mut key_bytes = vec![0u8; num_bytes];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    Ok(CfmlValue::String(base64_encode_bytes(&key_bytes)))
}

fn fn_encrypt(args: Vec<CfmlValue>) -> CfmlResult {
    let plaintext = get_str(&args, 0);
    let key_b64 = get_str(&args, 1);
    let algorithm = if args.len() >= 3 {
        get_str(&args, 2).to_uppercase()
    } else {
        "AES".to_string()
    };
    let encoding = if args.len() >= 4 {
        get_str(&args, 3).to_uppercase()
    } else {
        "UU".to_string()
    };

    let ciphertext = if algorithm == "CFMX_COMPAT" {
        cfmx_compat_encrypt(plaintext.as_bytes(), &key_b64)
    } else {
        let key_bytes = base64_decode_bytes(&key_b64);
        let plaintext_bytes = plaintext.as_bytes();

        match algorithm.as_str() {
            "AES" | "AES/CBC/PKCS5PADDING" | "AES/CBC/PKCS7PADDING" => {
                aes_cbc_encrypt(plaintext_bytes, &key_bytes)
                    .map_err(|e| CfmlError::runtime(e))?
            }
            "DES" | "DES/CBC/PKCS5PADDING" => {
                des_cbc_encrypt(plaintext_bytes, &key_bytes)
                    .map_err(|e| CfmlError::runtime(e))?
            }
            "DESEDE" | "DESEDE/CBC/PKCS5PADDING" => {
                desede_cbc_encrypt(plaintext_bytes, &key_bytes)
                    .map_err(|e| CfmlError::runtime(e))?
            }
            "BLOWFISH" | "BLOWFISH/CBC/PKCS5PADDING" => {
                blowfish_cbc_encrypt(plaintext_bytes, &key_bytes)
                    .map_err(|e| CfmlError::runtime(e))?
            }
            _ => return Err(CfmlError::runtime(format!("Unsupported encryption algorithm: {}", algorithm)))
        }
    };

    let encoded = match encoding.as_str() {
        "UU" => uu_encode(&ciphertext),
        "BASE64" => base64_encode_bytes(&ciphertext),
        "HEX" => hex_encode(&ciphertext),
        _ => return Err(CfmlError::runtime(format!("Unsupported encoding: {}", encoding)))
    };

    Ok(CfmlValue::String(encoded))
}

fn fn_decrypt(args: Vec<CfmlValue>) -> CfmlResult {
    let encoded_str = get_str(&args, 0);
    let key_b64 = get_str(&args, 1);
    let algorithm = if args.len() >= 3 {
        get_str(&args, 2).to_uppercase()
    } else {
        "AES".to_string()
    };
    let encoding = if args.len() >= 4 {
        get_str(&args, 3).to_uppercase()
    } else {
        "UU".to_string()
    };

    let ciphertext = match encoding.as_str() {
        "UU" => uu_decode(&encoded_str),
        "BASE64" => base64_decode_bytes(&encoded_str),
        "HEX" => hex_decode(&encoded_str).map_err(|e| CfmlError::runtime(e))?,
        _ => return Err(CfmlError::runtime(format!("Unsupported encoding: {}", encoding)))
    };

    let plaintext_bytes = if algorithm == "CFMX_COMPAT" {
        cfmx_compat_decrypt(&ciphertext, &key_b64)
    } else {
        let key_bytes = base64_decode_bytes(&key_b64);

        match algorithm.as_str() {
            "AES" | "AES/CBC/PKCS5PADDING" | "AES/CBC/PKCS7PADDING" => {
                aes_cbc_decrypt(&ciphertext, &key_bytes)
                    .map_err(|e| CfmlError::runtime(e))?
            }
            "DES" | "DES/CBC/PKCS5PADDING" => {
                des_cbc_decrypt(&ciphertext, &key_bytes)
                    .map_err(|e| CfmlError::runtime(e))?
            }
            "DESEDE" | "DESEDE/CBC/PKCS5PADDING" => {
                desede_cbc_decrypt(&ciphertext, &key_bytes)
                    .map_err(|e| CfmlError::runtime(e))?
            }
            "BLOWFISH" | "BLOWFISH/CBC/PKCS5PADDING" => {
                blowfish_cbc_decrypt(&ciphertext, &key_bytes)
                    .map_err(|e| CfmlError::runtime(e))?
            }
            _ => return Err(CfmlError::runtime(format!("Unsupported decryption algorithm: {}", algorithm)))
        }
    };

    let result = String::from_utf8(plaintext_bytes)
        .map_err(|e| CfmlError::runtime(format!("Decrypted data is not valid UTF-8: {}", e)))?;
    Ok(CfmlValue::String(result))
}

// ==== SYSTEM FUNCTIONS ====

fn fn_get_base_template_path(_args: Vec<CfmlValue>) -> CfmlResult {
    // VM-intercepted — this stub only runs if VM intercept misses
    Err(CfmlError::runtime("getBaseTemplatePath() requires VM context".to_string()))
}

fn fn_get_time_zone(_args: Vec<CfmlValue>) -> CfmlResult {
    // VM-intercepted — this stub only runs if VM intercept misses
    Err(CfmlError::runtime("getTimeZone() requires VM context".to_string()))
}

// ==== XML FUNCTIONS ====

#[cfg(feature = "xml")]
fn fn_xml_parse(args: Vec<CfmlValue>) -> CfmlResult {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let xml_str = get_str(&args, 0);
    let mut reader = Reader::from_str(&xml_str);

    let mut stack: Vec<HashMap<String, CfmlValue>> = Vec::new();
    let mut root: Option<HashMap<String, CfmlValue>> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let mut element = HashMap::new();
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                element.insert("xmlName".to_string(), CfmlValue::String(tag_name));
                element.insert("xmlType".to_string(), CfmlValue::String("ELEMENT".to_string()));
                element.insert("xmlText".to_string(), CfmlValue::String(String::new()));
                element.insert("xmlChildren".to_string(), CfmlValue::Array(Vec::new()));

                let mut attrs = HashMap::new();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    attrs.insert(key, CfmlValue::String(val));
                }
                element.insert("xmlAttributes".to_string(), CfmlValue::Struct(attrs));

                stack.push(element);
            }
            Ok(Event::End(_)) => {
                if let Some(completed) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        if let Some(CfmlValue::Array(ref mut children)) = parent.get_mut("xmlChildren") {
                            children.push(CfmlValue::Struct(completed));
                        }
                    } else {
                        root = Some(completed);
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let mut element = HashMap::new();
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                element.insert("xmlName".to_string(), CfmlValue::String(tag_name));
                element.insert("xmlType".to_string(), CfmlValue::String("ELEMENT".to_string()));
                element.insert("xmlText".to_string(), CfmlValue::String(String::new()));
                element.insert("xmlChildren".to_string(), CfmlValue::Array(Vec::new()));

                let mut attrs = HashMap::new();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    attrs.insert(key, CfmlValue::String(val));
                }
                element.insert("xmlAttributes".to_string(), CfmlValue::Struct(attrs));

                if let Some(parent) = stack.last_mut() {
                    if let Some(CfmlValue::Array(ref mut children)) = parent.get_mut("xmlChildren") {
                        children.push(CfmlValue::Struct(element));
                    }
                } else {
                    root = Some(element);
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or(std::borrow::Cow::Borrowed("")).to_string();
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    if let Some(current) = stack.last_mut() {
                        if let Some(CfmlValue::String(ref mut s)) = current.get_mut("xmlText") {
                            if !s.is_empty() { s.push(' '); }
                            s.push_str(&trimmed);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(CfmlError::runtime(format!("XML parse error: {}", e))),
            _ => {}
        }
    }

    match root {
        Some(root_element) => {
            let mut doc = HashMap::new();
            doc.insert("xmlRoot".to_string(), CfmlValue::Struct(root_element));
            doc.insert("xmlType".to_string(), CfmlValue::String("DOCUMENT".to_string()));
            Ok(CfmlValue::Struct(doc))
        }
        None => Err(CfmlError::runtime("Empty or invalid XML document".to_string()))
    }
}

#[cfg(feature = "xml")]
fn fn_xml_search(args: Vec<CfmlValue>) -> CfmlResult {
    let doc = args.get(0).cloned().unwrap_or(CfmlValue::Null);
    let path_expr = get_str(&args, 1);

    let mut results = Vec::new();

    let search_root = if let CfmlValue::Struct(ref s) = doc {
        if let Some(root) = s.get("xmlRoot") {
            root.clone()
        } else {
            doc.clone()
        }
    } else {
        doc.clone()
    };

    if path_expr.starts_with("//") {
        let tag_name = &path_expr[2..];
        xml_search_descendants(&search_root, tag_name, &mut results);
    } else {
        let parts: Vec<&str> = path_expr.trim_start_matches('/').split('/').collect();
        xml_search_path(&search_root, &parts, 0, &mut results);
    }

    Ok(CfmlValue::Array(results))
}

#[cfg(feature = "xml")]
fn xml_search_descendants(node: &CfmlValue, tag_name: &str, results: &mut Vec<CfmlValue>) {
    if let CfmlValue::Struct(ref s) = node {
        if let Some(CfmlValue::String(ref name)) = s.get("xmlName") {
            if name == tag_name || tag_name == "*" {
                results.push(node.clone());
            }
        }
        if let Some(CfmlValue::Array(ref children)) = s.get("xmlChildren") {
            for child in children {
                xml_search_descendants(child, tag_name, results);
            }
        }
    }
}

#[cfg(feature = "xml")]
fn xml_search_path(node: &CfmlValue, parts: &[&str], depth: usize, results: &mut Vec<CfmlValue>) {
    if depth >= parts.len() {
        results.push(node.clone());
        return;
    }

    let target = parts[depth];

    if let CfmlValue::Struct(ref s) = node {
        if let Some(CfmlValue::String(ref name)) = s.get("xmlName") {
            if name == target || target == "*" {
                if depth == parts.len() - 1 {
                    results.push(node.clone());
                } else if let Some(CfmlValue::Array(ref children)) = s.get("xmlChildren") {
                    for child in children {
                        xml_search_path(child, parts, depth + 1, results);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "xml")]
fn fn_is_xml(args: Vec<CfmlValue>) -> CfmlResult {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let s = get_str(&args, 0);
    let mut reader = Reader::from_str(&s);
    let mut found_element = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) | Ok(Event::Empty(_)) => { found_element = true; }
            Ok(Event::Eof) => break,
            Err(_) => return Ok(CfmlValue::Bool(false)),
            _ => {}
        }
    }
    Ok(CfmlValue::Bool(found_element))
}

#[cfg(feature = "xml")]
fn fn_xml_transform_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("xmlTransform() is not supported (requires XSLT engine)".to_string()))
}

#[cfg(feature = "xml")]
fn fn_xml_validate_stub(_args: Vec<CfmlValue>) -> CfmlResult {
    Err(CfmlError::runtime("xmlValidate() is not supported (requires schema validation engine)".to_string()))
}
