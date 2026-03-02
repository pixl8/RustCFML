<cfscript>
suiteBegin("Type: Null");

// --- null literal ---
assertTrue("null literal is null", isNull(null));

// --- isNull checks ---
assertTrue("isNull(null) is true", isNull(null));
assertFalse("isNull(string) is false", isNull("hello"));
assertFalse("isNull(number) is false", isNull(42));
assertFalse("isNull(boolean) is false", isNull(true));
assertFalse("isNull(empty string) is false", isNull(""));

// --- nullValue() function ---
assertTrue("nullValue() returns null", isNull(nullValue()));

// --- null comparison ---
assertTrue("null eq null is true", isNull(null) == isNull(nullValue()));

// --- isDefined for undefined variable ---
assertFalse("isDefined for undefined var", isDefined("doesNotExistAtAll_xyz"));

// --- isDefined for defined variable ---
definedVar = "exists";
assertTrue("isDefined for defined var", isDefined("definedVar"));

suiteEnd();
</cfscript>
