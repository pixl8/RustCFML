<cfscript>
// Debug test for component accessors
component name="TestComponent" accessors="true" {
    property name="name" default="test";
}

tc = new TestComponent();
tc.name = "John";

writeOutput("Before setName:\n");
writeOutput("  tc.name = " & tc.name & "\n");
writeOutput("  tc.getName() = " & tc.getName() & "\n");

tc.setName("Jane");

writeOutput("After setName:\n");
writeOutput("  tc.name = " & tc.name & "\n");
writeOutput("  tc.getName() = " & tc.getName() & "\n");

// Check if name property exists in tc
if (structKeyExists(tc, "name")) {
    writeOutput("  tc has 'name' property\n");
} else {
    writeOutput("  tc does NOT have 'name' property\n");
}

// Check all keys
writeOutput("  tc keys: " & structKeyList(tc) & "\n");
</cfscript>
