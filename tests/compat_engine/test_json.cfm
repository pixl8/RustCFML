<cfscript>
// Lucee 7 Compatibility Tests: JSON Functions
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness

// ============================================================
// serializeJSON basics (from Lucee SerializeJSON1.cfc)
// ============================================================
suiteBegin("Lucee7: serializeJSON basics");
assert("serializeJSON integer", serializeJSON(1), "1");
assert("serializeJSON zero", serializeJSON(0), "0");
assertTrue("serializeJSON string contains value", find("hello", serializeJSON("hello")) > 0);
assertTrue("serializeJSON boolean true", find("true", serializeJSON(true)) > 0);
assert("serializeJSON simple array", serializeJSON([1,2,3]), "[1,2,3]");
assert("serializeJSON empty array", serializeJSON([]), "[]");
assertTrue("serializeJSON null", serializeJSON(nullValue()) == "null");
suiteEnd();

// ============================================================
// serializeJSON structs (from Lucee SerializeJSON1.cfc)
// ============================================================
suiteBegin("Lucee7: serializeJSON structs");
s = {name:"test"};
json = serializeJSON(s);
assertTrue("serializeJSON struct is valid JSON", isJSON(json));
rt = deserializeJSON(json);
assert("serializeJSON struct roundtrip", rt.name, "test");

s2 = {a:1, b:"hello", c:true};
json2 = serializeJSON(s2);
assertTrue("serializeJSON multi-key struct is JSON", isJSON(json2));
rt2 = deserializeJSON(json2);
assert("serializeJSON struct roundtrip a", rt2.a, 1);
assert("serializeJSON struct roundtrip b", rt2.b, "hello");
suiteEnd();

// ============================================================
// deserializeJSON basics (from Lucee DeSerializeJSON.cfc)
// ============================================================
suiteBegin("Lucee7: deserializeJSON basics");
obj = deserializeJSON('{"key":"value"}');
assert("deserializeJSON object key", obj.key, "value");

arr = deserializeJSON("[1,2,3]");
assertTrue("deserializeJSON array is array", isArray(arr));
assert("deserializeJSON array len", arrayLen(arr), 3);
assert("deserializeJSON array first", arr[1], 1);
assert("deserializeJSON array last", arr[3], 3);

num = deserializeJSON("42");
assert("deserializeJSON number", num, 42);

str = deserializeJSON('"hello"');
assert("deserializeJSON string", str, "hello");

b = deserializeJSON("true");
assert("deserializeJSON boolean", b, true);

n = deserializeJSON("null");
assertTrue("deserializeJSON null", isNull(n));
suiteEnd();

// ============================================================
// Round-trip struct (from Lucee SerializeJSON1.cfc)
// ============================================================
suiteBegin("Lucee7: JSON round-trip struct");
orig = {a:1, b:"hello", c:true, d:[1,2]};
json = serializeJSON(orig);
assertTrue("roundtrip struct is JSON", isJSON(json));
rt = deserializeJSON(json);
assert("roundtrip struct a", rt.a, 1);
assert("roundtrip struct b", rt.b, "hello");
assertTrue("roundtrip struct d is array", isArray(rt.d));
assert("roundtrip struct d len", arrayLen(rt.d), 2);
assert("roundtrip struct d[1]", rt.d[1], 1);
assert("roundtrip struct d[2]", rt.d[2], 2);
suiteEnd();

// ============================================================
// Round-trip array (from Lucee SerializeJSON1.cfc)
// ============================================================
suiteBegin("Lucee7: JSON round-trip array");
orig = [1, "two", true, {a:1}];
json = serializeJSON(orig);
assertTrue("roundtrip array is JSON", isJSON(json));
rt = deserializeJSON(json);
assertTrue("roundtrip array is array", isArray(rt));
assert("roundtrip array len", arrayLen(rt), 4);
assert("roundtrip array[1]", rt[1], 1);
assert("roundtrip array[2]", rt[2], "two");
assert("roundtrip array[3]", rt[3], true);
assertTrue("roundtrip array[4] is struct", isStruct(rt[4]));
assert("roundtrip array[4].a", rt[4].a, 1);
suiteEnd();

// ============================================================
// Nested JSON (from Lucee DeSerializeJSON.cfc)
// ============================================================
suiteBegin("Lucee7: JSON nested structures");
json = '{"users":[{"name":"John","age":30},{"name":"Jane","age":25}]}';
d = deserializeJSON(json);
assertTrue("nested is struct", isStruct(d));
assertTrue("nested users is array", isArray(d.users));
assert("nested users len", arrayLen(d.users), 2);
assert("nested user 1 name", d.users[1].name, "John");
assert("nested user 1 age", d.users[1].age, 30);
assert("nested user 2 name", d.users[2].name, "Jane");
assert("nested user 2 age", d.users[2].age, 25);
suiteEnd();

// ============================================================
// Query serialization (from Lucee SerializeJSON1.cfc)
// ============================================================
suiteBegin("Lucee7: JSON query serialization");
q = queryNew("name,age", "varchar,integer", [["John",30],["Jane",25]]);
json = serializeJSON(q);
assertTrue("query serialization is JSON", isJSON(json));
assertTrue("query serialization is string", len(json) > 0);

// round-trip: deserialize and verify structure
rt = deserializeJSON(json);
assertTrue("query JSON roundtrip is not null", !isNull(rt));
suiteEnd();

// ============================================================
// Edge cases
// ============================================================
suiteBegin("Lucee7: JSON edge cases");
// empty struct
assert("serializeJSON empty struct", serializeJSON({}), "{}");

// deeply nested
deep = {level1:{level2:{level3:"found"}}};
json = serializeJSON(deep);
rt = deserializeJSON(json);
assert("deep nesting roundtrip", rt.level1.level2.level3, "found");

// array of arrays
nested = [[1,2],[3,4]];
json = serializeJSON(nested);
rt = deserializeJSON(json);
assert("array of arrays [1][1]", rt[1][1], 1);
assert("array of arrays [2][2]", rt[2][2], 4);

// special characters in strings
special = {msg:"hello ""world"""};
json = serializeJSON(special);
assertTrue("special chars is JSON", isJSON(json));

// numeric precision
nums = {intVal:42, floatVal:3.14};
json = serializeJSON(nums);
rt = deserializeJSON(json);
assert("numeric precision int", rt.intVal, 42);
suiteEnd();
</cfscript>
