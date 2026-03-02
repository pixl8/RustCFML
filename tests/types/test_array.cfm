<cfscript>
suiteBegin("Type: Array");

// --- Array literal ---
arr = [1, 2, 3];
assert("array literal length", arrayLen(arr), 3);

// --- 1-based indexing ---
assert("1-based index first", arr[1], 1);
assert("1-based index second", arr[2], 2);
assert("1-based index third", arr[3], 3);

// --- arrayLen ---
assert("arrayLen", arrayLen(arr), 3);

// --- arrayAppend ---
arr2 = [10, 20];
arrayAppend(arr2, 30);
assert("arrayAppend length", arrayLen(arr2), 3);
assert("arrayAppend value", arr2[3], 30);

// --- arrayDeleteAt ---
arr3 = ["a", "b", "c", "d"];
arrayDeleteAt(arr3, 2);
assert("arrayDeleteAt length", arrayLen(arr3), 3);
assert("arrayDeleteAt shifted", arr3[2], "c");

// --- arrayPrepend ---
arr4 = [2, 3];
arrayPrepend(arr4, 1);
assert("arrayPrepend first element", arr4[1], 1);
assert("arrayPrepend length", arrayLen(arr4), 3);

// --- Nested arrays ---
nested = [[1, 2], [3, 4]];
assert("nested array access", nested[1][2], 2);
assert("nested array second row", nested[2][1], 3);

// --- Empty array ---
empty = [];
assert("empty array length", arrayLen(empty), 0);

// --- Array of mixed types ---
mixed = [1, "two", true, 4.5];
assert("mixed array numeric", mixed[1], 1);
assert("mixed array string", mixed[2], "two");
assert("mixed array boolean", mixed[3], true);
assert("mixed array decimal", mixed[4], 4.5);

// --- Array copy with duplicate ---
original = [1, 2, 3];
copied = duplicate(original);
arrayAppend(copied, 4);
assert("duplicate original unchanged", arrayLen(original), 3);
assert("duplicate copy modified", arrayLen(copied), 4);

// --- Array modification via index ---
modArr = [10, 20, 30];
modArr[2] = 99;
assert("array set by index", modArr[2], 99);

suiteEnd();
</cfscript>
