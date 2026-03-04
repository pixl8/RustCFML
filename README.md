## RustCFML

![RustCFML Mascot](crab.svg)

A CFML (ColdFusion&reg; Markup Language) Interpreter written in Rust.

ColdFusion is a registered trademark of Adobe Inc. This project is not affiliated with or endorsed by Adobe.

![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)

## Usage

**[Try RustCFML in your browser](https://pixl8.github.io/RustCFML/demo/)** — interactive demo running on WebAssembly.

RustCFML requires Rust stable (>= 1.75.0). Install via [rustup.rs](https://rustup.rs/).

### Building from Source

```plaintext
git clone https://github.com/pixl8/RustCFML.git
cd RustCFML
cargo build --release
```

### Running CFML Files

```plaintext
cargo run --release -- myapp.cfm          # Run a .cfm template
cargo run --release -- -c 'writeOutput("Hello!")' # Inline code
cargo run --release -- -r                 # Interactive REPL
```

### Web Server Mode

Serve `.cfm` files over HTTP with full CFML web scopes (CGI, URL, Form, Cookie, Session, Application, Request):

```plaintext
cargo run --release -- --serve                          # Current dir on port 8500
cargo run --release -- --serve examples/miniapp --port 3000  # Custom root and port
```

The server is built on [Axum](https://github.com/tokio-rs/axum) with concurrent request handling. It serves `.cfm` files and static assets from the document root. Directory requests serve `index.cfm` if present. Path info routing is supported (`/index.cfm/users/123` resolves to `index.cfm` with path info `/users/123`). Bytecode caching skips recompilation for unchanged files across requests.

#### URL Rewriting

Place a `urlrewrite.xml` file in your document root for Tuckey-compatible URL rewriting. This enables clean URLs and REST-style routing:

```xml
<?xml version="1.0" encoding="utf-8"?>
<urlrewrite>
    <rule>
        <from>^/([a-zA-Z][a-zA-Z0-9_/-]*)$</from>
        <to>/index.cfm/$1</to>
    </rule>
    <rule>
        <from>^/old-page$</from>
        <to type="permanent-redirect">/new-page</to>
    </rule>
</urlrewrite>
```

Supported features:
- **Regex and wildcard patterns** with backreference substitution (`$1`, `$2`)
- **Forward**, **redirect** (302), and **permanent-redirect** (301) actions
- **Conditions** on HTTP method, port, and headers
- **Rule chaining** with `last="true"` to stop processing

#### Application.cfc Lifecycle

If an `Application.cfc` file exists in the document root (or any parent directory), it is automatically loaded and its lifecycle methods are called:

- `onApplicationStart()` — runs once when the application is first accessed
- `onRequestStart(targetPage)` — runs before each request
- `onRequest(targetPage)` — handles the request (replaces default page execution)
- `onRequestEnd(targetPage)` — runs after each request
- `onError(exception, eventName)` — handles uncaught errors

Application state (`application` scope) persists across requests in serve mode. Component mappings defined via `this.mappings` in Application.cfc are supported for virtual path resolution.

### Installing Globally

```plaintext
cargo install --path crates/cli
rustcfml myapp.cfm
```

### Shell Scripts (Shebang Support)

RustCFML scripts can be executed directly as shell scripts using a shebang line. The file extension does not matter.

```bash
#!/usr/bin/env rustcfml
writeOutput("Hello from a shell script!" & chr(10));
var x = 2 + 2;
writeOutput("2 + 2 = " & x & chr(10));
```

```plaintext
chmod +x myscript.cfm
./myscript.cfm
```

## Performance

Benchmarked serving a "Hello World" `.cfm` page using Apache Bench (`ab -n 100 -c 1`):

| Metric | RustCFML | Lucee 7.0.1 | BoxLang 1.10 |
|---|---|---|---|
| **Memory (RSS)** | **~8 MB** | ~350 MB | ~305 MB |
| **Requests/sec** | **1,949 req/s** | 635 req/s | 293 req/s |
| **Avg response time** | **0.5 ms** | 1.6 ms | 3.4 ms |
| **Startup** | instant | ~15s | ~15s |

RustCFML compiles to a native binary with no runtime VM overhead, resulting in significantly lower memory usage and faster response times compared to JVM-based CFML engines.

## Features

RustCFML covers a substantial portion of the CFML language:

- **Full CFScript and CFML tag syntax** — tag preprocessor converts 50+ CFML tags to CFScript automatically
- **Stack-based bytecode VM** with compilation caching in serve mode
- **400+ built-in functions** — strings, arrays, structs, math, dates, lists, queries, JSON, file I/O, regex, security, caching, hashing, encoding, XML, INI files, locale formatting, password hashing (bcrypt/scrypt/argon2)
- **Member functions and method chaining** — `"hello".ucase().reverse()`, `[1,2,3].len()`
- **Higher-order functions** — map, filter, reduce, each, some, every on arrays, structs, lists, queries, strings, and collections
- **Components** — inheritance, interfaces, implicit accessors, `onMissingMethod`, metadata, `createObject`
- **Web server** — Application.cfc lifecycle, sessions, cookies, authentication, URL rewriting, file uploads, component mappings
- **Database** — `queryExecute` with SQLite, MySQL, PostgreSQL, MSSQL; connection pooling, `cfqueryparam`, `cftransaction`
- **HTTP client** — `cfhttp`/`cfhttpparam` for GET/POST/PUT/DELETE/PATCH
- **Email** — `cfmail`/`cfmailparam`/`cfmailpart` with SMTP sending
- **Threading** — `cfthread` tag (sequential execution model)
- **Closures** — scope capture with parent write-back, arrow functions, spread operator
- **WASM target** — compile to WebAssembly via `wasm-bindgen`

See [Work.md](Work.md) for detailed implementation status.

### Not Supported

- **Query-of-Queries (QoQ)** — SQL SELECT on in-memory query objects
- Image functions, Spreadsheet functions, ORM, SOAP/WSDL, Flash/Flex, PDF, LDAP, Registry
- `cfschedule`, `cfwddx`, real concurrent `cfthread` execution

## Architecture

```plaintext
CFML Source (.cfm / .cfc)
    → Tag Preprocessor → CFScript
    → Lexer → Tokens
    → Parser → AST
    → Compiler → Bytecode
    → VM → Output
```

```plaintext
crates/
├── cfml-common/     # Shared types: CfmlValue, CfmlError
├── cfml-compiler/   # Lexer, Parser, AST, Tag Preprocessor
├── cfml-codegen/    # Bytecode compiler (AST → BytecodeOp)
├── cfml-vm/         # Stack-based bytecode VM
├── cfml-stdlib/     # 400+ built-in functions
├── cli/             # CLI + Axum web server
└── wasm/            # WebAssembly target
```

## Embedding in Rust

```rust
use cfml_codegen::compiler::CfmlCompiler;
use cfml_compiler::parser::Parser;
use cfml_stdlib::builtins::{get_builtin_functions, get_builtins};
use cfml_vm::CfmlVirtualMachine;

let source = r#"writeOutput("Hello from Rust!");"#;
let ast = Parser::new(source.to_string()).parse().unwrap();
let program = CfmlCompiler::new().compile(ast);
let mut vm = CfmlVirtualMachine::new(program);
for (name, value) in get_builtins() { vm.globals.insert(name, value); }
for (name, func) in get_builtin_functions() { vm.builtins.insert(name, func); }
vm.execute().unwrap();
println!("{}", vm.output_buffer);
```

## Compiling to WebAssembly

```plaintext
cargo install wasm-pack
wasm-pack build crates/wasm --target web
```

```javascript
import init, { CfmlEngine } from './pkg/rustcfml_wasm.js';
await init();
const output = CfmlEngine.new().execute('writeOutput("Hello from WASM!");');
```

## Testing

```plaintext
cargo run -- tests/runner.cfm    # 1181 assertions across 89 suites
cargo test                        # Rust unit tests
```

See [TESTING.md](TESTING.md) for the full testing guide.

## Related Projects

- [Lucee](https://github.com/lucee/Lucee) — open-source CFML engine (Java)
- [BoxLang](https://github.com/ortus-boxlang/BoxLang) — modern CFML+ runtime (Java)
- [RustPython](https://github.com/RustPython/RustPython) — Python interpreter in Rust (architectural reference)

## License

MIT
