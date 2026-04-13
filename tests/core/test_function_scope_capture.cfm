<cfscript>
suiteBegin("Function Reference Scope Capture");

// Bug #9: Functions stored in structs should retain access to their defining scope

// Setup: define a variable in the current scope, then a function that reads it
variables.myConfig = "hello world";

function getMyConfig() {
    return variables.myConfig;
}

// Store function reference in a struct (like service registry pattern)
svc = {};
svc.get = getMyConfig;

// Test 1: Function reference in struct can read variables scope
assert("struct fn reads variables scope", svc.get(), "hello world");

// Test 2: Direct call still works
assert("direct call still works", getMyConfig(), "hello world");

// Test 3: Variable set before function and struct, new function
variables.anotherVar = "test value";
function getAnotherVar() {
    return variables.anotherVar;
}
svc2 = {};
svc2.get = getAnotherVar;
assert("fn ref reads var set before capture", svc2.get(), "test value");

// Test 4: Nested struct - service stored in another struct
registry = {};
registry.configService = svc;
assert("nested struct fn access", registry.configService.get(), "hello world");

// Test 5: Function reference passed through another function
function useSvc(required any service) {
    return arguments.service.get();
}
assert("fn ref through function call", useSvc(svc), "hello world");

// Test 6: Multiple functions sharing the same defining scope
function getConfig2() {
    return variables.myConfig;
}
svc3 = {};
svc3.get1 = getMyConfig;
svc3.get2 = getConfig2;
assert("multiple fn refs same scope", svc3.get1(), svc3.get2());

suiteEnd();
</cfscript>
