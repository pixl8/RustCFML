<cfscript>
suiteBegin("Error Handling");

// --- try/catch basic ---
caught = false;
try {
    throw(message="Test error", type="Application");
} catch (any e) {
    caught = true;
}
assertTrue("try/catch basic", caught);

// --- catch any exception ---
anyMsg = "";
try {
    throw(message="any error");
} catch (any e) {
    anyMsg = e.message;
}
assert("catch any - message", anyMsg, "any error");

// --- Typed catch (Application) ---
typedCatch = "";
try {
    throw(message="app error", type="Application");
} catch (Application e) {
    typedCatch = "caught application";
}
assert("typed catch Application", typedCatch, "caught application");

// --- catch variable properties: e.message, e.type ---
errMsg = "";
errType = "";
try {
    throw(message="detailed error", type="CustomType");
} catch (any e) {
    errMsg = e.message;
    errType = e.type;
}
assert("catch e.message", errMsg, "detailed error");
assert("catch e.type", errType, "CustomType");

// --- throw with type and detail ---
errDetail = "";
try {
    throw(message="main msg", type="MyType", detail="extra info");
} catch (any e) {
    errDetail = e.detail;
}
assert("throw with detail", errDetail, "extra info");

// --- finally block execution ---
finallyRan = false;
try {
    x = 1;
} catch (any e) {
    // should not reach here
} finally {
    finallyRan = true;
}
assertTrue("finally runs on success", finallyRan);

finallyAfterError = false;
try {
    throw(message="boom");
} catch (any e) {
    // caught
} finally {
    finallyAfterError = true;
}
assertTrue("finally runs after error", finallyAfterError);

// --- Nested try/catch ---
innerCaught = "";
outerCaught = "";
try {
    try {
        throw(message="inner error", type="InnerType");
    } catch (InnerType e) {
        innerCaught = e.message;
    }
    throw(message="outer error", type="OuterType");
} catch (OuterType e) {
    outerCaught = e.message;
}
assert("nested try - inner caught", innerCaught, "inner error");
assert("nested try - outer caught", outerCaught, "outer error");

// --- assertThrows usage ---
assertThrows("assertThrows with throw", function() {
    throw(message="expected");
});

assertThrows("assertThrows with expression error", function() {
    var result = 1 / 0;
});

// --- rethrow ---
rethrown = "";
try {
    try {
        throw(message="rethrow me", type="Application");
    } catch (any e) {
        rethrow;
    }
} catch (any e) {
    rethrown = e.message;
}
assert("rethrow caught by outer", rethrown, "rethrow me");

// --- Error in catch does not swallow original ---
catchError = "";
try {
    try {
        throw(message="original");
    } catch (any e) {
        throw(message="from catch");
    }
} catch (any e) {
    catchError = e.message;
}
assert("throw from catch block", catchError, "from catch");

// --- finally runs even when rethrowing ---
finallyOnRethrow = false;
try {
    try {
        throw(message="rethrow finally test");
    } catch (any e) {
        rethrow;
    } finally {
        finallyOnRethrow = true;
    }
} catch (any e) {
    // outer catch
}
assertTrue("finally runs on rethrow", finallyOnRethrow);

suiteEnd();
</cfscript>
