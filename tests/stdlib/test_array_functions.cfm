<cfscript>
suiteBegin("Array Functions");

// --- arrayNew / arrayLen ---
arr = arrayNew(1);
assert("arrayNew creates empty array", arrayLen(arr), 0);

// --- arrayAppend ---
arrayAppend(arr, "a");
assert("arrayAppend increases length", arrayLen(arr), 1);
assert("arrayAppend value", arr[1], "a");

// --- arrayPrepend ---
arrayPrepend(arr, "z");
assert("arrayPrepend first element", arr[1], "z");
assert("arrayPrepend length", arrayLen(arr), 2);

// --- arrayDeleteAt ---
arrayDeleteAt(arr, 1);
assert("arrayDeleteAt removes element", arr[1], "a");
assert("arrayDeleteAt length", arrayLen(arr), 1);

// --- arrayInsertAt ---
arr = [1, 3];
arrayInsertAt(arr, 2, 2);
assert("arrayInsertAt value", arr[2], 2);
assert("arrayInsertAt length", arrayLen(arr), 3);

// --- arrayLen with literal ---
assert("arrayLen literal", arrayLen([1, 2, 3]), 3);

// --- arrayContains ---
assertTrue("arrayContains found", arrayContains([1, 2, 3], 2));
assertFalse("arrayContains not found", arrayContains([1, 2, 3], 9));

// --- arrayFind / arrayFindNoCase ---
assert("arrayFind", arrayFind(["a", "b", "c"], "b"), 2);
assert("arrayFindNoCase", arrayFindNoCase(["A", "B"], "a"), 1);

// --- arraySort ---
sortArr = [3, 1, 2];
arraySort(sortArr, "numeric");
assert("arraySort first", sortArr[1], 1);
assert("arraySort last", sortArr[3], 3);

// --- arrayReverse ---
revArr = arrayReverse([1, 2, 3]);
assert("arrayReverse", arrayToList(revArr), "3,2,1");

// --- arraySlice ---
sliced = arraySlice([1, 2, 3, 4, 5], 2, 3);
assert("arraySlice", arrayToList(sliced), "2,3,4");

// --- arrayToList ---
assert("arrayToList default", arrayToList([1, 2, 3]), "1,2,3");
assert("arrayToList pipe", arrayToList([1, 2, 3], "|"), "1|2|3");

// --- arrayClear ---
clearArr = [1, 2, 3];
arrayClear(clearArr);
assert("arrayClear empties", arrayLen(clearArr), 0);

// --- arrayFirst / arrayLast ---
assert("arrayFirst", arrayFirst([10, 20]), 10);
assert("arrayLast", arrayLast([10, 20]), 20);

// --- arrayIsEmpty ---
assertTrue("arrayIsEmpty on empty", arrayIsEmpty([]));
assertFalse("arrayIsEmpty on non-empty", arrayIsEmpty([1]));

// --- arrayMin / arrayMax / arraySum / arrayAvg ---
assert("arrayMin", arrayMin([3, 1, 2]), 1);
assert("arrayMax", arrayMax([3, 1, 2]), 3);
assert("arraySum", arraySum([1, 2, 3]), 6);
assert("arrayAvg", arrayAvg([2, 4, 6]), 4);

// --- arrayMerge ---
merged = arrayMerge([1, 2], [3, 4]);
assert("arrayMerge length", arrayLen(merged), 4);

suiteEnd();
</cfscript>
