<cfscript>
suiteBegin("Struct Functions");

// --- structNew ---
s = structNew();
assertTrue("structNew creates struct", isStruct(s));
assert("structNew is empty", structCount(s), 0);

// --- structCount ---
assert("structCount", structCount({a: 1, b: 2}), 2);

// --- structKeyExists ---
assertTrue("structKeyExists found", structKeyExists({a: 1}, "a"));
assertFalse("structKeyExists not found", structKeyExists({a: 1}, "z"));

// --- structKeyList ---
keyList = structKeyList({a: 1, b: 2});
assertTrue("structKeyList contains A", listFindNoCase(keyList, "A") > 0);
assertTrue("structKeyList contains B", listFindNoCase(keyList, "B") > 0);

// --- structKeyArray ---
keys = structKeyArray({a: 1, b: 2});
assert("structKeyArray length", arrayLen(keys), 2);

// --- structDelete ---
del = {a: 1, b: 2};
structDelete(del, "a");
assertFalse("structDelete removes key", structKeyExists(del, "a"));
assert("structDelete count", structCount(del), 1);

// --- structInsert ---
ins = {};
structInsert(ins, "x", 99);
assert("structInsert", ins.x, 99);

// --- structUpdate ---
upd = {a: 1};
structUpdate(upd, "a", 42);
assert("structUpdate", upd.a, 42);

// --- structFind ---
assert("structFind", structFind({a: 1, b: 2}, "a"), 1);

// --- structClear ---
clr = {a: 1, b: 2};
structClear(clr);
assert("structClear empties", structCount(clr), 0);

// --- structCopy ---
original = {a: 1, b: 2};
copied = structCopy(original);
assert("structCopy count", structCount(copied), 2);
assert("structCopy value", copied.a, 1);

// --- structAppend ---
base = {a: 1};
extra = {b: 2, c: 3};
structAppend(base, extra);
assert("structAppend count", structCount(base), 3);
assert("structAppend merged value", base.b, 2);

// --- structIsEmpty ---
assertTrue("structIsEmpty on empty", structIsEmpty({}));
assertFalse("structIsEmpty on non-empty", structIsEmpty({a: 1}));

// --- structSort ---
sortStruct = {b: 2, a: 1, c: 3};
sorted = structSort(sortStruct, "text");
assert("structSort first key", sorted[1], "A");

// --- structValueArray ---
vals = structValueArray({a: 1});
assert("structValueArray length", arrayLen(vals), 1);
assert("structValueArray value", vals[1], 1);

// --- isEmpty on structs ---
assertTrue("isEmpty empty struct", isEmpty({}));
assertFalse("isEmpty non-empty struct", isEmpty({a: 1}));

suiteEnd();
</cfscript>
