<cfscript>
suiteBegin("cfparam (script) — static and dynamic names");

// Static-name forms (compile-time expansion)
param name="p_simple" default=5;
assert("static simple sets default", p_simple, 5);

p_pre = 9;
param name="p_pre" default=5;
assert("static simple keeps existing", p_pre, 9);

p_dot = {};
param name="p_dot.a" default=1;
assert("static dotted sets nested default", p_dot.a, 1);

p_dot2 = { a: 7 };
param name="p_dot2.a" default=1;
assert("static dotted keeps existing", p_dot2.a, 7);

// Dynamic-name form: a.b.c['#expr#']  — the Taffy idiom.
function dynParam(required struct data) {
    var ext = "html";
    param name="arguments.data.mimeExtensions['#ext#']" default="text/html";
    return arguments.data;
}
shell = { mimeExtensions: {} };
result = dynParam(shell);
assertTrue("dynamic-bracket: key inserted", structKeyExists(result.mimeExtensions, "html"));
assert("dynamic-bracket: value is default", result.mimeExtensions.html, "text/html");
// Mutation must propagate back to the caller (pass-by-reference).
assertTrue("dynamic-bracket: writeback to caller", structKeyExists(shell.mimeExtensions, "html"));
assert("dynamic-bracket: writeback value", shell.mimeExtensions.html, "text/html");

// Dynamic-name form must NOT overwrite an existing key.
function dynParamPre(required struct data) {
    var ext = "html";
    param name="arguments.data.mimeExtensions['#ext#']" default="text/html";
    return arguments.data;
}
shell2 = { mimeExtensions: { html: "application/xhtml" } };
result2 = dynParamPre(shell2);
assert("dynamic-bracket: existing value preserved", result2.mimeExtensions.html, "application/xhtml");

// Dynamic with double-quoted bracket
function dynParamDq(required struct data) {
    var ext = "json";
    param name='arguments.data.mimeExtensions["#ext#"]' default="application/json";
    return arguments.data;
}
shell3 = { mimeExtensions: {} };
result3 = dynParamDq(shell3);
assert("dynamic-bracket double-quoted", result3.mimeExtensions.json, "application/json");

// Pattern B: trailing-dot interpolation — `<path>.#expr#`
function dynParamDot(required struct data) {
    var k = "title";
    param name="arguments.data.#k#" default="untitled";
    return arguments.data;
}
d1 = {};
r1 = dynParamDot(d1);
assert("dotted-interp: key inserted", r1.title, "untitled");
assertTrue("dotted-interp: writeback to caller", structKeyExists(d1, "title"));

d2 = { title: "kept" };
r2 = dynParamDot(d2);
assert("dotted-interp: existing preserved", r2.title, "kept");

// Parser pattern B + runtime variables-scope: `param name="variables.#k#" ...`
// The parser lowers this to a structKeyExists guard against `variables`.
varName = "_param_var_test";
if (structKeyExists(variables, varName)) structDelete(variables, varName);
param name="variables.#varName#" default=42;
assert("variables.<dyn>: default applied", variables._param_var_test, 42);
param name="variables.#varName#" default=99;
assert("variables.<dyn>: existing kept", variables._param_var_test, 42);

// Runtime __cfparam fallback: bare expression as the name (no string literal,
// no interpolation) — this bypasses both static and pattern lowerings and hits
// the VM intercept, which writes into the variables scope by default.
runtimeName = "_param_runtime_test";
if (structKeyExists(variables, runtimeName)) structDelete(variables, runtimeName);
param name=runtimeName default="rt";
assert("runtime fallback: default applied", variables._param_runtime_test, "rt");
param name=runtimeName default="other";
assert("runtime fallback: existing kept", variables._param_runtime_test, "rt");

suiteEnd();
</cfscript>
