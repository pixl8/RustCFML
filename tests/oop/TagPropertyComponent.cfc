<cfcomponent>
    <cfproperty name="myService" inject="MyService">
    <cfproperty name="helper" inject="HelperService" hint="A helper">
    <cfproperty name="title" type="string" default="Untitled">

    <cffunction name="getName" access="public" returntype="string">
        <cfreturn "TagPropertyComponent">
    </cffunction>
</cfcomponent>
