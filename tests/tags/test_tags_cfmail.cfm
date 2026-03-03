<cfscript>suiteBegin("Tags: cfmail");</cfscript>

<!--- Self-closing cfmail (no server = log only) --->
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

<!--- Real SMTP tests if credentials provided via env vars --->
<cfscript>
smtpServer = getEnvironmentVariable("RUSTCFML_TEST_SMTP_SERVER");
smtpPort = getEnvironmentVariable("RUSTCFML_TEST_SMTP_PORT");
smtpUser = getEnvironmentVariable("RUSTCFML_TEST_SMTP_USERNAME");
smtpPass = getEnvironmentVariable("RUSTCFML_TEST_SMTP_PASSWORD");
smtpTo = getEnvironmentVariable("RUSTCFML_TEST_SMTP_TO");
smtpFrom = getEnvironmentVariable("RUSTCFML_TEST_SMTP_FROM");
</cfscript>

<cfif len(smtpServer) GT 0 AND len(smtpTo) GT 0 AND len(smtpFrom) GT 0>

<!--- Test: plain text email --->
<cfmail
    to="#smtpTo#"
    from="#smtpFrom#"
    subject="RustCFML Test: Plain Text"
    type="text"
    server="#smtpServer#"
    port="#smtpPort#"
    username="#smtpUser#"
    password="#smtpPass#">
This is a plain text email from the RustCFML test suite.
Line 2 of the message.
</cfmail>
<cfscript>assertTrue("cfmail SMTP plain text", true);</cfscript>

<!--- Test: HTML email --->
<cfmail
    to="#smtpTo#"
    from="#smtpFrom#"
    subject="RustCFML Test: HTML"
    type="html"
    server="#smtpServer#"
    port="#smtpPort#"
    username="#smtpUser#"
    password="#smtpPass#">
<html>
<body>
<h1>RustCFML HTML Email Test</h1>
<p>This is an <strong>HTML email</strong> from the test suite.</p>
<ul>
<li>Item 1</li>
<li>Item 2</li>
</ul>
</body>
</html>
</cfmail>
<cfscript>assertTrue("cfmail SMTP html", true);</cfscript>

<!--- Test: email without type (defaults to text) --->
<cfmail
    to="#smtpTo#"
    from="#smtpFrom#"
    subject="RustCFML Test: Default Type"
    server="#smtpServer#"
    port="#smtpPort#"
    username="#smtpUser#"
    password="#smtpPass#">
This email has no explicit type attribute - should default to plain text.
</cfmail>
<cfscript>assertTrue("cfmail SMTP default type", true);</cfscript>

<!--- Create a temp file for attachment test --->
<cfscript>
attachPath = getTempDirectory() & "rustcfml_test_attachment.txt";
fileWrite(attachPath, "This is a test attachment from RustCFML.");
</cfscript>

<!--- Test: email with attachment via cfmailparam --->
<cfmail
    to="#smtpTo#"
    from="#smtpFrom#"
    subject="RustCFML Test: With Attachment"
    type="text"
    server="#smtpServer#"
    port="#smtpPort#"
    username="#smtpUser#"
    password="#smtpPass#">
This email should have a text file attachment.
<cfmailparam file="#attachPath#">
</cfmail>
<cfscript>assertTrue("cfmail SMTP with attachment", true);</cfscript>

<!--- Clean up temp file --->
<cfscript>
if (fileExists(attachPath)) { fileDelete(attachPath); }
</cfscript>

<cfelse>
<cfscript>assertTrue("cfmail SMTP skipped (no RUSTCFML_TEST_SMTP_* env vars)", true);</cfscript>
</cfif>

<cfscript>suiteEnd();</cfscript>
