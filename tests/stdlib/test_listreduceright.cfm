<cfscript>
suiteBegin("listReduceRight");

// Basic: sum from right to left
var result = listReduceRight("1,2,3,4", function(acc, item) {
    return acc + item;
}, 0);
assert("listReduceRight sum", result, 10);

// With initial value: string concatenation (verify right-to-left order)
var result2 = listReduceRight("a,b,c", function(acc, item) {
    return acc & item;
}, "");
assert("listReduceRight right-to-left order", result2, "cba");

// With custom delimiter
var result3 = listReduceRight("x|y|z", function(acc, item) {
    return acc & item;
}, "", "|");
assert("listReduceRight custom delimiter", result3, "zyx");

// Single element list
var result4 = listReduceRight("only", function(acc, item) {
    return acc & item;
}, "start:");
assert("listReduceRight single element", result4, "start:only");

suiteEnd();
</cfscript>
