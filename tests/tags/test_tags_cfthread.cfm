<cfscript>
suiteBegin("cfthread Tag");
</cfscript>

<!--- cfthread action=run (default) --->
<cfthread name="t1">
    <cfset request.threadTest1 = "hello from thread">
</cfthread>

<cfscript>
assert("cfthread run sets variable", request.threadTest1, "hello from thread");
</cfscript>

<!--- cfthread with explicit action=run --->
<cfthread name="t2" action="run">
    <cfset request.threadTest2 = "explicit run">
</cfthread>

<cfscript>
assert("cfthread explicit action=run", request.threadTest2, "explicit run");
</cfscript>

<!--- cfthread join (no-op since sequential) --->
<cfthread action="join" name="t1" timeout="1000"/>

<cfscript>
assert("cfthread join completes", true, true);
</cfscript>

<!--- cfthread terminate (no-op since already complete) --->
<cfthread action="terminate" name="t1"/>

<cfscript>
assert("cfthread terminate completes", true, true);

// Check thread scope metadata
assertTrue("cfthread scope exists", isDefined("cfthread"));
assertTrue("cfthread.t1 exists", isDefined("cfthread.t1"));
assert("cfthread.t1 status", cfthread.t1.status, "COMPLETED");
</cfscript>

<!--- Output capture --->
<cfthread name="t3">
    <cfoutput>hello from thread</cfoutput>
</cfthread>

<cfscript>
assert("cfthread output capture", cfthread.t3.output, "hello from thread");
</cfscript>

<!--- Error capture --->
<cfthread name="t4">
    <cfthrow message="thread error test">
</cfthread>

<cfscript>
assertTrue("cfthread error captured", len(cfthread.t4.error) > 0);
assert("cfthread status after error", cfthread.t4.status, "COMPLETED");
</cfscript>

<!--- Elapsed time --->
<cfscript>
assertTrue("cfthread elapsedtime is numeric", isNumeric(cfthread.t3.elapsedtime));
</cfscript>

<!--- Thread scope --->
<cfthread name="t5">
    <cfset thread.result = "done">
    <cfset thread.count = 42>
</cfthread>

<cfscript>
assert("cfthread thread scope string", cfthread.t5.result, "done");
assert("cfthread thread scope number", cfthread.t5.count, 42);

suiteEnd();
</cfscript>
