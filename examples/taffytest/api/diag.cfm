<cfscript>
writeOutput("=== Taffy Diagnostics ===\n\n");
writeOutput("Base template: " & getBaseTemplatePath() & "\n");
basedir = getDirectoryFromPath(getBaseTemplatePath());
writeOutput("Base directory: " & basedir & "\n\n");

resPath = basedir & "resources/";
writeOutput("Resources path: " & resPath & "\n");
writeOutput("Resources exists: " & directoryExists(resPath) & "\n\n");

if (directoryExists(resPath)) {
    files = directoryList(resPath, true, "path", "*.cfc");
    writeOutput("CFC files found: " & arrayLen(files) & "\n");
    for (f in files) {
        writeOutput("  - " & f & "\n");
    }
    writeOutput("\n");

    writeOutput("--- Loading HelloResource ---\n");
    try {
        obj = createObject("component", "resources.HelloResource");
        writeOutput("Created OK\n");
        meta = getMetaData(obj);
        writeOutput("Metadata type: " & meta.getClass().getName() & "\n");
    } catch(any e) {
        writeOutput("ERROR creating: " & e.message & "\n");
    }

    writeOutput("\n--- Try getComponentMetadata ---\n");
    try {
        meta = getComponentMetadata("resources.HelloResource");
        writeOutput("Got metadata\n");
        for (k in meta) {
            writeOutput("  " & k & " = ");
            try { writeOutput(meta[k]); } catch(any e2) { writeOutput("[complex]"); }
            writeOutput("\n");
        }
    } catch(any e) {
        writeOutput("ERROR: " & e.message & "\n");
    }
}
</cfscript>
