<cfscript>
// Debug test for component accessors
component name="DebugComponent" accessors="true" {
    property name="name" default="test";
}

dc = new DebugComponent();

// Check what's in the component
writeOutput("Component name property: " & dc.name & "\n");

// Check if getter exists
if (structKeyExists(dc, "getName")) {
    writeOutput("getName exists: " & dc.getName() & "\n");
} else {
    writeOutput("getName does NOT exist\n");
}

// Check if setter exists
if (structKeyExists(dc, "setName")) {
    writeOutput("setName exists\n");
    dc.setName("modified");
    writeOutput("After setName, name = " & dc.name & "\n");
} else {
    writeOutput("setName does NOT exist\n");
}
</cfscript>
