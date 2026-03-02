<cfscript>suiteBegin("Tags: Basic");</cfscript>

<!--- cfset simple assignment --->
<cfset myVar = 42>
<cfscript>assert("cfset numeric", myVar, 42);</cfscript>

<!--- cfset with expression --->
<cfset result = 10 + 20>
<cfscript>assert("cfset expression", result, 30);</cfscript>

<!--- cfset string assignment --->
<cfset greeting = "hello">
<cfscript>assert("cfset string", greeting, "hello");</cfscript>

<!--- cfset boolean --->
<cfset flag = true>
<cfscript>assertTrue("cfset boolean", flag);</cfscript>

<!--- cfoutput with variable interpolation --->
<cfset name = "World">
<cfsavecontent variable="outputResult"><cfoutput>Hello #name#</cfoutput></cfsavecontent>
<cfscript>assert("cfoutput interpolation", trim(outputResult), "Hello World");</cfscript>

<!--- cfoutput with expression --->
<cfsavecontent variable="exprOutput"><cfoutput>#3 + 4#</cfoutput></cfsavecontent>
<cfscript>assert("cfoutput expression", trim(exprOutput), "7");</cfscript>

<!--- cfif / cfelse --->
<cfset x = 10>
<cfif x GT 5>
    <cfset ifResult = "greater">
<cfelse>
    <cfset ifResult = "less">
</cfif>
<cfscript>assert("cfif GT true branch", ifResult, "greater");</cfscript>

<!--- cfif false branch --->
<cfset x = 2>
<cfif x GT 5>
    <cfset ifResult2 = "greater">
<cfelse>
    <cfset ifResult2 = "less">
</cfif>
<cfscript>assert("cfif GT false branch", ifResult2, "less");</cfscript>

<!--- cfelseif --->
<cfset x = 5>
<cfif x GT 10>
    <cfset ifResult3 = "big">
<cfelseif x EQ 5>
    <cfset ifResult3 = "five">
<cfelse>
    <cfset ifResult3 = "small">
</cfif>
<cfscript>assert("cfelseif", ifResult3, "five");</cfscript>

<!--- cfloop index (for loop) --->
<cfset loopResult = "">
<cfloop index="i" from="1" to="5">
    <cfset loopResult = loopResult & i>
</cfloop>
<cfscript>assert("cfloop index", loopResult, "12345");</cfscript>

<!--- cfloop array --->
<cfset arr = [10, 20, 30]>
<cfset arrSum = 0>
<cfloop array="#arr#" index="item">
    <cfset arrSum = arrSum + item>
</cfloop>
<cfscript>assert("cfloop array", arrSum, 60);</cfscript>

<!--- cfloop condition (while) --->
<cfset wResult = "">
<cfset wCounter = 1>
<cfloop condition="wCounter LTE 3">
    <cfset wResult = wResult & wCounter>
    <cfset wCounter = wCounter + 1>
</cfloop>
<cfscript>assert("cfloop condition", wResult, "123");</cfscript>

<!--- cfloop list --->
<cfset listResult = "">
<cfloop list="a,b,c" index="item">
    <cfset listResult = listResult & item>
</cfloop>
<cfscript>assert("cfloop list", listResult, "abc");</cfscript>

<!--- cfloop struct --->
<cfset s = {x: 1, y: 2}>
<cfset structKeyCount = 0>
<cfloop collection="#s#" item="key">
    <cfset structKeyCount = structKeyCount + 1>
</cfloop>
<cfscript>assert("cfloop struct key count", structKeyCount, 2);</cfscript>

<!--- cfloop with step --->
<cfset stepResult = "">
<cfloop index="i" from="0" to="10" step="2">
    <cfset stepResult = stepResult & i>
</cfloop>
<cfscript>assert("cfloop step", stepResult, "0246810");</cfscript>

<cfscript>suiteEnd();</cfscript>
