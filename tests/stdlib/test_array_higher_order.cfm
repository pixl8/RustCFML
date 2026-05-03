<cfscript>
suiteBegin("Array Higher-Order Functions");

// --- arrayMap ---
doubled = arrayMap([1, 2, 3], function(item) { return item * 2; });
assert("arrayMap doubles", arrayToList(doubled), "2,4,6");

// --- arrayFilter ---
evens = arrayFilter([1, 2, 3, 4, 5], function(item) { return item % 2 == 0; });
assert("arrayFilter evens", arrayToList(evens), "2,4");

// --- arrayReduce ---
total = arrayReduce([1, 2, 3, 4, 5], function(acc, item) { return acc + item; }, 0);
assert("arrayReduce sum", total, 15);

// --- arrayEach ---
result = "";
arrayEach([1, 2, 3], function(item) { result &= item; });
assert("arrayEach side effect", result, "123");

// --- arraySome ---
assertTrue("arraySome found", arraySome([1, 2, 3], function(item) { return item > 2; }));
assertFalse("arraySome not found", arraySome([1, 2, 3], function(item) { return item > 5; }));

// --- arrayEvery ---
assertTrue("arrayEvery all match", arrayEvery([1, 2, 3], function(item) { return item > 0; }));
assertFalse("arrayEvery not all match", arrayEvery([1, 2, 3], function(item) { return item > 2; }));

// --- Member syntax: map ---
memberDoubled = [1, 2, 3].map(function(item) { return item * 2; });
assert("member map", arrayToList(memberDoubled), "2,4,6");

// --- Member syntax: filter ---
memberEvens = [1, 2, 3, 4, 5].filter(function(item) { return item % 2 == 0; });
assert("member filter", arrayToList(memberEvens), "2,4");

// --- Member syntax: reduce ---
memberTotal = [1, 2, 3].reduce(function(acc, item) { return acc + item; }, 0);
assert("member reduce", memberTotal, 6);

// --- Chaining: filter then map ---
chained = [1, 2, 3, 4, 5].filter(function(item) { return item % 2 == 0; }).map(function(item) { return item * 10; });
assert("chained filter+map", arrayToList(chained), "20,40");

// --- arrayFindAll ---
ones = arrayFindAll([1, 2, 1, 3, 1], function(item) { return item == 1; });
assert("arrayFindAll count", arrayLen(ones), 3);
assert("arrayFindAll first index", ones[1], 1);
assert("arrayFindAll second index", ones[2], 3);
assert("arrayFindAll third index", ones[3], 5);

// --- Closure mutating captured outer-scope variable via member .each() ---
// The mutation must propagate back to the caller's scope. Both function-syntax
// and arrow-syntax callbacks must work.
collected = [];
[1, 2, 3].each(function(n) { collected.append(n); });
assert("member each propagates array append (fn)", arrayToList(collected), "1,2,3");

collected2 = [];
[1, 2, 3].each((n) => { collected2.append(n); });
assert("member each propagates array append (arrow)", arrayToList(collected2), "1,2,3");

// Query mutation via member each — the dashboard demo pattern.
team = queryNew("name,role", "varchar,varchar");
[{ name: "A", role: "X" }, { name: "B", role: "Y" }].each((m) => team.addRow(m));
assert("member each propagates query addRow", team.recordcount, 2);

suiteEnd();
</cfscript>
