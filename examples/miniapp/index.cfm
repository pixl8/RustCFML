<cfscript>
pageTitle = "Welcome to MiniApp";

// Greet by name if URL param provided: /?name=World
if (structKeyExists(url, "name")) {
    greeting = "Hello, " & url.name & "!";
} else {
    greeting = "Hello from RustCFML!";
}
</cfscript>

<cfinclude template="header.cfm">

<h2><cfoutput>#greeting#</cfoutput></h2>
<p>This is a simple web app served by RustCFML.</p>

<cfoutput>
<h3>Request Info</h3>
<ul>
    <li><strong>Method:</strong> #cgi.request_method#</li>
    <li><strong>Path:</strong> #cgi.path_info#</li>
    <li><strong>Query String:</strong> #cgi.query_string#</li>
    <li><strong>Server:</strong> #cgi.server_name#:#cgi.server_port#</li>
</ul>
</cfoutput>

<p>Try: <a href="/?name=World">/?name=World</a></p>

<cfinclude template="footer.cfm">
