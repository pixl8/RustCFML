<cfcomponent taffy_uri="/users">

	<cffunction name="get" access="public" output="false">
		<cfset var users = [] />
		<cfset arrayAppend(users, {"id": 1, "name": "Alice", "email": "alice@example.com"}) />
		<cfset arrayAppend(users, {"id": 2, "name": "Bob", "email": "bob@example.com"}) />
		<cfset arrayAppend(users, {"id": 3, "name": "Carol", "email": "carol@example.com"}) />
		<cfreturn users />
	</cffunction>

	<cffunction name="post" access="public" output="false">
		<cfargument name="name" type="string" required="true" />
		<cfargument name="email" type="string" required="true" />
		<cfset var newUser = {"id": 4, "name": arguments.name, "email": arguments.email} />
		<cfreturn newUser />
	</cffunction>

</cfcomponent>
