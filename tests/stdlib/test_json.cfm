<cfscript>
suiteBegin("JSON Functions");

// --- serializeJSON ---
assert("serializeJSON integer", serializeJSON(42), "42");
assert("serializeJSON string", serializeJSON("hello"), '"hello"');
assert("serializeJSON boolean", serializeJSON(true), "true");

jsonArr = serializeJSON([1, 2, 3]);
assertTrue("serializeJSON array is valid JSON", isJSON(jsonArr));
assertTrue("serializeJSON array contains brackets", find("[", jsonArr) > 0);

jsonObj = serializeJSON({name: "test"});
assertTrue("serializeJSON struct is valid JSON", isJSON(jsonObj));
assertTrue("serializeJSON struct contains brace", find("{", jsonObj) > 0);

// --- deserializeJSON ---
assert("deserializeJSON integer", deserializeJSON("42"), 42);
assert("deserializeJSON string", deserializeJSON('"hello"'), "hello");

parsedArr = deserializeJSON("[1,2,3]");
assertTrue("deserializeJSON array isArray", isArray(parsedArr));
assert("deserializeJSON array length", arrayLen(parsedArr), 3);

parsedObj = deserializeJSON('{"name":"test"}');
assertTrue("deserializeJSON struct has key", structKeyExists(parsedObj, "name"));
assert("deserializeJSON struct value", parsedObj.name, "test");

// --- isJSON ---
assertTrue("isJSON number", isJSON("42"));
assertTrue("isJSON array", isJSON("[1,2,3]"));
assertTrue("isJSON object", isJSON('{"a":1}'));
assertFalse("isJSON invalid", isJSON("not json {"));

// --- round-trip ---
original = {a: 1, b: [2, 3]};
roundTrip = deserializeJSON(serializeJSON(original));
assert("round-trip struct key a", roundTrip.a, 1);
assertTrue("round-trip struct key b is array", isArray(roundTrip.b));
assert("round-trip array length", arrayLen(roundTrip.b), 2);

suiteEnd();
</cfscript>
