<cfscript>suiteBegin("Tags: cfexecute");</cfscript>

<!--- Capture stdout to variable --->
<cfexecute name="echo" arguments="hello world" variable="execResult" timeout="10" />
<cfscript>assert("cfexecute capture stdout", trim(execResult), "hello world");</cfscript>

<!--- errorVariable captures stderr (may be empty) --->
<cfexecute name="echo" arguments="test" variable="out2" errorVariable="err2" timeout="10" />
<cfscript>
assert("cfexecute stdout", trim(out2), "test");
assertTrue("cfexecute errorVariable is string", isSimpleValue(err2));
</cfscript>

<!--- No variable = output to buffer (captured via savecontent) --->
<cfsavecontent variable="bufferOutput">
<cfexecute name="echo" arguments="buffered" timeout="10" />
</cfsavecontent>
<cfscript>assertTrue("cfexecute buffer output", find("buffered", trim(bufferOutput)) > 0);</cfscript>

<cfscript>suiteEnd();</cfscript>
