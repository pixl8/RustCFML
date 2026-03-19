// Lucee 7 Compatibility Tests: Language Operators
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness

// ============================================================
// Elvis Operator (from Lucee general/Elvis.cfc)
// ============================================================
suiteBegin("Lucee7: Elvis Operator");
x = "hello";
assert("elvis with existing var", x ?: "default", "hello");
assert("elvis with null", nullValue() ?: "default", "default");
result = (isDefined("nonExistentVar")) ? "exists" : "nope";
assert("isDefined nonexistent", result, "nope");
y = "";
assert("elvis with empty string", y ?: "default", "");
z = 0;
assert("elvis with zero", z ?: "default", 0);
suiteEnd();

// ============================================================
// Ternary Operator
// ============================================================
suiteBegin("Lucee7: Ternary Operator");
assert("ternary true", true ? "yes" : "no", "yes");
assert("ternary false", false ? "yes" : "no", "no");
assert("ternary expression", (1 > 0) ? "greater" : "less", "greater");
assert("ternary nested", true ? (false ? "a" : "b") : "c", "b");
assert("ternary with math", (2 + 2 == 4) ? "correct" : "wrong", "correct");
suiteEnd();

// ============================================================
// Unary Operators (from Lucee general/Unary.cfc)
// ============================================================
suiteBegin("Lucee7: Unary Operators");
x = 5;
x++;
assert("post increment", x, 6);

x = 5;
x--;
assert("post decrement", x, 4);

x = 5;
assert("prefix increment", ++x, 6);
assert("prefix increment persists", x, 6);

x = 5;
assert("prefix decrement", --x, 4);
assert("prefix decrement persists", x, 4);

x = 5;
x += 3;
assert("plus equals", x, 8);

x = 10;
x -= 3;
assert("minus equals", x, 7);

x = 5;
x *= 3;
assert("times equals", x, 15);

x = 10;
x /= 2;
assert("divide equals", x, 5);

x = 10;
x %= 3;
assert("mod equals", x, 1);

x = "hello";
x &= " world";
assert("concat equals", x, "hello world");
suiteEnd();

// ============================================================
// Comparison Operators
// ============================================================
suiteBegin("Lucee7: Comparison Operators");
assertTrue("eq", 1 == 1);
assertFalse("neq", 1 == 2);
assertTrue("neq operator", 1 != 2);
assertFalse("neq operator false", 1 != 1);
assertTrue("gt", 2 > 1);
assertFalse("gt false", 1 > 2);
assertTrue("gte", 2 >= 2);
assertTrue("gte greater", 3 >= 2);
assertTrue("lt", 1 < 2);
assertFalse("lt false", 2 < 1);
assertTrue("lte", 2 <= 2);
assertTrue("lte less", 1 <= 2);
assertTrue("string eq", "hello" == "hello");
assertTrue("string eq case insensitive", "Hello" == "hello");
assertTrue("string compare EQ keyword", "Hello" EQ "hello");
assertTrue("string NEQ keyword", "hello" NEQ "world");
assertTrue("GT keyword", 5 GT 3);
assertTrue("LT keyword", 3 LT 5);
assertTrue("GTE keyword", 5 GTE 5);
assertTrue("LTE keyword", 5 LTE 5);
suiteEnd();

// ============================================================
// Logical Operators
// ============================================================
suiteBegin("Lucee7: Logical Operators");
assertTrue("and true", true && true);
assertFalse("and false", true && false);
assertFalse("and both false", false && false);
assertTrue("or true", true || false);
assertTrue("or both true", true || true);
assertFalse("or both false", false || false);
assertTrue("not false", !false);
assertFalse("not true", !true);
assertTrue("AND keyword", true AND true);
assertFalse("AND keyword false", true AND false);
assertTrue("OR keyword", false OR true);
assertFalse("OR keyword false", false OR false);
assertTrue("NOT keyword", NOT false);
assertFalse("NOT keyword true", NOT true);
suiteEnd();

// ============================================================
// String Concatenation
// ============================================================
suiteBegin("Lucee7: String Concatenation");
assert("concat basic", "hello" & " " & "world", "hello world");
x = "a";
assert("concat with var", x & "b", "ab");
assert("concat number coerce", "num:" & 42, "num:42");
assert("concat empty", "" & "test", "test");
assert("concat multiple", "a" & "b" & "c" & "d", "abcd");
suiteEnd();

// ============================================================
// Arithmetic Operators
// ============================================================
suiteBegin("Lucee7: Arithmetic Operators");
assert("addition", 2 + 3, 5);
assert("subtraction", 10 - 4, 6);
assert("multiplication", 3 * 4, 12);
assert("division", 10 / 2, 5);
assert("power", 2 ^ 3, 8);
assert("mod keyword", 10 MOD 3, 1);
assert("modulus operator", 10 % 3, 1);
assert("integer divide", 7 \ 2, 3);
assert("integer divide exact", 10 \ 5, 2);
assert("integer divide remainder discarded", 11 \ 3, 3);
assert("negative arithmetic", -5 + 3, -2);
assert("unary minus", -(5), -5);
assert("decimal arithmetic", 1.5 + 2.5, 4);
assert("order of operations", 2 + 3 * 4, 14);
assert("parentheses", (2 + 3) * 4, 20);
suiteEnd();

// ============================================================
// Null Coalescing (from Lucee SafeNavOp.cfc patterns)
// ============================================================
suiteBegin("Lucee7: Null Coalescing");
assert("null coalesce with nullValue()", nullValue() ?: "default", "default");
val = "exists";
assert("null coalesce with value", val ?: "default", "exists");
assert("null coalesce chain", nullValue() ?: nullValue() ?: "final", "final");
suiteEnd();
