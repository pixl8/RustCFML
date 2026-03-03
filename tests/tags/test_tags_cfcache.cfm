<cfscript>suiteBegin("Tags: cfcache");</cfscript>

<!--- action=flush --->
<cfcache action="flush">
<cfscript>assertTrue("cfcache flush no error", true);</cfscript>

<!--- action=clientcache --->
<cfcache action="clientcache">
<cfscript>assertTrue("cfcache clientcache no error", true);</cfscript>

<!--- action=optimal (default-ish) --->
<cfcache action="optimal">
<cfscript>assertTrue("cfcache optimal no error", true);</cfscript>

<cfscript>suiteEnd();</cfscript>
