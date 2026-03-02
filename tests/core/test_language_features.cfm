<cfscript>
suiteBegin("Language Features");

// --- String interpolation: simple variable ---
name = "World";
interpolated = "Hello #name#";
assert("string interpolation simple", interpolated, "Hello World");

// --- String interpolation: expression ---
x = 3;
y = 4;
exprInterp = "Sum is #x + y#";
assert("string interpolation expression", exprInterp, "Sum is 7");

// --- String interpolation: function call ---
fnInterp = "Length is #len('abc')#";
assert("string interpolation function call", fnInterp, "Length is 3");

// --- String interpolation: nested in struct value ---
label = "test";
st = { msg: "label=#label#" };
assert("interpolation in struct literal", st.msg, "label=test");

// --- String interpolation: with pound in single quotes (no interpolation) ---
noInterp = 'Hello #name#';
assert("single quotes no interpolation", noInterp, "Hello ##name##");

// --- Elvis operator: null case ---
nullVar = javacast("null", "");
elvisResult = nullVar ?: "default";
assert("elvis operator null", elvisResult, "default");

// --- Elvis operator: non-null case ---
nonNull = "present";
elvisNonNull = nonNull ?: "default";
assert("elvis operator non-null", elvisNonNull, "present");

// --- Elvis operator: empty string is NOT null ---
emptyStr = "";
elvisEmpty = emptyStr ?: "default";
assert("elvis empty string not null", elvisEmpty, "");

// --- Ternary with complex expressions ---
arr = [1, 2, 3, 4, 5];
ternaryComplex = (arrayLen(arr) > 3) ? "long" : "short";
assert("ternary complex condition", ternaryComplex, "long");

// --- Ternary chained ---
score = 75;
grade = (score >= 90) ? "A" : (score >= 80) ? "B" : (score >= 70) ? "C" : "F";
assert("ternary chained", grade, "C");

// --- Array merge (spread alternative) ---
base = [1, 2, 3];
extended = arrayMerge([0], arrayMerge(base, [4, 5]));
assert("array merge length", arrayLen(extended), 6);
assert("array merge first elem", extended[1], 0);
assert("array merge spread elem", extended[2], 1);
assert("array merge last elem", extended[6], 5);

// --- Null-safe navigation ---
nullObj = javacast("null", "");
assertTrue("null-safe navigation on null", isNull(nullObj?.property));

suiteEnd();
</cfscript>
