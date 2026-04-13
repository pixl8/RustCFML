<cfscript>
// Test Java shims - System class static methods
// Note: In real Java, System is a final class and cannot be instantiated.
// You call static methods directly on the class without init()
include "../harness.cfm";

suiteBegin("Java System Static Methods");

// Test currentTimeMillis (static method) - no init() needed on real Java
javaSystem = createObject("java", "java.lang.System");
currentTime = javaSystem.currentTimeMillis();
assertTrue("currentTimeMillis returns numeric", isNumeric(currentTime));
assertTrue("currentTimeMillis is positive", currentTime gt 0);

// Test getProperty (static method)
osName = javaSystem.getProperty("os.name");
assertTrue("getProperty returns string", isSimpleValue(osName));
assertTrue("os.name is not empty", len(osName) gt 0);

// Test getProperty for different keys
fileSep = javaSystem.getProperty("file.separator");
assertTrue("file.separator is / or \\", fileSep eq "/" or fileSep eq "\\");

userDir = javaSystem.getProperty("user.dir");
assertTrue("user.dir returns string", isSimpleValue(userDir));

// Test getEnv (returns a Map-like object)
env = javaSystem.getEnv();
assertTrue("getEnv returns something", isStruct(env) or isObject(env));

// Test nanoTime (static method)
nano = javaSystem.nanoTime();
assertTrue("nanoTime returns numeric", isNumeric(nano));
assertTrue("nanoTime is positive", nano gt 0);

// Test System.out (static field) - on real Java, access via createObject without init
out = javaSystem.out;
assertTrue("System.out exists", isObject(out));

suiteEnd();
</cfscript>