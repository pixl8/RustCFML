<cfscript>suiteBegin("Tags: Savecontent");</cfscript>

<!--- cfsavecontent captures literal text --->
<cfsavecontent variable="captured">Hello there</cfsavecontent>
<cfscript>assert("savecontent literal", trim(captured), "Hello there");</cfscript>

<!--- cfsavecontent with cfoutput interpolation --->
<cfset planet = "Mars">
<cfsavecontent variable="interpolated"><cfoutput>Visit #planet#</cfoutput></cfsavecontent>
<cfscript>assert("savecontent with cfoutput", trim(interpolated), "Visit Mars");</cfscript>

<!--- cfsavecontent with loop inside --->
<cfsavecontent variable="loopCapture"><cfloop index="i" from="1" to="3"><cfoutput>#i#</cfoutput></cfloop></cfsavecontent>
<cfscript>assert("savecontent with loop", trim(loopCapture), "123");</cfscript>

<!--- Verify captured content is a string --->
<cfsavecontent variable="typeCheck">test content</cfsavecontent>
<cfscript>assertTrue("savecontent is string", isSimpleValue(typeCheck));</cfscript>

<!--- cfsavecontent with multiple cfset inside --->
<cfsavecontent variable="multiLine"><cfoutput>Line1</cfoutput>
<cfoutput>Line2</cfoutput></cfsavecontent>
<cfscript>assertTrue("savecontent multiline has Line1", find("Line1", multiLine) GT 0);</cfscript>

<cfscript>suiteEnd();</cfscript>
