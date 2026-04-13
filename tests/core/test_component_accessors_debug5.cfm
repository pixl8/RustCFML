<cfscript>
// Debug test for component accessors with defaults
component name="DefaultComponent" accessors="true" {
    property name="count" default=0;
    property name="message" default="hello";
}

dc = new DefaultComponent();

writeOutput("dc keys: " & structKeyList(dc) & "\n");
writeOutput("dc.__variables keys: " & structKeyList(dc.__variables) & "\n");
writeOutput("dc.count = " & dc.count & "\n");
writeOutput("dc.__variables.count = " & dc.__variables.count & "\n");
writeOutput("dc.message = " & dc.message & "\n");
writeOutput("dc.__variables.message = " & dc.__variables.message & "\n");
</cfscript>
