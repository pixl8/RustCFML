<cfscript>
suiteBegin("Variables Scope in Includes");

variables.mainVar = "fromMain";
include "helper_variables_scope.cfm";

assert("variable set in include accessible from main", variables.helperVar, "fromHelper");
assert("function from include can read variables set in include", getHelperVar(), "fromHelper");
assert("function from include can read variables set in main", getMainVar(), "fromMain");

suiteEnd();
</cfscript>
