<cfscript>
suiteBegin("Struct Higher-Order Functions");

// --- structEach ---
eachResult = "";
structEach({a: 1, b: 2}, function(key, value) {
    eachResult &= key & "=" & value & ";";
});
assertTrue("structEach builds string", len(eachResult) > 0);
assertTrue("structEach contains a=1", find("A=1", uCase(eachResult)) > 0);

// --- structMap ---
mapped = structMap({a: 1, b: 2}, function(key, value) {
    return value * 10;
});
assert("structMap a", mapped.a, 10);
assert("structMap b", mapped.b, 20);

// --- structFilter ---
filtered = structFilter({a: 1, b: 2, c: 3}, function(key, value) {
    return value > 1;
});
assert("structFilter count", structCount(filtered), 2);
assertFalse("structFilter excludes a", structKeyExists(filtered, "a"));
assertTrue("structFilter includes b", structKeyExists(filtered, "b"));

// --- structReduce ---
reduced = structReduce({a: 1, b: 2, c: 3}, function(acc, key, value) {
    return acc + value;
}, 0);
assert("structReduce sum", reduced, 6);

// --- structSome ---
assertTrue("structSome found", structSome({a: 1, b: 5, c: 3}, function(key, value) {
    return value > 4;
}));
assertFalse("structSome not found", structSome({a: 1, b: 2}, function(key, value) {
    return value > 10;
}));

// --- structEvery ---
assertTrue("structEvery all positive", structEvery({a: 1, b: 2, c: 3}, function(key, value) {
    return value > 0;
}));
assertFalse("structEvery not all > 2", structEvery({a: 1, b: 2, c: 3}, function(key, value) {
    return value > 2;
}));

// --- Member syntax ---
memberMapped = {a: 1, b: 2}.map(function(key, value) { return value * 5; });
assert("member map a", memberMapped.a, 5);
assert("member map b", memberMapped.b, 10);

memberFiltered = {a: 1, b: 2, c: 3}.filter(function(key, value) { return value >= 2; });
assert("member filter count", structCount(memberFiltered), 2);

memberReduced = {a: 10, b: 20}.reduce(function(acc, key, value) { return acc + value; }, 0);
assert("member reduce", memberReduced, 30);

suiteEnd();
</cfscript>
