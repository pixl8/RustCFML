<cfscript>
// Example 08: Built-in Functions
// Using CFML built-in functions

str = "  Hello World  ";

writeOutput("Original: [");
writeOutput(str);
writeOutput("]");

writeOutput("Trimmed: [");
writeOutput(trim(str));
writeOutput("]");

writeOutput("Uppercase: ");
writeOutput(ucase(str));

writeOutput("Lowercase: ");
writeOutput(lcase("HELLO"));

writeOutput("String length: ");
writeOutput(len("Hello"));

writeOutput("Replace: ");
writeOutput(replace("Hello World", "World", "Rust"));
</cfscript>
