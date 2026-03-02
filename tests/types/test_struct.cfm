<cfscript>
suiteBegin("Type: Struct");

// --- Struct literal ---
s = {name: "Alice", age: 30};
assert("struct literal name", s.name, "Alice");
assert("struct literal age", s.age, 30);

// --- Quoted keys ---
sq = {"first-name": "Bob", "last-name": "Smith"};
assert("quoted key access", sq["first-name"], "Bob");
assert("quoted key second", sq["last-name"], "Smith");

// --- Dot notation access ---
person = {city: "London"};
assert("dot notation", person.city, "London");

// --- Bracket notation access ---
assert("bracket notation", person["city"], "London");
key = "city";
assert("bracket with variable", person[key], "London");

// --- Case-insensitive key access ---
cs = {MyKey: "value"};
assert("case-insensitive dot lower", cs.mykey, "value");
assert("case-insensitive dot upper", cs.MYKEY, "value");
assert("case-insensitive bracket", cs["mykey"], "value");

// --- structCount ---
sc = {a: 1, b: 2, c: 3};
assert("structCount", structCount(sc), 3);

// --- structKeyExists ---
assertTrue("structKeyExists found", structKeyExists(sc, "a"));
assertFalse("structKeyExists missing", structKeyExists(sc, "z"));

// --- Nested structs ---
ns = {outer: {inner: "deep"}};
assert("nested struct access", ns.outer.inner, "deep");

// --- Struct modification ---
mutable = {x: 1};
mutable.y = 2;
assert("struct add key", mutable.y, 2);
assert("struct count after add", structCount(mutable), 2);
mutable.x = 99;
assert("struct update key", mutable.x, 99);

// --- structDelete ---
del = {a: 1, b: 2, c: 3};
structDelete(del, "b");
assert("structDelete count", structCount(del), 2);
assertFalse("structDelete key removed", structKeyExists(del, "b"));

// --- Empty struct ---
empty = {};
assert("empty struct count", structCount(empty), 0);

// --- Ordered struct ---
ordered = structNew("ordered");
ordered["first"] = 1;
ordered["second"] = 2;
ordered["third"] = 3;
keys = structKeyArray(ordered);
assert("ordered struct first key", keys[1], "first");
assert("ordered struct second key", keys[2], "second");
assert("ordered struct third key", keys[3], "third");

suiteEnd();
</cfscript>
