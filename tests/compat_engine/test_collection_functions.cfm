// Lucee 7 Compatibility Tests: Collection & String Higher-Order Functions
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness
<cfscript>
suiteBegin("Lucee7: Collection & String Higher-Order Functions");

// ============================================================
// CollectionEach (from Lucee CollectionEach.cfc)
// ============================================================
result = 0;
collectionEach([1, 2, 3], function(item) { result = result + item; });
assert("collectionEach array", result, 6);

keys = "";
collectionEach({a: 1, b: 2}, function(key) { keys = listAppend(keys, key); });
assertTrue("collectionEach struct", listLen(keys) == 2);

// ============================================================
// CollectionMap (from Lucee collectionMap.cfc)
// ============================================================
mapped = collectionMap([1, 2, 3], function(item) { return item * 2; });
assertTrue("collectionMap array is array", isArray(mapped));
assert("collectionMap arr first", mapped[1], 2);
assert("collectionMap arr second", mapped[2], 4);
assert("collectionMap arr third", mapped[3], 6);

// ============================================================
// CollectionFilter (from Lucee CollectionFilter.cfc)
// ============================================================
filtered = collectionFilter([1, 2, 3, 4, 5], function(item) { return item > 3; });
assert("collectionFilter len", arrayLen(filtered), 2);

// ============================================================
// CollectionReduce (from Lucee collectionReduce.cfc)
// ============================================================
assert("collectionReduce array", collectionReduce([1, 2, 3], function(acc, item) { return acc + item; }, 0), 6);

// ============================================================
// CollectionSome / CollectionEvery (from Lucee CollectionSome.cfc, CollectionEvery.cfc)
// ============================================================
assertTrue("collectionSome", collectionSome([1, 2, 3], function(item) { return item > 2; }));
assertTrue("collectionEvery", collectionEvery([2, 4, 6], function(item) { return item mod 2 == 0; }));

// ============================================================
// StringEach (from Lucee StringEach.cfc)
// ============================================================
chars = "";
stringEach("abc", function(c) { chars = chars & c & ","; });
assert("stringEach", chars, "a,b,c,");

// ============================================================
// StringMap (from Lucee StringMap.cfc)
// ============================================================
assert("stringMap", stringMap("abc", function(c) { return uCase(c); }), "ABC");

// ============================================================
// StringFilter (from Lucee StringFilter.cfc)
// ============================================================
assert("stringFilter", stringFilter("a1b2c3", function(c) { return isNumeric(c); }), "123");

// ============================================================
// StringReduce (from Lucee StringReduce.cfc)
// ============================================================
assert("stringReduce", stringReduce("abc", function(acc, c) { return acc & uCase(c); }, ""), "ABC");

// ============================================================
// StringSome / StringEvery (from Lucee StringSome.cfc, StringEvery.cfc)
// ============================================================
assertTrue("stringSome", stringSome("abc1", function(c) { return isNumeric(c); }));
assertFalse("stringSome no match", stringSome("abc", function(c) { return isNumeric(c); }));
assertTrue("stringEvery", stringEvery("123", function(c) { return isNumeric(c); }));
assertFalse("stringEvery fail", stringEvery("12a", function(c) { return isNumeric(c); }));

// ============================================================
// StringSort (from Lucee StringSort.cfc)
// ============================================================
assert("stringSort", stringSort("cba"), "abc");

suiteEnd();
</cfscript>
