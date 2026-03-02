<cfscript>suiteBegin("Tags: Custom Tags");</cfscript>

<!--- Test 1: Self-closing cf_ prefix tag --->
<cfsavecontent variable="ct1"><cf_greeting name="World"></cfsavecontent>
<cfscript>assertTrue("cf_ self-closing greeting", findNoCase("Hello, World!", ct1) GT 0);</cfscript>

<!--- Test 2: cfmodule template= form --->
<cfsavecontent variable="ct2"><cfmodule template="customtags/greeting.cfm" name="Test"></cfsavecontent>
<cfscript>assertTrue("cfmodule template greeting", findNoCase("Hello, Test!", ct2) GT 0);</cfscript>

<!--- Test 3: caller write-back with cf_ prefix --->
<cf_setter value="hello from tag">
<cfscript>assert("cf_ caller writeback", result, "hello from tag");</cfscript>

<!--- Test 4: caller write-back with cfmodule --->
<cfmodule template="customtags/setter.cfm" value="cfmodule setter">
<cfscript>assert("cfmodule caller writeback", result, "cfmodule setter");</cfscript>

<!--- Test 5: Body tag with cf_ prefix --->
<cfsavecontent variable="ct5"><cf_wrapper>inner</cf_wrapper></cfsavecontent>
<cfscript>assertTrue("cf_ body tag has div", findNoCase("<div", ct5) GT 0);</cfscript>
<cfscript>assertTrue("cf_ body tag has content", findNoCase("inner", ct5) GT 0);</cfscript>

<!--- Test 6: Body tag with cfmodule --->
<cfsavecontent variable="ct6"><cfmodule template="customtags/wrapper.cfm">module body</cfmodule></cfsavecontent>
<cfscript>assertTrue("cfmodule body tag has div", findNoCase("<div", ct6) GT 0);</cfscript>
<cfscript>assertTrue("cfmodule body tag has content", findNoCase("module body", ct6) GT 0);</cfscript>

<!--- Test 7: Missing custom tag throws error --->
<cfset errorThrown = false>
<cftry>
    <cf_nonexistent_tag_xyz>
    <cfcatch type="any">
        <cfset errorThrown = true>
    </cfcatch>
</cftry>
<cfscript>assertTrue("missing custom tag throws error", errorThrown);</cfscript>

<cfscript>suiteEnd();</cfscript>
