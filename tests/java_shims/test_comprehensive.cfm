<cfscript>
// Test Java shims
include "../harness.cfm";

var sb = "";
var result = "";
var system = "";
var file = "";
var thread = "";
var map = "";
var lmap = "";
var queue = "";
var paths = "";
var osName = "";
var fileSep = "";
var userDir = "";
var out = "";
var keys = "";
var lkeys = "";
var parent = "";

suiteBegin("Java Shims Comprehensive");

// Test StringBuilder
writeOutput("1. StringBuilder.init: ");
sb = createObject("java", "java.lang.StringBuilder").init("");
writeOutput("OK<br>");

writeOutput("2. StringBuilder.append: ");
sb = sb.append("Hello");
sb = sb.append(" ");
sb = sb.append("World");
writeOutput("OK<br>");

writeOutput("3. StringBuilder.toString: ");
result = sb.toString();
writeOutput("OK - " & result & "<br>");

writeOutput("4. StringBuilder.length: ");
result = sb.length();
writeOutput("OK - " & result & "<br>");

// Test System
writeOutput("5. System.getProperty os.name: ");
system = createObject("java", "java.lang.System");
osName = system.getProperty("os.name");
writeOutput("OK - " & osName & "<br>");

writeOutput("6. System.getProperty file.separator: ");
fileSep = system.getProperty("file.separator");
writeOutput("OK - " & fileSep & "<br>");

writeOutput("7. System.getProperty user.dir: ");
userDir = system.getProperty("user.dir");
writeOutput("OK - " & userDir & "<br>");

writeOutput("8. System.getenv HOME: ");
var envHome = system.getenv("HOME");
writeOutput("OK - " & envHome & "<br>");

// Test File
writeOutput("9. File.init: ");
file = createObject("java", "java.io.File").init("/tmp");
writeOutput("OK<br>");

writeOutput("10. File.isAbsolute: ");
result = file.isAbsolute();
writeOutput("OK - " & result & "<br>");

writeOutput("11. File.getAbsolutePath: ");
result = file.getAbsolutePath();
writeOutput("OK - " & result & "<br>");

writeOutput("12. File.exists: ");
result = file.exists();
writeOutput("OK - " & result & "<br>");

writeOutput("13. File.isDirectory: ");
result = file.isDirectory();
writeOutput("OK - " & result & "<br>");

writeOutput("14. File.lastModified: ");
result = file.lastModified();
writeOutput("OK - " & result & "<br>");

// Test Thread
writeOutput("15. Thread.currentThread: ");
thread = createObject("java", "java.lang.Thread").currentThread();
writeOutput("OK<br>");

writeOutput("16. Thread.getName: ");
result = thread.getName();
writeOutput("OK - " & result & "<br>");

writeOutput("17. Thread.getPriority: ");
result = thread.getPriority();
writeOutput("OK - " & result & "<br>");

writeOutput("18. Thread.isDaemon: ");
result = thread.isDaemon();
writeOutput("OK - " & result & "<br>");

// Test TreeMap
writeOutput("19. TreeMap.init: ");
map = createObject("java", "java.util.TreeMap").init({z=3, a=1, m=2});
writeOutput("OK<br>");

writeOutput("20. TreeMap.keySet: ");
keys = map.keySet();
writeOutput("OK - count: " & arrayLen(keys) & "<br>");

writeOutput("21. TreeMap.get: ");
result = map.get("a");
writeOutput("OK - " & result & "<br>");

writeOutput("22. TreeMap.size: ");
result = map.size();
writeOutput("OK - " & result & "<br>");

writeOutput("23. TreeMap.containsKey: ");
result = map.containsKey("a");
writeOutput("OK - " & result & "<br>");

// Test LinkedHashMap
writeOutput("24. LinkedHashMap.init: ");
lmap = createObject("java", "java.util.LinkedHashMap").init();
writeOutput("OK<br>");

writeOutput("25. LinkedHashMap.put: ");
lmap = lmap.put("first", 1);
lmap = lmap.put("second", 2);
lmap = lmap.put("third", 3);
writeOutput("OK<br>");

writeOutput("26. LinkedHashMap.keySet: ");
lkeys = lmap.keySet();
writeOutput("OK - count: " & arrayLen(lkeys) & "<br>");

writeOutput("27. LinkedHashMap.get: ");
result = lmap.get("first");
writeOutput("OK - " & result & "<br>");

writeOutput("28. LinkedHashMap.size: ");
result = lmap.size();
writeOutput("OK - " & result & "<br>");

// Test ConcurrentLinkedQueue
writeOutput("29. ConcurrentLinkedQueue.init: ");
queue = createObject("java", "java.util.concurrent.ConcurrentLinkedQueue").init();
writeOutput("OK<br>");

writeOutput("30. ConcurrentLinkedQueue.offer: ");
queue = queue.offer("a");
queue = queue.offer("b");
writeOutput("OK<br>");

writeOutput("31. ConcurrentLinkedQueue.size: ");
result = queue.size();
writeOutput("OK - " & result & "<br>");

writeOutput("32. ConcurrentLinkedQueue.peek: ");
result = queue.peek();
writeOutput("OK - " & result & "<br>");

writeOutput("33. ConcurrentLinkedQueue.poll: ");
queue = queue.poll();
writeOutput("OK<br>");

// Test Paths
writeOutput("34. Paths.get: ");
paths = createObject("java", "java.nio.file.Paths").get("/tmp/test.txt");
writeOutput("OK<br>");

writeOutput("35. Paths.getParent: ");
parent = paths.getParent();
writeOutput("OK<br>");

writeOutput("36. Paths.isAbsolute: ");
result = paths.isAbsolute();
writeOutput("OK - " & result & "<br>");

writeOutput("37. Paths.toString: ");
result = paths.toString();
writeOutput("OK - " & result & "<br>");

suiteEnd();
</cfscript>