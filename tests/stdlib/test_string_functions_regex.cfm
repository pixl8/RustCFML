<cfscript>
suiteBegin("String Functions: Regex");

// --- reFind ---
assertTrue("reFind digits found", reFind("[0-9]+", "abc123def") > 0);
assert("reFind no digits", reFind("[0-9]+", "abcdef"), 0);

// --- reFindNoCase ---
assertTrue("reFindNoCase letters", reFindNoCase("[A-Z]+", "hello") > 0);

// --- reReplace ---
assert("reReplace first", reReplace("abc123def", "[0-9]+", "NUM"), "abcNUMdef");
assert("reReplace all", reReplace("abc123def456", "[0-9]+", "NUM", "all"), "abcNUMdefNUM");

// --- reReplaceNoCase ---
assert("reReplaceNoCase all", reReplaceNoCase("Hello World", "[a-z]+", "X", "all"), "X X");

// --- reMatch ---
matches = reMatch("[0-9]+", "abc123def456");
assert("reMatch count", arrayLen(matches), 2);
assert("reMatch first", matches[1], "123");
assert("reMatch second", matches[2], "456");

// --- reMatchNoCase ---
wordMatches = reMatchNoCase("[a-z]+", "Hello World");
assert("reMatchNoCase count", arrayLen(wordMatches), 2);
assert("reMatchNoCase first", wordMatches[1], "Hello");
assert("reMatchNoCase second", wordMatches[2], "World");

suiteEnd();
</cfscript>
