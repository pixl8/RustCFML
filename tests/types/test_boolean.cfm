<cfscript>
suiteBegin("Type: Boolean");

// --- true and false literals ---
t = true;
f = false;
assertTrue("true literal is truthy", t);
assertFalse("false literal is falsy", f);

// --- Boolean to string ---
assert("toString(true)", toString(true), "true");
assert("toString(false)", toString(false), "false");

// --- Numeric truthy ---
assertTrue("1 is truthy", 1);
assertFalse("0 is falsy", 0);

// --- String truthy ---
assertTrue("string 'yes' is truthy", "yes");
assertFalse("string 'no' is falsy", "no");
assertTrue("string 'true' is truthy", "true");
assertFalse("string 'false' is falsy", "false");

// --- Boolean operators result in booleans ---
andResult = true AND true;
assertTrue("true AND true is truthy", andResult);
orResult = false OR true;
assertTrue("false OR true is truthy", orResult);
notResult = NOT false;
assertTrue("NOT false is truthy", notResult);

// --- isBoolean checks ---
assertTrue("isBoolean(true)", isBoolean(true));
assertTrue("isBoolean(false)", isBoolean(false));
assertTrue("isBoolean('yes')", isBoolean("yes"));
assertTrue("isBoolean('no')", isBoolean("no"));
assertTrue("isBoolean(1)", isBoolean(1));
assertTrue("isBoolean(0)", isBoolean(0));
assertFalse("isBoolean('hello')", isBoolean("hello"));

suiteEnd();
</cfscript>
