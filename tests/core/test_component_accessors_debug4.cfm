<cfscript>
// Debug test for component accessors
component name="TestComponent" accessors="true" {
    property name="name" default="test";
}

tc = new TestComponent();
tc.name = "John";

writeOutput("Before setName:\n");
writeOutput("  tc.name = " & tc.name & "\n");
writeOutput("  tc.__variables.name = " & tc.__variables.name & "\n");

tc.setName("Jane");

writeOutput("After setName:\n");
writeOutput("  tc.name = " & tc.name & "\n");
writeOutput("  tc.__variables.name = " & tc.__variables.name & "\n");
writeOutput("  tc.getName() = " & tc.getName() & "\n");
</cfscript>
