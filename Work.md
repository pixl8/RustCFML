# RustCFML Development Status

## Overview
RustCFML is a CFML interpreter written in Rust, inspired by RustPython's architecture. This document tracks the implementation status of various CFML features.

---

## ✅ COMPLETED / WORKING

### Compiler Pipeline
- [x] Lexer - full tokenization of CFML source (~50 token types)
- [x] Parser - recursive descent parser with proper operator precedence
- [x] Compiler - bytecode generation from AST (30+ opcodes)
- [x] VM - stack-based bytecode execution engine
- [x] CFML Tag Preprocessor - converts tag syntax to CFScript before parsing

### Data Types (CfmlValue)
- [x] Null
- [x] Boolean (true/false/yes/no)
- [x] Integer (i64)
- [x] Double (f64)
- [x] String (with CFML case-insensitive comparison)
- [x] Array (1-based indexing)
- [x] Struct (case-insensitive key lookup)
- [x] Function reference
- [x] Closure
- [x] Component
- [x] Query
- [x] Binary

### Variables & Assignment
- [x] Variable assignment (`x = 5`, `x = 1 + 2`)
- [x] Variable declaration (`var x = 5`)
- [x] Compound assignments (`+=`, `-=`, `*=`, `/=`, `&=`)
- [x] Postfix increment/decrement (`i++`, `i--`)

### Arithmetic Operators
- [x] `+`, `-`, `*`, `/`, `%` (modulus)
- [x] `^` (power)
- [x] `\` (integer division)
- [x] `-` (unary negation)

### Comparison Operators
- [x] `==`, `!=`, `<`, `<=`, `>`, `>=`
- [x] CFML keyword operators: `EQ`, `NEQ`, `GT`, `GTE`/`GE`, `LT`, `LTE`/`LE`
- [x] `IS` (identity)

### Logical Operators
- [x] `&&` / `AND`
- [x] `||` / `OR`
- [x] `!` / `NOT`
- [x] `XOR`
- [x] `EQV` (equivalence)
- [x] `IMP` (implication)

### String Operators
- [x] `&` (concatenation)
- [x] `CONTAINS` / `DOES NOT CONTAIN`

### Control Flow
- [x] `if`/`else if`/`elseif`/`else` statements
- [x] Ternary operator (`condition ? then : else`)
- [x] `for` loops (C-style: `for (var i = 1; i <= 10; i++)`)
- [x] `for-in` loops (`for (var item in collection)`) — arrays and structs
- [x] `while` loops
- [x] `do/while` loops
- [x] `switch/case/default` statements
- [x] `break` and `continue` (with proper loop context)
- [x] `try/catch/finally` exception handling
- [x] `throw` with message

### Functions
- [x] User-defined functions (`function name(params) { ... }`)
- [x] Function parameters with proper binding
- [x] Return statements with values
- [x] Recursive function calls
- [x] Access modifiers (public/private/remote/package)
- [x] Closures (`function(params) { ... }`)
- [x] Arrow functions (`(params) => expression`)
- [x] Case-insensitive function lookup

### Member Functions
- [x] String member functions: `.len()`, `.ucase()`, `.lcase()`, `.trim()`, `.reverse()`, `.left()`, `.right()`, `.mid()`, `.find()`, `.replace()`, `.contains()`, `.split()`, `.toJSON()`, `.val()`, etc.
- [x] Array member functions: `.len()`, `.append()`, `.prepend()`, `.first()`, `.last()`, `.isEmpty()`, `.sort()`, `.reverse()`, `.slice()`, `.toList()`, `.sum()`, `.avg()`, `.min()`, `.max()`, `.contains()`, `.find()`, `.toJSON()`, etc.
- [x] Struct member functions: `.count()`, `.keyList()`, `.keyArray()`, `.keyExists()`, `.isEmpty()`, `.delete()`, `.insert()`, `.copy()`, `.toJSON()`, etc.
- [x] Number member functions: `.toString()`, `.abs()`, `.ceiling()`, `.floor()`, `.round()`
- [x] Method chaining (`"hello".ucase().reverse()`)

### Higher-Order Functions (with closure support)
- [x] `arrayMap()` / `.map()` - transform array elements
- [x] `arrayFilter()` / `.filter()` - filter array elements
- [x] `arrayReduce()` / `.reduce()` - reduce array to value
- [x] `arrayEach()` / `.each()` - iterate array elements
- [x] `structMap()` / `.map()` - transform struct values
- [x] `structFilter()` / `.filter()` - filter struct entries
- [x] `structEach()` / `.each()` - iterate struct entries

### Scopes
- [x] Local scope (`var x`, `local.x`)
- [x] Variables scope (`variables.x`)
- [x] Arguments scope (`arguments.name`, `arguments[1]`)
- [x] Parent scope inheritance (closures see parent variables)
- [x] Case-insensitive variable lookup

### CFML Tag Syntax
- [x] `<cfset variable = value>` - variable assignment
- [x] `<cfoutput>#expression#</cfoutput>` - output with expression interpolation
- [x] `<cfif condition>...<cfelseif>...<cfelse>...</cfif>` - conditionals
- [x] `<cfloop>` - loops (from/to/index, condition, array, list, collection)
- [x] `<cfscript>...</cfscript>` - embedded script blocks
- [x] `<cffunction name="..." ...>...</cffunction>` - function definition
- [x] `<cfargument name="..." default="...">` - function parameters
- [x] `<cfreturn expression>` - return values
- [x] `<cfinclude template="path">` - file inclusion
- [x] `<cfdump var="#expression#">` - debug dump
- [x] `<cfthrow message="...">` - throw exception
- [x] `<cftry>...<cfcatch>...</cfcatch></cftry>` - error handling
- [x] `<cfabort>` - abort processing
- [x] `<cfparam name="..." default="...">` - parameter defaults
- [x] `<cfcomponent>...</cfcomponent>` - component definition
- [x] `<cfproperty name="..." ...>` - property definition
- [x] Hash expression evaluation (`#expr#`) in text regions
- [x] Automatic tag-to-script conversion

### Standard Library (200+ Built-in Functions)

**String Functions (35+)**
- [x] `len()`, `ucase()`, `lcase()`, `trim()`, `ltrim()`, `rtrim()`
- [x] `find()`, `findNoCase()`, `findOneOf()`, `mid()`, `left()`, `right()`
- [x] `replace()`, `replaceNoCase()`, `reverse()`, `repeatString()`
- [x] `insert()`, `removeChars()`, `spanIncluding()`, `spanExcluding()`
- [x] `compare()`, `compareNoCase()`, `asc()`, `chr()`
- [x] `reFind()`, `reFindNoCase()`, `reReplace()`, `reReplaceNoCase()`, `reMatch()`, `reMatchNoCase()` (via `regex` crate)
- [x] `wrap()`, `stripCr()`, `toBase64()`, `toBinary()`
- [x] `urlEncodedFormat()`, `urlDecode()`
- [x] `htmlEditFormat()`, `htmlCodeFormat()`
- [x] `lJustify()`, `rJustify()`
- [x] `numberFormat()`, `decimalFormat()`
- [x] `formatBaseN()`, `inputBaseN()`

**Array Functions (25+)**
- [x] `arrayNew()`, `arrayLen()`, `arrayAppend()`, `arrayPrepend()`
- [x] `arrayDeleteAt()`, `arrayInsertAt()`, `arraySet()`, `arraySwap()`
- [x] `arrayContains()`, `arrayContainsNoCase()`
- [x] `arrayFind()`, `arrayFindNoCase()`
- [x] `arraySort()`, `arrayReverse()`, `arraySlice()`
- [x] `arrayToList()`, `arrayMerge()`, `arrayClear()`
- [x] `arrayIsDefined()`, `arrayMin()`, `arrayMax()`, `arrayAvg()`, `arraySum()`
- [x] `arrayMap()`, `arrayFilter()`, `arrayReduce()`, `arrayEach()`

**Struct Functions (20+)**
- [x] `structNew()`, `structCount()`, `structKeyExists()`, `structKeyList()`
- [x] `structKeyArray()`, `structDelete()`, `structInsert()`, `structUpdate()`
- [x] `structFind()`, `structClear()`, `structCopy()`, `structAppend()`
- [x] `structIsEmpty()`, `structSort()`
- [x] `structEach()`, `structMap()`, `structFilter()`

**Math Functions (25+)**
- [x] `abs()`, `ceiling()`, `floor()`, `round()`, `int()`, `fix()`
- [x] `max()`, `min()`, `sgn()`, `sqr()`
- [x] `exp()`, `log()`, `log10()`
- [x] `sin()`, `cos()`, `tan()`, `asin()`, `acos()`, `atan()`
- [x] `pi()`, `rand()`, `randRange()`, `randomize()`
- [x] `bitAnd()`, `bitOr()`, `bitXor()`, `bitNot()`, `bitSHLN()`, `bitSHRN()`

**Date/Time Functions (25+)**
- [x] `now()`, `createDate()`, `createDateTime()`, `createODBCDate()`, `createODBCDateTime()`
- [x] `year()`, `month()`, `day()`, `hour()`, `minute()`, `second()`
- [x] `dayOfWeek()`, `dayOfYear()`, `daysInMonth()`, `daysInYear()`
- [x] `quarter()`, `firstDayOfMonth()`, `monthAsString()`, `dayOfWeekAsString()`
- [x] `dateAdd()`, `dateDiff()`, `dateFormat()`, `timeFormat()`
- [x] `getTickCount()`

**List Functions (23)**
- [x] `listLen()`, `listFirst()`, `listLast()`, `listRest()`
- [x] `listGetAt()`, `listSetAt()`, `listDeleteAt()`, `listInsertAt()`
- [x] `listFind()`, `listFindNoCase()`, `listContains()`, `listContainsNoCase()`
- [x] `listAppend()`, `listPrepend()`, `listSort()`, `listRemoveDuplicates()`
- [x] `listToArray()`, `listValueCount()`, `listValueCountNoCase()`
- [x] `listQualify()`, `listChangeDelims()`, `listEach()`

**Type Checking Functions (12)**
- [x] `isNull()`, `isDefined()`, `isNumeric()`, `isBoolean()`
- [x] `isDate()`, `isArray()`, `isStruct()`, `isQuery()`
- [x] `isSimpleValue()`, `isBinary()`, `isValid()`, `isCustomFunction()`

**Conversion Functions (6)**
- [x] `toString()`, `toNumeric()`, `toBoolean()`, `val()`
- [x] `javacast()`, `parseNumber()`

**JSON Functions (3)**
- [x] `serializeJSON()`, `deserializeJSON()`, `isJSON()`

**Query Functions (4)**
- [x] `queryNew()`, `queryAddRow()`, `querySetCell()`, `queryAddColumn()`

**Utility Functions (7+)**
- [x] `writeOutput()`, `writeDump()`, `dump()`
- [x] `sleep()`, `getTickCount()`
- [x] `duplicate()`, `hash()`

### String Interpolation
- [x] `#variable#` interpolation in double-quoted strings
- [x] `#expression#` interpolation (e.g., `"2 + 2 = #2 + 2#"`)
- [x] Single-quoted strings remain literal (no interpolation)

### Elvis Operator & Null-Safe Navigation
- [x] Elvis operator `?:` — null coalescing (`value ?: "default"`)
- [x] Null-safe property access `?.` — returns null instead of erroring
- [x] Chained null-safe (`obj?.a?.b?.c`)

### Include Support
- [x] `include "file.cfm"` — executes in current scope
- [x] Path resolution relative to source file or CWD
- [x] Tag preprocessor converts `<cfinclude template="path">`

### File I/O
- [x] `fileRead()`, `fileWrite()`, `fileAppend()`
- [x] `fileExists()`, `fileDelete()`, `fileMove()`, `fileCopy()`
- [x] `directoryCreate()`, `directoryExists()`, `directoryDelete()`
- [x] `directoryList()` with recurse and filter
- [x] `getTempDirectory()`, `getTempFile()`
- [x] `getFileInfo()` — returns struct with name, size, lastModified, type, etc.
- [x] `expandPath()`

### Hashing & Validation
- [x] `hash(input, algorithm)` — MD5, SHA-256, SHA-384, SHA-512 (via `md-5` and `sha2` crates)
- [x] `createUUID()` — UUID v4 generation
- [x] `isValid(type, value)` — email, url, integer, numeric, date, uuid, regex, creditcard, boolean
- [x] `encodeForHTML()`, `encodeForURL()`, `encodeForCSS()`, `encodeForJavaScript()`
- [x] `arrayPop()`, `arrayShift()`

### Infrastructure
- [x] CLI with file execution support
- [x] `-c` / `--code` inline code execution
- [x] `-d` / `--debug` debug output (tokens, AST, bytecode)
- [x] `-r` / `--repl` interactive REPL mode
- [x] `--version` version info
- [x] WASM compilation target
- [x] Error handling with line/column info
- [x] Automatic CFML tag detection and conversion

---

## ⚠️ PARTIALLY IMPLEMENTED / NEEDS WORK

### Components (.cfc)
- [x] Component AST definition
- [x] Property AST
- [x] Component parsing
- [x] Component compilation (struct-based)
- [x] Component instantiation (`new Component()`)
- [x] Init/constructor pattern with args
- [x] `this` scope within components
- [x] .cfc file loading (`new Greeter()` loads `Greeter.cfc`)
- [ ] Inheritance (`extends`)
- [ ] Interfaces (`implements`)
- [ ] Property getters/setters

### Closures
- [x] Closure definition and invocation
- [x] Parent scope read access
- [ ] Parent scope write access (closures get copy, can't mutate parent)

---

## ❌ NOT IMPLEMENTED

### Advanced Features
- [ ] Spread operator
- [ ] Custom tag support
- [ ] Layouts and views
- [ ] ORM support
- [ ] Web service support (WSDL, REST)
- [x] Database connectivity (`queryExecute`) — SQLite (rusqlite), MySQL (mysql crate), PostgreSQL (postgres crate)
  - Positional (`?`) and named (`:param`) parameters for all drivers
  - SQLite: in-memory (`:memory:`) and file-based
  - MySQL: `mysql://user:pass@host:port/db` datasource URL
  - PostgreSQL: `postgres://user:pass@host:port/db` datasource URL (auto-rewrites `:name` → `$N`)
  - Feature flags: `sqlite`, `mysql_db`, `postgres_db`, `all-databases`
  - SELECT → CfmlQuery, INSERT/UPDATE/DELETE → struct with recordCount/generatedKey
  - `<cfquery>` tag support with datasource attribute
- [ ] Session/application/server scopes
- [ ] Threading (`cfthread`)
- [x] HTTP client (`cfhttp`) — GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS, headers, body, timeout, response struct
- [ ] Mail (`cfmail`)
- [ ] Scheduled tasks
- [ ] Custom tags / tag libraries

### Runtime
- [ ] Proper call stack frames (currently flat)
- [ ] Stack traces on error
- [ ] Source maps for tag→script errors
- [ ] Hot reload
- [ ] JIT compilation (future)

### Standard Library (Missing)
- [ ] Security: `encrypt()`, `decrypt()`, `generateSecretKey()`, `hmac()` (beyond basic `hash()`)
- [ ] XML: `xmlParse()`, `xmlSearch()`, `xmlTransform()`, `xmlValidate()`
- [ ] HTTP: `httpService`, `getHttpRequestData()`, `getHttpTimeString()`
- [ ] Image: `imageNew()`, `imageRead()`, `imageWrite()`, etc.
- [ ] Spreadsheet: `spreadsheetNew()`, `spreadsheetAddRow()`, etc.
- [ ] System: `getCurrentTemplatePath()`, `getBaseTemplatePath()`, `getTimeZone()`

---

## Architecture

### Execution Pipeline
```
CFML Source Code (tags or script)
    ↓
Tag Preprocessor (tag_parser.rs) → CFScript [if needed]
    ↓
Lexer (lexer.rs) → Tokens
    ↓
Parser (parser.rs) → AST (ast.rs)
    ↓
Compiler (compiler.rs) → Bytecode (BytecodeProgram)
    ↓
VM (lib.rs) → Execution with output buffer
```

### Crate Structure
```
RustCFML/
├── crates/
│   ├── cfml-common/     # Shared types (CfmlValue, CfmlError, Position)
│   ├── cfml-compiler/   # Lexer, Parser, AST, Tag Preprocessor
│   ├── cfml-codegen/    # Bytecode compiler (AST → BytecodeOp)
│   ├── cfml-vm/         # Virtual machine (stack-based bytecode execution)
│   ├── cfml-stdlib/     # Built-in functions (200+)
│   ├── cli/             # Command-line interface
│   └── wasm/            # WebAssembly target
```

### Reference Resources
- BoxLang ANTLR Grammar: `BoxLang/src/main/antlr/CFGrammar.g4`
- Lucee Expression Grammar: `Lucee/core/src/main/java/lucee/transformer/cfml/expression/`
- RustPython: `/Users/alexskinner/Repos/opensource/cfml_rust/RustPython/`

---

*Last Updated: 2026-02-21*
