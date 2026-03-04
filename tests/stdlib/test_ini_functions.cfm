<cfscript>
suiteBegin("INI File Functions");

tempDir = getTempDirectory();
iniFile = tempDir & "rustcfml_test_" & createUUID() & ".ini";

// --- setProfileString creates file and writes entry ---
setProfileString(iniFile, "Database", "server", "localhost");
assertTrue("setProfileString creates file", fileExists(iniFile));

// --- getProfileString reads value back ---
assert("getProfileString reads value", getProfileString(iniFile, "Database", "server"), "localhost");

// --- setProfileString adds more entries ---
setProfileString(iniFile, "Database", "port", "3306");
setProfileString(iniFile, "Database", "name", "mydb");
assert("read port", getProfileString(iniFile, "Database", "port"), "3306");
assert("read name", getProfileString(iniFile, "Database", "name"), "mydb");

// --- setProfileString updates existing entry ---
setProfileString(iniFile, "Database", "port", "5432");
assert("updated port", getProfileString(iniFile, "Database", "port"), "5432");

// --- multiple sections ---
setProfileString(iniFile, "App", "title", "MyApp");
setProfileString(iniFile, "App", "version", "1.0");
assert("read from App section", getProfileString(iniFile, "App", "title"), "MyApp");
assert("Database still intact", getProfileString(iniFile, "Database", "server"), "localhost");

// --- case-insensitive section and key lookup ---
assert("CI section lookup", getProfileString(iniFile, "database", "SERVER"), "localhost");
assert("CI key lookup", getProfileString(iniFile, "DATABASE", "port"), "5432");

// --- missing entry returns empty string ---
assert("missing entry", getProfileString(iniFile, "Database", "nonexistent"), "");
assert("missing section", getProfileString(iniFile, "NoSection", "key"), "");

// --- getProfileSections ---
sections = getProfileSections(iniFile);
assertTrue("sections is struct", isStruct(sections));
assertTrue("has Database section", structKeyExists(sections, "Database"));
assertTrue("has App section", structKeyExists(sections, "App"));
assertTrue("Database keys contains server", listFind(sections["Database"], "server") > 0);
assertTrue("App keys contains title", listFind(sections["App"], "title") > 0);

// --- values with spaces and special characters ---
setProfileString(iniFile, "Paths", "root", "C:\Program Files\MyApp");
assert("value with spaces", getProfileString(iniFile, "Paths", "root"), "C:\Program Files\MyApp");

setProfileString(iniFile, "Paths", "url", "https://example.com/api?key=abc");
assert("value with special chars", getProfileString(iniFile, "Paths", "url"), "https://example.com/api?key=abc");

// Cleanup
fileDelete(iniFile);

suiteEnd();
</cfscript>
