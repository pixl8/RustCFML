<cfscript>
// ============================================================
// RustCFML Test Harness
// ============================================================
// State stored in request scope so it persists across includes.
// Uses explicit assignment (no ++/+=) for RustCFML compatibility.

// Grand totals
request._test_totalPassed  = 0;
request._test_totalFailed  = 0;
request._test_totalSuites  = 0;
request._test_failedSuites = 0;
request._test_failures     = [];

// Per-suite state
request._test_suiteName   = "";
request._test_suitePassed = 0;
request._test_suiteFailed = 0;
request._test_suiteFailures = [];

// ---- suiteBegin(name) ----
function suiteBegin(required string name) {
    request._test_suiteName     = arguments.name;
    request._test_suitePassed   = 0;
    request._test_suiteFailed   = 0;
    request._test_suiteFailures = [];
}

// ---- suiteEnd() ----
function suiteEnd() {
    var total = request._test_suitePassed + request._test_suiteFailed;
    request._test_totalPassed = request._test_totalPassed + request._test_suitePassed;
    request._test_totalFailed = request._test_totalFailed + request._test_suiteFailed;
    request._test_totalSuites = request._test_totalSuites + 1;

    if (request._test_suiteFailed > 0) {
        request._test_failedSuites = request._test_failedSuites + 1;
        writeOutput("FAIL | " & request._test_suiteName & " | "
            & request._test_suitePassed & "/" & total & " passed ("
            & request._test_suiteFailed & " failed)" & chr(10));
        for (var f in request._test_suiteFailures) {
            writeOutput("       FAIL: " & f & chr(10));
            arrayAppend(request._test_failures,
                request._test_suiteName & " > " & f);
        }
    } else {
        writeOutput("PASS | " & request._test_suiteName & " | "
            & request._test_suitePassed & "/" & total & " passed" & chr(10));
    }
}

// ---- assert(label, actual, expected) ----
function assert(required string label, required actual, required expected) {
    if (toString(arguments.actual) == toString(arguments.expected)) {
        request._test_suitePassed = request._test_suitePassed + 1;
    } else {
        request._test_suiteFailed = request._test_suiteFailed + 1;
        arrayAppend(request._test_suiteFailures,
            arguments.label & " | expected: [" & toString(arguments.expected)
            & "] | got: [" & toString(arguments.actual) & "]");
    }
}

// ---- assertTrue(label, value) ----
function assertTrue(required string label, required value) {
    if (arguments.value) {
        request._test_suitePassed = request._test_suitePassed + 1;
    } else {
        request._test_suiteFailed = request._test_suiteFailed + 1;
        arrayAppend(request._test_suiteFailures,
            arguments.label & " | expected truthy | got: [" & toString(arguments.value) & "]");
    }
}

// ---- assertFalse(label, value) ----
function assertFalse(required string label, required value) {
    if (!arguments.value) {
        request._test_suitePassed = request._test_suitePassed + 1;
    } else {
        request._test_suiteFailed = request._test_suiteFailed + 1;
        arrayAppend(request._test_suiteFailures,
            arguments.label & " | expected falsy | got: [" & toString(arguments.value) & "]");
    }
}

// ---- assertNull(label, value) ----
function assertNull(required string label, value) {
    if (isNull(arguments.value)) {
        request._test_suitePassed = request._test_suitePassed + 1;
    } else {
        request._test_suiteFailed = request._test_suiteFailed + 1;
        arrayAppend(request._test_suiteFailures,
            arguments.label & " | expected null | got: [" & toString(arguments.value) & "]");
    }
}

// ---- assertNotNull(label, value) ----
function assertNotNull(required string label, value) {
    if (!isNull(arguments.value)) {
        request._test_suitePassed = request._test_suitePassed + 1;
    } else {
        request._test_suiteFailed = request._test_suiteFailed + 1;
        arrayAppend(request._test_suiteFailures,
            arguments.label & " | expected not null | got null");
    }
}

// ---- assertThrows(label, callback) ----
function assertThrows(required string label, required callback) {
    try {
        callback();
        request._test_suiteFailed = request._test_suiteFailed + 1;
        arrayAppend(request._test_suiteFailures,
            arguments.label & " | expected exception | none thrown");
    } catch (any e) {
        request._test_suitePassed = request._test_suitePassed + 1;
    }
}

// ---- printSummary() ----
function printSummary() {
    var grandTotal = request._test_totalPassed + request._test_totalFailed;
    writeOutput(chr(10) & "============================================================" & chr(10));
    writeOutput("SUMMARY: " & request._test_totalPassed & "/" & grandTotal
        & " passed across " & request._test_totalSuites & " suites" & chr(10));
    if (request._test_totalFailed > 0) {
        writeOutput("FAILED:  " & request._test_totalFailed & " assertion(s) in "
            & request._test_failedSuites & " suite(s)" & chr(10));
        writeOutput(chr(10) & "All failures:" & chr(10));
        for (var f in request._test_failures) {
            writeOutput("  - " & f & chr(10));
        }
    } else {
        writeOutput("ALL TESTS PASSED" & chr(10));
    }
    writeOutput("============================================================" & chr(10));
}
</cfscript>
