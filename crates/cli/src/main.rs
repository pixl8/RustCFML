mod rewrite;

use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

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
        run_server(&doc_root, args.port, args.debug);
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

    // Inject extra globals (web scopes, etc.)
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
        Err(e) if e.message == "__cflocation_redirect" => Ok(CfmlValue::Null),
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

fn run_server(doc_root: &Path, port: u16, debug: bool) {
    let addr = format!("0.0.0.0:{}", port);
    let server = tiny_http::Server::http(&addr).unwrap_or_else(|e| {
        eprintln!("Failed to start server on {}: {}", addr, e);
        exit(1);
    });

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

    println!("RustCFML server running on http://127.0.0.1:{}", port);
    println!("Document root: {}", fs::canonicalize(doc_root).unwrap_or_else(|_| doc_root.to_path_buf()).display());
    println!("Press Ctrl+C to stop\n");

    for request in server.incoming_requests() {
        handle_request(request, doc_root, port, debug, &server_state, &rewrite_rules);
    }
}

fn handle_request(
    mut request: tiny_http::Request,
    doc_root: &Path,
    port: u16,
    debug: bool,
    server_state: &ServerState,
    rewrite_rules: &[rewrite::RewriteRule],
) {
    let method = request.method().to_string();
    let url = request.url().to_string();

    if debug {
        println!("{} {}", method, url);
    }

    // Split URL into path and query string
    let (raw_path, original_qs) = match url.find('?') {
        Some(pos) => (&url[..pos], &url[pos + 1..]),
        None => (url.as_str(), ""),
    };

    // Apply URL rewrite rules before file resolution
    let mut path = raw_path.to_string();
    let mut query_string = original_qs.to_string();
    if !rewrite_rules.is_empty() {
        let mut headers = HashMap::new();
        for header in request.headers() {
            headers.insert(
                header.field.as_str().as_str().to_string(),
                header.value.as_str().to_string(),
            );
        }

        if let Some(result) = rewrite::apply_rewrite_rules(rewrite_rules, &path, &method, port, &headers) {
            match result.rewrite_type {
                rewrite::RewriteType::PermanentRedirect => {
                    if debug {
                        println!("  -> 301 redirect to {}", result.new_path);
                    }
                    let response = tiny_http::Response::from_string("")
                        .with_status_code(tiny_http::StatusCode(301))
                        .with_header(
                            tiny_http::Header::from_bytes(b"Location", result.new_path.as_bytes()).unwrap(),
                        );
                    let _ = request.respond(response);
                    return;
                }
                rewrite::RewriteType::Redirect => {
                    if debug {
                        println!("  -> 302 redirect to {}", result.new_path);
                    }
                    let response = tiny_http::Response::from_string("")
                        .with_status_code(tiny_http::StatusCode(302))
                        .with_header(
                            tiny_http::Header::from_bytes(b"Location", result.new_path.as_bytes()).unwrap(),
                        );
                    let _ = request.respond(response);
                    return;
                }
                rewrite::RewriteType::Forward => {
                    if debug && result.new_path != path {
                        println!("  -> rewrite to {}", result.new_path);
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
    let path = path.as_str();
    let query_string = query_string.as_str();

    // Resolve file path from URL
    let resolved = resolve_file(doc_root, path);

    match resolved {
        Some(file_path) if file_path.extension().map_or(false, |e| e == "cfm") => {
            // Execute .cfm file
            let source = match fs::read_to_string(&file_path) {
                Ok(s) => s,
                Err(e) => {
                    let _ = respond_error(request, 500, &format!("Error reading file: {}", e));
                    return;
                }
            };

            // Build web scopes
            let (extra_globals, http_request_data) = build_web_scopes(&mut request, path, query_string, port);

            match compile_and_run(
                &source,
                debug,
                Some(file_path.to_string_lossy().to_string()),
                extra_globals,
                Some(server_state),
                Some(http_request_data),
            ) {
                Ok(response) => {
                    // Check for redirect
                    if let Some(ref redirect_url) = response.redirect_url {
                        let status_code = response.response_status
                            .as_ref()
                            .map(|(c, _)| *c)
                            .unwrap_or(302);
                        let mut http_response = tiny_http::Response::from_string("")
                            .with_status_code(tiny_http::StatusCode(status_code))
                            .with_header(
                                tiny_http::Header::from_bytes(b"Location", redirect_url.as_bytes()).unwrap(),
                            );
                        // Add any extra response headers
                        for (name, value) in &response.response_headers {
                            if name.to_lowercase() != "location" {
                                if let Ok(h) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
                                    http_response.add_header(h);
                                }
                            }
                        }
                        let _ = request.respond(http_response);
                        return;
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

                    let mut http_response = tiny_http::Response::from_string(body)
                        .with_status_code(tiny_http::StatusCode(status_code))
                        .with_header(
                            tiny_http::Header::from_bytes(b"Content-Type", content_type.as_bytes()).unwrap(),
                        );

                    // Add response headers
                    for (name, value) in &response.response_headers {
                        if let Ok(h) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
                            http_response.add_header(h);
                        }
                    }

                    let _ = request.respond(http_response);
                }
                Err(e) => {
                    let body = format!(
                        "<html><body><h1>500 Internal Server Error</h1><pre>{}</pre></body></html>",
                        html_escape(&e)
                    );
                    let _ = respond_error_body(request, 500, &body);
                }
            }
        }
        Some(file_path) => {
            // Serve static file
            serve_static(request, &file_path);
        }
        None => {
            let body = format!(
                "<html><body><h1>404 Not Found</h1><p>The requested URL {} was not found.</p></body></html>",
                html_escape(path)
            );
            let _ = respond_error_body(request, 404, &body);
        }
    }
}

/// Resolve a URL path to a file in the document root.
fn resolve_file(doc_root: &Path, url_path: &str) -> Option<PathBuf> {
    // Normalize: strip leading slash, default to index.cfm
    let relative = url_path.trim_start_matches('/');

    // Try exact path first
    if !relative.is_empty() {
        let candidate = doc_root.join(relative);
        if candidate.is_file() {
            return Some(candidate);
        }
        // Try as directory with index.cfm
        let dir_index = doc_root.join(relative).join("index.cfm");
        if dir_index.is_file() {
            return Some(dir_index);
        }
    } else {
        // Root path → index.cfm
        let index = doc_root.join("index.cfm");
        if index.is_file() {
            return Some(index);
        }
    }

    None
}

/// Build CGI, URL, and Form scopes from the HTTP request.
/// Takes `&mut` because reading the POST body requires mutable access.
fn build_web_scopes(
    request: &mut tiny_http::Request,
    path: &str,
    query_string: &str,
    port: u16,
) -> (HashMap<String, CfmlValue>, CfmlValue) {
    let mut globals = HashMap::new();

    // CGI scope
    let mut cgi = HashMap::new();
    cgi.insert("request_method".to_string(), CfmlValue::String(request.method().to_string()));
    cgi.insert("path_info".to_string(), CfmlValue::String(path.to_string()));
    cgi.insert("query_string".to_string(), CfmlValue::String(query_string.to_string()));
    cgi.insert("server_name".to_string(), CfmlValue::String("127.0.0.1".to_string()));
    cgi.insert("server_port".to_string(), CfmlValue::String(port.to_string()));

    // Extract headers
    let mut content_type = String::new();
    let mut user_agent = String::new();
    for header in request.headers() {
        let name = header.field.as_str().as_str().to_lowercase();
        if name == "content-type" {
            content_type = header.value.as_str().to_string();
        } else if name == "user-agent" {
            user_agent = header.value.as_str().to_string();
        }
    }
    cgi.insert("content_type".to_string(), CfmlValue::String(content_type.clone()));
    cgi.insert("http_user_agent".to_string(), CfmlValue::String(user_agent));

    globals.insert("cgi".to_string(), CfmlValue::Struct(cgi));

    // URL scope — parsed query string
    let url_scope = parse_query_string(query_string);
    globals.insert("url".to_string(), CfmlValue::Struct(url_scope));

    // Read raw body BEFORE form parsing consumes it
    let raw_body = {
        let body_len = request.body_length().unwrap_or(0);
        if body_len > 0 && body_len < 10_000_000 {
            let mut body = String::new();
            if std::io::Read::read_to_string(request.as_reader(), &mut body).is_ok() {
                body
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };

    // Form scope — parsed POST body (application/x-www-form-urlencoded)
    let form_scope = if request.method().as_str() == "POST"
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
    for header in request.headers() {
        headers_struct.insert(
            header.field.as_str().as_str().to_string(),
            CfmlValue::String(header.value.as_str().to_string()),
        );
    }

    let mut http_request_data = HashMap::new();
    http_request_data.insert("headers".to_string(), CfmlValue::Struct(headers_struct));
    http_request_data.insert("content".to_string(), CfmlValue::String(raw_body));
    http_request_data.insert("method".to_string(), CfmlValue::String(request.method().to_string()));
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

fn serve_static(request: tiny_http::Request, path: &Path) {
    match fs::read(path) {
        Ok(data) => {
            let ct = content_type_for(path);
            let response = tiny_http::Response::from_data(data).with_header(
                tiny_http::Header::from_bytes(b"Content-Type", ct.as_bytes()).unwrap(),
            );
            let _ = request.respond(response);
        }
        Err(_) => {
            let _ = respond_error(request, 500, "Error reading file");
        }
    }
}

fn respond_error(request: tiny_http::Request, code: u16, message: &str) -> Result<(), ()> {
    let body = format!(
        "<html><body><h1>{} Error</h1><p>{}</p></body></html>",
        code,
        html_escape(message)
    );
    respond_error_body(request, code, &body)
}

fn respond_error_body(request: tiny_http::Request, code: u16, body: &str) -> Result<(), ()> {
    let status = tiny_http::StatusCode(code);
    let response = tiny_http::Response::from_string(body)
        .with_status_code(status)
        .with_header(
            tiny_http::Header::from_bytes(b"Content-Type", b"text/html; charset=utf-8").unwrap(),
        );
    request.respond(response).map_err(|_| ())
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
