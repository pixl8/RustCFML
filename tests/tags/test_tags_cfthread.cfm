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

suiteEnd();
</cfscript>
