<cfif thisTag.executionMode EQ "start">
    <cfoutput><div class="wrapped"></cfoutput>
<cfelse>
    <cfoutput>#thisTag.generatedContent#</div></cfoutput>
    <cfset thisTag.generatedContent = "">
</cfif>
