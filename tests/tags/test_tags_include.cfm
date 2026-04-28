<cfscript>suiteBegin("Tags: Include");</cfscript>

<!--- cfinclude of a helper file --->
<cfinclude template="_include_target.cfm">
<cfscript>assert("cfinclude sets variable", request._includeTest, "included");</cfscript>

<!--- Verify the type of the included value --->
<cfscript>assertTrue("cfinclude value is string", isSimpleValue(request._includeTest));</cfscript>

<!--- Verify we can use the value after include --->
<cfset includeUpper = uCase(request._includeTest)>
<cfscript>assert("cfinclude value usable", includeUpper, "INCLUDED");</cfscript>

<!--- Bug H: cfinclude path with .. segments must canonicalise.
      `customtags/../_include_target.cfm` should resolve to
      `_include_target.cfm` in the same directory. --->
<cfset request._includeTest = "">
<cfinclude template="customtags/../_include_target.cfm">
<cfscript>assert("cfinclude canonicalises .. segments", request._includeTest, "included");</cfscript>

<cfscript>suiteEnd();</cfscript>
