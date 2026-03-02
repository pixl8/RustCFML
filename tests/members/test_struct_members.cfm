<cfscript>
suiteBegin("Struct Member Functions");

// --- count ---
assert("struct.count()", {a: 1, b: 2}.count(), 2);

// --- isEmpty ---
assertFalse("struct.isEmpty() with keys", {a: 1, b: 2}.isEmpty());
assertTrue("empty struct.isEmpty()", {}.isEmpty());

// --- keyExists ---
assertTrue("struct.keyExists(a)", {a: 1, b: 2}.keyExists("a"));
assertFalse("struct.keyExists(z)", {a: 1, b: 2}.keyExists("z"));

// --- keyList ---
// Struct key order is not guaranteed; check that keyList contains expected keys
kl = {a: 1, b: 2}.keyList();
assertTrue("struct.keyList() contains A", findNoCase("A", kl) > 0);
assertTrue("struct.keyList() contains B", findNoCase("B", kl) > 0);

// --- keyArray ---
assert("struct.keyArray().len()", {a: 1, b: 2}.keyArray().len(), 2);

// --- insert (mutating) ---
s1 = {a: 1};
s1.insert("b", 2);
assert("struct.insert() then count", s1.count(), 2);

// --- delete (mutating) ---
s2 = {a: 1, b: 2};
s2.delete("a");
assert("struct.delete() then count", s2.count(), 1);

// --- find ---
assert("struct.find(a)", {a: 1, b: 2}.find("a"), 1);

// --- copy ---
s3 = {a: 1, b: 2};
s3copy = s3.copy();
assert("struct.copy().count()", s3copy.count(), 2);

// --- append ---
s4 = {a: 1};
s4.append({b: 2});
assert("struct.append() then count", s4.count(), 2);

suiteEnd();
</cfscript>
