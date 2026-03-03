<cfscript>
suiteBegin("cfzip Tag");

// Setup: create a temp directory with test files
tmpDir = getTempDirectory() & "cfzip_test_" & createUUID();
directoryCreate(tmpDir);
fileWrite(tmpDir & "/file1.txt", "Hello World");
fileWrite(tmpDir & "/file2.txt", "Goodbye World");

zipFile = tmpDir & "/test.zip";
</cfscript>

<!--- Create zip from directory --->
<cfzip action="zip" file="#zipFile#" source="#tmpDir#" filter="*.txt">

<cfscript>
assertTrue("zip file created", fileExists(zipFile));
</cfscript>

<!--- List zip contents --->
<cfzip action="list" file="#zipFile#" name="qList">

<cfscript>
assertTrue("list returns query", isQuery(qList));
assertTrue("list has entries", qList.recordCount >= 2);
</cfscript>

<!--- Read entry as text --->
<cfset entryName = qList.name[1]>
<cfzip action="read" file="#zipFile#" entrypath="#entryName#" variable="content">

<cfscript>
assertTrue("read returns string", isSimpleValue(content));
</cfscript>

<!--- Unzip --->
<cfset unzipDir = tmpDir & "/unzipped">
<cfzip action="unzip" file="#zipFile#" destination="#unzipDir#">

<cfscript>
assertTrue("unzip directory created", directoryExists(unzipDir));
</cfscript>

<!--- ReadBinary --->
<cfzip action="readBinary" file="#zipFile#" entrypath="#entryName#" variable="binContent">

<cfscript>
assertTrue("readBinary returns binary", isBinary(binContent));
</cfscript>

<!--- Delete entry --->
<cfscript>
origCount = qList.recordCount;
</cfscript>
<cfzip action="delete" file="#zipFile#" entrypath="#entryName#">
<cfzip action="list" file="#zipFile#" name="qListAfter">

<cfscript>
assert("delete reduced count", qListAfter.recordCount, origCount - 1);

// Cleanup
directoryDelete(tmpDir, true);

suiteEnd();
</cfscript>
