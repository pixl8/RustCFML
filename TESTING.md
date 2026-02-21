# RustCFML Testing Guide

## Quick Start

```bash
cd RustCFML

# Build the project
cargo build

# Run a CFML file
cargo run -- examples/01_hello.cfm

# Run inline code
cargo run -- -c 'writeOutput("Hello World");'

# Run with debug output (tokens, AST, bytecode)
cargo run -- -d -c 'var x = 1 + 2; writeOutput(x);'

# Start the REPL
cargo run -- -r
```

---

## Testing Methods

### 1. Inline Code (`-c` flag)

The fastest way to test individual features:

```bash
# Basic output
cargo run -- -c 'writeOutput("Hello World");'

# Variables and arithmetic
cargo run -- -c 'var x = 10; var y = 20; writeOutput(x + y);'

# String concatenation
cargo run -- -c 'writeOutput("Hello" & " " & "World");'
```

### 2. CFML Files

Create `.cfm` files and execute them:

```bash
cargo run -- mytest.cfm
```

### 3. Debug Mode (`-d` flag)

Inspect the full pipeline — tokens, AST, and bytecode:

```bash
cargo run -- -d -c 'var x = 5; writeOutput(x);'
```

This outputs:
- `=== TOKENS ===` — lexer output
- `=== AST ===` — parsed syntax tree
- `=== BYTECODE ===` — compiled instructions
- Then the actual execution output

For CFML tag files, you also get `=== TAG CONVERSION ===` showing the tag-to-script output.

### 4. REPL Mode (`-r` flag)

Interactive line-by-line execution:

```bash
cargo run -- -r
cfml> writeOutput("Hello")
Hello
cfml> var x = 42
cfml> writeOutput(x)
42
cfml> exit
```

Note: each REPL line executes independently (no shared state between lines).

### 5. Rust Unit Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p cfml-compiler

# Run a specific test
cargo test -p cfml-compiler tag_parser::tests::test_cfset
```

Currently tests exist in `crates/cfml-compiler/src/tag_parser.rs`. See [Adding Unit Tests](#adding-unit-tests) below.

### 6. Example Files

Pre-built examples in `examples/`:

```bash
cargo run -- examples/01_hello.cfm       # Hello World
cargo run -- examples/02_variables.cfm    # Variables and arithmetic
cargo run -- examples/03_conditionals.cfm # If/else
cargo run -- examples/04_arrays.cfm       # Arrays
cargo run -- examples/05_ternary.cfm      # Nested conditionals
cargo run -- examples/06_expressions.cfm  # Parenthesised expressions
cargo run -- examples/07_booleans.cfm     # Boolean logic
cargo run -- examples/08_builtins.cfm     # Built-in functions
```

---

## What to Test

### Variables & Types

```bash
cargo run -- -c 'var s = "hello"; var n = 42; var d = 3.14; var b = true; var x = null; writeOutput(s & " " & n & " " & d & " " & b);'
```

### Arithmetic

```bash
cargo run -- -c 'writeOutput(2 + 3);'          # 5
cargo run -- -c 'writeOutput(10 - 4);'         # 6
cargo run -- -c 'writeOutput(3 * 7);'          # 21
cargo run -- -c 'writeOutput(20 / 4);'         # 5
cargo run -- -c 'writeOutput(17 % 5);'         # 2
cargo run -- -c 'writeOutput(2 ^ 10);'         # 1024
```

### String Operations

```bash
cargo run -- -c 'writeOutput(ucase("hello"));'                     # HELLO
cargo run -- -c 'writeOutput(reverse("hello"));'                   # olleh
cargo run -- -c 'writeOutput(len("hello world"));'                 # 11
cargo run -- -c 'writeOutput(replace("hello world", "world", "CFML"));'  # hello CFML
cargo run -- -c 'writeOutput("hello".ucase());'                    # HELLO (member function)
cargo run -- -c 'writeOutput("  hello  ".trim());'                 # hello
cargo run -- -c 'writeOutput("hello".left(3));'                    # hel
```

### Arrays

```bash
cargo run -- -c 'var arr = [1, 2, 3, 4, 5]; writeOutput(arr[1] & " " & arr.len());'
cargo run -- -c 'var arr = ["a","b","c"]; writeOutput(arr.toList());'
cargo run -- -c 'writeOutput([1,2,3,4,5].sum());'
cargo run -- -c 'writeOutput([10,20,30].first() & " " & [10,20,30].last());'
```

### Structs

```bash
cargo run -- -c 'var s = {name: "Alex", age: 30}; writeOutput(s.name & " is " & s.age);'
cargo run -- -c 'var s = {a: 1, b: 2}; writeOutput(s.keyList() & " count=" & s.count());'
```

### Control Flow

```bash
# If/else
cargo run -- -c 'var x = 10; if (x > 5) { writeOutput("big"); } else { writeOutput("small"); }'

# For loop
cargo run -- -c 'for (var i = 1; i <= 5; i++) { writeOutput(i & " "); }'

# While loop
cargo run -- -c 'var i = 1; while (i <= 5) { writeOutput(i & " "); i++; }'

# Switch/case
cargo run -- -c 'var x = 2; switch(x) { case 1: writeOutput("one"); break; case 2: writeOutput("two"); break; default: writeOutput("other"); }'

# Ternary
cargo run -- -c 'var x = 10; writeOutput(x > 5 ? "big" : "small");'

# Break/continue
cargo run -- -c 'for (var i = 1; i <= 10; i++) { if (i == 5) break; writeOutput(i & " "); }'
```

### CFML Keyword Operators

```bash
cargo run -- -c 'writeOutput(5 GT 3);'                     # true
cargo run -- -c 'writeOutput(5 LT 3);'                     # false
cargo run -- -c 'writeOutput(5 EQ 5);'                     # true
cargo run -- -c 'writeOutput(5 NEQ 3);'                    # true
cargo run -- -c 'writeOutput("hello" CONTAINS "ell");'     # true
cargo run -- -c 'writeOutput(true AND false);'             # false
cargo run -- -c 'writeOutput(true OR false);'              # true
```

### Functions

```bash
# User-defined function
cargo run -- -c 'function add(a, b) { return a + b; } writeOutput(add(3, 4));'

# Recursive function
cargo run -- -c 'function fib(n) { if (n <= 1) return n; return fib(n-1) + fib(n-2); } writeOutput(fib(10));'

# Closures
cargo run -- -c 'var doubled = [1,2,3].map(function(n) { return n * 2; }); writeOutput(doubled.toList());'
```

### Higher-Order Functions

```bash
# map
cargo run -- -c 'writeOutput([1,2,3,4,5].map(function(n) { return n * 10; }).toList());'

# filter
cargo run -- -c 'writeOutput([1,2,3,4,5,6].filter(function(n) { return n % 2 == 0; }).toList());'

# reduce
cargo run -- -c 'writeOutput([1,2,3,4,5].reduce(function(acc, n) { return acc + n; }, 0));'

# each
cargo run -- -c '[1,2,3].each(function(n) { writeOutput(n & " "); });'

# Standalone form
cargo run -- -c 'var result = arrayMap([1,2,3], function(n) { return n * 2; }); writeOutput(result.toList());'
```

### Member Function Chaining

```bash
cargo run -- -c 'writeOutput("hello world".ucase().reverse());'
cargo run -- -c 'writeOutput("Hello".left(3) & "World".right(3));'
```

### Error Handling

```bash
cargo run -- -c 'try { throw("oops"); } catch (any e) { writeOutput("Caught: " & e); }'
```

### Math Functions

```bash
cargo run -- -c 'writeOutput(abs(-5) & " " & ceiling(3.2) & " " & floor(3.8) & " " & round(3.5));'
cargo run -- -c 'writeOutput(max(10, 20) & " " & min(10, 20) & " " & sqr(16));'
```

### JSON

```bash
cargo run -- -c 'var data = {name: "test", value: 42}; writeOutput(serializeJSON(data));'
```

### Type Checking

```bash
cargo run -- -c 'writeOutput(isNumeric(42) & " " & isNumeric("abc") & " " & isArray([1,2]) & " " & isStruct({a:1}));'
```

### CFML Tags

Create a `.cfm` file with tag syntax:

```cfml
<cfset name = "World">
<cfoutput>Hello #name#!</cfoutput>

<cfset score = 85>
<cfif score GTE 90>
    <cfoutput>Grade: A</cfoutput>
<cfelseif score GTE 80>
    <cfoutput>Grade: B</cfoutput>
<cfelse>
    <cfoutput>Grade: F</cfoutput>
</cfif>

<cfloop from="1" to="5" index="i">
    <cfoutput>#i# </cfoutput>
</cfloop>

<cfscript>
    writeOutput("Script block works too!");
</cfscript>
```

```bash
cargo run -- myfile.cfm
```

### Arguments Scope

```bash
cargo run -- -c 'function test(name) { writeOutput(arguments.name); } test("World");'
```

### Closures with Parent Scope

```bash
cargo run -- -c 'var x = 10; function inner() { return x + 5; } writeOutput(inner());'
```

---

## Adding Unit Tests

### Inline Tests (in source files)

Add `#[cfg(test)]` modules to any crate source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
        assert_eq!(result, expected);
    }
}
```

### Integration Test Pattern

To test the full pipeline (source → output), create a helper:

```rust
// In any crate's test module
fn run_cfml(source: &str) -> String {
    use cfml_codegen::compiler::CfmlCompiler;
    use cfml_compiler::parser::Parser;
    use cfml_stdlib::builtins::{get_builtin_functions, get_builtins};
    use cfml_vm::CfmlVirtualMachine;

    let mut parser = Parser::new(source.to_string());
    let ast = parser.parse().expect("parse failed");
    let compiler = CfmlCompiler::new();
    let program = compiler.compile(ast);
    let mut vm = CfmlVirtualMachine::new(program);

    for (name, value) in get_builtins() {
        vm.globals.insert(name, value);
    }
    for (name, func) in get_builtin_functions() {
        vm.builtins.insert(name, func);
    }

    vm.execute().expect("execution failed");
    vm.output_buffer
}

#[test]
fn test_hello_world() {
    assert_eq!(run_cfml(r#"writeOutput("Hello World");"#), "Hello World");
}

#[test]
fn test_arithmetic() {
    assert_eq!(run_cfml("writeOutput(2 + 3);"), "5");
}

#[test]
fn test_array_member_function() {
    assert_eq!(run_cfml("writeOutput([1,2,3].len());"), "3");
}
```

### Running Tests

```bash
cargo test                              # All tests
cargo test -p cfml-compiler             # Compiler crate only
cargo test -p cfml-compiler tag_parser  # Tag parser tests only
cargo test test_hello                   # Tests matching "test_hello"
```

---

## Debugging Tips

### Inspect the Pipeline

Use `-d` to see what happens at each stage:

```bash
cargo run -- -d -c 'var x = [1,2,3]; writeOutput(x.len());'
```

This shows:
1. **TOKENS** — verify the lexer tokenises correctly
2. **AST** — verify the parser builds the right tree
3. **BYTECODE** — verify the compiler emits correct instructions
4. **Output** — the final result

### Debug Tag Conversion

For CFML tag files, debug mode also shows the converted script:

```bash
cargo run -- -d myfile.cfm
```

Look for `=== TAG CONVERSION ===` in the output.

### Common Issues

| Symptom | Likely Cause |
|---------|-------------|
| Empty output | Expression result is `Null` (check variable names are correct) |
| `Parse Error` | Syntax issue — check the line/column in the error |
| `Runtime Error` | Usually a type mismatch or missing function |
| Struct keys return empty | Key might be case-sensitive — CFML is case-insensitive but check spelling |
| Function not found | Check spelling — function lookup is case-insensitive |
| Tag output has `\` chars | Shell escaping issue — test from a `.cfm` file instead of `-c` |

---

## Comprehensive Test Script

Save this as `test_all.cfm` and run with `cargo run -- test_all.cfm`:

```cfml
<cfscript>
writeOutput("=== RustCFML Test Suite ===" & chr(10));
var passed = 0;
var failed = 0;

function assert(label, actual, expected) {
    if (toString(actual) == toString(expected)) {
        passed++;
    } else {
        writeOutput("FAIL: " & label & " | expected: " & expected & " | got: " & actual & chr(10));
        failed++;
    }
}

// --- Arithmetic ---
assert("2 + 3", 2 + 3, 5);
assert("10 - 4", 10 - 4, 6);
assert("3 * 7", 3 * 7, 21);
assert("2 ^ 10", 2 ^ 10, 1024);
assert("17 % 5", 17 % 5, 2);

// --- Strings ---
assert("ucase", ucase("hello"), "HELLO");
assert("lcase", lcase("HELLO"), "hello");
assert("len", len("hello"), 5);
assert("left", left("hello", 3), "hel");
assert("right", right("hello", 3), "llo");
assert("reverse", reverse("hello"), "olleh");
assert("trim", trim("  hi  "), "hi");
assert("find", find("ll", "hello"), 3);
assert("replace", replace("hello", "ll", "r"), "hero");
assert("mid", mid("hello", 2, 3), "ell");
assert("repeatString", repeatString("ab", 3), "ababab");
assert("chr(65)", chr(65), "A");

// --- Member functions ---
assert("str.ucase()", "hello".ucase(), "HELLO");
assert("str.reverse()", "hello".reverse(), "olleh");
assert("str.len()", "hello".len(), 5);
assert("str.left(3)", "hello".left(3), "hel");
assert("str.trim()", "  hi  ".trim(), "hi");

// --- Arrays ---
var arr = [10, 20, 30, 40, 50];
assert("arr[1]", arr[1], 10);
assert("arr[3]", arr[3], 30);
assert("arr.len()", arr.len(), 5);
assert("arr.first()", arr.first(), 10);
assert("arr.last()", arr.last(), 50);
assert("arr.sum()", arr.sum(), 150);
assert("[1,2,3].toList()", [1,2,3].toList(), "1,2,3");

// --- Structs ---
var s = {name: "Alex", age: 30};
assert("s.name", s.name, "Alex");
assert("s.age", s.age, 30);
assert("s.count()", s.count(), 2);

// --- CFML operators ---
assert("5 GT 3", 5 GT 3, true);
assert("5 LT 3", 5 LT 3, false);
assert("5 EQ 5", 5 EQ 5, true);
assert("5 NEQ 3", 5 NEQ 3, true);
assert("contains", "hello" CONTAINS "ell", true);
assert("AND", true AND false, false);
assert("OR", true OR false, true);

// --- Control flow ---
var result = "";
for (var i = 1; i <= 5; i++) { result = result & i; }
assert("for loop", result, "12345");

result = "";
var w = 1;
while (w <= 3) { result = result & w; w++; }
assert("while loop", result, "123");

assert("ternary true", (10 > 5) ? "yes" : "no", "yes");
assert("ternary false", (10 < 5) ? "yes" : "no", "no");

// --- Functions ---
function add(a, b) { return a + b; }
assert("user function", add(3, 4), 7);

function factorial(n) { if (n <= 1) return 1; return n * factorial(n - 1); }
assert("factorial(5)", factorial(5), 120);

// --- Closures ---
var doubled = [1,2,3].map(function(n) { return n * 2; });
assert("array.map", doubled.toList(), "2,4,6");

var evens = [1,2,3,4,5,6].filter(function(n) { return n % 2 == 0; });
assert("array.filter", evens.toList(), "2,4,6");

var total = [1,2,3,4,5].reduce(function(acc, n) { return acc + n; }, 0);
assert("array.reduce", total, 15);

// --- Math ---
assert("abs(-5)", abs(-5), 5);
assert("ceiling(3.2)", ceiling(3.2), 4);
assert("floor(3.8)", floor(3.8), 3);
assert("max(10,20)", max(10, 20), 20);
assert("min(10,20)", min(10, 20), 10);

// --- Type checking ---
assert("isNumeric(42)", isNumeric(42), true);
assert("isNumeric('abc')", isNumeric("abc"), false);
assert("isArray([1,2])", isArray([1,2]), true);
assert("isStruct({a:1})", isStruct({a:1}), true);
assert("isNull(null)", isNull(null), true);

// --- Error handling ---
var caught = false;
try {
    throw("test error");
} catch (any e) {
    caught = true;
}
assert("try/catch", caught, true);

// --- JSON ---
assert("serializeJSON", serializeJSON(42), "42");

// --- Report ---
writeOutput(chr(10) & "Passed: " & passed & chr(10));
writeOutput("Failed: " & failed & chr(10));
if (failed == 0) {
    writeOutput("ALL TESTS PASSED" & chr(10));
} else {
    writeOutput("SOME TESTS FAILED" & chr(10));
}
</cfscript>
```

Run it:

```bash
cargo run -- test_all.cfm
```
