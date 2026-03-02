<cfscript>suiteBegin("Tags: Param");</cfscript>

<!--- cfparam with default for undefined variable --->
<cfparam name="undefinedVar1" default="defaultValue">
<cfscript>assert("cfparam sets default", undefinedVar1, "defaultValue");</cfscript>

<!--- cfparam with numeric default --->
<cfparam name="undefinedNum" default="99">
<cfscript>assert("cfparam numeric default", undefinedNum, 99);</cfscript>

<!--- cfparam does not override existing variable --->
<cfset existingVar = "original">
<cfparam name="existingVar" default="overridden">
<cfscript>assert("cfparam no override", existingVar, "original");</cfscript>

<!--- cfparam with type="string" on valid string --->
<cfset validStr = "hello">
<cfparam name="validStr" type="string">
<cfscript>assert("cfparam type string valid", validStr, "hello");</cfscript>

<!--- cfparam with type="numeric" on valid number --->
<cfset validNum = 42>
<cfparam name="validNum" type="numeric">
<cfscript>assert("cfparam type numeric valid", validNum, 42);</cfscript>

<!--- cfparam with type="boolean" on valid boolean --->
<cfset validBool = true>
<cfparam name="validBool" type="boolean">
<cfscript>assertTrue("cfparam type boolean valid", validBool);</cfscript>

<!--- cfparam with default and type --->
<cfparam name="typedDefault" type="string" default="typed">
<cfscript>assert("cfparam default with type", typedDefault, "typed");</cfscript>

<!--- cfparam with type="array" default --->
<cfparam name="arrDefault" type="array" default="#[]#">
<cfscript>assertTrue("cfparam array default is array", isArray(arrDefault));</cfscript>

<cfscript>suiteEnd();</cfscript>
