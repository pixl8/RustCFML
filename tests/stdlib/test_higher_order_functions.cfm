<cfscript>
suiteBegin("Higher-Order Functions");

// --- listSome / listEvery ---
list = "1,2,3,4,5";
assert("listSome finds match", listSome(list, function(item) { return item > 3; }), true);
assert("listSome no match", listSome(list, function(item) { return item > 10; }), false);
assert("listEvery all match", listEvery(list, function(item) { return item > 0; }), true);
assert("listEvery not all match", listEvery(list, function(item) { return item > 2; }), false);

// listSome/listEvery with delimiter
pipeList = "a|b|c";
assert("listSome with delim", listSome(pipeList, function(item) { return item == "b"; }, "|"), true);
assert("listEvery with delim", listEvery(pipeList, function(item) { return len(item) == 1; }, "|"), true);

// --- String Higher-Order Functions ---
str = "Hello";

// stringEach
counter = 0;
stringEach(str, function(ch) { counter = counter + 1; });
assert("stringEach iterates", counter, 5);

// stringMap
assert("stringMap uppercase", stringMap(str, function(ch) { return uCase(ch); }), "HELLO");

// stringFilter
assert("stringFilter vowels", stringFilter(str, function(ch) { return findNoCase(ch, "aeiou") > 0; }), "eo");

// stringReduce
assert("stringReduce concat", stringReduce(str, function(acc, ch) { return acc & "-" & ch; }, ""), "-H-e-l-l-o");

// stringSome
assert("stringSome finds H", stringSome(str, function(ch) { return ch == "H"; }), true);
assert("stringSome no match", stringSome(str, function(ch) { return ch == "z"; }), false);

// stringEvery
assert("stringEvery all alpha", stringEvery(str, function(ch) { return reFind("[a-zA-Z]", ch) > 0; }), true);
assert("stringEvery not all H", stringEvery(str, function(ch) { return ch == "H"; }), false);

// stringSort
assert("stringSort default", stringSort("dcba"), "abcd");
assert("stringSort custom", stringSort("dcba", function(a, b) { return compare(b, a); }), "dcba");

// --- Collection Higher-Order Functions ---

// collectionEach with array
arr = [10, 20, 30];
total = 0;
collectionEach(arr, function(item) { total = total + item; });
assert("collectionEach array", total, 60);

// collectionEach with struct
s = { a: 1, b: 2 };
keys = "";
collectionEach(s, function(key, value) { keys = keys & key; });
assert("collectionEach struct", len(keys) > 0, true);

// collectionMap with array
result = collectionMap([1,2,3], function(item) { return item * 2; });
assert("collectionMap array", result[1], 2);

// collectionFilter with array
result = collectionFilter([1,2,3,4,5], function(item) { return item > 3; });
assert("collectionFilter array", arrayLen(result), 2);

// collectionReduce with array
result = collectionReduce([1,2,3,4], function(acc, item) { return acc + item; }, 0);
assert("collectionReduce array", result, 10);

// collectionSome / collectionEvery with array
assert("collectionSome array", collectionSome([1,2,3], function(item) { return item > 2; }), true);
assert("collectionEvery array", collectionEvery([1,2,3], function(item) { return item > 0; }), true);

// generic each() function
arr2 = [5, 10, 15];
sum = 0;
each(arr2, function(item) { sum = sum + item; });
assert("each() array", sum, 30);

suiteEnd();
</cfscript>
