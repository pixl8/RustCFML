<cfscript>
writeOutput("=== RustCFML Test Suite ===" & chr(10) & chr(10));
var results = [];

function assert(label, actual, expected) {
    if (toString(actual) == toString(expected)) {
        return "PASS";
    } else {
        writeOutput("  FAIL: " & label & " | expected: " & expected & " | got: " & actual & chr(10));
        return "FAIL";
    }
}

// --- Arithmetic ---
writeOutput("Arithmetic:" & chr(10));
var passed = 0;
var failed = 0;
var r = "";

r = assert("2 + 3", 2 + 3, 5);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("10 - 4", 10 - 4, 6);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("3 * 7", 3 * 7, 21);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("2 ^ 10", 2 ^ 10, 1024);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("17 % 5", 17 % 5, 2);
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Strings ---
writeOutput("String Functions:" & chr(10));
passed = 0;
failed = 0;

r = assert("ucase", ucase("hello"), "HELLO");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("lcase", lcase("HELLO"), "hello");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("len", len("hello"), 5);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("left", left("hello", 3), "hel");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("right", right("hello", 3), "llo");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("reverse", reverse("hello"), "olleh");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("trim", trim("  hi  "), "hi");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("find", find("ll", "hello"), 3);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("replace", replace("hello", "ll", "r"), "hero");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("mid", mid("hello", 2, 3), "ell");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("repeatString", repeatString("ab", 3), "ababab");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("chr(65)", chr(65), "A");
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Member Functions ---
writeOutput("Member Functions:" & chr(10));
passed = 0;
failed = 0;

r = assert("str.ucase()", "hello".ucase(), "HELLO");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("str.reverse()", "hello".reverse(), "olleh");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("str.len()", "hello".len(), 5);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("str.left(3)", "hello".left(3), "hel");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("str.trim()", "  hi  ".trim(), "hi");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("chain ucase.reverse", "hello".ucase().reverse(), "OLLEH");
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Arrays ---
writeOutput("Arrays:" & chr(10));
passed = 0;
failed = 0;

var arr = [10, 20, 30, 40, 50];

r = assert("arr[1]", arr[1], 10);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("arr[3]", arr[3], 30);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("arr.len()", arr.len(), 5);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("arr.first()", arr.first(), 10);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("arr.last()", arr.last(), 50);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("arr.sum()", arr.sum(), 150);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("[1,2,3].toList()", [1,2,3].toList(), "1,2,3");
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Structs ---
writeOutput("Structs:" & chr(10));
passed = 0;
failed = 0;

var s = {name: "Alex", age: 30};

r = assert("s.name", s.name, "Alex");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("s.age", s.age, 30);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("s.count()", s.count(), 2);
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- CFML Operators ---
writeOutput("CFML Operators:" & chr(10));
passed = 0;
failed = 0;

r = assert("5 GT 3", 5 GT 3, true);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("5 LT 3", 5 LT 3, false);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("5 EQ 5", 5 EQ 5, true);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("5 NEQ 3", 5 NEQ 3, true);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("contains", "hello" CONTAINS "ell", true);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("AND", true AND false, false);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("OR", true OR false, true);
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Control Flow ---
writeOutput("Control Flow:" & chr(10));
passed = 0;
failed = 0;

var result = "";
for (var i = 1; i <= 5; i++) { result = result & i; }
r = assert("for loop", result, "12345");
if (r == "PASS") { passed++; } else { failed++; }

result = "";
var w = 1;
while (w <= 3) { result = result & w; w++; }
r = assert("while loop", result, "123");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("ternary true", (10 > 5) ? "yes" : "no", "yes");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("ternary false", (10 < 5) ? "yes" : "no", "no");
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Functions ---
writeOutput("Functions:" & chr(10));
passed = 0;
failed = 0;

function add(a, b) { return a + b; }
r = assert("user function", add(3, 4), 7);
if (r == "PASS") { passed++; } else { failed++; }

function factorial(n) { if (n <= 1) return 1; return n * factorial(n - 1); }
r = assert("factorial(5)", factorial(5), 120);
if (r == "PASS") { passed++; } else { failed++; }

function fibonacci(n) { if (n <= 1) return n; return fibonacci(n-1) + fibonacci(n-2); }
r = assert("fibonacci(10)", fibonacci(10), 55);
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Closures & Higher-Order ---
writeOutput("Higher-Order Functions:" & chr(10));
passed = 0;
failed = 0;

var doubled = [1,2,3].map(function(n) { return n * 2; });
r = assert("array.map", doubled.toList(), "2,4,6");
if (r == "PASS") { passed++; } else { failed++; }

var evens = [1,2,3,4,5,6].filter(function(n) { return n % 2 == 0; });
r = assert("array.filter", evens.toList(), "2,4,6");
if (r == "PASS") { passed++; } else { failed++; }

var total = [1,2,3,4,5].reduce(function(acc, n) { return acc + n; }, 0);
r = assert("array.reduce", total, 15);
if (r == "PASS") { passed++; } else { failed++; }

var mapped = arrayMap([1,2,3], function(n) { return n * 10; });
r = assert("arrayMap standalone", mapped.toList(), "10,20,30");
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Math ---
writeOutput("Math Functions:" & chr(10));
passed = 0;
failed = 0;

r = assert("abs(-5)", abs(-5), 5);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("ceiling(3.2)", ceiling(3.2), 4);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("floor(3.8)", floor(3.8), 3);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("max(10,20)", max(10, 20), 20);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("min(10,20)", min(10, 20), 10);
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Type Checking ---
writeOutput("Type Checking:" & chr(10));
passed = 0;
failed = 0;

r = assert("isNumeric(42)", isNumeric(42), true);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("isNumeric('abc')", isNumeric("abc"), false);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("isArray([1,2])", isArray([1,2]), true);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("isStruct({a:1})", isStruct({a:1}), true);
if (r == "PASS") { passed++; } else { failed++; }

r = assert("isNull(null)", isNull(null), true);
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- Error Handling ---
writeOutput("Error Handling:" & chr(10));
passed = 0;
failed = 0;

var caught = "no";
try {
    throw("test error");
} catch (any e) {
    caught = "yes";
}
r = assert("try/catch", caught, "yes");
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

// --- JSON ---
writeOutput("JSON:" & chr(10));
passed = 0;
failed = 0;

r = assert("serializeJSON number", serializeJSON(42), "42");
if (r == "PASS") { passed++; } else { failed++; }

r = assert("serializeJSON string", serializeJSON("hello"), '"hello"');
if (r == "PASS") { passed++; } else { failed++; }

writeOutput("  " & passed & " passed, " & failed & " failed" & chr(10) & chr(10));

writeOutput("=== Test Suite Complete ===" & chr(10));
</cfscript>
