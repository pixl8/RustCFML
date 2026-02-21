// Build a large array using member function
var arr = [];
for (var i = 1; i <= 10000; i++) {
    arr.append("item_" & i);
}
writeOutput("Array len: " & arrayLen(arr) & chr(10));

// String concatenation
var big = "";
for (var i = 1; i <= 1000; i++) {
    big = big & "x";
}
writeOutput("String len: " & len(big) & chr(10));

// Nested computation
function fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
writeOutput("fib(25): " & fib(25) & chr(10));
