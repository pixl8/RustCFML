<cfscript>suiteBegin("Tags: cfmail");</cfscript>

<!--- Self-closing cfmail --->
<cfmail to="test@example.com" from="sender@example.com" subject="Test" type="text">
This is a test email body.
</cfmail>
<cfscript>assertTrue("cfmail basic no error", true);</cfscript>

<!--- cfmail with cfmailparam --->
<cfmail to="test@example.com" from="sender@example.com" subject="With Params">
Body text here.
<cfmailparam name="X-Custom-Header" value="custom-value">
<cfmailparam file="/tmp/attachment.txt">
</cfmail>
<cfscript>assertTrue("cfmail with params no error", true);</cfscript>

<!--- cfmail with type=html --->
<cfmail to="test@example.com" from="sender@example.com" subject="HTML Mail" type="html">
<html><body><h1>Hello</h1></body></html>
</cfmail>
<cfscript>assertTrue("cfmail html type no error", true);</cfscript>

<cfscript>suiteEnd();</cfscript>
