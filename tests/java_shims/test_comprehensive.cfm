<cfscript>
// Comprehensive Java shim exercise — written to match real Lucee Java-interop
// behaviour so the same code runs unchanged on RustCFML. Do not assign .put()
// / .offer() / .poll() results back to the map/queue variable: on Lucee
// those return previous-value / boolean / head (respectively) and reassigning
// over the receiver deletes or replaces it. Populate via standalone calls.

suiteBegin("Java Shims Comprehensive");

// Test StringBuilder
writeOutput("1. StringBuilder.init: ");
sb = createObject("java", "java.lang.StringBuilder").init("");
writeOutput("OK<br>");

writeOutput("2. StringBuilder.append: ");
sb.append("Hello");
sb.append(" ");
sb.append("World");
writeOutput("OK<br>");

writeOutput("3. StringBuilder.toString: ");
sbValue = sb.toString();
writeOutput("OK - " & sbValue & "<br>");

writeOutput("4. StringBuilder.length: ");
sbLen = sb.length();
writeOutput("OK - " & sbLen & "<br>");

// Test System — static class, no .init()
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
envHome = system.getenv("HOME");
writeOutput("OK - " & envHome & "<br>");

// Test File
writeOutput("9. File.init: ");
file = createObject("java", "java.io.File").init("/tmp");
writeOutput("OK<br>");

writeOutput("10. File.isAbsolute: ");
fIsAbs = file.isAbsolute();
writeOutput("OK - " & fIsAbs & "<br>");

writeOutput("11. File.getAbsolutePath: ");
fAbs = file.getAbsolutePath();
writeOutput("OK - " & fAbs & "<br>");

writeOutput("12. File.exists: ");
fExists = file.exists();
writeOutput("OK - " & fExists & "<br>");

writeOutput("13. File.isDirectory: ");
fIsDir = file.isDirectory();
writeOutput("OK - " & fIsDir & "<br>");

writeOutput("14. File.lastModified: ");
fLast = file.lastModified();
writeOutput("OK - " & fLast & "<br>");

// Test Thread
writeOutput("15. Thread.currentThread: ");
thread = createObject("java", "java.lang.Thread").currentThread();
writeOutput("OK<br>");

writeOutput("16. Thread.getName: ");
tName = thread.getName();
writeOutput("OK - " & tName & "<br>");

writeOutput("17. Thread.getPriority: ");
tPri = thread.getPriority();
writeOutput("OK - " & tPri & "<br>");

writeOutput("18. Thread.isDaemon: ");
tDaemon = thread.isDaemon();
writeOutput("OK - " & tDaemon & "<br>");

// Test TreeMap — populate via put(), not a struct literal (Lucee uppercases
// unquoted struct keys when converting to Java Map, breaking later .get()).
writeOutput("19. TreeMap.init: ");
map = createObject("java", "java.util.TreeMap").init();
map.put("z", 3);
map.put("a", 1);
map.put("m", 2);
writeOutput("OK<br>");

writeOutput("20. TreeMap.keySet: ");
keys = map.keySet().toArray();
writeOutput("OK - count: " & arrayLen(keys) & "<br>");

writeOutput("21. TreeMap.get: ");
tmGet = map.get("a");
writeOutput("OK - " & tmGet & "<br>");

writeOutput("22. TreeMap.size: ");
tmSize = map.size();
writeOutput("OK - " & tmSize & "<br>");

writeOutput("23. TreeMap.containsKey: ");
tmHas = map.containsKey("a");
writeOutput("OK - " & tmHas & "<br>");

// Test LinkedHashMap — do not reassign put() result (Lucee returns null).
writeOutput("24. LinkedHashMap.init: ");
lmap = createObject("java", "java.util.LinkedHashMap").init();
writeOutput("OK<br>");

writeOutput("25. LinkedHashMap.put: ");
lmap.put("first", 1);
lmap.put("second", 2);
lmap.put("third", 3);
writeOutput("OK<br>");

writeOutput("26. LinkedHashMap.keySet: ");
lkeys = lmap.keySet().toArray();
writeOutput("OK - count: " & arrayLen(lkeys) & "<br>");

writeOutput("27. LinkedHashMap.get: ");
lmGet = lmap.get("first");
writeOutput("OK - " & lmGet & "<br>");

writeOutput("28. LinkedHashMap.size: ");
lmSize = lmap.size();
writeOutput("OK - " & lmSize & "<br>");

// Test ConcurrentLinkedQueue
writeOutput("29. ConcurrentLinkedQueue.init: ");
queue = createObject("java", "java.util.concurrent.ConcurrentLinkedQueue").init();
writeOutput("OK<br>");

writeOutput("30. ConcurrentLinkedQueue.offer: ");
queue.offer("a");
queue.offer("b");
writeOutput("OK<br>");

writeOutput("31. ConcurrentLinkedQueue.size: ");
qSize = queue.size();
writeOutput("OK - " & qSize & "<br>");

writeOutput("32. ConcurrentLinkedQueue.peek: ");
qPeek = queue.peek();
writeOutput("OK - " & qPeek & "<br>");

writeOutput("33. ConcurrentLinkedQueue.poll: ");
qPoll = queue.poll();
writeOutput("OK - " & qPoll & "<br>");

// Test Paths — via File.toPath() (portable; Paths.get varargs doesn't dispatch on Lucee)
writeOutput("34. Paths.get: ");
paths = createObject("java", "java.io.File").init("/tmp/test.txt").toPath();
writeOutput("OK<br>");

writeOutput("35. Paths.getParent: ");
parent = paths.getParent();
writeOutput("OK<br>");

writeOutput("36. Paths.isAbsolute: ");
pIsAbs = paths.isAbsolute();
writeOutput("OK - " & pIsAbs & "<br>");

writeOutput("37. Paths.toString: ");
pStr = paths.toString();
writeOutput("OK - " & pStr & "<br>");

suiteEnd();
</cfscript>
