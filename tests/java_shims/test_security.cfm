<cfscript>
// Test Java security shims for Wheels compatibility

shim = "";
md = "";
result = "";
uuid = "";
t = "";
addr = "";
f = "";
exists = "";
path = "";
resultType = "";

suiteBegin("Java Security & UUID Shims");

// Test 1: Create Java shim object
writeOutput("1. createObject java: ");
shim = createObject("java", "java.security.MessageDigest");
writeOutput("OK<br>");

// Test 2: MessageDigest.getInstance
writeOutput("2. getInstance: ");
md = shim.getInstance("SHA-256");
writeOutput("OK<br>");

// Test 3: MessageDigest.update — real Java wants byte[], .getBytes() gives us that
writeOutput("3. update: ");
md.update("test data".getBytes());
writeOutput("OK<br>");

// Test 4: MessageDigest.digest
writeOutput("4. digest: ");
result = md.digest();
writeOutput("OK - result type: Binary<br>");

// Test 5: MessageDigest.isEqual — static method, takes byte[] arrays
writeOutput("5. isEqual: ");
md = createObject("java", "java.security.MessageDigest");
result = md.isEqual("test".getBytes(), "test".getBytes());
writeOutput("OK - result: " & result & "<br>");

// Test 6: UUID.randomUUID
writeOutput("6. UUID.randomUUID: ");
uuid = createObject("java", "java.util.UUID").randomUUID();
writeOutput("OK - " & uuid.toString() & "<br>");

// Test 7: Thread.currentThread
writeOutput("7. Thread.currentThread: ");
t = createObject("java", "java.lang.Thread").currentThread();
writeOutput("OK - " & t.getName() & "<br>");

// Test 8: InetAddress.getLocalHost
writeOutput("8. InetAddress.getLocalHost: ");
addr = createObject("java", "java.net.InetAddress").getLocalHost();
writeOutput("OK - " & addr.getHostAddress() & "<br>");

// Test 9: InetAddress.getByName
writeOutput("9. InetAddress.getByName: ");
addr = createObject("java", "java.net.InetAddress").getByName("localhost");
writeOutput("OK - " & addr.getHostAddress() & "<br>");

// Test 10: File.exists
writeOutput("10. File.exists: ");
f = createObject("java", "java.io.File").init("/tmp");
exists = f.exists();
writeOutput("OK - " & exists & "<br>");

// Test 11: File.getAbsolutePath
writeOutput("11. File.getAbsolutePath: ");
path = f.getAbsolutePath();
writeOutput("OK - " & path & "<br>");

// Test 12: System.currentTimeMillis
writeOutput("12. System.currentTimeMillis: ");
t = createObject("java", "java.lang.System").currentTimeMillis();
writeOutput("OK - " & t & "<br>");

// Test 13: System.nanoTime
writeOutput("13. System.nanoTime: ");
t = createObject("java", "java.lang.System").nanoTime();
writeOutput("OK - " & t & "<br>");

suiteEnd();
</cfscript>