<cfscript>
suiteBegin("Arguments Scope Pass-by-Reference Writeback");

// Test 1: modify struct param via arguments scope (arguments.param.prop = val)
function addViaArguments(required any obj) {
    arguments.obj.foo = "bar";
}
var s1 = { name: "test" };
addViaArguments(s1);
assertTrue("arguments.obj.prop writeback", structKeyExists(s1, "foo"));
assert("arguments.obj.prop value", s1.foo, "bar");

// Test 2: modify struct param directly (param.prop = val)
function addDirect(required any obj) {
    obj.color = "blue";
}
var s2 = { name: "test" };
addDirect(s2);
assertTrue("direct param.prop writeback", structKeyExists(s2, "color"));
assert("direct param.prop value", s2.color, "blue");

// Test 3: both patterns in same function
function addBoth(required any obj) {
    obj.directProp = "direct";
    arguments.obj.argsProp = "args";
}
var s3 = {};
addBoth(s3);
assertTrue("both: direct prop exists", structKeyExists(s3, "directProp"));
assertTrue("both: args prop exists", structKeyExists(s3, "argsProp"));

// Test 4: mixin pattern — inject function ref into struct via arguments
function injectMethod(required any target) {
    arguments.target.greet = function(name) { return "Hello " & arguments.name; };
}
var svc = {};
injectMethod(svc);
assertTrue("mixin: function injected", structKeyExists(svc, "greet"));
assert("mixin: function callable", svc.greet("World"), "Hello World");

// Test 5: structInsert via arguments scope
function addViaStructInsert(required any obj) {
    structInsert(arguments.obj, "inserted", "yes");
}
var s5 = {};
addViaStructInsert(s5);
assertTrue("structInsert via arguments", structKeyExists(s5, "inserted"));

// Test 6: nested struct modification
function modifyNested(required any obj) {
    arguments.obj.child = { nested: true };
}
var s6 = {};
modifyNested(s6);
assertTrue("nested struct added", structKeyExists(s6, "child"));
assertTrue("nested value preserved", s6.child.nested);

suiteEnd();
</cfscript>
