<cfscript>
suiteBegin("String Member Functions");

// --- len ---
assert("string.len()", "hello".len(), 5);

// --- ucase / lcase ---
assert("string.ucase()", "hello".ucase(), "HELLO");
assert("string.lcase()", "HELLO".lcase(), "hello");

// --- trim / ltrim / rtrim ---
assert("string.trim()", "  hi  ".trim(), "hi");
assert("string.ltrim()", "  hi".ltrim(), "hi");
assert("string.rtrim()", "hi  ".rtrim(), "hi");

// --- left / right / mid ---
assert("string.left(3)", "hello".left(3), "hel");
assert("string.right(3)", "hello".right(3), "llo");
assert("string.mid(2,3)", "hello".mid(2, 3), "ell");

// --- reverse ---
assert("string.reverse()", "hello".reverse(), "olleh");

// --- find / findNoCase ---
assert("string.find(ll)", "hello".find("ll"), 3);
assert("string.findNoCase(LL)", "hello".findNoCase("LL"), 3);

// --- replace ---
assert("string.replace(ll, r)", "hello".replace("ll", "r"), "hero");

// --- repeatString ---
assert("string.repeatString(3)", "hello".repeatString(3), "hellohellohello");
assert("ab.repeatString(3)", "ab".repeatString(3), "ababab");

// --- insert ---
assert("string.insert(X, 3)", "hello".insert("X", 3), "helXlo");

// --- chaining: ucase then reverse ---
assert("chain ucase().reverse()", "hello".ucase().reverse(), "OLLEH");

// --- chaining: trim then ucase ---
assert("chain trim().ucase()", "  hello  ".trim().ucase(), "HELLO");

// --- ucFirst ---
assert("string.ucFirst()", "hello world".ucFirst(), "Hello world");

// --- compare ---
assert("string.compare() equal", "Hello".compare("Hello"), 0);

suiteEnd();
</cfscript>
