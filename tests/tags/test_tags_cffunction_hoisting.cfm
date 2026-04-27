<cfscript>suiteBegin("Tags: cffunction template hoisting");</cfscript>

<!--- Standard CFML hoists template-level <cffunction> declarations:
      a function defined at the bottom of a template must be callable
      from anywhere above it. Reproduces the Taffy dashboard.cfm
      `getDocUrl undefined` failure. --->

<cfsavecontent variable="hoistOutput"><cfoutput>#greet("world")#</cfoutput></cfsavecontent>
<cfscript>assert("forward call to <cffunction> in cfoutput", trim(hoistOutput), "Hello, world");</cfscript>

<cfset earlyResult = addOne(41)>
<cfscript>assert("forward call to <cffunction> via cfset", earlyResult, 42);</cfscript>

<cffunction name="greet">
    <cfargument name="who" />
    <cfreturn "Hello, " & arguments.who />
</cffunction>

<cffunction name="addOne">
    <cfargument name="n" />
    <cfreturn arguments.n + 1 />
</cffunction>

<!--- Calls placed AFTER the declarations should also still work, of course. --->
<cfscript>
    assert("post-decl call still works", greet("again"), "Hello, again");
    assert("post-decl numeric still works", addOne(99), 100);
</cfscript>

<cfscript>suiteEnd();</cfscript>
