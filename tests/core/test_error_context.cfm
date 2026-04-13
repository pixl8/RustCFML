<cfscript>
suiteBegin("Error Context & tagContext");

// --- cfcatch.tagContext is populated ---
try {
    throw(message="test error", type="Application");
} catch (any e) {
    assertTrue("cfcatch.tagContext is an array", isArray(e.tagcontext));
    assertTrue("cfcatch.tagContext has at least one entry", arrayLen(e.tagcontext) >= 1);

    // Check the first entry has expected keys
    firstEntry = e.tagcontext[1];
    assertTrue("tagContext entry has 'template' key", structKeyExists(firstEntry, "template"));
    assertTrue("tagContext entry has 'line' key", structKeyExists(firstEntry, "line"));
    assertTrue("tagContext entry has 'id' key", structKeyExists(firstEntry, "id"));
    assertTrue("tagContext entry has 'raw_trace' key", structKeyExists(firstEntry, "raw_trace"));
    assertTrue("tagContext entry has 'column' key", structKeyExists(firstEntry, "column"));

    // Verify types of values
    assertTrue("tagContext template is a string", isSimpleValue(firstEntry.template));
    assertTrue("tagContext line is numeric", isNumeric(firstEntry.line));
    assertTrue("tagContext id is a string", isSimpleValue(firstEntry.id));
}

// --- tagContext from division by zero ---
try {
    x = 1 / 0;
} catch (any e) {
    assertTrue("div-by-zero tagContext is an array", isArray(e.tagcontext));
    assertTrue("div-by-zero tagContext has entries", arrayLen(e.tagcontext) >= 1);
}

// --- tagContext from function error ---
function throwError() {
    throw(message="inner error");
}
try {
    throwError();
} catch (any e) {
    assertTrue("function error tagContext is array", isArray(e.tagcontext));
    assertTrue("function error tagContext has entries", arrayLen(e.tagcontext) >= 1);
}

// --- structKeyExists on exception struct ---
try {
    throw(message="test for key exists", type="CustomType", detail="some detail");
} catch (any e) {
    assertTrue("exception has 'message'", structKeyExists(e, "message"));
    assertTrue("exception has 'type'", structKeyExists(e, "type"));
    assertTrue("exception has 'detail'", structKeyExists(e, "detail"));
    assertTrue("exception has 'tagcontext'", structKeyExists(e, "tagcontext"));
    assertFalse("exception missing key returns false", structKeyExists(e, "nonExistentKey"));
    // Case-insensitive check
    assertTrue("structKeyExists is case-insensitive", structKeyExists(e, "MESSAGE"));
    assertTrue("structKeyExists is case-insensitive (mixed)", structKeyExists(e, "TagContext"));
}

suiteEnd();
</cfscript>
