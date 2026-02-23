mod rewrite;

use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

use cfml_codegen::compiler::CfmlCompiler;
use cfml_common::dynamic::CfmlValue;
use cfml_compiler::lexer;
use cfml_compiler::parser::Parser as CfmlParser;
use cfml_compiler::tag_parser;
use cfml_stdlib::builtins::{get_builtin_functions, get_builtins};
use cfml_vm::{CfmlVirtualMachine, ServerState};

#[derive(Parser, Debug)]
#[command(name = "rustcfml")]
#[command(about = "A CFML interpreter written in Rust", long_about = None)]
struct Args {
    /// The CFML file to execute
    #[arg(default_value = "")]
    file: String,

    /// Execute code from command line
    #[arg(short, long)]
    code: Option<String>,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Run in interactive REPL mode
    #[arg(short, long)]
    repl: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Show version information
    #[arg(long)]
    version: bool,

    /// Start web server with document root (default: current directory)
    #[arg(long, num_args = 0..=1, default_missing_value = ".")]
    serve: Option<String>,

    /// Server port (default: 8500)
    #[arg(long, default_value = "8500")]
    port: u16,

    /// Use single-threaded async runtime (lower memory, lower concurrency)
    #[arg(long)]
    single_threaded: bool,
}

/// Encapsulates the full response from CFML execution, including HTTP metadata.
struct CfmlResponse {
    output: String,
    response_headers: Vec<(String, String)>,
    response_status: Option<(u16, String)>,
    response_content_type: Option<String>,
    response_body: Option<CfmlValue>,
    redirect_url: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.version {
        println!("RustCFML v{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }

    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    if let Some(ref doc_root) = args.serve {
        let doc_root = PathBuf::from(doc_root);
        if !doc_root.is_dir() {
            eprintln!("Error: Document root is not a directory: {}", doc_root.display());
            exit(1);
        }
        run_server(&doc_root, args.port, args.debug, args.single_threaded);
        return;
    }

    if args.repl {
        run_repl(args.debug);
        return;
    }

    if let Some(code) = args.code {
        execute_code(&code, args.debug);
        return;
    }

    if args.file.is_empty() {
        println!("RustCFML v{}", env!("CARGO_PKG_VERSION"));
        println!("Usage: rustcfml <file.cfm|.cfc>");
        println!("       rustcfml -c \"<code>\"");
        println!("       rustcfml -r (REPL mode)");
        println!("       rustcfml --serve [path] [--port 8500]");
        println!("       rustcfml --help");
        exit(0);
    }

    let path = PathBuf::from(&args.file);
    if !path.exists() {
        eprintln!("Error: File not found: {}", args.file);
        exit(1);
    }

    execute_file(&path, args.debug);
}

fn execute_file(path: &PathBuf, debug: bool) {
    let source = fs::read_to_string(path).expect("Failed to read file");
    execute_code_with_file(&source, debug, Some(path.to_string_lossy().to_string()));
}

fn execute_code(source: &str, debug: bool) {
    execute_code_with_file(source, debug, None);
}

fn execute_code_with_file(source: &str, debug: bool, source_file: Option<String>) {
    match compile_and_run(source, debug, source_file, HashMap::new(), None, None) {
        Ok(response) => {
            if !response.output.is_empty() {
                print!("{}", response.output);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}

/// Compile and execute CFML source, returning output as a String.
/// `extra_globals` are injected into vm.globals before execution (e.g. web scopes).
fn compile_and_run(
    source: &str,
    debug: bool,
    source_file: Option<String>,
    extra_globals: HashMap<String, CfmlValue>,
    server_state: Option<&ServerState>,
    http_request_data: Option<CfmlValue>,
) -> Result<CfmlResponse, String> {
    // Strip shebang line if present (e.g. #!/usr/bin/env rustcfml)
    let source = if source.starts_with("#!") {
        source.split_once('\n').map_or("", |(_shebang, rest)| rest)
    } else {
        source
    };

    // Pre-process: convert CFML tags to script if needed
    let source = if tag_parser::has_cfml_tags(source) {
        let converted = tag_parser::tags_to_script(source);
        if debug {
            println!("=== TAG CONVERSION ===");
            println!("{}", converted);
            println!();
        }
        converted
    } else {
        source.to_string()
    };
    let source = source.as_str();

    // Lexical analysis
    let tokens = lexer::tokenize(source.to_string());

    if debug {
        println!("=== TOKENS ===");
        for (i, tok) in tokens.iter().enumerate() {
            println!("{:3}: {:?}", i, tok.token);
        }
        println!();
    }

    // Parse to AST
    let mut parser = CfmlParser::new(source.to_string());
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            return Err(format!(
                "Parse Error [line {}, col {}]: {}",
                e.line, e.column, e.message
            ));
        }
    };

    if debug {
        println!("=== AST ===");
        println!("{:#?}", ast);
        println!();
    }

    // Compile to bytecode
    let compiler = CfmlCompiler::new();
    let program = compiler.compile(ast);

    if debug {
        println!("=== BYTECODE ===");
        for func in &program.functions {
            println!("Function: {} (params: {:?})", func.name, func.params);
            for (i, instr) in func.instructions.iter().enumerate() {
                match instr {
                    cfml_codegen::BytecodeOp::LineInfo(line, col) => {
                        println!("        ; line {}:{}", line, col);
                    }
                    _ => {
                        println!("  {:3}: {:?}", i, instr);
                    }
                }
            }
        }
        println!();
    }

    // Execute
    let mut vm = CfmlVirtualMachine::new(program);
    vm.base_template_path = source_file.clone();
    vm.source_file = source_file;

    // Register builtins
    for (name, value) in get_builtins() {
        vm.globals.insert(name, value);
    }
    for (name, func) in get_builtin_functions() {
        vm.builtins.insert(name, func);
    }

    // Ensure web scopes always exist (CFML guarantees url/cgi/form are always defined)
    vm.globals.entry("url".to_string()).or_insert_with(|| CfmlValue::Struct(HashMap::new()));
    vm.globals.entry("cgi".to_string()).or_insert_with(|| CfmlValue::Struct(HashMap::new()));
    vm.globals.entry("form".to_string()).or_insert_with(|| CfmlValue::Struct(HashMap::new()));

    // Inject extra globals (web scopes, etc.) — overrides defaults above in serve mode
    for (name, value) in extra_globals {
        vm.globals.insert(name, value);
    }

    // Wire up server state if provided (for --serve mode)
    if let Some(ss) = server_state {
        vm.server_state = Some(ss.clone());
    }

    // Wire up HTTP request data if provided
    vm.http_request_data = http_request_data;

    let result = vm.execute_with_lifecycle();

    // Catch redirect errors as success
    let result = match result {
        Err(e) if e.message == "__cflocation_redirect" || e.message == "__cfabort" => Ok(CfmlValue::Null),
        other => other,
    };

    match result {
        Ok(value) => {
            let mut output = String::new();
            if !vm.output_buffer.is_empty() {
                output.push_str(&vm.output_buffer);
            }
            if debug {
                println!("Result: {:?}", value);
            }
            Ok(CfmlResponse {
                output,
                response_headers: vm.response_headers,
                response_status: vm.response_status,
                response_content_type: vm.response_content_type,
                response_body: vm.response_body,
                redirect_url: vm.redirect_url,
            })
        }
        Err(e) => Err(format!("{}", e)),
    }
}

// ---------------------------------------------------------------------------
// Web server
// ---------------------------------------------------------------------------

struct AppState {
    doc_root: PathBuf,
    port: u16,
    debug: bool,
    server_state: ServerState,
    rewrite_rules: Vec<rewrite::RewriteRule>,
}

fn run_server(doc_root: &Path, port: u16, debug: bool, single_threaded: bool) {
    let rt = if single_threaded {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    };
    rt.block_on(async_run_server(doc_root, port, debug, single_threaded));
}

async fn async_run_server(doc_root: &Path, port: u16, debug: bool, single_threaded: bool) {
    let server_state = ServerState::new();

    // Load URL rewrite rules if urlrewrite.xml exists
    let rewrite_xml = doc_root.join("urlrewrite.xml");
    let rewrite_rules = if rewrite_xml.is_file() {
        let rules = rewrite::parse_urlrewrite_xml(&rewrite_xml);
        println!("Loaded {} URL rewrite rule(s) from urlrewrite.xml", rules.len());
        rules
    } else {
        Vec::new()
    };

    let mode = if single_threaded { "single-threaded" } else { "multi-threaded" };
    println!("RustCFML server running on http://127.0.0.1:{} ({})", port, mode);
    println!("Document root: {}", fs::canonicalize(doc_root).unwrap_or_else(|_| doc_root.to_path_buf()).display());
    println!("Press Ctrl+C to stop\n");

    let app_state = Arc::new(AppState {
        doc_root: doc_root.to_path_buf(),
        port,
        debug,
        server_state,
        rewrite_rules,
    });

    let app = axum::Router::new()
        .fallback(handle_request)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap_or_else(|e| {
        eprintln!("Failed to start server on 0.0.0.0:{}: {}", port, e);
        exit(1);
    });
    axum::serve(listener, app).await.unwrap();
}

async fn handle_request(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    req: axum::extract::Request,
) -> axum::response::Response {
    let (parts, body) = req.into_parts();
    let method = parts.method.to_string();
    let url = parts.uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/").to_string();

    // Extract headers as Vec<(String, String)>
    let headers: Vec<(String, String)> = parts.headers.iter()
        .map(|(name, value)| (name.as_str().to_string(), value.to_str().unwrap_or("").to_string()))
        .collect();

    // Read body bytes
    let body_bytes = axum::body::to_bytes(body, 10_000_000).await.unwrap_or_default();

    let debug = state.debug;

    log::debug!("{} {}", method, url);

    // Split URL into path and query string
    let (raw_path, original_qs) = match url.find('?') {
        Some(pos) => (&url[..pos], &url[pos + 1..]),
        None => (url.as_str(), ""),
    };

    // Apply URL rewrite rules before file resolution
    let mut path = raw_path.to_string();
    let mut query_string = original_qs.to_string();
    if !state.rewrite_rules.is_empty() {
        let mut header_map = HashMap::new();
        for (name, value) in &headers {
            header_map.insert(name.clone(), value.clone());
        }

        if let Some(result) = rewrite::apply_rewrite_rules(&state.rewrite_rules, &path, &method, state.port, &header_map) {
            match result.rewrite_type {
                rewrite::RewriteType::PermanentRedirect => {
                    log::debug!("  -> 301 redirect to {}", result.new_path);
                    return axum::response::Response::builder()
                        .status(301)
                        .header("Location", &result.new_path)
                        .body(axum::body::Body::empty())
                        .unwrap();
                }
                rewrite::RewriteType::Redirect => {
                    log::debug!("  -> 302 redirect to {}", result.new_path);
                    return axum::response::Response::builder()
                        .status(302)
                        .header("Location", &result.new_path)
                        .body(axum::body::Body::empty())
                        .unwrap();
                }
                rewrite::RewriteType::Forward => {
                    if result.new_path != path {
                        log::debug!("  -> rewrite to {}", result.new_path);
                    }
                    // Split rewritten path from its query string
                    if let Some(qpos) = result.new_path.find('?') {
                        let rewritten_qs = &result.new_path[qpos + 1..];
                        // Merge: rewritten QS params, then original QS params
                        if !rewritten_qs.is_empty() && !query_string.is_empty() {
                            query_string = format!("{}&{}", rewritten_qs, query_string);
                        } else if !rewritten_qs.is_empty() {
                            query_string = rewritten_qs.to_string();
                        }
                        path = result.new_path[..qpos].to_string();
                    } else {
                        path = result.new_path;
                    }
                }
            }
        }
    }

    // Resolve file path from URL
    let resolved = resolve_file(&state.doc_root, &path);

    match resolved {
        Some(ref rf) if rf.file_path.extension().map_or(false, |e| e == "cfm") => {
            // Execute .cfm file
            let source = match fs::read_to_string(&rf.file_path) {
                Ok(s) => s,
                Err(e) => {
                    return axum::response::Response::builder()
                        .status(500)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(axum::body::Body::from(format!(
                            "<html><body><h1>500 Internal Server Error</h1><p>Error reading file: {}</p></body></html>",
                            html_escape(&e.to_string())
                        )))
                        .unwrap();
                }
            };

            // Build web scopes using resolved script_name and path_info
            let (extra_globals, http_request_data) = build_web_scopes(
                &method, &headers, &body_bytes, &rf.script_name, &rf.path_info, &query_string, state.port,
            );

            let file_path = rf.file_path.to_string_lossy().to_string();
            let server_state = state.server_state.clone();

            let result = tokio::task::spawn_blocking(move || {
                compile_and_run(
                    &source,
                    debug,
                    Some(file_path),
                    extra_globals,
                    Some(&server_state),
                    Some(http_request_data),
                )
            }).await.unwrap();

            match result {
                Ok(response) => {
                    // Check for redirect
                    if let Some(ref redirect_url) = response.redirect_url {
                        let status_code = response.response_status
                            .as_ref()
                            .map(|(c, _)| *c)
                            .unwrap_or(302);
                        let mut builder = axum::response::Response::builder()
                            .status(status_code)
                            .header("Location", redirect_url.as_str());
                        for (name, value) in &response.response_headers {
                            if name.to_lowercase() != "location" {
                                builder = builder.header(name.as_str(), value.as_str());
                            }
                        }
                        return builder.body(axum::body::Body::empty()).unwrap();
                    }

                    // Determine content type
                    let content_type = response.response_content_type
                        .as_deref()
                        .unwrap_or("text/html; charset=utf-8");

                    // Determine body
                    let body = if let Some(ref body_override) = response.response_body {
                        body_override.as_string()
                    } else {
                        response.output
                    };

                    // Determine status code
                    let status_code = response.response_status
                        .as_ref()
                        .map(|(c, _)| *c)
                        .unwrap_or(200);

                    let mut builder = axum::response::Response::builder()
                        .status(status_code)
                        .header("Content-Type", content_type);

                    for (name, value) in &response.response_headers {
                        builder = builder.header(name.as_str(), value.as_str());
                    }

                    builder.body(axum::body::Body::from(body)).unwrap()
                }
                Err(e) => {
                    axum::response::Response::builder()
                        .status(500)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(axum::body::Body::from(format!(
                            "<html><body><h1>500 Internal Server Error</h1><pre>{}</pre></body></html>",
                            html_escape(&e)
                        )))
                        .unwrap()
                }
            }
        }
        Some(ref rf) => {
            // Serve static file
            match fs::read(&rf.file_path) {
                Ok(data) => {
                    let ct = content_type_for(&rf.file_path);
                    axum::response::Response::builder()
                        .status(200)
                        .header("Content-Type", ct)
                        .body(axum::body::Body::from(data))
                        .unwrap()
                }
                Err(_) => {
                    axum::response::Response::builder()
                        .status(500)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(axum::body::Body::from(
                            "<html><body><h1>500 Internal Server Error</h1><p>Error reading file</p></body></html>"
                        ))
                        .unwrap()
                }
            }
        }
        None => {
            axum::response::Response::builder()
                .status(404)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(axum::body::Body::from(format!(
                    "<html><body><h1>404 Not Found</h1><p>The requested URL {} was not found.</p></body></html>",
                    html_escape(&path)
                )))
                .unwrap()
        }
    }
}

/// Result of resolving a URL path to a file.
struct ResolvedFile {
    file_path: PathBuf,
    /// The script portion of the URL (e.g. "/index.cfm")
    script_name: String,
    /// Extra path info after the script (e.g. "/hello/world")
    path_info: String,
}

/// Resolve a URL path to a file in the document root.
fn resolve_file(doc_root: &Path, url_path: &str) -> Option<ResolvedFile> {
    // Normalize: strip leading slash, default to index.cfm
    let relative = url_path.trim_start_matches('/');

    // Try exact path first
    if !relative.is_empty() {
        let candidate = doc_root.join(relative);
        if candidate.is_file() {
            return Some(ResolvedFile {
                file_path: candidate,
                script_name: format!("/{}", relative),
                path_info: String::new(),
            });
        }
        // Try as directory with index.cfm
        let dir_index = doc_root.join(relative).join("index.cfm");
        if dir_index.is_file() {
            let script = if relative.is_empty() {
                "/index.cfm".to_string()
            } else {
                format!("/{}/index.cfm", relative)
            };
            return Some(ResolvedFile {
                file_path: dir_index,
                script_name: script,
                path_info: String::new(),
            });
        }
        // Try path info pattern: /script.cfm/extra/path
        // Walk up the path segments to find a .cfm file
        let mut parts: Vec<&str> = relative.split('/').collect();
        while parts.len() > 1 {
            parts.pop();
            let partial = parts.join("/");
            let candidate = doc_root.join(&partial);
            if candidate.is_file() && candidate.extension().map_or(false, |e| e == "cfm" || e == "cfc") {
                let script_name = format!("/{}", partial);
                let path_info = url_path[script_name.len()..].to_string();
                return Some(ResolvedFile {
                    file_path: candidate,
                    script_name,
                    path_info,
                });
            }
        }
    } else {
        // Root path → index.cfm
        let index = doc_root.join("index.cfm");
        if index.is_file() {
            return Some(ResolvedFile {
                file_path: index,
                script_name: "/index.cfm".to_string(),
                path_info: String::new(),
            });
        }
    }

    None
}

/// Build CGI, URL, and Form scopes from extracted HTTP request data.
fn build_web_scopes(
    method: &str,
    headers: &[(String, String)],
    body: &[u8],
    script_name: &str,
    path_info: &str,
    query_string: &str,
    port: u16,
) -> (HashMap<String, CfmlValue>, CfmlValue) {
    let mut globals = HashMap::new();

    // CGI scope
    let mut cgi = HashMap::new();
    cgi.insert("request_method".to_string(), CfmlValue::String(method.to_string()));
    let path_info = if path_info.is_empty() { "/" } else { path_info };
    cgi.insert("path_info".to_string(), CfmlValue::String(path_info.to_string()));
    cgi.insert("script_name".to_string(), CfmlValue::String(script_name.to_string()));
    cgi.insert("query_string".to_string(), CfmlValue::String(query_string.to_string()));
    cgi.insert("server_name".to_string(), CfmlValue::String("127.0.0.1".to_string()));
    cgi.insert("server_port".to_string(), CfmlValue::String(port.to_string()));

    // Extract headers
    let mut content_type = String::new();
    let mut user_agent = String::new();
    for (name, value) in headers {
        let lower = name.to_lowercase();
        if lower == "content-type" {
            content_type = value.clone();
        } else if lower == "user-agent" {
            user_agent = value.clone();
        }
    }
    cgi.insert("content_type".to_string(), CfmlValue::String(content_type.clone()));
    cgi.insert("http_user_agent".to_string(), CfmlValue::String(user_agent));

    globals.insert("cgi".to_string(), CfmlValue::Struct(cgi));

    // URL scope — parsed query string
    let url_scope = parse_query_string(query_string);
    globals.insert("url".to_string(), CfmlValue::Struct(url_scope));

    // Raw body as string
    let raw_body = String::from_utf8_lossy(body).to_string();

    // Form scope — parsed POST body (application/x-www-form-urlencoded)
    let form_scope = if method == "POST"
        && content_type.starts_with("application/x-www-form-urlencoded")
        && !raw_body.is_empty()
    {
        parse_query_string(&raw_body)
    } else {
        HashMap::new()
    };
    globals.insert("form".to_string(), CfmlValue::Struct(form_scope));

    // Build full HTTP request data
    let mut headers_struct = HashMap::new();
    for (name, value) in headers {
        headers_struct.insert(name.clone(), CfmlValue::String(value.clone()));
    }

    let mut http_request_data = HashMap::new();
    http_request_data.insert("headers".to_string(), CfmlValue::Struct(headers_struct));
    http_request_data.insert("content".to_string(), CfmlValue::String(raw_body));
    http_request_data.insert("method".to_string(), CfmlValue::String(method.to_string()));
    http_request_data.insert("protocol".to_string(), CfmlValue::String("HTTP/1.1".to_string()));

    (globals, CfmlValue::Struct(http_request_data))
}

/// Parse a query string like "name=World&id=1" into a HashMap.
fn parse_query_string(qs: &str) -> HashMap<String, CfmlValue> {
    let mut map = HashMap::new();
    if qs.is_empty() {
        return map;
    }
    for pair in qs.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let Some(key) = parts.next() {
            let value = parts.next().unwrap_or("");
            // Simple URL decoding: + → space, %XX → byte
            let key = url_decode(key);
            let value = url_decode(value);
            if !key.is_empty() {
                map.insert(key.to_lowercase(), CfmlValue::String(value));
            }
        }
    }
    map
}

/// Simple URL decoding.
fn url_decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            result.push(b' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                &String::from_utf8_lossy(&bytes[i + 1..i + 3]),
                16,
            ) {
                result.push(byte);
                i += 3;
            } else {
                result.push(bytes[i]);
                i += 1;
            }
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&result).to_string()
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// REPL
// ---------------------------------------------------------------------------

fn run_repl(debug: bool) {
    println!("RustCFML REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or 'quit' to exit\n");

    loop {
        print!("cfml> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).unwrap() == 0 {
            break;
        }

        let line = line.trim();
        if line == "exit" || line == "quit" {
            break;
        }

        if line.is_empty() {
            continue;
        }

        execute_code(line, debug);
    }
}
