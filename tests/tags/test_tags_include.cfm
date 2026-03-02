<cfscript>suiteBegin("Tags: Include");</cfscript>

<!--- cfinclude of a helper file --->
<cfinclude template="_include_target.cfm">
<cfscript>assert("cfinclude sets variable", request._includeTest, "included");</cfscript>

<!--- Verify the type of the included value --->
<cfscript>assertTrue("cfinclude value is string", isSimpleValue(request._includeTest));</cfscript>

<!--- Verify we can use the value after include --->
<cfset includeUpper = uCase(request._includeTest)>
<cfscript>assert("cfinclude value usable", includeUpper, "INCLUDED");</cfscript>

<cfscript>suiteEnd();</cfscript>
