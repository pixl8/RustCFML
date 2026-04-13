<cfscript>
include "../harness.cfm";

var sb = "";
var result = "";
var map = "";
var lmap = "";
var queue = "";
var paths = "";
var keys = "";

suiteBegin("More Java Shims");

// StringBuilder
writeOutput("1. StringBuilder: ");
sb = createObject("java", "java.lang.StringBuilder").init("Hello");
sb = sb.append(" World");
result = sb.toString();
writeOutput(result & "<br>");

writeOutput("2. StringBuilder length: ");
result = sb.length();
writeOutput(result & "<br>");

// TreeMap
writeOutput("3. TreeMap: ");
map = createObject("java", "java.util.TreeMap").init({z=3, a=1, m=2});
keys = map.keySet();
writeOutput("keys: " & arrayLen(keys) & "<br>");

writeOutput("4. TreeMap get: ");
result = map.get("a");
writeOutput(result & "<br>");

// LinkedHashMap
writeOutput("5. LinkedHashMap: ");
lmap = createObject("java", "java.util.LinkedHashMap").init();
lmap = lmap.put("first", 1);
lmap = lmap.put("second", 2);
writeOutput("size: " & lmap.size() & "<br>");

// ConcurrentLinkedQueue
writeOutput("6. ConcurrentLinkedQueue: ");
queue = createObject("java", "java.util.concurrent.ConcurrentLinkedQueue").init();
queue = queue.offer("a");
queue = queue.offer("b");
writeOutput("size: " & queue.size() & "<br>");

writeOutput("7. Queue peek: ");
result = queue.peek();
writeOutput(result & "<br>");

// Paths
writeOutput("8. Paths: ");
paths = createObject("java", "java.nio.file.Paths").get("/tmp/test.txt");
result = paths.toString();
writeOutput(result & "<br>");

suiteEnd();
</cfscript>