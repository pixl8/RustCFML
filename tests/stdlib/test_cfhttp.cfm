<cfscript>
suiteBegin("cfhttp");

// ============================================================
// Tag-style cfhttp (via tag preprocessor)
// ============================================================
</cfscript>

<!--- Basic GET request --->
<cfhttp url="https://httpbin.org/get" method="GET" result="getResult">

<cfscript>
assertTrue("GET result is struct", isStruct(getResult));
assertTrue("GET status_code is 200", getResult.status_code == 200);
assertTrue("GET statusCode contains 200", find("200", getResult.statusCode) > 0);
assertTrue("GET fileContent is not empty", len(getResult.fileContent) > 0);
assertTrue("GET responseHeader is struct", isStruct(getResult.responseHeader));
assertTrue("GET mimeType is json", find("json", getResult.mimeType) > 0);

// Parse the JSON response
getData = deserializeJSON(getResult.fileContent);
assertTrue("GET response has url key", structKeyExists(getData, "url"));
assert("GET response url", getData.url, "https://httpbin.org/get");

// ============================================================
// CFScript http statement syntax (semicolon form)
// ============================================================
http url="https://httpbin.org/get" method="GET" result="scriptResult";

assertTrue("CFScript http result is struct", isStruct(scriptResult));
assertTrue("CFScript http status 200", scriptResult.status_code == 200);
assertTrue("CFScript http fileContent not empty", len(scriptResult.fileContent) > 0);

// ============================================================
// CFScript http block form with httpparam
// ============================================================
http url="https://httpbin.org/get" method="GET" result="paramResult" {
    httpparam type="header" name="X-Custom-Header" value="TestValue123";
    httpparam type="url" name="foo" value="bar";
}

assertTrue("httpparam result is struct", isStruct(paramResult));
assertTrue("httpparam status 200", paramResult.status_code == 200);
paramData = deserializeJSON(paramResult.fileContent);
assertTrue("httpparam header sent", structKeyExists(paramData.headers, "X-Custom-Header"));
assert("httpparam header value", paramData.headers["X-Custom-Header"], "TestValue123");
assertTrue("httpparam url param sent", find("foo=bar", paramData.url) > 0);

// ============================================================
// POST with body
// ============================================================
http url="https://httpbin.org/post" method="POST" result="postResult" {
    httpparam type="header" name="Content-Type" value="application/json";
    httpparam type="body" value='{"name":"test","value":42}';
}

assertTrue("POST status 200", postResult.status_code == 200);
postData = deserializeJSON(postResult.fileContent);
assertTrue("POST body received", len(postData.data) > 0);
postedJson = deserializeJSON(postData.data);
assert("POST body name", postedJson.name, "test");
assert("POST body value", postedJson.value, 42);

// ============================================================
// POST with formfields
// ============================================================
http url="https://httpbin.org/post" method="POST" result="formResult" {
    httpparam type="formfield" name="username" value="testuser";
    httpparam type="formfield" name="email" value="test@example.com";
}

assertTrue("formfield POST status 200", formResult.status_code == 200);
formData = deserializeJSON(formResult.fileContent);
assertTrue("formfield has form key", structKeyExists(formData, "form"));
assert("formfield username", formData.form.username, "testuser");
assert("formfield email", formData.form.email, "test@example.com");

// ============================================================
// Custom headers
// ============================================================
http url="https://httpbin.org/headers" method="GET" result="headerResult" {
    httpparam type="header" name="X-Test-One" value="Alpha";
    httpparam type="header" name="X-Test-Two" value="Beta";
}

assertTrue("headers status 200", headerResult.status_code == 200);
headerData = deserializeJSON(headerResult.fileContent);
assert("custom header X-Test-One", headerData.headers["X-Test-One"], "Alpha");
assert("custom header X-Test-Two", headerData.headers["X-Test-Two"], "Beta");

// ============================================================
// PUT method
// ============================================================
http url="https://httpbin.org/put" method="PUT" result="putResult" {
    httpparam type="header" name="Content-Type" value="application/json";
    httpparam type="body" value='{"updated":true}';
}

assertTrue("PUT status 200", putResult.status_code == 200);
putData = deserializeJSON(putResult.fileContent);
assert("PUT body received", putData.data, '{"updated":true}');

// ============================================================
// DELETE method
// ============================================================
http url="https://httpbin.org/delete" method="DELETE" result="deleteResult";

assertTrue("DELETE status 200", deleteResult.status_code == 200);

// ============================================================
// User-Agent header
// ============================================================
http url="https://httpbin.org/user-agent" method="GET" result="uaResult" useragent="RustCFML-Test/1.0";

assertTrue("useragent status 200", uaResult.status_code == 200);
uaData = deserializeJSON(uaResult.fileContent);
assert("useragent value sent", uaData["user-agent"], "RustCFML-Test/1.0");

// ============================================================
// Cookie via httpparam
// ============================================================
http url="https://httpbin.org/cookies" method="GET" result="cookieResult" {
    httpparam type="cookie" name="session_id" value="abc123";
    httpparam type="cookie" name="theme" value="dark";
}

assertTrue("cookie status 200", cookieResult.status_code == 200);
cookieData = deserializeJSON(cookieResult.fileContent);
assertTrue("cookie has cookies key", structKeyExists(cookieData, "cookies"));
assert("cookie session_id", cookieData.cookies.session_id, "abc123");
assert("cookie theme", cookieData.cookies.theme, "dark");

// ============================================================
// Response headers
// ============================================================
http url="https://httpbin.org/response-headers?X-Custom-Response=HelloWorld" method="GET" result="respHeaderResult";

assertTrue("response header status 200", respHeaderResult.status_code == 200);
assertTrue("responseHeader has custom key", structKeyExists(respHeaderResult.responseHeader, "x-custom-response"));

// ============================================================
// Timeout (short timeout, should still work for fast endpoint)
// ============================================================
http url="https://httpbin.org/get" method="GET" result="timeoutResult" timeout="10";

assertTrue("timeout request succeeded", timeoutResult.status_code == 200);

// ============================================================
// PATCH method
// ============================================================
http url="https://httpbin.org/patch" method="PATCH" result="patchResult" {
    httpparam type="header" name="Content-Type" value="application/json";
    httpparam type="body" value='{"patched":true}';
}

assertTrue("PATCH status 200", patchResult.status_code == 200);
patchData = deserializeJSON(patchResult.fileContent);
assert("PATCH body received", patchData.data, '{"patched":true}');

suiteEnd();
</cfscript>
