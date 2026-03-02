<cfscript>
suiteBegin("Scopes");

// --- variables scope ---
variables.scopeTest = "in variables";
assert("variables scope explicit", variables.scopeTest, "in variables");

// --- Unscoped reads from variables scope ---
scopeTest2 = "unscoped";
assert("unscoped same as variables", variables.scopeTest2, "unscoped");

// --- request scope (set and read back) ---
request.testValue = "from request";
assert("request scope set and read", request.testValue, "from request");

// --- local scope inside function ---
function testLocalScope() {
    var localVar = "I am local";
    return localVar;
}
assert("local scope in function", testLocalScope(), "I am local");

// --- arguments scope inside function ---
function testArgsScope(a, b) {
    return arguments.a & "-" & arguments.b;
}
assert("arguments scope", testArgsScope("x", "y"), "x-y");

// --- arguments scope: count ---
function argCount() {
    return structCount(arguments);
}
assert("arguments count", argCount("a", "b", "c"), 3);

// --- Scope precedence: local shadows variables ---
variables.shadowed = "from variables";
function testShadowing() {
    var shadowed = "from local";
    return shadowed;
}
assert("local shadows variables", testShadowing(), "from local");
assert("variables still intact after shadow", variables.shadowed, "from variables");

// --- Function does not leak local vars ---
function testNoLeak() {
    var leakTest = "should not leak";
    return true;
}
testNoLeak();
leaked = false;
try {
    check = local.leakTest;
    leaked = true;
} catch (any e) {
    leaked = false;
}
assertFalse("function local does not leak", leaked);

// --- request scope persists ---
request.persistCheck = 42;
function readRequest() {
    return request.persistCheck;
}
assert("request scope persists into function", readRequest(), 42);

suiteEnd();
</cfscript>
