<cfscript>
// Example 05: Nested Conditionals
// If/else with nested blocks

score = 85;
passed = true;

if (score >= 60) {
    if (passed) {
        writeOutput("Passed");
    } else {
        writeOutput("Failed - not passed");
    }
} else {
    writeOutput("Failed");
}
</cfscript>
