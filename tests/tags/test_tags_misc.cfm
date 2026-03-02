<cfscript>suiteBegin("Tags: Miscellaneous");</cfscript>

<!--- cfsilent suppresses output --->
<cfsavecontent variable="silentOutput"><cfsilent>
    <cfset silentVar = "set inside silent">
    <cfoutput>This should not appear</cfoutput>
</cfsilent></cfsavecontent>
<cfscript>assert("cfsilent suppresses output", trim(silentOutput), "");</cfscript>

<!--- Variable set inside cfsilent is still accessible --->
<cfscript>assert("cfsilent var accessible", silentVar, "set inside silent");</cfscript>

<!--- cfthrow throws an exception (via assertThrows) --->
<cfscript>
assertThrows("cfthrow throws", function() {
    throw(message="test error", type="CustomError");
});
</cfscript>

<!--- cftry/cfcatch catches exceptions --->
<cftry>
    <cfthrow message="caught error" type="TestException">
    <cfcatch type="any">
        <cfset caughtMessage = cfcatch.message>
    </cfcatch>
</cftry>
<cfscript>assert("cftry/cfcatch catches", caughtMessage, "caught error");</cfscript>

<!--- cfcatch with specific type --->
<cftry>
    <cfthrow message="typed error" type="MyType">
    <cfcatch type="MyType">
        <cfset typedCatch = cfcatch.type>
    </cfcatch>
</cftry>
<cfscript>assert("cfcatch typed", typedCatch, "MyType");</cfscript>

<!--- cfcatch has message property --->
<cftry>
    <cfthrow message="has message" detail="has detail">
    <cfcatch type="any">
        <cfset catchHasMessage = len(cfcatch.message) GT 0>
    </cfcatch>
</cftry>
<cfscript>assertTrue("cfcatch has message", catchHasMessage);</cfscript>

<!--- cflock does not error --->
<cflock name="testlock" timeout="5" type="exclusive">
    <cfset lockVar = "locked">
</cflock>
<cfscript>assert("cflock executes body", lockVar, "locked");</cfscript>

<!--- cftry with no error --->
<cftry>
    <cfset noErrorVar = "success">
    <cfcatch type="any">
        <cfset noErrorVar = "failed">
    </cfcatch>
</cftry>
<cfscript>assert("cftry no error", noErrorVar, "success");</cfscript>

<cfscript>suiteEnd();</cfscript>
