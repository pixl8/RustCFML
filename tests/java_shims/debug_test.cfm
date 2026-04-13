<cfscript>
writeOutput("Test 1: createObject java.security.MessageDigest<br>");
shim = createObject("java", "java.security.MessageDigest");
writeOutput("shim: " & shim.toString() & "<br>");
writeOutput("Test 2: calling isEqual<br>");
result = shim.isEqual("test", "test");
writeOutput("result: [" & result & "]<br>");
writeOutput("Test 3: calling getInstance<br>");
md = shim.getInstance("SHA-256");
writeOutput("md toString: " & md.toString() & "<br>");
</cfscript>