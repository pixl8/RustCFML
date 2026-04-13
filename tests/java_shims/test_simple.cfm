<cfscript>
writeOutput( "\n=== Java Shims Simple Test ===\n" );

// Test StringBuilder
writeOutput( "\n--- Testing StringBuilder ---\n" );
var sb = createObject( "java", "java.lang.StringBuilder" ).init( "" );
writeOutput( "StringBuilder created: " & isObject( sb ) & "\n" );

sb.append( "Hello" ).append( " " ).append( "World" );
var result = sb.toString();
writeOutput( "StringBuilder result: " & result & "\n" );
writeOutput( "StringBuilder length: " & sb.length() & "\n" );

// Test System
writeOutput( "\n--- Testing System ---\n" );
var system = createObject( "java", "java.lang.System" ).init();
writeOutput( "System created: " & isObject( system ) & "\n" );

var osName = system.getProperty( "os.name" );
writeOutput( "OS Name: " & osName & "\n" );

var fileSep = system.getProperty( "file.separator" );
writeOutput( "File separator: " & fileSep & "\n" );

var userDir = system.getProperty( "user.dir" );
writeOutput( "User dir: " & userDir & "\n" );

// Test System.out
var out = system.out;
writeOutput( "System.out exists: " & isObject( out ) & "\n" );

// Test File
writeOutput( "\n--- Testing File ---\n" );
var file = createObject( "java", "java.io.File" ).init( "/tmp/test.txt" );
writeOutput( "File created: " & isObject( file ) & "\n" );
writeOutput( "File isAbsolute: " & file.isAbsolute() & "\n" );
writeOutput( "File canonicalPath: " & file.getCanonicalPath() & "\n" );
writeOutput( "File toString: " & file.toString() & "\n" );

// Test Thread
writeOutput( "\n--- Testing Thread ---\n" );
var thread = createObject( "java", "java.lang.Thread" ).currentThread();
writeOutput( "Thread created: " & isObject( thread ) & "\n" );
writeOutput( "Thread name: " & thread.getName() & "\n" );

var threadGroup = thread.getThreadGroup();
writeOutput( "Thread group exists: " & isObject( threadGroup ) & "\n" );
writeOutput( "Thread group name: " & threadGroup.getName() & "\n" );

// Test TreeMap
writeOutput( "\n--- Testing TreeMap ---\n" );
var map = createObject( "java", "java.util.TreeMap" ).init( { z=3, a=1, m=2 } );
writeOutput( "TreeMap created: " & isObject( map ) & "\n" );

var keys = map.keySet();
writeOutput( "TreeMap key count: " & arrayLen( keys ) & "\n" );
writeOutput( "TreeMap keys[1]: " & keys[ 1 ] & "\n" );
writeOutput( "TreeMap keys[2]: " & keys[ 2 ] & "\n" );
writeOutput( "TreeMap keys[3]: " & keys[ 3 ] & "\n" );

writeOutput( "TreeMap get('a'): " & map.get( "a" ) & "\n" );
writeOutput( "TreeMap size: " & map.size() & "\n" );
writeOutput( "TreeMap containsKey('a'): " & map.containsKey( "a" ) & "\n" );

// Test LinkedHashMap
writeOutput( "\n--- Testing LinkedHashMap ---\n" );
var lmap = createObject( "java", "java.util.LinkedHashMap" ).init();
writeOutput( "LinkedHashMap created: " & isObject( lmap ) & "\n" );

// Capture return values for method chaining (immutable pattern)
lmap = lmap.put( "first", 1 );
lmap = lmap.put( "second", 2 );
lmap = lmap.put( "third", 3 );

var lkeys = lmap.keySet();
writeOutput( "LinkedHashMap key count: " & arrayLen( lkeys ) & "\n" );
writeOutput( "LinkedHashMap keys[1]: " & lkeys[ 1 ] & "\n" );
writeOutput( "LinkedHashMap keys[2]: " & lkeys[ 2 ] & "\n" );
writeOutput( "LinkedHashMap keys[3]: " & lkeys[ 3 ] & "\n" );

writeOutput( "LinkedHashMap get('first'): " & lmap.get( "first" ) & "\n" );
writeOutput( "LinkedHashMap size: " & lmap.size() & "\n" );

// Test ConcurrentLinkedQueue
writeOutput( "\n--- Testing ConcurrentLinkedQueue ---\n" );
var queue = createObject( "java", "java.util.concurrent.ConcurrentLinkedQueue" ).init();
writeOutput( "Queue created: " & isObject( queue ) & "\n" );

// Capture return values for method chaining (immutable pattern)
queue = queue.offer( "a" );
queue = queue.offer( "b" );
queue = queue.offer( "c" );

writeOutput( "Queue size: " & queue.size() & "\n" );
writeOutput( "Queue peek: " & queue.peek() & "\n" );
queue = queue.poll();
writeOutput( "Queue size after poll: " & queue.size() & "\n" );

// Test Paths
writeOutput( "\n--- Testing Paths ---\n" );
var paths = createObject( "java", "java.nio.file.Paths" ).get( "/home/user/file.txt" );
writeOutput( "Paths created: " & isObject( paths ) & "\n" );

var parent = paths.getParent();
writeOutput( "Parent exists: " & isObject( parent ) & "\n" );
writeOutput( "Parent toString: " & parent.toString() & "\n" );

writeOutput( "Path isAbsolute: " & paths.isAbsolute() & "\n" );
writeOutput( "Path toString: " & paths.toString() & "\n" );

writeOutput( "\n=== All Java shims loaded successfully! ===\n" );
</cfscript>
