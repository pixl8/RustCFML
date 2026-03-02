<cfscript>
suiteBegin("Type: Numeric");

// --- Integer literals ---
i = 42;
assert("integer literal", i, 42);
assert("zero literal", 0, 0);

// --- Decimal/float literals ---
d = 3.14;
assert("decimal literal", d, 3.14);
small = 0.001;
assert("small decimal", small, 0.001);

// --- Negative numbers ---
neg = -10;
assert("negative integer", neg, -10);
negDec = -2.5;
assert("negative decimal", negDec, -2.5);

// --- Numeric precision (division) ---
divResult = 10 / 3;
// Lucee returns a high-precision decimal; check it starts correctly
assertTrue("10/3 is numeric", isNumeric(divResult));
assertTrue("10/3 greater than 3.33", divResult > 3.33);
assertTrue("10/3 less than 3.34", divResult < 3.34);

// --- Integer arithmetic ---
assert("integer addition", 5 + 3, 8);
assert("integer subtraction", 10 - 4, 6);
assert("integer multiplication", 6 * 7, 42);
assert("integer modulus", 10 % 3, 1);

// --- Large numbers ---
big = 999999999;
assert("large number", big, 999999999);
bigCalc = big + 1;
assert("large number + 1", bigCalc, 1000000000);

// --- isNumeric checks ---
assertTrue("isNumeric(42)", isNumeric(42));
assertTrue("isNumeric(3.14)", isNumeric(3.14));
assertTrue("isNumeric('123')", isNumeric("123"));
assertTrue("isNumeric('-5.5')", isNumeric("-5.5"));
assertFalse("isNumeric('abc')", isNumeric("abc"));
assertFalse("isNumeric('')", isNumeric(""));

// --- val() parsing ---
assert("val('123abc')", val("123abc"), 123);
assert("val('abc')", val("abc"), 0);
assert("val('  42  ')", val("  42  "), 42);

suiteEnd();
</cfscript>
