# RustCFML Development Status

Compatibility target: [cfdocs.org/functions](https://cfdocs.org/functions) and [cfdocs.org/tags](https://cfdocs.org/tags)

---

## What's Implemented

### Core Language
Full CFScript support: variables, operators (arithmetic, comparison, logical, string, bitwise), control flow (if/else, for, for-in, while, do-while, switch/case, break/continue, try/catch/finally, throw/rethrow), functions (UDFs, closures, arrow functions, recursion, access modifiers), string interpolation (`#expr#`), spread operator, elvis operator (`?:`), null coalescing (`??`), null-safe navigation (`?.`). `throw()` function form with named args.

### Data Types
Null, Boolean, Integer (i64), Double (f64), String (CI comparison), Array (1-based), Struct (CI keys), Function, Closure, Component, Query, Binary.

### Tag Syntax (50+ tags)
Tag preprocessor converts CFML tags to CFScript. Supports: cfset, cfoutput, cfif/cfelseif/cfelse, cfloop, cfscript, cffunction, cfargument, cfreturn, cfinclude, cfdump, cfthrow, cftry/cfcatch/cffinally, cfabort, cfparam, cfcomponent, cfinterface, cfproperty, cfhttp, cfhttpparam, cfquery, cfqueryparam, cfheader, cfcontent, cflocation, cfdirectory, cfsavecontent, cfinvoke, cftransaction, cfswitch/cfcase/cfdefaultcase, cfbreak, cfcontinue, cfwhile, cfrethrow, cflock, cfsilent, cflog, cfsetting, cfcookie, cffile, cfloginuser, cflogout, cfmodule, cf_ prefix custom tags, cfmail/cfmailparam/cfmailpart, cfcache, cfexecute, cfstoredproc/cfprocparam/cfprocresult, cfimport (taglib with .tld support), cfthread, cfzip. CFScript `lock` block syntax supported. Custom tags support self-closing and body modes, caller write-back, thisTag scope, this.customTagPaths.

### Standard Library (400+ functions)
- **String (50+)**: len, ucase, lcase, trim, ltrim, rtrim, find, findNoCase, findOneOf, mid, left, right, replace, replaceNoCase, replaceList, replaceListNoCase, reverse, repeatString, insert, removeChars, spanIncluding, spanExcluding, compare, compareNoCase, asc, chr, reFind, reFindNoCase, reReplace, reReplaceNoCase, reMatch, reMatchNoCase, wrap, stripCr, toBase64, toBinary, urlEncodedFormat, urlDecode, htmlEditFormat, htmlCodeFormat, encodeForHTML, xmlFormat, paragraphFormat, ucFirst, jsStringFormat, reEscape, getToken, newLine, lJustify, rJustify, cJustify, numberFormat, decimalFormat, formatBaseN, inputBaseN
- **Array (46+)**: arrayNew, arrayLen, arrayAppend, arrayPrepend, arrayDeleteAt, arrayInsertAt, arraySet, arraySwap, arrayContains, arrayContainsNoCase, arrayFind, arrayFindNoCase, arrayFindAll, arrayFindAllNoCase, arraySort, arrayReverse, arraySlice, arrayToList, arrayMerge, arrayClear, arrayIsDefined, arrayMin, arrayMax, arrayAvg, arraySum, arrayFirst, arrayLast, arrayIsEmpty, arrayDelete, arrayPop, arrayShift, arrayPush, arrayUnshift, arrayIndexExists, arrayResize, arrayMedian, arrayMid, arrayReduceRight, arraySplice, arrayRange, arrayToStruct, arrayDeleteNoCase + higher-order: map, filter, reduce, each, some, every
- **Struct (31+)**: structNew, structCount, structKeyExists, structKeyList, structKeyArray, structDelete, structInsert, structUpdate, structFind, structFindKey, structFindValue, structClear, structCopy, structAppend, structIsEmpty, structSort, structGet, structValueArray, structEquals, isEmpty, structToSorted, structIsOrdered, structIsCaseSensitive, structToQueryString, structGetMetadata, structSetMetadata + higher-order: each, map, filter, reduce, some, every
- **Math (25+)**: abs, ceiling, floor, round, int, fix, max, min, sgn, sqr, exp, log, log10, trig functions, pi, rand, randRange, randomize, bitwise operations
- **Date/Time (39)**: now, createDate, createDateTime, createTime, ODBC date functions, date part accessors, dateAdd, dateDiff, dateFormat, timeFormat, dateTimeFormat, parseDateTime, datePart, dateCompare, getTickCount, millisecond, dateConvert, getNumericDate, getHTTPTimeString, nowServer
- **List (32+)**: listLen, listFirst, listLast, listRest, listGetAt, listSetAt, listDeleteAt, listInsertAt, listFind, listFindNoCase, listContains, listContainsNoCase, listAppend, listPrepend, listSort, listRemoveDuplicates, listToArray, listValueCount, listValueCountNoCase, listQualify, listChangeDelims, listCompact, listSome, listEvery, listAvg, listItemTrim, listIndexExists, listReduceRight + higher-order: each, map, filter, reduce
- **Query (26)**: queryNew, queryAddRow, querySetCell, queryAddColumn, queryGetRow, queryGetCell, queryRecordCount, queryColumnCount, queryColumnList, queryDeleteRow, queryDeleteColumn, queryColumnExists, queryRowData, querySlice, queryGetResult, queryKeyExists, queryColumnData/queryColumnArray, queryCurrentRow, queryAppend, queryInsertAt, queryPrepend, queryReverse, queryRowSwap, querySetRow, valueList, quotedValueList + higher-order: each, map, filter, reduce, sort, some, every
- **Type checking (12)**: isNull, isDefined, isNumeric, isBoolean, isDate, isArray, isStruct, isQuery, isSimpleValue, isBinary, isValid, isCustomFunction
- **Conversion (15)**: toString, toNumeric, toBoolean, val, javacast, parseNumber, createTimeSpan, yesNoFormat, booleanFormat, trueFalseFormat, nullValue, incrementValue, decrementValue, de, dollarFormat
- **JSON (3)**: serializeJSON, deserializeJSON, isJSON
- **File I/O (23+)**: fileRead, fileWrite, fileAppend, fileExists, fileDelete, fileMove, fileCopy, fileOpen, fileClose, fileReadLine, fileWriteLine, fileReadBinary, fileGetMimeType, fileIsEOF, fileUpload, fileUploadAll, fileSetAccessMode, fileSetAttribute, fileSetLastModified, directoryCreate, directoryExists, directoryDelete, directoryList, directoryRename, directoryCopy, getTempDirectory, getTempFile, getFileInfo, expandPath
- **Security**: encrypt/decrypt (AES/DES/DESEDE/Blowfish/CFMX_COMPAT), hmac, generateSecretKey, hash (MD5/SHA family), createUUID, createGUID, encodeForHTML/URL/CSS/JavaScript, generatePBKDFKey, generateBCryptHash/verifyBCryptHash, generateSCryptHash/verifySCryptHash, generateArgon2Hash/argon2CheckHash, csrfGenerateToken/csrfVerifyToken
- **Encoding/Decoding**: charsetDecode, charsetEncode, encodeForHTMLAttribute, encodeForXML, encodeForXMLAttribute, encodeFor, decodeForHTML, decodeFromURL, urlEncode, canonicalize
- **Locale (13)**: lsDateFormat, lsTimeFormat, lsDateTimeFormat, lsCurrencyFormat, lsEuroCurrencyFormat, lsIsDate, lsIsNumeric, lsIsCurrency, lsParseCurrency, lsParseDateTime, lsNumberFormat, lsWeek, lsDayOfWeek
- **Error Handling**: cfcatch.tagContext (stack trace array with template, line, id, raw_trace, column), exceptionKeyExists
- **XML/HTML (11+)**: xmlParse, xmlSearch, isXML, xmlNew, xmlElemNew, xmlChildPos, xmlGetNodeType, xmlHasChild, isXMLDoc/Elem/Node/Root/Attribute, htmlParse
- **Caching (8)**: cachePut, cacheGet, cacheDelete, cacheClear, cacheKeyExists, cacheCount, cacheGetAll, cacheGetAllIds
- **Higher-Order Generics**: collectionEach/Map/Filter/Reduce/Some/Every, stringEach/Map/Filter/Reduce/Some/Every/Sort, each (generic)
- **Utility (23+)**: writeOutput, writeDump, dump, sleep, duplicate, writeLog, systemOutput, trace, location, applicationStop, getApplicationMetadata, getApplicationSettings, getFileFromPath, getCanonicalPath, getTemplatePath, setLocale, getLocale, setTimeZone, getTimeZone, getBaseTemplatePath, getCurrentTemplatePath, getDirectoryFromPath, setVariable, getVariable, getEnvironmentVariable
- **Session/Auth**: sessionInvalidate, sessionRotate, sessionGetMetaData, getAuthUser, isUserLoggedIn, isUserInRole
- **Bitmask**: bitMaskClear, bitMaskRead, bitMaskSet
- **Zip**: cfzip (actions: zip, unzip, list, read, readBinary, delete)
- **Stack/Precision**: callStackGet, callStackDump, precisionEvaluate
- **Misc**: soundex, metaphone, toScript

### Member Functions
String, Array, Struct, Number member functions with method chaining.

### Scopes
local, variables, arguments, request, application (persistent), server, session (CFID cookie), cookie. Case-insensitive lookup, closure scope capture with write-back.

### Components & Interfaces
extends (dot-path), super.method(), isInstanceOf, getMetadata, createObject, interface/implements with inheritance, implicit accessors, onMissingMethod, component mappings (this.mappings).

### Web Server (Axum)
`--serve` mode with: HTTP response infrastructure, form scope (url-encoded + multipart), url scope, cgi scope (remote_addr, server_name from Host header, all http_* headers), cookie scope, session management (onSessionStart/onSessionEnd lifecycle, configurable timeout), file uploads, Application.cfc lifecycle, URL rewrite engine (Tuckey-compatible). Real named locks with cflock/cfscript lock (RwLock-based concurrency). Bracket assignment for component mappings.

### Database
queryExecute with SQLite, MySQL, PostgreSQL, MSSQL. Connection pooling, cfqueryparam, cftransaction, structured query params.

### Threading (Sequential — No Real Concurrency)
cfthread tag with action=run/join/terminate. **Thread bodies execute inline sequentially** — no threads are spawned. This means cfthread code works functionally but does not run in parallel; sleep/long operations block the parent. cfthread scope with thread metadata (status, output, error, elapsedtime). Output capture, error capture, and thread scope (thread.varName) all work correctly within this sequential model.

### Infrastructure
CLI (file exec, `-c` inline, `-d` debug, `-r` REPL, `--serve`), WASM target, error handling with line/column info.

---

## Remaining Work

### Low Priority
- cfschedule, cfwddx
- INI file functions (getProfileString, setProfileString)
- Real concurrent cfthread execution (currently sequential)

### Explicitly Unsupported
- **Query-of-Queries (QoQ)**: SQL SELECT on in-memory query objects is not supported
- Image functions (80+), Spreadsheet functions (40+), ORM (20+), SOAP/WSDL, Flash/Flex UI tags, Exchange, PDF, LDAP, Registry, .NET, Gateway, JWT

### Taffy Framework
Believed feature-complete. cgi.remote_addr, cgi.server_name (from Host header), and all request headers (cgi.http_*) available for rate limiting and CORS.

---

## Architecture

```
CFML Source → Tag Preprocessor (tag_parser.rs) → CFScript
  → Lexer (lexer.rs) → Tokens
  → Parser (parser.rs) → AST (ast.rs)
  → Compiler (compiler.rs) → Bytecode
  → VM (lib.rs) → Execution
```

```
crates/
├── cfml-common/     # CfmlValue, CfmlError, Position
├── cfml-compiler/   # Lexer, Parser, AST, Tag Preprocessor
├── cfml-codegen/    # AST → BytecodeOp
├── cfml-vm/         # Stack-based VM
├── cfml-stdlib/     # 400+ built-in functions
├── cli/             # CLI + Axum web server
└── wasm/            # WebAssembly target
```

Reference: [cfdocs.org/functions](https://cfdocs.org/functions) | [cfdocs.org/tags](https://cfdocs.org/tags) | BoxLang ANTLR grammar | Lucee expression grammar

*Last Updated: 2026-03-03* | 1163 tests across 88 suites
