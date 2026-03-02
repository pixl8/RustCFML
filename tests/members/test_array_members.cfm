<cfscript>
suiteBegin("Array Member Functions");

// --- len ---
assert("array.len()", [1, 2, 3].len(), 3);

// --- first / last ---
assert("array.first()", [1, 2, 3].first(), 1);
assert("array.last()", [1, 2, 3].last(), 3);

// --- toList ---
assert("array.toList()", [1, 2, 3].toList(), "1,2,3");
assert("array.toList(|)", [1, 2, 3].toList("|"), "1|2|3");

// --- sum / avg ---
assert("array.sum()", [10, 20, 30].sum(), 60);
assert("array.avg()", [10, 20, 30].avg(), 20);

// --- min / max ---
assert("array.min()", [3, 1, 2].min(), 1);
assert("array.max()", [3, 1, 2].max(), 3);

// --- isEmpty ---
assertTrue("empty array.isEmpty()", [].isEmpty());
assertFalse("[1].isEmpty()", [1].isEmpty());

// --- reverse + toList ---
assert("array.reverse().toList()", [1, 2, 3].reverse().toList(), "3,2,1");

// --- slice + toList ---
assert("array.slice(2,2).toList()", [1, 2, 3].slice(2, 2).toList(), "2,3");

// --- append (mutating) ---
arr = [1, 2];
arr.append(3);
assert("array.append() then len", arr.len(), 3);

// --- deleteAt (mutating) ---
arr2 = [1, 2, 3];
arr2.deleteAt(2);
assert("array.deleteAt().toList()", arr2.toList(), "1,3");

// --- find ---
assert("array.find(2)", [1, 2, 3].find(2), 2);

// --- contains ---
assertTrue("array.contains(2)", [1, 2, 3].contains(2));
assertFalse("array.contains(9)", [1, 2, 3].contains(9));

suiteEnd();
</cfscript>
