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
- [x] `arraySome()` / `.some()` - true if any element matches
- [x] `arrayEvery()` / `.every()` - true if all elements match
- [x] `structMap()` / `.map()` - transform struct values
- [x] `structFilter()` / `.filter()` - filter struct entries
- [x] `structEach()` / `.each()` - iterate struct entries
- [x] `structReduce()` / `.reduce()` - reduce struct to value
- [x] `structSome()` / `.some()` - true if any entry matches
- [x] `structEvery()` / `.every()` - true if all entries match
- [x] `listMap()`, `listFilter()`, `listEach()`, `listReduce()` - list higher-order functions

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

### Standard Library (250+ Built-in Functions)

**String Functions (45+)**
- [x] `len()`, `ucase()`, `lcase()`, `trim()`, `ltrim()`, `rtrim()`
- [x] `find()`, `findNoCase()`, `findOneOf()` (with start param), `mid()`, `left()`, `right()`
- [x] `replace()`, `replaceNoCase()` (both support scope="all"), `reverse()`, `repeatString()`
- [x] `replaceList()`, `replaceListNoCase()` — list-based find/replace
- [x] `insert()`, `removeChars()`, `spanIncluding()`, `spanExcluding()`
- [x] `compare()`, `compareNoCase()`, `asc()`, `chr()`
- [x] `reFind()`, `reFindNoCase()`, `reReplace()`, `reReplaceNoCase()`, `reMatch()`, `reMatchNoCase()` (via `regex` crate)
- [x] `wrap()` (with strip param), `stripCr()`, `toBase64()`, `toBinary()` (base64 decode)
- [x] `urlEncodedFormat()` (%20 for spaces), `urlDecode()` (UTF-8 aware)
- [x] `htmlEditFormat()`, `htmlCodeFormat()`, `encodeForHTML()` (separate, encodes `'` and `/`)
- [x] `xmlFormat()`, `paragraphFormat()`
- [x] `lJustify()`, `rJustify()`, `cJustify()`
- [x] `numberFormat()` (mask engine: `9`, `0`, `,`, `$`, `()`, `+/-`), `decimalFormat()` (thousands separator)
- [x] `formatBaseN()` (radix 2-36), `inputBaseN()`

**Array Functions (35+)**
- [x] `arrayNew()`, `arrayLen()`, `arrayAppend()`, `arrayPrepend()`
- [x] `arrayDeleteAt()`, `arrayInsertAt()`, `arraySet()`, `arraySwap()`
- [x] `arrayContains()`, `arrayContainsNoCase()`
- [x] `arrayFind()`, `arrayFindNoCase()`, `arrayFindAll()`, `arrayFindAllNoCase()`
- [x] `arraySort()` (sortOrder, textnocase), `arrayReverse()`, `arraySlice()` (negative offset)
- [x] `arrayToList()`, `arrayMerge()` (leaveIndex param), `arrayClear()`
- [x] `arrayIsDefined()`, `arrayMin()`, `arrayMax()`, `arrayAvg()`, `arraySum()`
- [x] `arrayFirst()`, `arrayLast()`, `arrayIsEmpty()`, `arrayDelete()` (by value)
- [x] `arrayPop()`, `arrayShift()`
- [x] `arrayMap()`, `arrayFilter()`, `arrayReduce()`, `arrayEach()`, `arraySome()`, `arrayEvery()`

**Struct Functions (25+)**
- [x] `structNew()`, `structCount()`, `structKeyExists()` (CI), `structKeyList()`
- [x] `structKeyArray()`, `structDelete()` (CI), `structInsert()` (allowoverwrite), `structUpdate()`
- [x] `structFind()` (CI), `structFindKey()` (recursive), `structFindValue()` (recursive)
- [x] `structClear()`, `structCopy()`, `structAppend()` (overwriteFlag)
- [x] `structIsEmpty()`, `structSort()` (sortType, sortOrder)
- [x] `structGet()`, `structValueArray()`, `structEquals()`, `isEmpty()`
- [x] `structEach()`, `structMap()`, `structFilter()`, `structReduce()`, `structSome()`, `structEvery()`

**Math Functions (25+)**
- [x] `abs()`, `ceiling()`, `floor()`, `round()`, `int()` (uses floor), `fix()`
- [x] `max()`, `min()`, `sgn()`, `sqr()`
- [x] `exp()`, `log()`, `log10()`
- [x] `sin()`, `cos()`, `tan()`, `asin()`, `acos()`, `atan()`
- [x] `pi()`, `rand()`, `randRange()` (returns Int), `randomize()`
- [x] `bitAnd()`, `bitOr()`, `bitXor()`, `bitNot()` (32-bit), `bitSHLN()`, `bitSHRN()`

**Date/Time Functions (34)** — fully implemented with chrono, `parse_cfml_date()` central parser
- [x] `now()`, `createDate()`, `createDateTime()`, `createTime()`
- [x] `createODBCDate()`, `createODBCDateTime()`, `createODBCTime()`
- [x] `year()`, `month()`, `day()`, `hour()`, `minute()`, `second()`
- [x] `dayOfWeek()`, `dayOfYear()`, `daysInMonth()`, `daysInYear()`
- [x] `quarter()`, `firstDayOfMonth()`, `week()`, `isLeapYear()`
- [x] `monthAsString()`, `monthShortAsString()`, `dayOfWeekAsString()`, `dayOfWeekShortAsString()`
- [x] `dateAdd()`, `dateDiff()`, `dateFormat()`, `timeFormat()`, `dateTimeFormat()`
- [x] `parseDateTime()`, `datePart()`, `dateCompare()`
- [x] `getTickCount()`

**List Functions (27+)** — all use `cfml_list_split()` (multi-char delimiters, empty element filtering)
- [x] `listLen()`, `listFirst()`, `listLast()`, `listRest()`
- [x] `listGetAt()`, `listSetAt()`, `listDeleteAt()`, `listInsertAt()`
- [x] `listFind()`, `listFindNoCase()`, `listContains()`, `listContainsNoCase()`
- [x] `listAppend()`, `listPrepend()`, `listSort()` (sortType, sortOrder, delimiter), `listRemoveDuplicates()` (ignoreCase)
- [x] `listToArray()` (includeEmptyValues), `listValueCount()`, `listValueCountNoCase()`
- [x] `listQualify()`, `listChangeDelims()`, `listCompact()`
- [x] `listEach()`, `listMap()`, `listFilter()`, `listReduce()` — higher-order (VM-level closure dispatch)

**Type Checking Functions (12)**
- [x] `isNull()`, `isDefined()` (stub — needs VM bytecode), `isNumeric()`, `isBoolean()` (accepts numeric strings)
- [x] `isDate()`, `isArray()`, `isStruct()`, `isQuery()`
- [x] `isSimpleValue()` (excludes Null), `isBinary()`, `isValid()` (see Hashing & Validation), `isCustomFunction()`

**Conversion Functions (6)**
- [x] `toString()` (handles Binary→String), `toNumeric()` (errors on invalid), `toBoolean()`, `val()` (handles leading `+`)
- [x] `javacast()`, `parseNumber()`

**JSON Functions (3)** — powered by `serde_json`
- [x] `serializeJSON()` (handles Query as array-of-structs), `deserializeJSON()` (full recursive), `isJSON()`

**Query Functions (11)**
- [x] `queryNew()`, `queryAddRow()` (struct/array data), `querySetCell()`, `queryAddColumn()` (array values)
- [x] `queryGetRow()`, `queryGetCell()`, `queryRecordCount()`, `queryColumnCount()`, `queryColumnList()`
- [x] `queryDeleteRow()`, `queryDeleteColumn()`

**Utility Functions (7+)**
- [x] `writeOutput()`, `writeDump()`, `dump()`
- [x] `sleep()`, `getTickCount()`
- [x] `duplicate()`, `hash()`

### HTTP Client
- [x] `cfhttp` tag and function — GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS
- [x] Request headers, body, timeout
- [x] Response struct (statusCode, fileContent, headers)
- [x] Via `ureq` v2 (sync)

### Database Connectivity
- [x] `queryExecute()` — SQLite (rusqlite), MySQL (mysql crate), PostgreSQL (postgres crate)
- [x] Positional (`?`) and named (`:param`) parameters for all drivers
- [x] SQLite: in-memory (`:memory:`) and file-based
- [x] MySQL: `mysql://user:pass@host:port/db` datasource URL
- [x] PostgreSQL: `postgres://user:pass@host:port/db` datasource URL (auto-rewrites `:name` → `$N`)
- [x] Feature flags: `sqlite`, `mysql_db`, `postgres_db`, `all-databases`
- [x] SELECT → CfmlQuery, INSERT/UPDATE/DELETE → struct with recordCount/generatedKey
- [x] `<cfquery>` tag support with datasource attribute

### Component Inheritance
- [x] `extends` keyword with dot-path resolution (e.g., `extends taffy.core.resource`)
- [x] Recursive parent→child merge with circular inheritance detection
- [x] `super.method()` — calls parent method with child `this` binding
- [x] `isInstanceOf(obj, typeName)` — walks `__extends_chain`, case-insensitive
- [x] `getMetadata(component)` — name, extends, functions, properties, custom metadata
- [x] `createObject("component", "name")` — VM-intercepted dynamic instantiation
- [x] Component/function metadata attributes (`taffy_uri="/path"`, `taffy:mime="text/json"`)

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
- [x] `hash(input, algorithm)` — MD5, SHA-1, SHA-256, SHA-384, SHA-512 (via `md-5`, `sha1`, and `sha2` crates)
- [x] `createUUID()` — CFML format (8-4-4-16)
- [x] `createGUID()` — standard format (8-4-4-4-12)
- [x] `isValid(type, value)` — email, url, integer, numeric, date, uuid (CFML format), guid, range, regex, creditcard, boolean, zipcode, phone, ssn
- [x] `encodeForHTML()`, `encodeForURL()`, `encodeForCSS()`, `encodeForJavaScript()`

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
- [x] Component AST, parsing, compilation (struct-based), instantiation (`new Component()`)
- [x] Init/constructor, `this` scope, .cfc file loading
- [x] Inheritance, super, metadata, createObject (see Component Inheritance section above)
- [ ] Interfaces (`implements`)
- [ ] Property getters/setters (implicit accessors)

### Closures
- [x] Closure definition and invocation
- [x] Parent scope read access
- [ ] Parent scope write access (closures get copy, can't mutate parent)

---

## ❌ NOT IMPLEMENTED

### Language Features
- [ ] Spread operator
- [ ] Custom tag support / tag libraries
- [ ] Layouts and views
- [ ] ORM support
- [ ] Application/session/server scopes
- [ ] Threading (`cfthread`)
- [ ] Mail (`cfmail`)
- [ ] Scheduled tasks

### Runtime
- [ ] Proper call stack frames (currently flat)
- [ ] Stack traces on error
- [ ] Source maps for tag→script errors
- [ ] Hot reload
- [ ] JIT compilation (future)

### Standard Library (Missing)
- [ ] Security: `encrypt()`, `decrypt()`, `generateSecretKey()`, `hmac()`
- [ ] XML: `xmlParse()`, `xmlSearch()`, `xmlTransform()`, `xmlValidate()`
- [ ] HTTP: `getHttpRequestData()`, `getHttpTimeString()`
- [ ] Image: `imageNew()`, `imageRead()`, `imageWrite()`, etc.
- [ ] Spreadsheet: `spreadsheetNew()`, `spreadsheetAddRow()`, etc.
- [ ] System: `getCurrentTemplatePath()`, `getBaseTemplatePath()`, `getTimeZone()`

---

## 📋 TODO: Taffy Framework Support

The following features are needed to run the [Taffy REST framework](https://github.com/atuttle/Taffy) on RustCFML. Ordered by implementation priority.

### Phase 1: Quick Wins
- [x] `cfabort`/`abort` statement — `Statement::Exit` → `BytecodeOp::Halt` ✅
- [ ] `getCurrentTemplatePath()` — return path of currently executing file
- [ ] `getDirectoryFromPath()` — extract directory from file path
- [ ] `getComponentMetadata("dot.path")` — introspect component without instantiation

### Phase 2: Scopes & Lifecycle
- [ ] `application` scope — persistent shared state across requests (struct in VM, keyed by app name)
- [ ] `request` scope — per-request shared state (cleared each request)
- [ ] `Application.cfc` lifecycle — `onApplicationStart()`, `onRequestStart()`, `onRequest()`, `onError()`
- [ ] Scope cascading: `variables` → `local` → `arguments` → `application` → `request`

### Phase 3: HTTP Infrastructure
- [ ] Embedded HTTP server (e.g., `hyper` or `actix-web`) — listen, route, serve
- [ ] `getHTTPRequestData()` — returns struct with `method`, `headers`, `content`, `protocol`
- [ ] `cfheader` tag / `header()` function — set HTTP response headers
- [ ] `cfcontent` tag — set response content type and body
- [ ] URL/form scope population from HTTP requests
- [ ] REST-style URL path parsing (`/users/{id}` → `url.id`)

### Phase 4: Dynamic Invocation
- [ ] `cfinvoke` tag — invoke component method by name with `argumentcollection` and `returnvariable`
- [ ] `invoke(obj, "methodName", args)` — dynamic method invocation function
- [ ] `argumentCollection` support — pass struct as named arguments to any function call

### Phase 5: Component Enhancements
- [ ] `onMissingMethod(missingMethodName, missingMethodArguments)` — fallback handler
- [ ] Implicit property accessors (getters/setters from `<cfproperty>`)
- [ ] `cfdirectory` tag — list/create/rename/delete directories (Taffy uses for resource discovery)
- [ ] `cfsavecontent variable="x">...</cfsavecontent>` — capture output to variable

### Phase 6: Utility Functions
- [ ] `listFirst()` with multi-char delimiter support (verify existing impl)
- [ ] `reReplaceNoCase()` improvements for Taffy's URI pattern matching
- [ ] `serializeJSON()` — ensure proper Query-to-JSON serialization matches Taffy expectations
- [ ] `structKeyTranslate()` — convert struct keys to specified case
- [ ] `getMetadata()` — ensure function parameter metadata (hint, type, required) is preserved

### Stretch Goals (Not Required for Basic Taffy)
- [ ] `cfthread` — async task execution
- [ ] `cflock` — named/scoped locking for thread safety
- [ ] `cfcache` — response caching
- [ ] Rate limiting middleware support
- [ ] CORS header management

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

*Last Updated: 2026-02-22*
