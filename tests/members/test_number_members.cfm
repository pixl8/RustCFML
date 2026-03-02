<cfscript>
suiteBegin("Number Member Functions");

// --- abs ---
// Use standalone function for portability; member syntax (-5).abs() may not work
// because -5 is a unary expression. Use abs() function form.
assert("abs(-5)", abs(-5), 5);

// --- ceiling ---
assert("ceiling(3.7)", ceiling(3.7), 4);

// --- floor ---
assert("floor(3.7)", floor(3.7), 3);

// --- round ---
assert("round(3.5)", round(3.5), 4);
assert("round(3.4)", round(3.4), 3);

// --- number member: toString ---
// Wrapping in parentheses to use member syntax on a number literal
assert("(42).toString()", (42).toString(), "42");

// --- int (truncate toward zero) ---
assert("int(3.9)", int(3.9), 3);

// --- sgn (sign) ---
assert("sgn(42)", sgn(42), 1);
assert("sgn(-7)", sgn(-7), -1);
assert("sgn(0)", sgn(0), 0);

suiteEnd();
</cfscript>
