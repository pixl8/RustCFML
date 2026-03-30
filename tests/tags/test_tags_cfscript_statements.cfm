<cfscript>
suiteBegin("CFScript Tag Statements");

// ============================================================
// content (cfcontent)
// ============================================================

// content sets the response content type - just verify it parses and runs
content type="application/json";
assertTrue("content type parsed", true);

content type="text/html" reset="true";
assertTrue("content with reset parsed", true);

// ============================================================
// header (cfheader)
// ============================================================

// header sets response headers
header name="X-Test-Header" value="hello123";
assertTrue("header name/value parsed", true);

header statuscode="200" statustext="OK";
assertTrue("header statuscode parsed", true);

// ============================================================
// setting (cfsetting)
// ============================================================

setting requesttimeout="60";
assertTrue("setting requesttimeout parsed", true);

setting showdebugoutput="false";
assertTrue("setting showdebugoutput parsed", true);

// ============================================================
// cookie (cfcookie)
// ============================================================

cookie name="testcookie" value="cookievalue";
assertTrue("cookie name/value parsed", true);

cookie name="securecookie" value="secret" httponly="true" secure="true";
assertTrue("cookie with httponly/secure parsed", true);

// ============================================================
// log (cflog)
// ============================================================

log text="Test log message" type="information";
assertTrue("log text/type parsed", true);

log text="Debug message" type="debug" file="testlog";
assertTrue("log with file parsed", true);

// ============================================================
// location (cflocation) — throws redirect error, must catch
// ============================================================

try {
    location url="/redirect-target" statuscode="301";
    // Should not reach here
    assertTrue("location should have thrown", false);
} catch (any e) {
    assertTrue("location throws redirect", true);
}

try {
    location url="/another-redirect";
    assertTrue("location default should throw", false);
} catch (any e) {
    assertTrue("location default statuscode throws", true);
}

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
// Mixed usage: tag statements inside functions
// ============================================================

function setJsonResponse(required string data) {
    content type="application/json";
    header name="X-API-Version" value="1.0";
    return data;
}

result = setJsonResponse('{"status":"ok"}');
assert("tag stmts in function", result, '{"status":"ok"}');

// ============================================================
// Tag statements with dynamic expressions
// ============================================================

myType = "text/xml";
content type=myType;
assertTrue("content with variable expression", true);

myHeaderVal = "dynamic-" & "value";
header name="X-Dynamic" value=myHeaderVal;
assertTrue("header with expression", true);

cookieName = "dyncookie";
cookieVal = "dynval";
cookie name=cookieName value=cookieVal;
assertTrue("cookie with variable expressions", true);

suiteEnd();
</cfscript>
