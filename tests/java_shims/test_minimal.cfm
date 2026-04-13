<cfscript>
writeOutput("Starting test<br>");

var sb = createObject("java", "java.lang.StringBuilder");
writeOutput("After createObject<br>");

sb = sb.init("Hello");
writeOutput("After init<br>");

var str = sb.toString();
writeOutput("Result: [" & str & "]<br>");

writeOutput("Done");