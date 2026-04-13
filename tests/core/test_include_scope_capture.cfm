<cfscript>
suiteBegin("Include Function Scope Capture");

// Reproduces the crabpot service registry pattern: variables set in main,
// functions defined in same scope, stored in structs via another function,
// then called as struct methods.

variables.testHome = "/test/path";

function testConfigSave() {
    return variables.testHome & "/config.json";
}

// Test 1: Direct call works
assert("direct testConfigSave", testConfigSave(), "/test/path/config.json");

// Test 2: Struct method call (inline)
cfgSvc = {};
cfgSvc.save = testConfigSave;
assert("struct save method", cfgSvc.save(), "/test/path/config.json");

// Test 3: Function builds registry (simulating registerBuiltinServices)
function buildRegistry() {
    var reg = {};
    reg.save = testConfigSave;
    return reg;
}
builtReg = buildRegistry();
assert("built registry save", builtReg.save(), "/test/path/config.json");

// Test 4: Function that calls another function accessing variables scope
function testConfigSet(required string key) {
    return testConfigSave();
}
cfgSvc2 = {};
cfgSvc2.set = testConfigSet;
assert("struct set calls save", cfgSvc2.set("x"), "/test/path/config.json");

// Test 5: Registry pattern with set calling save
function buildRegistry2() {
    var reg = {};
    reg.set = testConfigSet;
    reg.save = testConfigSave;
    return reg;
}
builtReg2 = buildRegistry2();
assert("built registry set->save", builtReg2.set("y"), "/test/path/config.json");

suiteEnd();
</cfscript>
