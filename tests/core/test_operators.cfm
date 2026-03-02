<cfscript>
suiteBegin("Operators");

// --- Arithmetic ---
assert("addition", 3 + 4, 7);
assert("subtraction", 10 - 3, 7);
assert("multiplication", 6 * 7, 42);
assert("division", 20 / 4, 5);
assert("modulus %", 10 % 3, 1);
assert("MOD keyword", 10 MOD 3, 1);
assert("exponentiation ^", 2 ^ 3, 8);
assert("integer division \\", 7 \ 2, 3);

// --- String concatenation ---
assert("string concat &", "Hello" & " " & "World", "Hello World");
assert("concat with number", "Count: " & 5, "Count: 5");

// --- Comparison: symbolic ---
assertTrue("== equal", 5 == 5);
assertTrue("!= not equal", 5 != 4);
assertTrue("< less than", 3 < 5);
assertTrue("> greater than", 7 > 2);
assertTrue("<= less or equal", 5 <= 5);
assertTrue(">= greater or equal", 6 >= 6);

// --- Comparison: keyword ---
assertTrue("EQ", 10 EQ 10);
assertTrue("NEQ", 10 NEQ 11);
assertTrue("GT", 10 GT 5);
assertTrue("LT", 5 LT 10);
assertTrue("GTE equal case", 5 GTE 5);
assertTrue("LTE equal case", 5 LTE 5);

// --- CONTAINS / DOES NOT CONTAIN ---
assertTrue("CONTAINS", "Hello World" CONTAINS "World");
assertFalse("NOT CONTAINS false case", "Hello World" CONTAINS "Mars");
assertTrue("NOT CONTAINS", NOT ("Hello World" CONTAINS "Mars"));

// --- Logical: keyword ---
assertTrue("AND both true", true AND true);
assertFalse("AND one false", true AND false);
assertTrue("OR one true", false OR true);
assertFalse("OR both false", false OR false);
assertTrue("NOT false", NOT false);
assertFalse("NOT true", NOT true);

// --- Logical: symbolic ---
assertTrue("&& both true", true && true);
assertFalse("&& one false", true && false);
assertTrue("|| one true", false || true);
assertTrue("! false", !false);

// --- Ternary operator ---
x = 10;
assert("ternary true branch", (x > 5) ? "big" : "small", "big");
assert("ternary false branch", (x > 20) ? "big" : "small", "small");

// --- Unary: negation ---
pos = 5;
assert("unary negation", -pos, -5);

// --- Unary: ++ and -- ---
inc = 10;
inc++;
assert("post-increment", inc, 11);
inc--;
assert("post-decrement", inc, 10);

suiteEnd();
</cfscript>
