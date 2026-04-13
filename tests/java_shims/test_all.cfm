<cfinclude template="../harness.cfm">
<cfscript>
suiteBegin( "Java Shims" );

// Test StringBuilder
writeOutput( "\n--- Testing StringBuilder ---\n" );
var sb = createObject( "java", "java.lang.StringBuilder" ).init( "" );
assertTrue( "StringBuilder init", isObject( sb ) );

sb.append( "Hello" ).append( " " ).append( "World" );
var result = sb.toString();
var test1 = result == "Hello World";
assertTrue( "StringBuilder append", test1 );
var test2 = sb.length() == 11;
assertTrue( "StringBuilder length", test2 );

// Test System
writeOutput( "\n--- Testing System ---\n" );
var system = createObject( "java", "java.lang.System" ).init();
assertTrue( "System init", isObject( system ) );

var osName = system.getProperty( "os.name" );
var test3 = len( osName ) > 0;
assertTrue( "System.getProperty os.name", test3 );

var fileSep = system.getProperty( "file.separator" );
var test4 = len( fileSep ) == 1;
assertTrue( "System.getProperty file.separator", test4 );

var userDir = system.getProperty( "user.dir" );
var test5 = len( userDir ) > 0;
assertTrue( "System.getProperty user.dir", test5 );

// Test System.out
var out = system.out;
assertTrue( "System.out exists", isObject( out ) );

// Test File
writeOutput( "\n--- Testing File ---\n" );
var file = createObject( "java", "java.io.File" ).init( "/tmp/test.txt" );
assertTrue( "File init", isObject( file ) );

assertTrue( "File isAbsolute", file.isAbsolute() );
var test6 = len( file.getCanonicalPath() ) > 0;
assertTrue( "File getCanonicalPath", test6 );
var test7 = file.toString() == "/tmp/test.txt";
assertTrue( "File toString", test7 );

// Test Thread
writeOutput( "\n--- Testing Thread ---\n" );
var thread = createObject( "java", "java.lang.Thread" ).currentThread();
assertTrue( "Thread currentThread", isObject( thread ) );

var threadName = thread.getName();
var test8 = threadName == "main";
assertTrue( "Thread getName", test8 );

var threadGroup = thread.getThreadGroup();
assertTrue( "Thread getThreadGroup", isObject( threadGroup ) );
var test9 = len( threadGroup.getName() ) > 0;
assertTrue( "ThreadGroup getName", test9 );

// Test TreeMap
writeOutput( "\n--- Testing TreeMap ---\n" );
var map = createObject( "java", "java.util.TreeMap" ).init( { z=3, a=1, m=2 } );
assertTrue( "TreeMap init", isObject( map ) );

var keys = map.keySet();
var test10 = arrayLen( keys ) == 3;
assertTrue( "TreeMap keySet count", test10 );
var test11 = keys[ 1 ] == "a";
assertTrue( "TreeMap sorted keys[1]", test11 );
var test12 = keys[ 2 ] == "m";
assertTrue( "TreeMap sorted keys[2]", test12 );
var test13 = keys[ 3 ] == "z";
assertTrue( "TreeMap sorted keys[3]", test13 );

var test14 = map.get( "a" ) == 1;
assertTrue( "TreeMap get", test14 );
var test15 = map.size() == 3;
assertTrue( "TreeMap size", test15 );
assertTrue( "TreeMap containsKey", map.containsKey( "a" ) );

// Test LinkedHashMap
writeOutput( "\n--- Testing LinkedHashMap ---\n" );
var lmap = createObject( "java", "java.util.LinkedHashMap" ).init();
assertTrue( "LinkedHashMap init", isObject( lmap ) );

lmap.put( "first", 1 );
lmap.put( "second", 2 );
lmap.put( "third", 3 );

var lkeys = lmap.keySet();
var test16 = arrayLen( lkeys ) == 3;
assertTrue( "LinkedHashMap keySet count", test16 );
var test17 = lkeys[ 1 ] == "first";
assertTrue( "LinkedHashMap order[1]", test17 );
var test18 = lkeys[ 2 ] == "second";
assertTrue( "LinkedHashMap order[2]", test18 );
var test19 = lkeys[ 3 ] == "third";
assertTrue( "LinkedHashMap order[3]", test19 );

var test20 = lmap.get( "first" ) == 1;
assertTrue( "LinkedHashMap get", test20 );
var test21 = lmap.size() == 3;
assertTrue( "LinkedHashMap size", test21 );

// Test ConcurrentLinkedQueue
writeOutput( "\n--- Testing ConcurrentLinkedQueue ---\n" );
var queue = createObject( "java", "java.util.concurrent.ConcurrentLinkedQueue" ).init();
assertTrue( "Queue init", isObject( queue ) );

queue.offer( "a" );
queue.offer( "b" );
queue.offer( "c" );

var test22 = queue.size() == 3;
assertTrue( "Queue size", test22 );
var test23 = queue.peek() == "a";
assertTrue( "Queue peek", test23 );
var test24 = queue.poll() == "a";
assertTrue( "Queue poll", test24 );
var test25 = queue.size() == 2;
assertTrue( "Queue size after poll", test25 );

// Test Paths
writeOutput( "\n--- Testing Paths ---\n" );
var paths = createObject( "java", "java.nio.file.Paths" ).get( "/home/user/file.txt" );
assertTrue( "Paths get", isObject( paths ) );

var parent = paths.getParent();
assertTrue( "Path getParent", isObject( parent ) );
var test26 = parent.toString() == "/home/user";
assertTrue( "Parent toString", test26 );

assertTrue( "Path isAbsolute", paths.isAbsolute() );
var test27 = paths.toString() == "/home/user/file.txt";
assertTrue( "Path toString", test27 );

writeOutput( "\n=== All Java shim tests passed! ===\n" );

suiteEnd( "Java Shims" );
</cfscript>
