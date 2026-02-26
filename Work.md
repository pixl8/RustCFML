# RustCFML Development Status

## Overview
RustCFML is a CFML interpreter written in Rust, inspired by RustPython's architecture. This document tracks the implementation status of various CFML features.

Compatibility target: [cfdocs.org/functions](https://cfdocs.org/functions) and [cfdocs.org/tags](https://cfdocs.org/tags)

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
- [x] `queryEach()` / `.each()`, `queryMap()` / `.map()`, `queryFilter()` / `.filter()` - query iteration
- [x] `queryReduce()` / `.reduce()`, `querySort()` / `.sort()` - query manipulation
- [x] `querySome()` / `.some()`, `queryEvery()` / `.every()` - query predicates

### Scopes
- [x] Local scope (`var x`, `local.x`)
- [x] Variables scope (`variables.x`)
- [x] Arguments scope (`arguments.name`, `arguments[1]`)
- [x] `request` scope (per-request HashMap)
- [x] `application` scope (Arc<Mutex>, persists in serve mode)
- [x] `server` scope (read-only OS/version info)
- [x] Parent scope inheritance (closures see parent variables)
- [x] Parent scope write-back (closures can mutate parent variables via `closure_parent_writeback`)
- [x] Case-insensitive variable lookup

### CFML Tag Syntax (31 tags handled)
- [x] `<cfset>`, `<cfoutput>`, `<cfif>/<cfelseif>/<cfelse>`, `<cfloop>`
- [x] `<cfscript>`, `<cffunction>`, `<cfargument>`, `<cfreturn>`
- [x] `<cfinclude>`, `<cfdump>`, `<cfthrow>`, `<cftry>/<cfcatch>`
- [x] `<cfabort>`, `<cfparam>`, `<cfcomponent>`, `<cfinterface>`, `<cfproperty>`
- [x] `<cfhttp>`, `<cfquery>`, `<cfqueryparam>`, `<cfheader>`, `<cfcontent>`, `<cflocation>`
- [x] `<cfdirectory>`, `<cfsavecontent>`, `<cfinvoke>`, `<cftransaction>`
- [x] Hash expression evaluation (`#expr#`) in text regions
- [x] CFML comment stripping (`<!--- ... --->`)
- [x] Automatic tag-to-script conversion

### Standard Library (260+ Built-in Functions)

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
- [x] `isNull()`, `isDefined()` (VM bytecode for literals + runtime intercept for dynamic args), `isNumeric()`, `isBoolean()` (accepts numeric strings)
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
- [x] `queryExecute()` — SQLite (rusqlite), MySQL/MariaDB (mysql crate), PostgreSQL (postgres crate), MSSQL/SQL Server (tiberius)
- [x] Positional (`?`) and named (`:param`) parameters for all drivers
- [x] SQLite: in-memory (`:memory:`) and file-based
- [x] MySQL/MariaDB: `mysql://user:pass@host:port/db` datasource URL
- [x] PostgreSQL: `postgres://user:pass@host:port/db` datasource URL (auto-rewrites `:name` → `$N`)
- [x] MSSQL/SQL Server: `mssql://user:pass@host:port/db` or `sqlserver://` datasource URL
- [x] Feature flags: `sqlite`, `mysql_db`, `postgres_db`, `mssql_db`, `all-databases`
- [x] SELECT → CfmlQuery, INSERT/UPDATE/DELETE → struct with recordCount/generatedKey
- [x] `<cfquery>` tag support with datasource attribute
- [x] Connection pooling: r2d2 pools for SQLite/PostgreSQL, cached mysql::Pool for MySQL/MariaDB
- [x] `<cfqueryparam>` — parameterized queries with type coercion (`cfsqltype`), null support, list expansion
- [x] `<cftransaction>` — begin/commit/rollback with auto-rollback on exception, datasource auto-detection
- [x] Structured query params: array of `{value, cfsqltype, null, list, separator}` structs with type coercion

### Components & Interfaces
- [x] `extends` keyword with dot-path resolution (e.g., `extends taffy.core.resource`)
- [x] Recursive parent→child merge with circular inheritance detection
- [x] `super.method()` — calls parent method with child `this` binding
- [x] `isInstanceOf(obj, typeName)` — walks `__extends_chain` + `__implements_chain`, case-insensitive
- [x] `getMetadata(component)` — name, extends, functions, properties, custom metadata
- [x] `createObject("component", "name")` — VM-intercepted dynamic instantiation
- [x] Component/function metadata attributes (`taffy_uri="/path"`, `taffy:mime="text/json"`)
- [x] `interface` keyword + `<cfinterface>` tag — define method contracts
- [x] `implements` keyword — compile-time + runtime validation of interface methods
- [x] Interface inheritance (`interface IChild extends IParent`)
- [x] Multiple interface implementation (`component Foo implements IBar, IBaz`)
- [x] Transitive interface validation (parent interface methods enforced on implementors)

### Component Mappings
- [x] `this.mappings` in Application.cfc — virtual prefix → physical directory
- [x] Longest-prefix-first matching (e.g., `/taffy/core/` before `/taffy/` before `/`)
- [x] Case-insensitive mapping lookup
- [x] Relative mapping paths expanded relative to Application.cfc directory
- [x] Default `/` mapping (CWD or source file's directory) as fallback
- [x] Include path resolution via mappings (for paths starting with `/`)
- [x] Component body `this.xxx = val` statements now compiled and executed (enables Application.cfc config)
- **Limitation**: Mapping values must use struct literal syntax (`this.mappings = {"/taffy": "../Taffy"}`) — bracket assignment (`this.mappings["/taffy"] = "..."`) not yet supported due to nested SetIndex write-back limitation

### String Interpolation
- [x] `#variable#` interpolation in double-quoted strings
- [x] `#expression#` interpolation (e.g., `"2 + 2 = #2 + 2#"`)
- [x] Single-quoted strings remain literal (no interpolation)

### Spread Operator
- [x] Array spread: `[0, ...arr, 3]`
- [x] Struct spread: `{...defaults, key: override}`
- [x] Function argument spread: `func(...args)`

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

### Security Functions
- [x] `encrypt()` / `decrypt()` — AES-128/192/256, DES, DESEDE, Blowfish (CBC+PKCS7), CFMX_COMPAT XOR
- [x] `hmac()` — MD5, SHA-1, SHA-256, SHA-384, SHA-512
- [x] `generateSecretKey()` — AES/DES/DESEDE/Blowfish
- [x] UU/Base64/Hex encoding support

### XML Functions
- [x] `xmlParse()` — event-based parsing via quick-xml → nested structs
- [x] `xmlSearch()` — descendant (`//tag`) and path (`/a/b/c`) queries
- [x] `isXML()`

### Application.cfc Lifecycle
- [x] `Application.cfc` discovery — walks up directory tree
- [x] Lifecycle methods: `onApplicationStart`, `onRequestStart`, `onRequest`, `onRequestEnd`, `onError`
- [x] Function caching across requests in serve mode (ServerState)
- [x] `onMissingMethod` — component fallback handler
- [x] Implicit property accessors (`getXxx()`/`setXxx()` auto-generated)
- [x] `cfsavecontent` — `<cfsavecontent variable="x">body</cfsavecontent>`
- [x] `invoke()` — standalone function: `invoke(component, "method", argStruct)`

### URL Rewrite Engine
- [x] Tuckey-compatible `urlrewrite.xml` parser + rewrite engine
- [x] Regex/wildcard patterns, conditions (method/port/header)
- [x] Forward/redirect/permanent-redirect, rule chaining

### Web Server (Axum)
- [x] `--serve [path]` mode with `--port` option
- [x] HTTP response infrastructure (headers, status, content-type, redirect)
- [x] `cfheader`, `cfcontent`, `cflocation` tag support
- [x] `getHTTPRequestData()` — method, headers, content
- [x] `form` scope (application/x-www-form-urlencoded POST bodies)
- [x] `url` scope (query string parameters)
- [x] `cgi` scope (request metadata)

### System Functions
- [x] `getBaseTemplatePath()`, `getCurrentTemplatePath()`, `getDirectoryFromPath()`
- [x] `getTimeZone()` (TZ env + /etc/localtime)
- [x] `getComponentMetadata()` — VM intercept returning full component metadata

### Closures
- [x] Variable capture (closures retain defining scope)
- [x] Shared mutable state across invocations (via `Arc<RwLock<HashMap>>` shared environment)
- [x] Multiple closures from same scope share variables (accumulator/counter patterns)

### Infrastructure
- [x] CLI with file execution support (64MB thread stack for deep recursion)
- [x] `-c` / `--code` inline code execution
- [x] `-d` / `--debug` debug output (tokens, AST, bytecode)
- [x] `-r` / `--repl` interactive REPL mode
- [x] `--version` version info
- [x] WASM compilation target
- [x] Error handling with line/column info
- [x] Automatic CFML tag detection and conversion
- [x] Soft keywords as identifiers (`local`, `param`, `output`, `required`, etc.)
- [x] Keywords as property names after dot (`obj.default`, `obj.continue`)
- [x] Return type annotations (`private array function foo()`)
- [x] Dotted var declarations (`var local.x = 1`)

---

## 🔴 CRITICAL GAPS (blocking real-world usage)

> Audited against [cfdocs.org/functions](https://cfdocs.org/functions) and [cfdocs.org/tags](https://cfdocs.org/tags) — Feb 2026

### 1. Missing Control-Flow Tags
Tag-based CFML using switch/while/break/continue/finally fails — tag preprocessor doesn't handle them.
- [ ] `<cfswitch>/<cfcase>/<cfdefaultcase>` → convert to cfscript `switch/case/default`
- [ ] `<cfbreak>` → `break;`
- [ ] `<cfcontinue>` → `continue;`
- [ ] `<cfwhile>` → `while (...) { }`  (Lucee extension)
- [ ] `<cffinally>` → `finally { }`
- [ ] `<cfrethrow>` → `rethrow;` (also need cfscript `rethrow` keyword)

### 2. `<cflock>` — Concurrency / Locking
Required for safe shared-scope access in `--serve` mode. Almost every production app uses this.
- [ ] `<cflock name="..." type="exclusive|readonly" timeout="...">`
- [ ] cfscript `lock` block: `lock name="x" type="exclusive" timeout="5" { ... }`
- [ ] Named locks + scope locks (`scope="application"`, `scope="session"`)

### 3. `<cffile>` + File Upload + Multipart Form Parsing
The `<cffile>` tag is extremely common. File upload requires multipart/form-data parsing which doesn't exist.
- [ ] `<cffile action="upload" destination="..." filefield="..." nameconflict="...">`
- [ ] `<cffile action="read|write|append|copy|move|delete|rename">` — wrappers over existing functions
- [ ] `fileUpload(destination, formField, accept, nameConflict)` function
- [ ] `fileUploadAll(destination, accept, nameConflict)` function
- [ ] Multipart/form-data request body parsing in serve mode
- [ ] Upload result struct: `serverFile`, `serverDirectory`, `contentType`, `fileSize`, `clientFile`, `clientFileName`, `clientFileExt`, `timeCreated`
- [ ] `form` scope population from multipart fields (currently only handles url-encoded)

### 4. Session Scope & Authentication
No stateful web applications possible without sessions.
- [ ] `session` scope with configurable timeout (`this.sessionManagement`, `this.sessionTimeout`)
- [ ] Session ID generation + cookie management (`cfid`/`cftoken` or `jsessionid`)
- [ ] `sessionInvalidate()`, `sessionRotate()`
- [ ] `<cflogin>`, `<cfloginuser>`, `<cflogout>` tags
- [ ] `getAuthUser()`, `isUserLoggedIn()`, `isUserInRole()`, `isUserInAnyRole()`
- [ ] `onSessionStart`, `onSessionEnd` lifecycle methods in Application.cfc

---

## 🟠 HIGH PRIORITY GAPS (important for compatibility)

### 7. `<cfcookie>` + Cookie Scope
- [ ] `cookie` scope readable from request headers
- [ ] `<cfcookie name="..." value="..." expires="..." domain="..." path="..." secure="..." httponly="..." samesite="...">`
- [ ] Cookie scope in scope cascade

### 8. `<cfhttpparam>`
cfhttp tag exists but lacks child parameter support — real HTTP calls need headers/formfields/files.
- [ ] `<cfhttpparam type="header|formfield|url|body|file|cookie" name="..." value="...">`

### 9. `<cflog>` / `<cfsetting>` / `<cfsilent>`
- [ ] `<cflog>` tag / `writeLog(text, type, file, log)` function
- [ ] `<cfsetting enablecfoutputonly="..." requesttimeout="..." showdebugoutput="...">`
- [ ] `<cfsilent>` — suppress output in tag body

### 10. Missing String Functions
- [ ] `ucFirst()` — capitalize first letter
- [ ] `jsStringFormat()` — escape for JavaScript strings
- [ ] `reEscape()` — escape regex special characters
- [ ] `getToken()` — get nth token from delimited string
- [ ] `newline()` — platform newline character

### 11. Missing Type/Conversion Functions
- [ ] `createTimeSpan(days, hours, minutes, seconds)` — **used in every Application.cfc for timeouts**
- [ ] `yesNoFormat()` — boolean to "Yes"/"No"
- [ ] `booleanFormat()` — boolean to "true"/"false"
- [ ] `truefalseFormat()` — alias for booleanFormat
- [ ] `nullValue()` — return null
- [ ] `incrementValue()` / `decrementValue()` — add/subtract 1
- [ ] `de()` — delay evaluation (wraps string)
- [ ] `dollarFormat()` — format as currency
- [ ] `setVariable()` / `getVariable()` — dynamic variable access by name string

### 12. Missing Array Functions
- [ ] `arrayPush()` — alias for arrayAppend
- [ ] `arrayUnshift()` — alias for arrayPrepend
- [ ] `arrayIndexExists()` — check if index exists
- [ ] `arrayResize()` — resize array to N elements
- [ ] `arrayMedian()` — median of numeric array
- [ ] `arrayMid()` — sub-array extraction
- [ ] `arrayReduceRight()` — reduce from right
- [ ] `arraySplice()` — remove/insert elements at position
- [ ] `arrayRange()` — generate range array
- [ ] `arrayToStruct()` — convert to struct
- [ ] `arrayDeleteNoCase()` — delete by value, case-insensitive

### 13. Missing Query Functions
- [ ] `queryColumnExists()` — check if column exists
- [ ] `queryRowData()` — get row as struct
- [ ] `querySlice()` — slice rows from query
- [ ] `queryAppend()` — append rows from another query
- [ ] `queryGetResult()` — metadata from last query execution
- [ ] `queryKeyExists()` — check if key exists
- [ ] `queryColumnData()` / `queryColumnArray()` — column as array
- [ ] `queryInsertAt()` — insert row at position
- [ ] `queryPrepend()` — prepend rows
- [ ] `queryReverse()` — reverse row order
- [ ] `queryRowSwap()` — swap two rows
- [ ] `querySetRow()` — set entire row from struct
- [ ] `queryCurrentRow()` — current row in cfoutput loop

### 14. Missing Struct Functions
- [ ] `structToSorted()` — return ordered struct
- [ ] `structIsOrdered()` — check if ordered
- [ ] `structIsCaseSensitive()` — check case sensitivity
- [ ] `structToQueryString()` — convert to URL query string
- [ ] `structGetMetadata()` / `structSetMetadata()` — metadata access

### 15. Missing List Functions
- [ ] `listSome()` / `listEvery()` — higher-order predicates
- [ ] `listAvg()` — average of numeric list
- [ ] `listItemTrim()` — trim whitespace from items
- [ ] `listIndexExists()` — check if index exists
- [ ] `listReduceRight()` — reduce from right

### 16. Application/System Functions
- [ ] `applicationStop()` — stop the application
- [ ] `getApplicationMetadata()` / `getApplicationSettings()` — read app config
- [ ] `getFileFromPath()` — extract filename from path
- [ ] `getCanonicalPath()` — resolve canonical path
- [ ] `writeLog()` — write to log file
- [ ] `systemOutput()` — write to stdout/stderr
- [ ] `setLocale()` / `getLocale()` — locale management
- [ ] `setTimeZone()` — set timezone (have getTimeZone)
- [ ] `trace()` — debug tracing
- [ ] `getTemplatePath()` — alias for getCurrentTemplatePath
- [ ] `throw()` function form (in addition to `throw` keyword)
- [ ] `location()` function form — redirect

### 17. Missing File Functions
- [ ] `fileOpen()` / `fileClose()` / `fileReadLine()` / `fileWriteLine()` — streaming file I/O
- [ ] `fileReadBinary()` — read as binary
- [ ] `fileGetMimeType()` — detect MIME type
- [ ] `fileIsEOF()` — check end of file
- [ ] `fileSetAccessMode()` / `fileSetAttribute()` / `fileSetLastModified()`
- [ ] `directoryRename()` / `directoryCopy()`

---

## 🟡 MEDIUM PRIORITY GAPS (nice to have)

### 18. Date/Time Functions
- [ ] `createTimeSpan()` (arguably CRITICAL — used in Application.cfc)
- [ ] `dateConvert()` — convert between local/UTC
- [ ] `getNumericDate()` — date as numeric value
- [ ] `getHTTPTimeString()` — RFC 1123 date format
- [ ] `millisecond()` — get milliseconds component
- [ ] `nowServer()` — server time

### 19. Locale-Sensitive Functions
- [ ] `lsDateFormat()` / `lsTimeFormat()` / `lsDateTimeFormat()` — locale formatting
- [ ] `lsCurrencyFormat()` / `lsEuroCurrencyFormat()` — currency formatting
- [ ] `lsIsDate()` / `lsIsNumeric()` / `lsIsCurrency()` — locale validation
- [ ] `lsParseCurrency()` / `lsParseDateTime()` — locale parsing
- [ ] `lsNumberFormat()` — locale number formatting
- [ ] `lsWeek()` / `lsDayOfWeek()` — locale week handling

### 20. Encoding/Decoding Functions
- [ ] `binaryDecode()` / `binaryEncode()` — convert between binary and encoded strings
- [ ] `charsetDecode()` / `charsetEncode()` — character set conversions
- [ ] `encodeForHTMLAttribute()` — XSS-safe attribute encoding
- [ ] `encodeForXML()` / `encodeForXMLAttribute()` — XML encoding
- [ ] `encodeFor()` — generic encoder
- [ ] `decodeForHTML()` / `decodeFromURL()` — reverse encoding
- [ ] `urlEncode()` — simpler alias for urlEncodedFormat
- [ ] `canonicalize()` — anti-XSS canonicalization

### 21. Password Hashing & Security
- [ ] `generatePBKDFKey()` — PBKDF2 key derivation
- [ ] `generateBCryptHash()` / `verifyBCryptHash()` — BCrypt
- [ ] `generateSCryptHash()` / `verifySCryptHash()` — SCrypt
- [ ] `generateArgon2Hash()` / `argon2CheckHash()` — Argon2
- [ ] `csrfGenerateToken()` / `csrfVerifyToken()` — CSRF protection

### 22. Error Handling
- [ ] `rethrow` keyword in cfscript (currently only `throw`)
- [ ] `cfcatch.tagContext` — stack trace array in catch blocks
- [ ] `exceptionKeyExists()` — check exception keys

### 23. `<cfmail>` / `<cfmailparam>` / `<cfmailpart>`
- [ ] SMTP client integration for email sending
- [ ] Attachments, HTML/text parts, from/to/cc/bcc

### 24. Caching
- [ ] `<cfcache>` tag
- [ ] `cacheGet()`, `cachePut()`, `cacheDelete()`, `cacheClear()`
- [ ] `cacheKeyExists()`, `cacheCount()`, `cacheGetAll()`, `cacheGetAllIds()`
- [ ] In-memory cache implementation

### 25. `<cfexecute>` — OS Command Execution
- [ ] Execute OS commands with arguments, timeout, output capture

### 26. `<cfstoredproc>` / `<cfprocparam>` / `<cfprocresult>`
- [ ] Stored procedure execution for all database drivers

### 27. Higher-Order Collection Functions
- [ ] `collectionEach()` / `collectionMap()` / `collectionFilter()` / `collectionReduce()` / `collectionEvery()` / `collectionSome()`
- [ ] `stringEach()` / `stringMap()` / `stringFilter()` / `stringReduce()` / `stringSome()` / `stringEvery()` / `stringSort()`
- [ ] `each()` — generic iterator

### 28. Bit Manipulation
- [ ] `bitMaskClear()` / `bitMaskRead()` / `bitMaskSet()`

### 29. XML Construction/Manipulation
- [ ] `xmlNew()` — create new XML document
- [ ] `xmlElemNew()` — create element
- [ ] `xmlChildPos()`, `xmlGetNodeType()`, `xmlHasChild()`
- [ ] `isXMLDoc()`, `isXMLElem()`, `isXMLNode()`, `isXMLRoot()`, `isXMLAttribute()`

### 30. Miscellaneous String
- [ ] `soundex()` / `metaphone()` — phonetic algorithms
- [ ] `htmlParse()` — parse HTML
- [ ] `jsstringFormat()` — JavaScript string escaping
- [ ] `toScript()` — convert to JavaScript variable declaration

---

## 🟢 LOW PRIORITY / OUT OF SCOPE

### Not implementing (niche/legacy/heavy dependencies)
- [ ] Image functions (`imageNew`, `imageRead`, `imageWrite`, etc.) — 80+ functions, needs image library
- [ ] Spreadsheet functions (`spreadsheetNew`, `spreadsheetAddRow`, etc.) — 40+ functions
- [ ] ORM functions (`entityLoad`, `entitySave`, `ormFlush`, etc.) — 20+ functions
- [ ] SOAP/WSDL functions (`addSOAPRequestHeader`, `getSOAPRequest`, etc.)
- [ ] Flash/Flex UI tags (`cfcalendar`, `cfgrid`, `cfslider`, `cfmenu`, etc.)
- [ ] Exchange server integration (`cfexchange*`)
- [ ] PDF manipulation (`cfpdf`, `cfdocument` — needs rendering engine)
- [ ] LDAP (`cfldap`)
- [ ] Registry (`cfregistry`)
- [ ] `.NET` integration (`dotNetToCFType`)
- [ ] Gateway functions (`sendGatewayMessage`, `getGatewayHelper`)
- [ ] K2 server functions

### Low Priority but potentially useful
- [ ] JWT functions: `createSignedJWT()`, `verifySignedJWT()`, `createEncryptedJWT()`, `verifyEncryptedJWT()`
- [ ] `<cfzip>` / `<cfzipparam>` — zip/compress
- [ ] `<cfschedule>` — scheduled tasks
- [ ] `<cfthread>` / `threadNew()` / `threadJoin()` / `threadTerminate()` — threading
- [ ] `<cfwddx>` / `isWDDX()` — legacy serialization
- [ ] `callStackGet()` / `callStackDump()` — programmatic stack trace
- [ ] `precisionEvaluate()` — BigDecimal math
- [ ] `getProfileString()` / `setProfileString()` / `getProfileSections()` — INI file manipulation
- [ ] `valuelist()` / `quotedValueList()` — query column to delimited list
- [ ] `getMemoryUsage()` / `getCPUUsage()` / `getFreespace()` / `getTotalSpace()` — system monitoring

---

## 📋 Taffy Framework Support

Phases 1–6 complete. Remaining items:

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
│   ├── cfml-stdlib/     # Built-in functions (260+)
│   ├── cli/             # Command-line interface
│   └── wasm/            # WebAssembly target
```

### Reference Resources
- BoxLang ANTLR Grammar: `BoxLang/src/main/antlr/CFGrammar.g4`
- Lucee Expression Grammar: `Lucee/core/src/main/java/lucee/transformer/cfml/expression/`
- RustPython: `/Users/alexskinner/Repos/opensource/cfml_rust/RustPython/`
- cfdocs.org: [functions](https://cfdocs.org/functions) | [tags](https://cfdocs.org/tags)

---

*Last Updated: 2026-02-25 — Database improvements: connection pooling, cfqueryparam, cftransaction, MSSQL driver*
