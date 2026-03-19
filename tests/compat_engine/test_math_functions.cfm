// Lucee 7 Compatibility Tests: Math Functions
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test/functions
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness

// ============================================================
// Abs (from Lucee Abs.cfc)
// ============================================================
suiteBegin("Lucee7: Abs");
assert("abs(1)", abs(1), 1);
assert("abs(1.9)", abs(1.9), 1.9);
assert("abs(-1.9)", abs(-1.9), 1.9);
assert("abs(0)", abs(0), 0);
assert("abs(-0)", abs(-0), 0);
assert("abs(string '0')", abs("0"), 0);
suiteEnd();

// ============================================================
// Ceiling (from Lucee Ceiling.cfc)
// ============================================================
suiteBegin("Lucee7: Ceiling");
assert("ceiling(1.1)", ceiling(1.1), 2);
assert("ceiling(-1.1)", ceiling(-1.1), -1);
assert("ceiling(1)", ceiling(1), 1);
assert("ceiling(0)", ceiling(0), 0);
suiteEnd();

// ============================================================
// Floor (from Lucee Floor.cfc)
// ============================================================
suiteBegin("Lucee7: Floor");
assert("floor(1.9)", floor(1.9), 1);
assert("floor(-1.9)", floor(-1.9), -2);
assert("floor(1)", floor(1), 1);
assert("floor(0)", floor(0), 0);
suiteEnd();

// ============================================================
// Fix (from Lucee Fix.cfc)
// ============================================================
suiteBegin("Lucee7: Fix");
assert("fix(1.9)", fix(1.9), 1);
assert("fix(-1.9)", fix(-1.9), -1);
assert("fix(0)", fix(0), 0);
assert("fix(1)", fix(1), 1);
suiteEnd();

// ============================================================
// Round (from Lucee Round.cfc)
// ============================================================
suiteBegin("Lucee7: Round");
assert("round(1.5)", round(1.5), 2);
assert("round(2.5)", round(2.5), 3);
assert("round(-1.5)", round(-1.5), -2);
assert("round(1.4)", round(1.4), 1);
assert("round(0)", round(0), 0);
suiteEnd();

// ============================================================
// Int (from Lucee Int.cfc)
// ============================================================
suiteBegin("Lucee7: Int");
assert("int(1.9)", int(1.9), 1);
assert("int(-1.9)", int(-1.9), -2);
assert("int(0)", int(0), 0);
assert("int(1)", int(1), 1);
suiteEnd();

// ============================================================
// Sqr / Sqrt (from Lucee Sqr.cfc)
// ============================================================
suiteBegin("Lucee7: Sqr/Sqrt");
assert("sqr(4)", sqr(4), 2);
assert("sqr(9)", sqr(9), 3);
assert("sqr(0)", sqr(0), 0);
assert("sqrt(4)", sqrt(4), 2);
assert("sqr(1)", sqr(1), 1);
suiteEnd();

// ============================================================
// Pow (from Lucee Pow.cfc)
// ============================================================
suiteBegin("Lucee7: Pow");
assert("pow(2,3)", pow(2, 3), 8);
assert("pow(2,0)", pow(2, 0), 1);
assert("pow(10,2)", pow(10, 2), 100);
assert("pow(5,1)", pow(5, 1), 5);
suiteEnd();

// ============================================================
// Exp (from Lucee Exp.cfc)
// ============================================================
suiteBegin("Lucee7: Exp");
assert("exp(0)", exp(0), 1);
assertTrue("exp(1) approx 2.718", abs(exp(1) - 2.718281828) < 0.0001);
suiteEnd();

// ============================================================
// Log / Log10 (from Lucee Log.cfc, Log10.cfc)
// ============================================================
suiteBegin("Lucee7: Log/Log10");
assert("log(1)", log(1), 0);
assertTrue("log(exp(1)) approx 1", abs(log(exp(1)) - 1) < 0.0001);
assert("log10(100)", log10(100), 2);
assert("log10(1)", log10(1), 0);
assert("log10(10)", log10(10), 1);
suiteEnd();

// ============================================================
// Trig: Sin, Cos, Tan, ASin, ACos (from Lucee Sin.cfc, Cos.cfc, Tan.cfc, ASin.cfc, ACos.cfc)
// ============================================================
suiteBegin("Lucee7: Trig");
assert("sin(0)", sin(0), 0);
assert("cos(0)", cos(0), 1);
assert("tan(0)", tan(0), 0);
assert("asin(0)", asin(0), 0);
assert("acos(1)", acos(1), 0);
assertTrue("pi() approx 3.14159", abs(pi() - 3.14159265358979) < 0.0001);
assertTrue("sin(pi()/2) approx 1", abs(sin(pi() / 2) - 1) < 0.0001);
assertTrue("cos(pi()) approx -1", abs(cos(pi()) - (-1)) < 0.0001);
assertTrue("asin(1) approx pi/2", abs(asin(1) - pi() / 2) < 0.0001);
assertTrue("acos(0) approx pi/2", abs(acos(0) - pi() / 2) < 0.0001);
suiteEnd();

// ============================================================
// Min / Max (from Lucee Min.cfc, Max.cfc)
// ============================================================
suiteBegin("Lucee7: Min/Max");
assert("min(1,2)", min(1, 2), 1);
assert("max(1,2)", max(1, 2), 2);
assert("min(-1,1)", min(-1, 1), -1);
assert("max(-1,1)", max(-1, 1), 1);
assert("min(0,0)", min(0, 0), 0);
assert("max(0,0)", max(0, 0), 0);
suiteEnd();

// ============================================================
// Sgn (from Lucee Sgn.cfc)
// ============================================================
suiteBegin("Lucee7: Sgn");
assert("sgn(5)", sgn(5), 1);
assert("sgn(-5)", sgn(-5), -1);
assert("sgn(0)", sgn(0), 0);
assert("sgn(100)", sgn(100), 1);
assert("sgn(-0.5)", sgn(-0.5), -1);
suiteEnd();

// ============================================================
// Rand / RandRange (from Lucee Rand.cfc, RandRange.cfc)
// ============================================================
suiteBegin("Lucee7: Rand/RandRange");
r = rand();
assertTrue("rand() >= 0", r >= 0);
assertTrue("rand() < 1", r < 1);
rr = randRange(1, 10);
assertTrue("randRange(1,10) >= 1", rr >= 1);
assertTrue("randRange(1,10) <= 10", rr <= 10);
rr2 = randRange(5, 5);
assert("randRange(5,5)", rr2, 5);
suiteEnd();

// ============================================================
// BitAnd / BitOr / BitXor / BitNot / BitSHLN / BitSHRN
// (from Lucee BitAnd.cfc, BitOr.cfc, BitXor.cfc, BitNot.cfc, BitSHLN.cfc, BitSHRN.cfc)
// ============================================================
suiteBegin("Lucee7: Bitwise");
assert("bitAnd(3,5)", bitAnd(3, 5), 1);
assert("bitOr(3,5)", bitOr(3, 5), 7);
assert("bitXor(3,5)", bitXor(3, 5), 6);
assert("bitNot(0)", bitNot(0), -1);
assert("bitSHLN(1,3)", bitSHLN(1, 3), 8);
assert("bitSHRN(8,3)", bitSHRN(8, 3), 1);
assert("bitAnd(0,0)", bitAnd(0, 0), 0);
assert("bitOr(0,0)", bitOr(0, 0), 0);
assert("bitXor(255,255)", bitXor(255, 255), 0);
suiteEnd();

// ============================================================
// FormatBaseN / InputBaseN (from Lucee FormatBaseN.cfc, InputBaseN.cfc)
// ============================================================
suiteBegin("Lucee7: FormatBaseN/InputBaseN");
assert("formatBaseN(255,16)", formatBaseN(255, 16), "ff");
assert("inputBaseN('ff',16)", inputBaseN("ff", 16), 255);
assert("formatBaseN(10,2)", formatBaseN(10, 2), "1010");
assert("inputBaseN('1010',2)", inputBaseN("1010", 2), 10);
assert("formatBaseN(255,8)", formatBaseN(255, 8), "377");
assert("inputBaseN('377',8)", inputBaseN("377", 8), 255);
assert("formatBaseN(0,16)", formatBaseN(0, 16), "0");
suiteEnd();

// ============================================================
// IncrementValue / DecrementValue (from Lucee IncrementValue.cfc, DecrementValue.cfc)
// ============================================================
suiteBegin("Lucee7: IncrementValue/DecrementValue");
assert("incrementValue(1)", incrementValue(1), 2);
assert("decrementValue(1)", decrementValue(1), 0);
assert("incrementValue(1.5)", incrementValue(1.5), 2.5);
assert("decrementValue(1.5)", decrementValue(1.5), 0.5);
assert("incrementValue(0)", incrementValue(0), 1);
assert("decrementValue(0)", decrementValue(0), -1);
assert("incrementValue(-1)", incrementValue(-1), 0);
suiteEnd();

// ============================================================
// Val (from Lucee Val.cfc)
// ============================================================
suiteBegin("Lucee7: Val");
assert("val('123abc')", val("123abc"), 123);
assert("val('abc')", val("abc"), 0);
assert("val('1.5')", val("1.5"), 1.5);
assert("val('')", val(""), 0);
assert("val('0')", val("0"), 0);
assert("val('-5.5xyz')", val("-5.5xyz"), -5.5);
suiteEnd();

// ============================================================
// IsNumeric (from Lucee IsNumeric.cfc)
// ============================================================
suiteBegin("Lucee7: IsNumeric");
assertTrue("isNumeric(1)", isNumeric(1));
assertFalse("isNumeric('abc')", isNumeric("abc"));
assertTrue("isNumeric('1.5')", isNumeric("1.5"));
assertTrue("isNumeric(0)", isNumeric(0));
assertTrue("isNumeric(-1)", isNumeric(-1));
assertTrue("isNumeric('100')", isNumeric("100"));
assertFalse("isNumeric('')", isNumeric(""));
suiteEnd();
