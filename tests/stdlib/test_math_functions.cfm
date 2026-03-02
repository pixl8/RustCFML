<cfscript>
suiteBegin("Math Functions");

// --- abs ---
assert("abs(-5)", abs(-5), 5);
assert("abs(5)", abs(5), 5);

// --- ceiling ---
assert("ceiling(3.2)", ceiling(3.2), 4);
assert("ceiling(-3.2)", ceiling(-3.2), -3);

// --- floor ---
assert("floor(3.8)", floor(3.8), 3);
assert("floor(-3.8)", floor(-3.8), -4);

// --- round ---
assert("round(3.5)", round(3.5), 4);
assert("round(3.4)", round(3.4), 3);

// --- int ---
assert("int(3.9)", int(3.9), 3);

// --- fix ---
assert("fix(-3.9)", fix(-3.9), -3);

// --- max / min ---
assert("max(10, 20)", max(10, 20), 20);
assert("min(10, 20)", min(10, 20), 10);

// --- sgn ---
assert("sgn(-5)", sgn(-5), -1);
assert("sgn(0)", sgn(0), 0);
assert("sgn(5)", sgn(5), 1);

// --- sqr ---
assert("sqr(16)", sqr(16), 4);

// --- exp ---
assert("exp(0)", exp(0), 1);

// --- log ---
assert("log(1)", log(1), 0);

// --- bitwise ---
assert("bitAnd(15, 9)", bitAnd(15, 9), 9);
assert("bitOr(9, 6)", bitOr(9, 6), 15);
assert("bitXor(15, 9)", bitXor(15, 9), 6);

// --- rand ---
r = rand();
assertTrue("rand() >= 0", r >= 0);
assertTrue("rand() < 1", r < 1);

// --- randRange ---
rr = randRange(1, 10);
assertTrue("randRange(1,10) >= 1", rr >= 1);
assertTrue("randRange(1,10) <= 10", rr <= 10);

suiteEnd();
</cfscript>
