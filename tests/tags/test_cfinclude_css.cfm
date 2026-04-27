<cfscript>
suiteBegin("cfinclude of static CSS keeps hash literal");

// Regression: pre-fix, the tag preprocessor interpolated hash-expr-hash even
// outside cfoutput, and the lexer interpolated hash inside __writeText() string
// literals. CSS hex colors thus crashed with parse errors.
savecontent variable="captured" {
    include "literal_styles.css";
}

hash = chr(35);
assert("first hex color survives",  findNoCase(hash & "f3f4f6", captured) > 0, true);
assert("second hex color survives", findNoCase(hash & "111827", captured) > 0, true);
assert("third hex color survives",  findNoCase(hash & "fef2f2", captured) > 0, true);
assert("fourth hex color survives", findNoCase(hash & "fecaca", captured) > 0, true);
assert("CSS rule structure intact", findNoCase("background-color:", captured) > 0, true);

suiteEnd();
</cfscript>
