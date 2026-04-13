<cfscript>
// Test Java shims - File methods (simple output version)
writeOutput( "<h3>Java IO File Methods Test</h3>" );

// Test File init and basic methods
currentFile = createObject("java", "java.io.File").init( getCurrentTemplatePath() );
writeOutput( "File created: " & isObject(currentFile) & "<br>" );

// Test lastModified
lastMod = currentFile.lastModified();
writeOutput( "lastModified: " & lastMod & " (isNumeric: " & isNumeric(lastMod) & ", positive: " & (lastMod gt 0) & ")<br>" );

// Test exists
writeOutput( "exists: " & currentFile.exists() & "<br>" );

// Test isFile
writeOutput( "isFile: " & currentFile.isFile() & "<br>" );

// Test isDirectory
isDir = currentFile.isDirectory();
writeOutput( "isDirectory: " & isDir & "<br>" );

// Test getName
name = currentFile.getName();
writeOutput( "getName: " & name & " (len: " & len(name) & ")<br>" );

// Test getParent
parent = currentFile.getParent();
writeOutput( "getParent: " & parent & "<br>" );

// Test getAbsolutePath
absPath = currentFile.getAbsolutePath();
writeOutput( "getAbsolutePath: " & absPath & " (len: " & len(absPath) & ")<br>" );

// Test canRead
canRead = currentFile.canRead();
writeOutput( "canRead: " & canRead & "<br>" );

// Test length
len = currentFile.length();
writeOutput( "length: " & len & " (isNumeric: " & isNumeric(len) & ")<br>" );

writeOutput( "<p>All File tests completed!</p>" );
</cfscript>