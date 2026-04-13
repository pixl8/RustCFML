<cfscript>
suiteBegin("CFScript Tag Statements");

// ============================================================
// setting (cfsetting) — safe, no HTTP side-effects
// ============================================================

setting requesttimeout="60";
assertTrue("setting requesttimeout parsed", true);

setting showdebugoutput="false";
assertTrue("setting showdebugoutput parsed", true);

// ============================================================
// log (cflog) — safe, no HTTP side-effects
// ============================================================

log text="Test log message" type="information";
assertTrue("log text/type parsed", true);

log text="Debug message" type="debug" file="testlog";
assertTrue("log with file parsed", true);

// ============================================================
// thread (cfthread) — run/join/terminate actions
// ============================================================

// thread run with body
thread name="testThread1" action="run" {
    // Thread body - just needs to parse and run
    var x = 42;
}
thread name="testThread1" action="join" timeout="5";
assertTrue("thread run/join parsed", true);

// thread with default action (run)
thread name="testThread2" {
    var y = 100;
}
thread name="testThread2" action="join";
assertTrue("thread default action parsed", true);

// ============================================================
// HTTP-affecting statements (header, cookie, content, location)
// Tested via cfhttp to a target page so headers don't bleed
// into the test runner's own HTTP response.
// ============================================================

baseUrl = "http://127.0.0.1:" & (cgi.server_port ?: "8585");
targetPath = "/tests/tags/http_statements_target.cfm";

// --- header ---
cfhttp(url=baseUrl & targetPath & "?test=header", method="GET", result="headerResult");
assert("header target responds", headerResult.statuscode, "200 OK");
assert("header body", trim(headerResult.filecontent), "header-ok");
assertTrue("header X-Test-Header set",
    structKeyExists(headerResult.responseheader, "X-Test-Header")
    && headerResult.responseheader["X-Test-Header"] == "hello123");

// --- cookie ---
cfhttp(url=baseUrl & targetPath & "?test=cookie", method="GET", result="cookieResult");
assert("cookie target responds", cookieResult.statuscode, "200 OK");
assert("cookie body", trim(cookieResult.filecontent), "cookie-ok");

// --- content type ---
cfhttp(url=baseUrl & targetPath & "?test=content", method="GET", result="contentResult");
assert("content target responds", contentResult.statuscode, "200 OK");
assert("content body", trim(contentResult.filecontent), '{"status":"ok"}');
assertTrue("content type is json",
    findNoCase("application/json", contentResult.responseheader["Content-Type"]) > 0);

// --- location (redirect) ---
cfhttp(url=baseUrl & targetPath & "?test=location", method="GET", redirect="false", result="locResult");
assertTrue("location returns 3xx",
    left(locResult.statuscode, 1) == "3");
assertTrue("location header set",
    structKeyExists(locResult.responseheader, "Location")
    && findNoCase("redirect-target", locResult.responseheader["Location"]) > 0);

suiteEnd();
</cfscript>
