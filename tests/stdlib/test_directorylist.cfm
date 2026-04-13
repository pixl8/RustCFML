<cfscript>
suiteBegin("directoryList");

// Create temp test structure
tmpDir = getTempDirectory() & "rustcfml_dirlist_test_" & createUUID();
directoryCreate(tmpDir);
directoryCreate(tmpDir & "/subdir1");
directoryCreate(tmpDir & "/subdir2");
fileWrite(tmpDir & "/file1.txt", "hello");
fileWrite(tmpDir & "/file2.cfm", "world");
fileWrite(tmpDir & "/subdir1/nested.txt", "nested");

// Default: list all (files AND directories), non-recursive, path mode
all = directoryList(tmpDir);
assertTrue("default lists files and dirs", arrayLen(all) >= 4); // subdir1, subdir2, file1.txt, file2.cfm

// Check directories are included
hasDir = false;
for (item in all) {
    if (find("subdir1", item)) hasDir = true;
}
assertTrue("directories included in results", hasDir);

// Non-recursive should not include nested files
hasNested = false;
for (item in all) {
    if (find("nested.txt", item)) hasNested = true;
}
assertFalse("non-recursive excludes nested", hasNested);

// Recursive
recursive = directoryList(tmpDir, true);
foundNested = false;
for (item in recursive) {
    if (find("nested.txt", item)) foundNested = true;
}
assertTrue("recursive includes nested files", foundNested);

// Name mode
names = directoryList(tmpDir, false, "name");
hasFileName = false;
for (item in names) {
    if (item == "file1.txt") hasFileName = true;
}
assertTrue("name mode returns filenames", hasFileName);

// Filter by extension - should only return matching files, not dirs
cfmOnly = directoryList(tmpDir, false, "name", "*.cfm");
assert("filter returns matching files", arrayLen(cfmOnly), 1);
assert("filter matches correct file", cfmOnly[1], "file2.cfm");

// Cleanup
directoryDelete(tmpDir, true);

suiteEnd();
</cfscript>
