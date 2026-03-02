<cfscript>
suiteBegin("Type Checking Functions");

// --- isNull ---
assertTrue("isNull(nullValue())", isNull(nullValue()));
assertFalse("isNull('hi')", isNull("hi"));

// --- isNumeric ---
assertTrue("isNumeric(42)", isNumeric(42));
assertTrue("isNumeric('42')", isNumeric("42"));
assertFalse("isNumeric('abc')", isNumeric("abc"));

// --- isBoolean ---
assertTrue("isBoolean(true)", isBoolean(true));
assertTrue("isBoolean('yes')", isBoolean("yes"));
assertFalse("isBoolean('maybe')", isBoolean("maybe"));

// --- isArray ---
assertTrue("isArray([1,2])", isArray([1, 2]));
assertFalse("isArray('nope')", isArray("nope"));

// --- isStruct ---
assertTrue("isStruct({a:1})", isStruct({a: 1}));
assertFalse("isStruct('nope')", isStruct("nope"));

// --- isQuery ---
assertTrue("isQuery(queryNew('a'))", isQuery(queryNew("a")));

// --- isSimpleValue ---
assertTrue("isSimpleValue('hello')", isSimpleValue("hello"));
assertTrue("isSimpleValue(42)", isSimpleValue(42));
assertFalse("isSimpleValue([1])", isSimpleValue([1]));
assertFalse("isSimpleValue({a:1})", isSimpleValue({a: 1}));

// --- isDefined ---
assertFalse("isDefined undefinedVar", isDefined("variables.undefinedVar123"));

// --- isCustomFunction ---
myFn = function() { return 1; };
assertTrue("isCustomFunction(fn)", isCustomFunction(myFn));
assertFalse("isCustomFunction('nope')", isCustomFunction("nope"));

suiteEnd();
</cfscript>
