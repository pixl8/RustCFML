<cfscript>
suiteBegin("Sequential calls to function stored on plain struct");

// Regression: a function stored on a plain struct (non-CFC) was getting
// `__variables` written into it after a previous CFC method call left
// `method_variables_writeback` set. The pollution made the struct look
// like a CFC on subsequent calls, breaking dispatch.

// 1) Plain struct with a stored function — multiple calls in sequence.
encode = { string: function(d) { return chr(2) & d; } };
a = encode.string("first");
b = encode.string("second");
c = encode.string("third");
assert("first call",  len(a), 6);  // chr(2) + "first"
assert("second call", len(b), 7);  // chr(2) + "second"
assert("third call",  len(c), 6);  // chr(2) + "third"

// 2) Same shape inside a CFC pseudo-constructor (this is what stock
//    Taffy's `taffy.core.resource` chain depends on — was Bug-G adjacent).
suiteBegin_dummy = "guard"; // touch __main__ locals so closure_env grows
component_under_test = createObject("component", "core.SequentialMethodCFC").init();
got = component_under_test.results();
assert("cfc body — first call",  len(got.a), 4); // chr(2) + "aaa"
assert("cfc body — second call", len(got.b), 4); // chr(2) + "bbb"
assert("cfc body — third call",  len(got.c), 4); // chr(2) + "ccc"

suiteEnd();
</cfscript>
