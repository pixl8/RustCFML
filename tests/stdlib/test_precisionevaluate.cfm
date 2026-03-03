suiteBegin("precisionEvaluate");

// --- Basic arithmetic ---
assert("addition", precisionEvaluate("1 + 2"), "3");
assert("subtraction", precisionEvaluate("10 - 3"), "7");
assert("multiplication", precisionEvaluate("6 * 7"), "42");
assert("division", precisionEvaluate("10 / 4"), "2.5");

// --- Decimal precision ---
assert("decimal add", precisionEvaluate("0.1 + 0.2"), "0.3");
assert("large number", precisionEvaluate("99999999999999999 + 1"), "100000000000000000");

// --- Operator precedence ---
assert("precedence mul before add", precisionEvaluate("2 + 3 * 4"), "14");
assert("precedence div before sub", precisionEvaluate("10 - 6 / 3"), "8");

// --- Parentheses ---
assert("parentheses", precisionEvaluate("(2 + 3) * 4"), "20");
assert("nested parens", precisionEvaluate("((1 + 2) * (3 + 4))"), "21");

// --- Negative numbers ---
assert("unary minus", precisionEvaluate("-5 + 10"), "5");
assert("negative result", precisionEvaluate("3 - 8"), "-5");

// --- Division by zero throws ---
assertThrows("division by zero", function() {
    precisionEvaluate("1 / 0");
});

// --- Modulo ---
assert("modulo", precisionEvaluate("10 % 3"), "1");

suiteEnd();
