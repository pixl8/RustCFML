// Lucee 7 Compatibility Tests: File/Directory Functions
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness
<cfscript>
suiteBegin("Lucee7: File/Directory Functions");

tempDir = getTempDirectory();
testFile = tempDir & "rustcfml_lucee7_test_" & createUUID() & ".txt";
testDir = tempDir & "rustcfml_lucee7_dir_" & createUUID();

try {

    // ============================================================
    // FileWrite / FileExists / FileRead (from Lucee FileWrite.cfc, FileExists.cfc)
    // ============================================================
    fileWrite(testFile, "hello world");
    assertTrue("file exists after write", fileExists(testFile));
    assert("file read matches write", fileRead(testFile), "hello world");

    // ============================================================
    // FileAppend (from Lucee FileAppend.cfc)
    // ============================================================
    fileAppend(testFile, " more");
    assert("file append", fileRead(testFile), "hello world more");

    // ============================================================
    // GetFileInfo (from Lucee GetFileInfo.cfc)
    // ============================================================
    info = getFileInfo(testFile);
    assertTrue("getFileInfo returns struct", isStruct(info));
    assertTrue("has size key", structKeyExists(info, "size"));

    // ============================================================
    // FileCopy (from Lucee FileCopy.cfc)
    // ============================================================
    copyFile = testFile & ".copy";
    fileCopy(testFile, copyFile);
    assertTrue("copy exists", fileExists(copyFile));
    fileDelete(copyFile);

    // ============================================================
    // FileMove (from Lucee FileMove.cfc)
    // ============================================================
    movedFile = testFile & ".moved";
    fileMove(testFile, movedFile);
    assertTrue("moved exists", fileExists(movedFile));
    assertFalse("original gone", fileExists(testFile));
    fileMove(movedFile, testFile);

    // ============================================================
    // FileDelete (from Lucee FileDelete.cfc)
    // ============================================================
    fileDelete(testFile);
    assertFalse("deleted", fileExists(testFile));

    // ============================================================
    // DirectoryCreate / DirectoryExists (from Lucee DirectoryCreate.cfc)
    // ============================================================
    directoryCreate(testDir);
    assertTrue("dir exists", directoryExists(testDir));

    // ============================================================
    // DirectoryList (from Lucee DirectoryList.cfc)
    // ============================================================
    fileWrite(testDir & "/test.txt", "content");
    listing = directoryList(testDir);
    assertTrue("listing is array or query", isArray(listing) || isQuery(listing));

    // ============================================================
    // DirectoryDelete (from Lucee DirectoryDelete.cfc)
    // ============================================================
    directoryDelete(testDir, true);
    assertFalse("dir deleted", directoryExists(testDir));

    // ============================================================
    // Path Functions (from Lucee GetDirectoryFromPath.cfc, etc.)
    // ============================================================
    path = "/path/to/file.txt";
    assert("getDirectoryFromPath", getDirectoryFromPath(path), "/path/to/");
    assert("getFileFromPath", getFileFromPath(path), "file.txt");
    assertTrue("getTempDirectory not empty", len(getTempDirectory()) > 0);
    assertTrue("expandPath returns string", len(expandPath(".")) > 0);
    tmpFile = getTempFile(getTempDirectory(), "rcfml");
    assertTrue("getTempFile returns string", len(tmpFile) > 0);

} finally {
    // Cleanup
    try { if (fileExists(testFile)) fileDelete(testFile); } catch (any e) {}
    try { if (fileExists(testFile & ".copy")) fileDelete(testFile & ".copy"); } catch (any e) {}
    try { if (fileExists(testFile & ".moved")) fileDelete(testFile & ".moved"); } catch (any e) {}
    try { if (directoryExists(testDir)) directoryDelete(testDir, true); } catch (any e) {}
    try { if (len(tmpFile) > 0 && fileExists(tmpFile)) fileDelete(tmpFile); } catch (any e) {}
}

suiteEnd();
</cfscript>
