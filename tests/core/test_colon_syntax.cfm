<cfscript>
// Test colon syntax for named arguments
include "../harness.cfm";

suiteBegin("Colon Syntax for Named Arguments");

// Test 1: throw() with colon syntax
try {
    throw(
        type   : "TestError",
        message: "Test message with colon syntax"
    );
} catch (Any e) {
    assertTrue("throw colon type", e.type eq "TestError");
    assertTrue("throw colon message", e.message eq "Test message with colon syntax");
}

// Test 2: Mixed colon and equals syntax
try {
    throw(
        type = "MixedError",
        message : "Mixed syntax test"
    );
} catch (Any e) {
    assertTrue("throw mixed type", e.type eq "MixedError");
    assertTrue("throw mixed message", e.message eq "Mixed syntax test");
}

// Test 3: Function call with colon syntax (if supported)
// This tests the general parse_arguments() change
testStruct = {
    key1 : "value1",
    key2 : "value2"
};
assertTrue("struct colon syntax key1", testStruct.key1 eq "value1");
assertTrue("struct colon syntax key2", testStruct.key2 eq "value2");

suiteEnd();
</cfscript>
