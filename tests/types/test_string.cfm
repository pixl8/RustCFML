<cfscript>
suiteBegin("Type: String");

// --- String literals (double quotes) ---
s = "hello world";
assert("double-quoted string", s, "hello world");

// --- Single-quoted strings ---
sq = 'single quoted';
assert("single-quoted string", sq, "single quoted");

// --- Empty string ---
empty = "";
assert("empty string", empty, "");
assert("empty string length", len(empty), 0);

// --- String concatenation with & ---
first = "Hello";
second = " World";
assert("string concatenation", first & second, "Hello World");

// --- String comparison is case-insensitive ---
assert("compare() case-insensitive eq", compare("abc", "abc"), 0);
assertTrue("case-insensitive equality", "abc" EQ "ABC");
assertTrue("case-insensitive EQ keyword", "Hello" EQ "hello");

// --- String length with len() ---
assert("len of string", len("abcdef"), 6);
assert("len of single char", len("x"), 1);

// --- String with special characters ---
special = "it's a ""test""";
assert("double-quote escaping", len(special) > 0, true);

// --- String numeric coercion ---
numStr = "42";
assert("string + number coercion", numStr + 0, 42);

// --- String interpolation with #expr# ---
name = "World";
interpolated = "Hello #name#!";
assert("string interpolation variable", interpolated, "Hello World!");
calcInterp = "Result: #1 + 2#";
assert("string interpolation expression", calcInterp, "Result: 3");

// --- Multiline string ---
multi = "line1
line2";
assertTrue("multiline string has content", len(multi) > 5);

suiteEnd();
</cfscript>
