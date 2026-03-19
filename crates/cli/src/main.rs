mod rewrite;

use clap::Parser;
use std::collections::HashMap;
use indexmap::IndexMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

use cfml_codegen::compiler::CfmlCompiler;
use cfml_common::dynamic::CfmlValue;
use cfml_common::vfs::{self, Vfs};
use cfml_compiler::lexer;
use cfml_compiler::parser::Parser as CfmlParser;
use cfml_compiler::tag_parser;
use cfml_stdlib::builtins::{get_builtin_functions, get_builtins};
use cfml_vm::{CfmlVirtualMachine, ServerState, compile_file_cached};

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

    /// Build a self-contained binary: embed a CFML app into a single executable
    /// Usage: rustcfml --build <app-dir> [-o output-binary] [--mode serve|cli]
    #[arg(long, value_name = "APP_DIR")]
    build: Option<String>,

    /// Output path for the built binary (default: ./app)
    #[arg(short, long, default_value = "app")]
    output: String,

    /// Build mode: "serve" for web server (default), "cli" for command-line tool
    #[arg(long, default_value = "serve")]
    mode: String,

    /// Entry point for CLI mode (default: main.cfm)
    #[arg(long, default_value = "main.cfm")]
    entry: String,
}

/// Encapsulates the full response from CFML execution, including HTTP metadata.
struct CfmlResponse {
    output: String,
    response_headers: Vec<(String, String)>,
    response_status: Option<(u16, String)>,
    response_content_type: Option<String>,
    response_body: Option<CfmlValue>,
    redirect_url: Option<String>,
    session_id: Option<String>,
}

/// Error from CFML execution, carrying any output generated before the error.
struct CfmlRunError {
    output: String,
    message: String,
}

fn main() {
    // Spawn a thread with a large stack (64 MB) so deep recursion in the VM
    // doesn't blow the default ~8 MB main-thread stack (especially in debug builds).
    const STACK_SIZE: usize = 64 * 1024 * 1024;
    let builder = std::thread::Builder::new().stack_size(STACK_SIZE);
    let handler = builder.spawn(real_main).expect("failed to spawn main thread");
    if let Err(e) = handler.join() {
        eprintln!("Fatal: {:?}", e);
        exit(1);
    }
}

fn real_main() {
    // Check for embedded archive — if present, run as self-contained app
    if let Some(files) = vfs::extract_embedded_archive() {
        run_embedded_app(files);
        return;
    }

    let args = Args::parse();

    if args.version {
        println!("RustCFML v{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }

    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    // Handle --build <app-dir>
    if let Some(ref app_dir) = args.build {
        let mode = args.mode.to_lowercase();
        if mode != "serve" && mode != "cli" {
            eprintln!("Error: --mode must be 'serve' or 'cli'");
            exit(1);
        }
        build_self_contained(app_dir, &args.output, &mode, &args.entry);
        return;
    }

    if let Some(ref doc_root) = args.serve {
        let doc_root = PathBuf::from(doc_root);
        if !doc_root.is_dir() {
            eprintln!("Error: Document root is not a directory: {}", doc_root.display());
            exit(1);
        }
        run_server(&doc_root, args.port, args.debug, args.single_threaded, vfs::real_fs(), false);
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
        println!("       rustcfml --build <app-dir> [-o output]");
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
    match compile_and_run(source, debug, source_file, IndexMap::new(), None, None, None, vfs::real_fs(), false) {
        Ok(response) => {
            if !response.output.is_empty() {
                print!("{}", response.output);
            }
        }
        Err(e) => {
            if !e.output.is_empty() {
                print!("{}", e.output);
            }
            eprintln!("{}", e.message);
            exit(1);
        }
    }
}

/// Compile and execute CFML source, returning output as a String.
/// `extra_globals` are injected into vm.globals before execution (e.g. web scopes).
fn compile_and_run_with_session(
    source: &str,
    debug: bool,
    source_file: Option<String>,
    extra_globals: IndexMap<String, CfmlValue>,
    server_state: Option<&ServerState>,
    http_request_data: Option<CfmlValue>,
    session_id: Option<String>,
    vfs: Arc<dyn Vfs>,
    sandbox: bool,
) -> Result<CfmlResponse, CfmlRunError> {
    compile_and_run(source, debug, source_file, extra_globals, server_state, http_request_data, session_id, vfs, sandbox)
}

fn compile_and_run(
    source: &str,
    debug: bool,
    source_file: Option<String>,
    extra_globals: IndexMap<String, CfmlValue>,
    server_state: Option<&ServerState>,
    http_request_data: Option<CfmlValue>,
    session_id: Option<String>,
    vfs: Arc<dyn Vfs>,
    sandbox: bool,
) -> Result<CfmlResponse, CfmlRunError> {
    // In serve mode with a source file, use the bytecode cache to skip recompilation
    let program = if !debug && source_file.is_some() && server_state.is_some() {
        let path = source_file.as_ref().unwrap();
        let cache = &server_state.unwrap().bytecode_cache;
        compile_file_cached(path, Some(cache), vfs.as_ref()).map_err(|e| CfmlRunError { output: String::new(), message: format!("{}", e) })?
    } else {
        // CLI mode / inline code / debug: full pipeline
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
                return Err(CfmlRunError {
                    output: String::new(),
                    message: format!("Parse Error [line {}, col {}]: {}", e.line, e.column, e.message),
                });
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

        program
    };

    // Execute
    let mut vm = CfmlVirtualMachine::new(program);
    vm.vfs = vfs;
    vm.sandbox = sandbox;
    vm.base_template_path = source_file.clone();
    vm.source_file = source_file;

    // Register builtins
    for (name, value) in get_builtins() {
        vm.globals.insert(name, value);
    }
    for (name, func) in get_builtin_functions() {
        vm.builtins.insert(name, func);
    }

    // Register database transaction function pointers
    vm.txn_begin = Some(cfml_stdlib::builtins::txn_begin_boxed);
    vm.txn_commit = Some(cfml_stdlib::builtins::txn_commit_boxed);
    vm.txn_rollback = Some(cfml_stdlib::builtins::txn_rollback_boxed);
    vm.txn_execute = Some(cfml_stdlib::builtins::txn_execute_boxed);
    vm.query_execute_fn = Some(cfml_stdlib::builtins::fn_query_execute);

    // Ensure web scopes always exist (CFML guarantees url/cgi/form are always defined)
    vm.globals.entry("url".to_string()).or_insert_with(|| CfmlValue::Struct(IndexMap::new()));
    vm.globals.entry("cgi".to_string()).or_insert_with(|| CfmlValue::Struct(IndexMap::new()));
    vm.globals.entry("form".to_string()).or_insert_with(|| CfmlValue::Struct(IndexMap::new()));

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

    // Wire up session ID
    vm.session_id = session_id;

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
                session_id: vm.session_id,
            })
        }
        Err(e) => {
            // Preserve any output generated before the error
            let mut output = String::new();
            if !vm.output_buffer.is_empty() {
                output.push_str(&vm.output_buffer);
            }
            Err(CfmlRunError { output, message: format!("{}", e) })
        }
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
    vfs: Arc<dyn Vfs>,
    sandbox: bool,
}

fn run_server(doc_root: &Path, port: u16, debug: bool, single_threaded: bool, vfs: Arc<dyn Vfs>, sandbox: bool) {
    let rt = if single_threaded {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_stack_size(8 * 1024 * 1024) // 8MB stack like main thread
            .build()
            .unwrap()
    };
    rt.block_on(async_run_server(doc_root, port, debug, single_threaded, vfs, sandbox));
}

async fn async_run_server(doc_root: &Path, port: u16, debug: bool, single_threaded: bool, vfs: Arc<dyn Vfs>, sandbox: bool) {
    let server_state = ServerState::new();

    // Load URL rewrite rules if urlrewrite.xml exists
    let rewrite_xml = doc_root.join("urlrewrite.xml");
    let rewrite_xml_str = rewrite_xml.to_string_lossy().to_string();
    let rewrite_rules = if vfs.is_file(&rewrite_xml_str) {
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
        vfs,
        sandbox,
    });

    let app = axum::Router::new()
        .fallback(handle_request)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap_or_else(|e| {
        eprintln!("Failed to start server on 0.0.0.0:{}: {}", port, e);
        exit(1);
    });
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();
}

async fn handle_request(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    req: axum::extract::Request,
) -> axum::response::Response {
    let (parts, body) = req.into_parts();
    let method = parts.method.to_string();
    let remote_addr = addr.ip().to_string();
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
    let resolved = resolve_file(&state.doc_root, &path, state.vfs.as_ref());

    match resolved {
        Some(ref rf) if rf.file_path.extension().map_or(false, |e| e == "cfm") => {
            // Execute .cfm file — in non-debug serve mode the bytecode cache reads the
            // file itself, so we pass an empty source and skip the redundant read.
            let source = if debug {
                match fs::read_to_string(&rf.file_path) {
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
                }
            } else {
                String::new()
            };

            // Build web scopes using resolved script_name and path_info
            let (extra_globals, http_request_data) = build_web_scopes(
                &method, &headers, &body_bytes, &rf.script_name, &rf.path_info, &query_string, state.port, &remote_addr,
            );

            let file_path = rf.file_path.to_string_lossy().to_string();
            let server_state = state.server_state.clone();
            let vfs = state.vfs.clone();
            let sandbox = state.sandbox;

            // Extract or generate session ID from cookies
            let session_id = {
                let cookie_header = headers.iter()
                    .find(|(n, _)| n.to_lowercase() == "cookie")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let existing_sid = cookie_header.split(';')
                    .find_map(|c| {
                        let c = c.trim();
                        if c.starts_with("CFID=") {
                            Some(c[5..].to_string())
                        } else {
                            None
                        }
                    });
                existing_sid.unwrap_or_else(|| {
                    // Generate a new session ID
                    use std::time::SystemTime;
                    let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos();
                    format!("{:x}", ts)
                })
            };
            let session_id_clone = session_id.clone();

            let result = tokio::task::spawn_blocking(move || {
                compile_and_run_with_session(
                    &source,
                    debug,
                    Some(file_path),
                    extra_globals,
                    Some(&server_state),
                    Some(http_request_data),
                    Some(session_id_clone),
                    vfs,
                    sandbox,
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

                    // Set session cookie
                    if let Some(ref sid) = response.session_id {
                        builder = builder.header("Set-Cookie", format!("CFID={}; Path=/; HttpOnly", sid));
                    }

                    builder.body(axum::body::Body::from(body)).unwrap()
                }
                Err(e) => {
                    let mut body = String::new();
                    if !e.output.is_empty() {
                        body.push_str(&e.output);
                    }
                    body.push_str(&format!(
                        "<html><body><h1>500 Internal Server Error</h1><pre>{}</pre></body></html>",
                        html_escape(&e.message)
                    ));
                    axum::response::Response::builder()
                        .status(500)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(axum::body::Body::from(body))
                        .unwrap()
                }
            }
        }
        Some(ref rf) => {
            // Serve static file (via VFS for embedded support)
            match state.vfs.read(&rf.file_path.to_string_lossy()) {
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
fn resolve_file(doc_root: &Path, url_path: &str, vfs: &dyn Vfs) -> Option<ResolvedFile> {
    // Normalize: strip leading slash, default to index.cfm
    let relative = url_path.trim_start_matches('/');

    // Try exact path first
    if !relative.is_empty() {
        let candidate = doc_root.join(relative);
        let candidate_str = candidate.to_string_lossy().to_string();
        if vfs.is_file(&candidate_str) {
            return Some(ResolvedFile {
                file_path: candidate,
                script_name: format!("/{}", relative),
                path_info: String::new(),
            });
        }
        // Try as directory with index.cfm
        let dir_index = doc_root.join(relative).join("index.cfm");
        let dir_index_str = dir_index.to_string_lossy().to_string();
        if vfs.is_file(&dir_index_str) {
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
            let candidate_str = candidate.to_string_lossy().to_string();
            if vfs.is_file(&candidate_str) && candidate.extension().map_or(false, |e| e == "cfm" || e == "cfc") {
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
        let index_str = index.to_string_lossy().to_string();
        if vfs.is_file(&index_str) {
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
    remote_addr: &str,
) -> (IndexMap<String, CfmlValue>, CfmlValue) {
    let mut globals = IndexMap::new();

    // CGI scope
    let mut cgi = IndexMap::new();
    cgi.insert("request_method".to_string(), CfmlValue::String(method.to_string()));
    let path_info = if path_info.is_empty() { "/" } else { path_info };
    cgi.insert("path_info".to_string(), CfmlValue::String(path_info.to_string()));
    cgi.insert("script_name".to_string(), CfmlValue::String(script_name.to_string()));
    cgi.insert("query_string".to_string(), CfmlValue::String(query_string.to_string()));
    cgi.insert("server_port".to_string(), CfmlValue::String(port.to_string()));
    cgi.insert("remote_addr".to_string(), CfmlValue::String(remote_addr.to_string()));
    cgi.insert("remote_host".to_string(), CfmlValue::String(remote_addr.to_string()));

    // Extract headers into CGI scope with http_ prefix (standard CGI convention)
    let mut content_type = String::new();
    let mut server_name = "127.0.0.1".to_string();
    for (name, value) in headers {
        let lower = name.to_lowercase();
        if lower == "content-type" {
            content_type = value.clone();
            cgi.insert("content_type".to_string(), CfmlValue::String(value.clone()));
        }
        if lower == "host" {
            // server_name from Host header (strip port if present)
            server_name = value.split(':').next().unwrap_or(value).to_string();
        }
        // Map all headers to cgi.http_* (replacing - with _)
        let cgi_key = format!("http_{}", lower.replace('-', "_"));
        cgi.insert(cgi_key, CfmlValue::String(value.clone()));
    }
    cgi.insert("server_name".to_string(), CfmlValue::String(server_name));

    globals.insert("cgi".to_string(), CfmlValue::Struct(cgi));

    // URL scope — parsed query string
    let url_scope = parse_query_string(query_string);
    globals.insert("url".to_string(), CfmlValue::Struct(url_scope));

    // Raw body as string
    let raw_body = String::from_utf8_lossy(body).to_string();

    // Form scope — parsed POST body (application/x-www-form-urlencoded or multipart/form-data)
    let form_scope = if method == "POST"
        && content_type.starts_with("application/x-www-form-urlencoded")
        && !raw_body.is_empty()
    {
        parse_query_string(&raw_body)
    } else if method == "POST" && content_type.starts_with("multipart/form-data") {
        parse_multipart_sync(&content_type, body)
    } else {
        IndexMap::new()
    };
    globals.insert("form".to_string(), CfmlValue::Struct(form_scope));

    // Cookie scope — parsed from Cookie header
    let cookie_scope = {
        let mut cookies = IndexMap::new();
        for (name, value) in headers {
            if name.to_lowercase() == "cookie" {
                for cookie in value.split(';') {
                    let cookie = cookie.trim();
                    if let Some(eq) = cookie.find('=') {
                        let cname = cookie[..eq].trim().to_string();
                        let cvalue = cookie[eq+1..].trim().to_string();
                        cookies.insert(cname, CfmlValue::String(cvalue));
                    }
                }
            }
        }
        cookies
    };
    globals.insert("cookie".to_string(), CfmlValue::Struct(cookie_scope));

    // Build full HTTP request data
    let mut headers_struct = IndexMap::new();
    for (name, value) in headers {
        headers_struct.insert(name.clone(), CfmlValue::String(value.clone()));
    }

    let mut http_request_data = IndexMap::new();
    http_request_data.insert("headers".to_string(), CfmlValue::Struct(headers_struct));
    http_request_data.insert("content".to_string(), CfmlValue::String(raw_body));
    http_request_data.insert("method".to_string(), CfmlValue::String(method.to_string()));
    http_request_data.insert("protocol".to_string(), CfmlValue::String("HTTP/1.1".to_string()));

    (globals, CfmlValue::Struct(http_request_data))
}

/// Parse a query string like "name=World&id=1" into a HashMap.
fn parse_query_string(qs: &str) -> IndexMap<String, CfmlValue> {
    let mut map = IndexMap::new();
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

/// Parse multipart/form-data body synchronously.
/// Extracts form fields and file uploads into the form scope.
/// File uploads are stored as structs with file metadata + temp path.
fn parse_multipart_sync(content_type: &str, body: &[u8]) -> IndexMap<String, CfmlValue> {
    let mut form = IndexMap::new();

    // Extract boundary from content-type
    let boundary = content_type
        .split(';')
        .find_map(|part| {
            let trimmed = part.trim();
            if trimmed.to_lowercase().starts_with("boundary=") {
                Some(trimmed[9..].trim_matches('"').to_string())
            } else {
                None
            }
        });

    let boundary = match boundary {
        Some(b) => b,
        None => return form,
    };

    let delimiter = format!("--{}", boundary);
    let end_delimiter = format!("--{}--", boundary);

    // Split body by boundary
    let body_str = String::from_utf8_lossy(body);
    let parts: Vec<&str> = body_str.split(&delimiter).collect();

    for part in parts {
        let part = part.trim_start_matches("\r\n").trim_end_matches("\r\n");
        if part.is_empty() || part == "--" || part.starts_with(&end_delimiter) {
            continue;
        }

        // Split headers from body
        let header_end = if let Some(pos) = part.find("\r\n\r\n") {
            pos
        } else if let Some(pos) = part.find("\n\n") {
            pos
        } else {
            continue;
        };

        let header_section = &part[..header_end];
        let body_start = if part[header_end..].starts_with("\r\n\r\n") {
            header_end + 4
        } else {
            header_end + 2
        };
        let part_body = &part[body_start..];

        // Parse Content-Disposition
        let mut field_name = String::new();
        let mut file_name = None;
        let mut part_content_type = None;

        for line in header_section.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("content-disposition:") {
                // Extract name
                if let Some(pos) = line.find("name=\"") {
                    let rest = &line[pos + 6..];
                    if let Some(end) = rest.find('"') {
                        field_name = rest[..end].to_string();
                    }
                }
                // Extract filename
                if let Some(pos) = line.find("filename=\"") {
                    let rest = &line[pos + 10..];
                    if let Some(end) = rest.find('"') {
                        file_name = Some(rest[..end].to_string());
                    }
                }
            } else if lower.starts_with("content-type:") {
                part_content_type = Some(line[13..].trim().to_string());
            }
        }

        if field_name.is_empty() {
            continue;
        }

        if let Some(fname) = file_name {
            // File upload — save to temp directory and store metadata
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!("cfupload_{}", fname));
            let _ = std::fs::write(&temp_path, part_body.as_bytes());

            let mut file_info = IndexMap::new();
            file_info.insert("serverFile".to_string(), CfmlValue::String(fname.clone()));
            file_info.insert("clientFile".to_string(), CfmlValue::String(fname.clone()));
            file_info.insert("serverDirectory".to_string(), CfmlValue::String(temp_dir.to_string_lossy().to_string()));
            file_info.insert("serverFileName".to_string(), CfmlValue::String(fname.clone()));
            file_info.insert("tempFilePath".to_string(), CfmlValue::String(temp_path.to_string_lossy().to_string()));
            file_info.insert("contentType".to_string(), CfmlValue::String(part_content_type.unwrap_or_else(|| "application/octet-stream".to_string())));
            file_info.insert("fileSize".to_string(), CfmlValue::Int(part_body.len() as i64));
            file_info.insert("fileWasSaved".to_string(), CfmlValue::Bool(true));

            form.insert(field_name.to_lowercase(), CfmlValue::Struct(file_info));
        } else {
            // Regular form field
            form.insert(field_name.to_lowercase(), CfmlValue::String(part_body.to_string()));
        }
    }

    form
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

// ---------------------------------------------------------------------------
// Self-contained binary: build & run
// ---------------------------------------------------------------------------

/// Build a self-contained binary by embedding all files from `app_dir` into a
/// copy of the current rustcfml executable.
fn build_self_contained(app_dir: &str, output: &str, mode: &str, entry: &str) {
    use std::collections::HashMap;

    let app_path = PathBuf::from(app_dir);
    if !app_path.is_dir() {
        eprintln!("Error: '{}' is not a directory", app_dir);
        exit(1);
    }

    let app_path = fs::canonicalize(&app_path).unwrap_or(app_path);
    println!("Embedding app from: {}", app_path.display());
    println!("Mode: {}, Entry: {}", mode, entry);

    // Walk directory and collect all files
    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    collect_files(&app_path, &app_path, &mut files);

    if files.is_empty() {
        eprintln!("Error: No files found in '{}'", app_dir);
        exit(1);
    }

    // Validate entry point exists for CLI mode
    if mode == "cli" {
        let entry_lower = entry.to_lowercase();
        if !files.keys().any(|k| k.to_lowercase() == entry_lower) {
            eprintln!("Error: Entry point '{}' not found in '{}'", entry, app_dir);
            eprintln!("Available files: {}", files.keys().cloned().collect::<Vec<_>>().join(", "));
            exit(1);
        }
    }

    // Embed metadata: mode and entry point
    let meta = format!("mode={}\nentry={}", mode, entry);
    files.insert("__rustcfml_meta__".to_string(), meta.into_bytes());

    let total_size: usize = files.values().map(|v| v.len()).sum();
    println!("Collected {} files ({:.1} KB)", files.len() - 1, total_size as f64 / 1024.0);

    // Read the current executable as the base binary
    let exe_path = std::env::current_exe().expect("Cannot determine current executable path");
    let base_binary = fs::read(&exe_path).expect("Cannot read current executable");

    // If the current exe already has an archive, strip it first
    let base_binary = strip_existing_archive(&base_binary);

    // On macOS, strip the code signature from the base binary before appending.
    // Apple Silicon requires signed binaries, so we'll re-sign after writing.
    #[cfg(target_os = "macos")]
    let base_binary = strip_macos_signature(base_binary.to_vec());
    #[cfg(not(target_os = "macos"))]
    let base_binary = base_binary.to_vec();

    // Create self-contained binary
    let output_data = vfs::create_self_contained_binary(&base_binary, &files);

    // Write output
    let output_path = PathBuf::from(output);
    fs::write(&output_path, &output_data).expect("Failed to write output binary");

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&output_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&output_path, perms).unwrap();
    }

    // On macOS (especially Apple Silicon), binaries must be code-signed.
    // The base binary's signature was stripped before appending; now re-sign.
    #[cfg(target_os = "macos")]
    {
        let out = output_path.to_str().unwrap_or("");
        let status = std::process::Command::new("codesign")
            .args(["--force", "--sign", "-", "--no-strict", out])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if let Ok(s) = status {
            if !s.success() {
                eprintln!("Warning: Failed to re-sign binary (codesign). The binary may not run on macOS.");
            }
        }
    }

    // Verify the archive is extractable from the final binary
    let final_data = fs::read(&output_path).expect("Cannot read output binary for verification");
    if vfs::extract_archive_from_bytes(&final_data).is_none() {
        eprintln!("Error: Archive verification failed — the embedded archive is not readable.");
        eprintln!("This may be caused by code signing. Try running: codesign --remove-signature {}", output_path.display());
        exit(1);
    }

    println!("Built: {} ({:.1} MB)", output_path.display(), final_data.len() as f64 / (1024.0 * 1024.0));
    println!("Run with: ./{}", output_path.display());
}

/// Recursively collect files from a directory into a HashMap.
/// Keys are relative paths with forward slashes.
fn collect_files(base: &Path, dir: &Path, files: &mut std::collections::HashMap<String, Vec<u8>>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Warning: Cannot read directory {}: {}", dir.display(), e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories and common non-app dirs
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == ".git" {
                continue;
            }
            collect_files(base, &path, files);
        } else if path.is_file() {
            let relative = path.strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            match fs::read(&path) {
                Ok(data) => {
                    files.insert(relative, data);
                }
                Err(e) => {
                    eprintln!("Warning: Cannot read {}: {}", path.display(), e);
                }
            }
        }
    }
}

/// Strip macOS code signature from binary data using codesign CLI.
/// This is needed because appending data to a signed Mach-O invalidates the
/// signature, and codesign won't re-sign a binary with a stale signature.
#[cfg(target_os = "macos")]
fn strip_macos_signature(data: Vec<u8>) -> Vec<u8> {
    let tmp = std::env::temp_dir().join("rustcfml_strip_sig");
    if fs::write(&tmp, &data).is_ok() {
        let status = std::process::Command::new("codesign")
            .args(["--remove-signature", tmp.to_str().unwrap_or("")])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if let Ok(s) = status {
            if s.success() {
                if let Ok(stripped) = fs::read(&tmp) {
                    let _ = fs::remove_file(&tmp);
                    return stripped;
                }
            }
        }
        let _ = fs::remove_file(&tmp);
    }
    data
}

/// Strip an existing embedded archive from a binary (if present).
fn strip_existing_archive(data: &[u8]) -> &[u8] {
    let len = data.len();
    if len < vfs::ARCHIVE_MAGIC.len() + 8 {
        return data;
    }
    let magic_start = len - vfs::ARCHIVE_MAGIC.len();
    if &data[magic_start..] != vfs::ARCHIVE_MAGIC.as_slice() {
        return data;
    }
    let len_start = magic_start - 8;
    let archive_len = u64::from_le_bytes(data[len_start..len_start + 8].try_into().unwrap()) as usize;
    let archive_start = len_start - archive_len;
    &data[..archive_start]
}

/// Parse metadata from the embedded archive.
fn parse_embedded_meta(files: &std::collections::HashMap<String, Vec<u8>>) -> (String, String) {
    let meta = files.get("__rustcfml_meta__")
        .map(|data| String::from_utf8_lossy(data).to_string())
        .unwrap_or_default();
    let mut mode = "serve".to_string();
    let mut entry = "main.cfm".to_string();
    for line in meta.lines() {
        if let Some(val) = line.strip_prefix("mode=") {
            mode = val.to_string();
        } else if let Some(val) = line.strip_prefix("entry=") {
            entry = val.to_string();
        }
    }
    (mode, entry)
}

/// Run the embedded app (self-contained binary mode).
/// Supports both "serve" (web server) and "cli" (command-line) modes.
fn run_embedded_app(mut files: std::collections::HashMap<String, Vec<u8>>) {
    use cfml_common::vfs::EmbeddedFs;

    let (mode, entry) = parse_embedded_meta(&files);

    // Remove metadata file from the archive so it's not visible to CFML code
    files.remove("__rustcfml_meta__");
    let file_count = files.len();

    // Determine base dir: use CWD as the virtual base
    let base_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .to_string_lossy()
        .to_string();

    let vfs: Arc<dyn Vfs> = Arc::new(EmbeddedFs::new(files, base_dir.clone()));

    if mode == "cli" {
        run_embedded_cli(vfs, &base_dir, &entry, file_count);
    } else {
        run_embedded_serve(vfs, &base_dir, file_count);
    }
}

/// Run embedded app in CLI mode: execute entry point with command-line args.
fn run_embedded_cli(vfs: Arc<dyn Vfs>, base_dir: &str, entry: &str, file_count: usize) {
    let cli_args: Vec<String> = std::env::args().collect();

    let mut sandbox = false;
    // Check for --help / --version / --sandbox
    for arg in &cli_args[1..] {
        match arg.as_str() {
            "--version" => {
                println!("Built with RustCFML v{} ({} embedded files)", env!("CARGO_PKG_VERSION"), file_count);
                exit(0);
            }
            "--sandbox" => { sandbox = true; }
            _ => {}
        }
    }

    // Build the entry point path and read source from VFS
    let entry_path = format!("{}/{}", base_dir, entry);
    let source = match vfs.read_to_string(&entry_path) {
        Ok(s) => s,
        Err(_) => {
            // Try just the entry name (relative)
            match vfs.read_to_string(entry) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: Cannot read entry point '{}': {}", entry, e);
                    exit(1);
                }
            }
        }
    };

    // Parse CLI args into the "cli" scope (ordered struct).
    // Works like CFML's arguments scope:
    //   --name value  → cli.name = "value"   (named)
    //   --flag        → cli.flag = true       (boolean flag)
    //   positional    → cli[1], cli[2], ...   (1-based numeric keys)
    let mut cli_scope = IndexMap::new();
    let mut positional_idx: usize = 1;
    let mut i = 1;
    while i < cli_args.len() {
        let arg = &cli_args[i];
        if arg.starts_with("--") {
            let key = arg.trim_start_matches('-').to_lowercase();
            if i + 1 < cli_args.len() && !cli_args[i + 1].starts_with("--") {
                cli_scope.insert(key, CfmlValue::String(cli_args[i + 1].clone()));
                i += 2;
            } else {
                cli_scope.insert(key, CfmlValue::Bool(true));
                i += 1;
            }
        } else if arg.starts_with("-") && arg.len() == 2 {
            let key = arg[1..].to_lowercase();
            if i + 1 < cli_args.len() && !cli_args[i + 1].starts_with("-") {
                cli_scope.insert(key, CfmlValue::String(cli_args[i + 1].clone()));
                i += 2;
            } else {
                cli_scope.insert(key, CfmlValue::Bool(true));
                i += 1;
            }
        } else {
            // Positional: 1-based numeric key like CFML arguments scope
            cli_scope.insert(positional_idx.to_string(), CfmlValue::String(arg.clone()));
            positional_idx += 1;
            i += 1;
        }
    }

    let mut extra_globals = IndexMap::new();
    extra_globals.insert("cli".to_string(), CfmlValue::Struct(cli_scope));

    // Execute
    match compile_and_run(&source, false, Some(entry_path), extra_globals, None, None, None, vfs, sandbox) {
        Ok(response) => {
            if !response.output.is_empty() {
                print!("{}", response.output);
            }
        }
        Err(e) => {
            if !e.output.is_empty() {
                print!("{}", e.output);
            }
            eprintln!("{}", e.message);
            exit(1);
        }
    }
}

/// Run embedded app in serve mode with start/stop/foreground support.
fn run_embedded_serve(vfs: Arc<dyn Vfs>, base_dir: &str, file_count: usize) {
    let cli_args: Vec<String> = std::env::args().collect();

    // Parse args
    let mut port: u16 = 8500;
    let mut single_threaded = false;
    let mut sandbox = false;
    let mut command = ""; // "", "start", "stop", "status"
    let mut i = 1;
    while i < cli_args.len() {
        match cli_args[i].as_str() {
            "--port" if i + 1 < cli_args.len() => {
                port = cli_args[i + 1].parse().unwrap_or(8500);
                i += 2;
            }
            "--single-threaded" => {
                single_threaded = true;
                i += 1;
            }
            "--sandbox" => {
                sandbox = true;
                i += 1;
            }
            "--version" => {
                println!("RustCFML v{} (self-contained, {} files)", env!("CARGO_PKG_VERSION"), file_count);
                exit(0);
            }
            "start" | "stop" | "status" => {
                command = match cli_args[i].as_str() {
                    "start" => "start",
                    "stop" => "stop",
                    "status" => "status",
                    _ => "",
                };
                i += 1;
            }
            _ => { i += 1; }
        }
    }

    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "app".to_string());
    let pid_file = format!("/tmp/{}.pid", exe_name);

    match command {
        "stop" => {
            embedded_stop(&pid_file);
        }
        "status" => {
            embedded_status(&pid_file);
        }
        "start" => {
            embedded_start(&pid_file, port, file_count);
            // After daemonizing, the child process continues here
            if sandbox { println!("Sandbox mode: host filesystem access disabled"); }
            println!("RustCFML self-contained app ({} embedded files)", file_count);
            let doc_root = PathBuf::from(base_dir);
            run_server(&doc_root, port, false, single_threaded, vfs, sandbox);
        }
        _ => {
            // Foreground mode (default: just run)
            if sandbox { println!("Sandbox mode: host filesystem access disabled"); }
            println!("RustCFML self-contained app ({} embedded files)", file_count);
            let doc_root = PathBuf::from(base_dir);
            run_server(&doc_root, port, false, single_threaded, vfs, sandbox);
        }
    }
}

/// Daemonize: fork to background and write PID file.
#[cfg(unix)]
fn embedded_start(pid_file: &str, port: u16, file_count: usize) {
    use std::io::Write;

    // Check if already running
    if let Ok(pid_str) = fs::read_to_string(pid_file) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Check if process is alive
            if unsafe { libc::kill(pid, 0) } == 0 {
                eprintln!("Already running (PID {})", pid);
                exit(1);
            }
        }
    }

    // Fork
    let pid = unsafe { libc::fork() };
    match pid {
        -1 => {
            eprintln!("Failed to fork");
            exit(1);
        }
        0 => {
            // Child process — continue to run the server
            // Create new session
            unsafe { libc::setsid() };

            // Write PID file
            let child_pid = std::process::id();
            let mut f = std::fs::File::create(pid_file).expect("Cannot create PID file");
            write!(f, "{}", child_pid).expect("Cannot write PID file");

            // Redirect stdout/stderr to log file
            let log_path = pid_file.replace(".pid", ".log");
            if let Ok(log_file) = std::fs::File::create(&log_path) {
                use std::os::unix::io::AsRawFd;
                let fd = log_file.as_raw_fd();
                unsafe {
                    libc::dup2(fd, 1); // stdout
                    libc::dup2(fd, 2); // stderr
                }
            }
            // Child continues to the server startup code
        }
        _ => {
            // Parent process — report and exit
            println!("Started in background (PID {})", pid);
            println!("Listening on http://127.0.0.1:{} ({} embedded files)", port, file_count);
            println!("Stop with: {} stop", std::env::args().next().unwrap_or_default());
            exit(0);
        }
    }
}

#[cfg(not(unix))]
fn embedded_start(pid_file: &str, _port: u16, _file_count: usize) {
    // On non-Unix, just write PID and run in foreground
    let pid = std::process::id();
    let _ = fs::write(pid_file, format!("{}", pid));
}

/// Stop a daemonized instance by reading its PID file.
#[cfg(unix)]
fn embedded_stop(pid_file: &str) {
    match fs::read_to_string(pid_file) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                if unsafe { libc::kill(pid, libc::SIGTERM) } == 0 {
                    println!("Stopped (PID {})", pid);
                    let _ = fs::remove_file(pid_file);
                } else {
                    eprintln!("Process {} not running", pid);
                    let _ = fs::remove_file(pid_file);
                }
            } else {
                eprintln!("Invalid PID file");
            }
        }
        Err(_) => {
            eprintln!("Not running (no PID file)");
        }
    }
    exit(0);
}

#[cfg(not(unix))]
fn embedded_stop(pid_file: &str) {
    eprintln!("Stop command not supported on this platform");
    eprintln!("PID file: {}", pid_file);
    exit(1);
}

/// Check status of a daemonized instance.
fn embedded_status(pid_file: &str) {
    match fs::read_to_string(pid_file) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                #[cfg(unix)]
                {
                    if unsafe { libc::kill(pid, 0) } == 0 {
                        println!("Running (PID {})", pid);
                    } else {
                        println!("Not running (stale PID file, was PID {})", pid);
                    }
                }
                #[cfg(not(unix))]
                {
                    println!("PID file exists (PID {})", pid);
                }
            } else {
                println!("Invalid PID file");
            }
        }
        Err(_) => {
            println!("Not running (no PID file)");
        }
    }
    exit(0);
}
