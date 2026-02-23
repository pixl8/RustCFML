<cfcomponent taffy_uri="/hello">

	<cffunction name="get" access="public" output="false">
		<cfset var data = {} />
		<cfset data.message = "Hello from Taffy on RustCFML!" />
		<cfset data.timestamp = now() />
		<cfreturn data />
	</cffunction>

</cfcomponent>
