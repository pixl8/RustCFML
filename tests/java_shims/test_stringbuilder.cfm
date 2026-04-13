<cfscript>

suiteBegin("StringBuilder Test");

sb = createObject("java", "java.lang.StringBuilder");
writeOutput("Created StringBuilder<br>");

sb = sb.init("Hello");
writeOutput("After init: buffer = '" & sb.toString() & "'<br>");

sb = sb.append(" World");
writeOutput("After append: buffer = '" & sb.toString() & "'<br>");

writeOutput("Length = " & sb.length() & "<br>");

sb = sb.append("!");
writeOutput("After second append: buffer = '" & sb.toString() & "'<br>");
writeOutput("Final length = " & sb.length() & "<br>");

result = sb.toString();
assert("toString result", result, "Hello World!");

suiteEnd();
</cfscript>