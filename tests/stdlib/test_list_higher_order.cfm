<cfscript>
suiteBegin("List Higher-Order Functions");

// --- listEach ---
result = "";
listEach("a,b,c", function(item) {
    result &= item;
});
assert("listEach builds abc", result, "abc");

// --- listMap ---
mapped = listMap("1,2,3", function(item) {
    return item * 2;
});
assert("listMap first item doubled", listGetAt(mapped, 1), 2);
assert("listMap second item doubled", listGetAt(mapped, 2), 4);
assert("listMap third item doubled", listGetAt(mapped, 3), 6);

// --- listFilter ---
filtered = listFilter("1,2,3,4,5", function(item) {
    return item % 2 == 0;
});
assert("listFilter keeps evens len", listLen(filtered), 2);
assert("listFilter first even", listFirst(filtered), 2);
assert("listFilter last even", listLast(filtered), 4);

// --- listReduce ---
total = listReduce("1,2,3,4,5", function(acc, item) {
    return acc + item;
}, 0);
assert("listReduce sum", total, 15);

// --- listMap with string transform ---
upper = listMap("a,b,c", function(item) {
    return uCase(item);
});
assert("listMap uCase", upper, "A,B,C");

// --- listFilter with string condition ---
long = listFilter("hi,hello,hey,howdy", function(item) {
    return len(item) > 3;
});
assert("listFilter by length", listLen(long), 2);

suiteEnd();
</cfscript>
