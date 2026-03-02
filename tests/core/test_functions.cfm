<cfscript>
suiteBegin("Functions");

// --- Basic UDF ---
function add(a, b) {
    return a + b;
}
assert("basic UDF", add(3, 4), 7);

// --- Function with no return (returns null implicitly) ---
function doNothing() {
    var x = 1;
}
assertTrue("function with no explicit return", isNull(doNothing()));

// --- Recursion (factorial) ---
function factorial(n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
assert("recursion factorial(5)", factorial(5), 120);
assert("recursion factorial(1)", factorial(1), 1);
assert("recursion factorial(0)", factorial(0), 1);

// --- Default argument values ---
function greet(name, greeting="Hello") {
    return greeting & " " & name;
}
assert("default arg used", greet("World"), "Hello World");
assert("default arg overridden", greet("World", "Hi"), "Hi World");

// --- Closures ---
multiply = function(a, b) {
    return a * b;
};
assert("closure basic", multiply(3, 5), 15);

// --- Arrow functions ---
double = (x) => x * 2;
assert("arrow function", double(7), 14);

// --- Arrow function with block body ---
clamp = function(val, lo, hi) {
    if (val < lo) return lo;
    if (val > hi) return hi;
    return val;
};
assert("closure block body - clamped low", clamp(-5, 0, 10), 0);
assert("closure block body - clamped high", clamp(15, 0, 10), 10);
assert("closure block body - in range", clamp(5, 0, 10), 5);

// --- Function as argument (higher-order) ---
function applyOp(a, b, op) {
    return op(a, b);
}
result = applyOp(10, 3, function(x, y) { return x - y; });
assert("function as argument", result, 7);

arrowResult = applyOp(4, 5, function(x, y) { return x + y; });
assert("closure as argument", arrowResult, 9);

// --- Nested function calls ---
function square(n) { return n * n; }
function sumOfSquares(a, b) { return square(a) + square(b); }
assert("nested function calls", sumOfSquares(3, 4), 25);

// --- Access modifiers ---
public function pubFn() { return "public"; }
private function privFn() { return "private"; }
assert("public function", pubFn(), "public");
assert("private function (direct call)", privFn(), "private");

// --- Function returning function (closure factory) ---
function makeAdder(n) {
    return function(x) {
        return x + n;
    };
}
addFive = makeAdder(5);
assert("closure factory", addFive(10), 15);

addTen = makeAdder(10);
assert("closure factory second instance", addTen(3), 13);

// --- Closure captures variable by reference ---
base = 100;
addToBase = function(x) { return base + x; };
assert("closure captures outer var", addToBase(5), 105);

suiteEnd();
</cfscript>
