<cfscript>
// Debug test for component accessors
component name="TestComponent" accessors="true" {
    property name="name" default="test";
}

tc = new TestComponent();

// Check template
writeOutput("Template keys: " & structKeyList(TestComponent) & "\n");
writeOutput("Template has __variables: " & structKeyExists(TestComponent, "__variables") & "\n");

// Check instance
writeOutput("Instance keys: " & structKeyList(tc) & "\n");
writeOutput("Instance has __variables: " & structKeyExists(tc, "__variables") & "\n");

if (structKeyExists(tc, "__variables")) {
    writeOutput("Instance __variables keys: " & structKeyList(tc.__variables) & "\n");
}
</cfscript>
