<cfscript>
// Lucee 7 Compatibility Tests: Array Functions
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test/functions
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness

// ============================================================
// ArrayAppend (from Lucee ArrayAppend.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayAppend");
arr = [];
arrayAppend(arr, 1);
arrayAppend(arr, 2);
arrayAppend(arr, 3);
assert("arrayAppend basic", arrayLen(arr), 3);
assert("arrayAppend value check 1", arr[1], 1);
assert("arrayAppend value check 2", arr[2], 2);
assert("arrayAppend value check 3", arr[3], 3);

arr = [1];
arrayAppend(arr, [1,2,3]);
assert("arrayAppend array as element - len", arrayLen(arr), 2);
assertTrue("arrayAppend array as element - isArray", isArray(arr[2]));
assert("arrayAppend array as element - nested len", arrayLen(arr[2]), 3);

arr = [10];
arrayAppend(arr, 20);
arrayAppend(arr, 30);
assert("arrayAppend sequential - list", arrayToList(arr), "10,20,30");
suiteEnd();

// ============================================================
// ArrayLen / ArrayIsEmpty (from Lucee ArrayLen.cfc, ArrayIsEmpty.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayLen / ArrayIsEmpty");
assert("arrayLen empty", arrayLen([]), 0);
assert("arrayLen one element", arrayLen([1]), 1);
assert("arrayLen three elements", arrayLen([1,2,3]), 3);
assertTrue("arrayIsEmpty on empty array", arrayIsEmpty([]));
assertFalse("arrayIsEmpty on non-empty array", arrayIsEmpty([1]));
assertFalse("arrayIsEmpty on multi-element array", arrayIsEmpty([1,2,3]));
suiteEnd();

// ============================================================
// ArrayPrepend (from Lucee ArrayPrepend.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayPrepend");
arr = [2,3];
arrayPrepend(arr, 1);
assert("arrayPrepend first element", arr[1], 1);
assert("arrayPrepend preserves order", arrayToList(arr), "1,2,3");
assert("arrayPrepend len", arrayLen(arr), 3);

arr = ["b","c"];
arrayPrepend(arr, "a");
assert("arrayPrepend strings", arrayToList(arr), "a,b,c");
suiteEnd();

// ============================================================
// ArrayDeleteAt / ArrayInsertAt (from Lucee ArrayDeleteAt.cfc, ArrayInsertAt.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayDeleteAt / ArrayInsertAt");
arr = [1,2,3];
arrayDeleteAt(arr, 2);
assert("arrayDeleteAt middle", arrayToList(arr), "1,3");
assert("arrayDeleteAt len", arrayLen(arr), 2);

arr = [1,2,3];
arrayDeleteAt(arr, 1);
assert("arrayDeleteAt first", arrayToList(arr), "2,3");

arr = [1,2,3];
arrayDeleteAt(arr, 3);
assert("arrayDeleteAt last", arrayToList(arr), "1,2");

arr = [1,3];
arrayInsertAt(arr, 2, 2);
assert("arrayInsertAt middle", arrayToList(arr), "1,2,3");
assert("arrayInsertAt len", arrayLen(arr), 3);

arr = [2,3];
arrayInsertAt(arr, 1, 1);
assert("arrayInsertAt at start", arrayToList(arr), "1,2,3");
suiteEnd();

// ============================================================
// ArrayFind / ArrayFindNoCase (from Lucee ArrayFind.cfc, ArrayFindNoCase.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayFind / ArrayFindNoCase");
assert("arrayFind found", arrayFind([1,2,3], 2), 2);
assert("arrayFind not found", arrayFind([1,2,3], 4), 0);
assert("arrayFind first element", arrayFind([1,2,3], 1), 1);
assert("arrayFind last element", arrayFind([1,2,3], 3), 3);
assert("arrayFind string", arrayFind(["a","b","c"], "b"), 2);
assert("arrayFind in empty array", arrayFind([], 1), 0);

assert("arrayFindNoCase", arrayFindNoCase(["A","B","C"], "b"), 2);
assert("arrayFindNoCase lowercase search", arrayFindNoCase(["hello","world"], "HELLO"), 1);
assert("arrayFindNoCase not found", arrayFindNoCase(["A","B","C"], "d"), 0);
suiteEnd();

// ============================================================
// ArrayFindAll / ArrayFindAllNoCase (from Lucee ArrayFindAll.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayFindAll / ArrayFindAllNoCase");
result = arrayFindAll([1,2,3,2], function(item){ return item == 2; });
assertTrue("arrayFindAll returns array", isArray(result));
assert("arrayFindAll count", arrayLen(result), 2);
assert("arrayFindAll first index", result[1], 2);
assert("arrayFindAll second index", result[2], 4);

result = arrayFindAll([1,2,3], function(item){ return item > 5; });
assert("arrayFindAll no match", arrayLen(result), 0);

result = arrayFindAll([10,20,30,40,50], function(item){ return item > 25; });
assert("arrayFindAll gt filter count", arrayLen(result), 3);
assert("arrayFindAll gt filter first", result[1], 3);
assert("arrayFindAll gt filter last", result[3], 5);
suiteEnd();

// ============================================================
// ArrayContains / ArrayContainsNoCase (from Lucee arrayContains.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayContains / ArrayContainsNoCase");
assertTrue("arrayContains found", arrayContains([1,2,3], 2));
assertFalse("arrayContains not found", arrayContains([1,2,3], 5));
assertTrue("arrayContains string", arrayContains(["hello","world"], "hello"));
assertFalse("arrayContains empty array", arrayContains([], 1));

assertTrue("arrayContainsNoCase found", arrayContainsNoCase(["Hello","World"], "hello"));
assertTrue("arrayContainsNoCase uppercase", arrayContainsNoCase(["Hello","World"], "WORLD"));
assertFalse("arrayContainsNoCase not found", arrayContainsNoCase(["Hello","World"], "foo"));
suiteEnd();

// ============================================================
// ArraySort (from Lucee ArraySort.cfc)
// ============================================================
suiteBegin("Lucee7: ArraySort");
arr = [3,1,2];
arraySort(arr, "numeric");
assert("arraySort numeric asc", arrayToList(arr), "1,2,3");

arr = [3,1,2];
arraySort(arr, "numeric", "desc");
assert("arraySort numeric desc", arrayToList(arr), "3,2,1");

arr = ["c","a","b"];
arraySort(arr, "text");
assert("arraySort text asc", arrayToList(arr), "a,b,c");

arr = ["c","a","b"];
arraySort(arr, "text", "desc");
assert("arraySort text desc", arrayToList(arr), "c,b,a");

arr = [5,3,1,4,2];
arraySort(arr, "numeric");
assert("arraySort numeric five elements", arrayToList(arr), "1,2,3,4,5");
suiteEnd();

// ============================================================
// ArrayReverse (from Lucee ArrayReverse.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayReverse");
assert("arrayReverse basic", arrayToList(arrayReverse([1,2,3])), "3,2,1");
assert("arrayReverse single", arrayToList(arrayReverse([1])), "1");
assert("arrayReverse empty", arrayLen(arrayReverse([])), 0);
assert("arrayReverse strings", arrayToList(arrayReverse(["a","b","c"])), "c,b,a");
suiteEnd();

// ============================================================
// ArraySlice / ArrayMid (from Lucee ArraySlice.cfc, ArrayMid.cfc)
// ============================================================
suiteBegin("Lucee7: ArraySlice / ArrayMid");
assert("arraySlice with length", arrayToList(arraySlice([1,2,3,4,5], 2, 3)), "2,3,4");
assert("arraySlice from start", arrayToList(arraySlice([1,2,3,4,5], 1, 2)), "1,2");
assert("arraySlice to end", arrayToList(arraySlice([1,2,3,4,5], 3)), "3,4,5");
assert("arraySlice single element", arrayToList(arraySlice([1,2,3,4,5], 3, 1)), "3");

assert("arrayMid with length", arrayToList(arrayMid([1,2,3,4,5], 2, 3)), "2,3,4");
assert("arrayMid from start", arrayToList(arrayMid([1,2,3,4,5], 1, 2)), "1,2");
suiteEnd();

// ============================================================
// ArrayToList (from Lucee ArrayToList.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayToList");
assert("arrayToList default delimiter", arrayToList([1,2,3]), "1,2,3");
assert("arrayToList custom delimiter", arrayToList([1,2,3], "-"), "1-2-3");
assert("arrayToList pipe delimiter", arrayToList([1,2,3], "|"), "1|2|3");
assert("arrayToList single element", arrayToList([1]), "1");
assert("arrayToList empty", arrayToList([]), "");
suiteEnd();

// ============================================================
// ArrayMerge
// ============================================================
suiteBegin("Lucee7: ArrayMerge");
assert("arrayMerge basic", arrayToList(arrayMerge([1,2], [3,4])), "1,2,3,4");
assert("arrayMerge empty first", arrayToList(arrayMerge([], [1,2])), "1,2");
assert("arrayMerge empty second", arrayToList(arrayMerge([1,2], [])), "1,2");
assert("arrayMerge both empty", arrayLen(arrayMerge([], [])), 0);
assert("arrayMerge strings", arrayToList(arrayMerge(["a","b"], ["c","d"])), "a,b,c,d");
suiteEnd();

// ============================================================
// ArrayClear (from Lucee ArrayClear.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayClear");
arr = [1,2,3];
arrayClear(arr);
assert("arrayClear len", arrayLen(arr), 0);
assertTrue("arrayClear isEmpty", arrayIsEmpty(arr));

arr = ["a","b","c"];
arrayClear(arr);
assert("arrayClear strings", arrayLen(arr), 0);
suiteEnd();

// ============================================================
// ArraySet
// ============================================================
suiteBegin("Lucee7: ArraySet");
arr = [];
arraySet(arr, 1, 5, 0);
assert("arraySet len", arrayLen(arr), 5);
assert("arraySet first value", arr[1], 0);
assert("arraySet last value", arr[5], 0);
assert("arraySet middle value", arr[3], 0);
suiteEnd();

// ============================================================
// ArraySwap (from Lucee ArraySwap.cfc)
// ============================================================
suiteBegin("Lucee7: ArraySwap");
arr = [1,2,3];
arraySwap(arr, 1, 3);
assert("arraySwap first", arr[1], 3);
assert("arraySwap last", arr[3], 1);
assert("arraySwap middle unchanged", arr[2], 2);

arr = ["a","b","c"];
arraySwap(arr, 1, 2);
assert("arraySwap strings first", arr[1], "b");
assert("arraySwap strings second", arr[2], "a");
suiteEnd();

// ============================================================
// ArrayMin / ArrayMax / ArrayAvg / ArraySum / ArrayMedian
// ============================================================
suiteBegin("Lucee7: Array Math Functions");
assert("arrayMin", arrayMin([3,1,2]), 1);
assert("arrayMin negative", arrayMin([-5,0,5]), -5);
assert("arrayMin single", arrayMin([42]), 42);
assert("arrayMax", arrayMax([3,1,2]), 3);
assert("arrayMax negative", arrayMax([-5,0,5]), 5);
assert("arrayMax single", arrayMax([42]), 42);
assert("arrayAvg", arrayAvg([2,4,6]), 4);
assert("arrayAvg decimal", arrayAvg([1,2]), 1.5);
assert("arraySum", arraySum([1,2,3]), 6);
assert("arraySum single", arraySum([10]), 10);
assert("arrayMedian odd", arrayMedian([1,2,3]), 2);
assert("arrayMedian even", arrayMedian([1,2,3,4]), 2.5);
assert("arrayMedian single", arrayMedian([5]), 5);
suiteEnd();

// ============================================================
// ArrayFilter (from Lucee ArrayFilter.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayFilter");
arr = ["hello","world"];
result = arrayFilter(arr, function(item){ return item == "hello"; });
assert("arrayFilter len", arrayLen(result), 1);
assert("arrayFilter value", result[1], "hello");

result = arrayFilter([1,2,3,4,5], function(item){ return item > 3; });
assert("arrayFilter numeric", arrayLen(result), 2);
assert("arrayFilter numeric values", arrayToList(result), "4,5");

result = arrayFilter([1,2,3], function(item){ return item > 10; });
assert("arrayFilter no match", arrayLen(result), 0);

result = arrayFilter([1,2,3,4,5,6], function(item){ return item mod 2 == 0; });
assert("arrayFilter even numbers", arrayToList(result), "2,4,6");
suiteEnd();

// ============================================================
// ArrayMap (from Lucee arrayMap.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayMap");
arr = [1,2,3];
result = arrayMap(arr, function(item){ return item * 2; });
assert("arrayMap double", arrayToList(result), "2,4,6");
assert("arrayMap original unchanged", arrayToList(arr), "1,2,3");

result = arrayMap(["a","b","c"], function(item){ return uCase(item); });
assert("arrayMap uCase", arrayToList(result), "A,B,C");

result = arrayMap([10,20,30], function(item){ return item + 1; });
assert("arrayMap add one", arrayToList(result), "11,21,31");
suiteEnd();

// ============================================================
// ArrayReduce (from Lucee arrayReduce.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayReduce");
assert("arrayReduce sum", arrayReduce([1,2,3,4], function(acc, item){ return acc + item; }, 0), 10);
assert("arrayReduce concat", arrayReduce(["a","b","c"], function(acc, item){ return acc & item; }, ""), "abc");
assert("arrayReduce with initial", arrayReduce([1,2,3], function(acc, item){ return acc + item; }, 100), 106);
assert("arrayReduce product", arrayReduce([2,3,4], function(acc, item){ return acc * item; }, 1), 24);
suiteEnd();

// ============================================================
// ArrayEach (from Lucee ArrayEach.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayEach");
total = 0;
arrayEach([1,2,3], function(item){ total += item; });
assert("arrayEach sum", total, 6);

items = [];
arrayEach(["a","b","c"], function(item){ arrayAppend(items, item); });
assert("arrayEach collect", arrayToList(items), "a,b,c");

count = 0;
arrayEach([10,20,30,40], function(item){ count++; });
assert("arrayEach count iterations", count, 4);
suiteEnd();

// ============================================================
// ArraySome / ArrayEvery (from Lucee arraySome.cfc, arrayEvery.cfc)
// ============================================================
suiteBegin("Lucee7: ArraySome / ArrayEvery");
assertTrue("arraySome found", arraySome([1,2,3], function(item){ return item > 2; }));
assertFalse("arraySome not found", arraySome([1,2,3], function(item){ return item > 5; }));
assertFalse("arraySome empty", arraySome([], function(item){ return true; }));
assertTrue("arraySome string match", arraySome(["a","b","c"], function(item){ return item == "b"; }));

assertTrue("arrayEvery all match", arrayEvery([2,4,6], function(item){ return item mod 2 == 0; }));
assertFalse("arrayEvery not all match", arrayEvery([2,3,6], function(item){ return item mod 2 == 0; }));
assertTrue("arrayEvery all positive", arrayEvery([1,2,3], function(item){ return item > 0; }));
suiteEnd();

// ============================================================
// ArrayFirst / ArrayLast (from Lucee ArrayFirst.cfc, ArrayLast.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayFirst / ArrayLast");
assert("arrayFirst numeric", arrayFirst([1,2,3]), 1);
assert("arrayFirst string", arrayFirst(["hello","world"]), "hello");
assert("arrayFirst single", arrayFirst([42]), 42);
assert("arrayLast numeric", arrayLast([1,2,3]), 3);
assert("arrayLast string", arrayLast(["hello","world"]), "world");
assert("arrayLast single", arrayLast([42]), 42);
suiteEnd();

// ============================================================
// ArrayPop / ArrayShift (from Lucee ArrayPop.cfc, ArrayShift.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayPop / ArrayShift");
arr = [1,2,3];
v = arrayPop(arr);
assert("arrayPop returns last", v, 3);

arr = [1,2,3];
v = arrayShift(arr);
assert("arrayShift returns first", v, 1);

v = arrayPop([10,20,30]);
assert("arrayPop inline", v, 30);

v = arrayShift([10,20,30]);
assert("arrayShift inline", v, 10);
suiteEnd();

// ============================================================
// ArraySplice (from Lucee ArraySplice.cfc)
// ============================================================
suiteBegin("Lucee7: ArraySplice");
// arraySplice returns the removed elements
removed = arraySplice([1,2,3,4,5], 2, 2);
assert("arraySplice removed elements", arrayToList(removed), "2,3");
assert("arraySplice removed count", arrayLen(removed), 2);

removed = arraySplice([1,2,3,4,5], 1, 3);
assert("arraySplice remove from start", arrayToList(removed), "1,2,3");

removed = arraySplice([1,2,3,4,5], 4, 2);
assert("arraySplice remove from end", arrayToList(removed), "4,5");
suiteEnd();

// ============================================================
// ArrayToStruct (from Lucee ArrayToStruct.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayToStruct");
s = arrayToStruct(["a","b","c"]);
assertTrue("arrayToStruct is struct", isStruct(s));
assert("arrayToStruct key 1", s[1], "a");
assert("arrayToStruct key 2", s[2], "b");
assert("arrayToStruct key 3", s[3], "c");
suiteEnd();

// ============================================================
// ArrayDelete / ArrayDeleteNoCase
// ============================================================
suiteBegin("Lucee7: ArrayDelete / ArrayDeleteNoCase");
// arrayDelete modifies in place and returns boolean
delArr = ["a","b","c"];
assertTrue("arrayDelete found", arrayDelete(delArr, "b"));
assert("arrayDelete result", arrayToList(delArr), "a,c");
assert("arrayDelete len", arrayLen(delArr), 2);

delArr2 = ["x","y","z"];
assertFalse("arrayDelete not found", arrayDelete(delArr2, "q"));
assert("arrayDelete unchanged", arrayToList(delArr2), "x,y,z");

// arrayDeleteNoCase returns boolean indicating if element was found
assertTrue("arrayDeleteNoCase found", arrayDeleteNoCase(["A","B","C"], "b"));
assertFalse("arrayDeleteNoCase not found", arrayDeleteNoCase(["A","B","C"], "x"));
suiteEnd();

// ============================================================
// ArrayResize (from Lucee ArrayResize.cfc)
// ============================================================
suiteBegin("Lucee7: ArrayResize");
arr = [];
arrayResize(arr, 5);
assert("arrayResize len", arrayLen(arr), 5);
suiteEnd();

// ============================================================
// ArrayIndexExists / ArrayIsDefined
// ============================================================
suiteBegin("Lucee7: ArrayIndexExists / ArrayIsDefined");
arr = [1,2,3];
assertTrue("arrayIndexExists valid", arrayIndexExists(arr, 2));
assertFalse("arrayIndexExists out of bounds", arrayIndexExists(arr, 5));
assertFalse("arrayIndexExists zero", arrayIndexExists(arr, 0));
assertTrue("arrayIndexExists first", arrayIndexExists(arr, 1));
assertTrue("arrayIndexExists last", arrayIndexExists(arr, 3));

assertTrue("arrayIsDefined valid", arrayIsDefined(arr, 2));
assertFalse("arrayIsDefined out of bounds", arrayIsDefined(arr, 5));
suiteEnd();
</cfscript>
