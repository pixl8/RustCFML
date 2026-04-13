<cfscript>
suiteBegin("More Java Shims");

// StringBuilder — chained append() works on both engines (real Java returns this).
writeOutput("1. StringBuilder: ");
sb = createObject("java", "java.lang.StringBuilder").init("Hello");
sb.append(" World");
sbValue = sb.toString();
writeOutput(sbValue & "<br>");

writeOutput("2. StringBuilder length: ");
sbLen = sb.length();
writeOutput(sbLen & "<br>");

// TreeMap — populate via put() so key casing is preserved on Lucee.
writeOutput("3. TreeMap: ");
map = createObject("java", "java.util.TreeMap").init();
map.put("z", 3);
map.put("a", 1);
map.put("m", 2);
keys = map.keySet().toArray();
writeOutput("keys: " & arrayLen(keys) & "<br>");

writeOutput("4. TreeMap get: ");
tmGet = map.get("a");
writeOutput(tmGet & "<br>");

// LinkedHashMap — do NOT reassign put() result (Lucee returns null).
writeOutput("5. LinkedHashMap: ");
lmap = createObject("java", "java.util.LinkedHashMap").init();
lmap.put("first", 1);
lmap.put("second", 2);
writeOutput("size: " & lmap.size() & "<br>");

// ConcurrentLinkedQueue — same: don't reassign offer() return (boolean in Lucee).
writeOutput("6. ConcurrentLinkedQueue: ");
queue = createObject("java", "java.util.concurrent.ConcurrentLinkedQueue").init();
queue.offer("a");
queue.offer("b");
writeOutput("size: " & queue.size() & "<br>");

writeOutput("7. Queue peek: ");
qPeek = queue.peek();
writeOutput(qPeek & "<br>");

// Paths — via File.toPath() (portable)
writeOutput("8. Paths: ");
paths = createObject("java", "java.io.File").init("/tmp/test.txt").toPath();
pStr = paths.toString();
writeOutput(pStr & "<br>");

suiteEnd();
</cfscript>
