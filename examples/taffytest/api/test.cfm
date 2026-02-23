<cfscript>
writeOutput("=== Taffy on RustCFML ===" & chr(10));
writeOutput("app._taffy exists: " & structKeyExists(application, "_taffy") & chr(10));
if (structKeyExists(application, "_taffy")) {
    writeOutput("beanList: " & application._taffy.beanList & chr(10));
    writeOutput("endpoints: " & structCount(application._taffy.endpoints) & chr(10));
    writeOutput("URIMatchOrder: " & arrayToList(application._taffy.URIMatchOrder) & chr(10));

    // Show registered endpoints
    var uris = application._taffy.URIMatchOrder;
    var i = 0;
    for (i = 1; i <= arrayLen(uris); i = i + 1) {
        var uri = uris[i];
        var ep = application._taffy.endpoints[uri];
        writeOutput("  " & uri & " => " & ep.beanName & chr(10));
    }
}
</cfscript>
