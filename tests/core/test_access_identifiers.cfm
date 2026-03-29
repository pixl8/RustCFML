<cfscript>
include "../harness.cfm";
suiteBegin("Access Modifier Keywords as Identifiers");

// private/public can be used as variable names in CFML
var private = "secret";
var public = "open";
assert("var private as identifier", private, "secret");
assert("var public as identifier", public, "open");

// Reassignment without var
private = "updated";
assert("private reassignment", private, "updated");

// Inside a function
function testAccessWords() {
    var private = "func-private";
    var public = "func-public";
    return private & "-" & public;
}
assert("access words in function", testAccessWords(), "func-private-func-public");

// private function declarations still work
private function secretFunc() { return "I am private"; }
assert("private function decl", secretFunc(), "I am private");

suiteEnd();
</cfscript>
