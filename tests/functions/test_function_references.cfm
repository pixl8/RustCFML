<cfscript>
suiteBegin("First-Class Function References");

// Basic: assign a UDF to a variable and call it
function foobar(sname) {
    return arguments.sname;
}
myvar = foobar;
assert("func ref call", myvar("hello"), "hello");

// Function should be in variables scope
assertTrue("func in variables", structKeyExists(variables, "foobar"));
assertTrue("assigned ref in variables", structKeyExists(variables, "myvar"));

// Both should be callable
assert("original still works", foobar("world"), "world");
assert("ref still works", myvar("world"), "world");

// Cross-function reference: access UDF from inside another function
function getGreeting(name) {
    return "Hi " & arguments.name;
}

function doTest() {
    var svc = {};
    svc.greet = getGreeting;
    return svc.greet("Alex");
}
assert("cross-function ref", doTest(), "Hi Alex");

// Mixin pattern: inject UDF into a struct and call it
function mixinMethod(val) {
    return val * 2;
}
var obj = { name: "test" };
obj.doubler = mixinMethod;
assert("mixin on struct", obj.doubler(21), 42);

// Multiple function references in a service struct
function add(a, b) { return arguments.a + arguments.b; }
function multiply(a, b) { return arguments.a * arguments.b; }

function buildMathService() {
    var svc = {};
    svc.add = add;
    svc.multiply = multiply;
    return svc;
}
var math = buildMathService();
assert("service add", math.add(3, 4), 7);
assert("service multiply", math.multiply(3, 4), 12);

suiteEnd();
</cfscript>
