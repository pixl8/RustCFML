# RustCFML - Claude Code Guide

## Project Overview

CFML (ColdFusion Markup Language) interpreter written in Rust. Compiles CFML source through a pipeline: **Tag Preprocessor -> Lexer -> Parser -> AST -> Bytecode Compiler -> Stack-based VM**. Inspired by RustPython.

## Build & Test

```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo run -- tests/runner.cfm        # Run all tests (~1181 assertions, 89 suites)
cargo run --release -- file.cfm      # Run a CFML file
cargo run --release -- --serve       # Start web server on port 8500
cargo test                           # Rust unit tests (tag_parser, etc.)
```

Tests are CFML-based, not Rust-based. The test runner (`tests/runner.cfm`) includes all test files and uses the harness (`tests/harness.cfm`) which provides `assert()`, `assertTrue()`, `assertFalse()`, `assertNull()`, `assertThrows()`, `suiteBegin()`, `suiteEnd()`.

### Cross-engine testing (Lucee)

The same test suite runs against Lucee to verify compatibility with the reference engine. Start Lucee via CommandBox (served out of the project root), then hit `tests/runner.cfm` over HTTP:

```bash
box server start cfengine=lucee@be   # starts Lucee on a CommandBox-assigned port (e.g. 127.0.0.1:8585)
curl -s http://127.0.0.1:8585/tests/runner.cfm -o /tmp/lucee_out.txt
grep -E "^(SUMMARY|FAIL \||ERROR)" /tmp/lucee_out.txt
box server status                    # show port if you don't see it in startup output
box server stop                      # shut it down
```

**Writing tests that pass on both engines:**
- Do NOT use `var` at page scope — Lucee rejects it ("Unsupported Context for Local Scope"). Declare without `var` at page level, or wrap the test body in a function.
- Always close `<cfscript>` blocks with `</cfscript>` — Lucee's parser is strict about this; RustCFML tolerates EOF.
- The test runner includes `harness.cfm` once at the top. Individual test files must NOT re-include it, because the harness body resets `request._test_total*` counters and masks the grand summary.
- HTTP-dependent tests (`tests/tags/test_tags_cfscript_statements.cfm`) discover the port from `cgi.server_port` (set by the server at request-time) and skip the HTTP subtests when the runner is invoked from the CLI with no server available. Don't hardcode a port.
- Lucee and RustCFML both run through the same `tests/runner.cfm`, so a green run on both is the compatibility bar.

## Architecture

```
crates/
  cfml-common/      # CfmlValue enum, CfmlError, Position (small, ~470 lines)
  cfml-compiler/    # Lexer, Parser, AST, Tag Preprocessor (~5,700 lines)
  cfml-codegen/     # AST -> BytecodeOp compiler (~1,850 lines)
  cfml-vm/          # Stack-based bytecode VM (~6,900 lines) - THE BIG FILE
  cfml-stdlib/      # 400+ built-in functions (~9,800 lines) - THE OTHER BIG FILE
  cli/              # CLI entry point + Axum web server (~1,500 lines)
  wasm/             # WebAssembly target (small wrapper)
```

### Compilation Pipeline

1. **Tag Preprocessor** (`tag_parser.rs`): Converts `<cfset x=1>` style tags to CFScript (`x=1;`). All CFML tags become function calls or script constructs. Body tags use `find_closing_tag` to extract content.

2. **Lexer** (`lexer.rs`): Tokenizes CFScript source. Tokens defined in `token.rs`.

3. **Parser** (`parser.rs`): Recursive descent parser producing AST nodes defined in `ast.rs`.

4. **Codegen** (`compiler.rs`): Walks AST and emits `BytecodeOp` instructions defined in `bytecode.rs`.

5. **VM** (`lib.rs`): Stack-based execution engine. The main loop is `execute_function_with_args()` which processes bytecode ops.

## Key Patterns

### Adding a New Built-in Function

**Simple (pure function, no VM state needed):**
1. Register in `builtins.rs` → `get_builtin_functions()` with section comment
2. Implement as `fn fn_name(args: Vec<CfmlValue>) -> CfmlResult`

**VM-intercepted (needs access to VM state like output_buffer, globals, closures):**
1. Register a stub in `builtins.rs` that returns an error
2. Add the function name (lowercase) to the intercept list in `lib.rs` `call_function()` (~line 1718)
3. Add the handler in `call_function()` after the intercept check

Examples of VM-intercepted: `writeOutput`, `writeDump`, `sleep`, `include`, all higher-order functions (arrayMap, structFilter, etc.), savecontent, cfthread.

### Adding a New CFML Tag

1. Add the tag name match arm in `tag_parser.rs` → `tags_to_script_impl()`
2. Convert to CFScript equivalent (function call, block, or statement)
3. Body tags: use `find_closing_tag(chars, tag_end, len, "tagname")` to find `</cftagname>`
4. For tags needing VM support: register stub functions + add VM intercepts (see cfthread, cfsavecontent patterns)

### Adding a New Operator

1. Add token variant in `token.rs`
2. Add lexer recognition in `lexer.rs`
3. Add parser handling in `parser.rs` (check precedence level)
4. Add AST node if needed in `ast.rs`
5. Add codegen in `compiler.rs` → emit bytecode ops
6. Add VM handling in `lib.rs` if new bytecode op

### Adding Tests

1. Create `tests/<category>/test_<feature>.cfm`
2. Use the harness: `suiteBegin("Name")`, `assert("label", actual, expected)`, `suiteEnd()`
3. Add `try { include "category/test_file.cfm"; } catch ...` line in `tests/runner.cfm`

## Important Conventions

- **Case-insensitive**: All CFML identifiers, function names, scope keys are case-insensitive. Use `.to_lowercase()` or `eq_ignore_ascii_case()` for comparisons.
- **IndexMap, not HashMap**: Use `IndexMap<String, CfmlValue>` for ordered key-value structures (structs, scopes). `HashMap` only for internal lookups (builtins, user_functions).
- **CfmlValue**: The core enum — `Null`, `Boolean(bool)`, `Int(i64)`, `Double(f64)`, `String(String)`, `Array(Vec<CfmlValue>)`, `Struct(IndexMap<String, CfmlValue>)`, `Function(CfmlFunction)`, `Query(...)`, `Binary(Vec<u8>)`, `Component(...)`.
- **CfmlResult**: `Result<CfmlValue, CfmlError>`. Functions return `Ok(CfmlValue::Null)` for void operations.
- **1-based arrays**: CFML arrays are 1-based. Convert to 0-based for Rust Vec access.
- **Output buffering**: `self.output_buffer` collects all output. `saved_output_buffers` is a stack for nested capture (cfsavecontent, cfsilent, cfthread).
- **Closure scope capture**: Closures carry `captured_scope: Option<Arc<RwLock<IndexMap>>>`. Sibling closures share the same Arc. Write-back propagates mutations to parent scope.

## Scope Resolution Order

Variable lookup checks scopes in CFML-standard order: `local` -> `arguments` -> `thread` (inside cfthread) -> `variables` -> `cgi` -> `url` -> `form` -> `cookie` -> `request` -> `application` -> `server` -> `session`.

Explicit scope prefix (`variables.x`, `request.x`) bypasses the search chain.

## Common Gotchas

- `lib.rs` and `builtins.rs` are very large files (~7k and ~10k lines). Use grep to find specific handlers rather than reading the whole file.
- The VM intercept pattern means some "builtin" functions are actually handled in `lib.rs`, not `builtins.rs`. Always check both.
- Tag preprocessing happens before parsing — the parser only sees CFScript. Debug tag issues by checking the tag_parser output first.
- Query column access via dot notation (`q.name`) returns an Array of column values. Bracket notation (`q["name"]`) is different.
- The `request` scope persists across includes within a single execution — tests use it for state (harness counters).

## Git Commit Style

Short, descriptive first line summarizing the change. Examples from history:
```
Optimize VM function call dispatch: 5.9x faster recursive calls
Add getProfileString, setProfileString, getProfileSections; simplify README
Add bytecode cache for serve mode, fix all build warnings
```

**NEVER add Co-Authored-By lines to commits.**

## Reference

- Compatibility target: [cfdocs.org](https://cfdocs.org) (functions and tags)
- [Work.md](Work.md) — detailed implementation status
- [TESTING.md](TESTING.md) — testing guide
- Repo: github.com/pixl8/RustCFML
