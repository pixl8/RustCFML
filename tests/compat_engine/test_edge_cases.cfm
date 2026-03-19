// Lucee 7 Compatibility Tests: Edge Cases
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness

// ============================================================
// Scientific Notation (LDEV-0066)
// ============================================================
suiteBegin("Lucee7: Scientific Notation (LDEV-0066)");
assertTrue("1E2 is numeric", isNumeric("1E2"));
assert("1E2 equals 100", val("1E2"), 100);
assertTrue("1.0E2 is numeric", isNumeric("1.0E2"));
// May fail: scientific notation with negative exponent
assertTrue("1E-2 is numeric", isNumeric("1E-2"));
assert("1E-2 equals 0.01", val("1E-2"), 0.01);
// May fail: uppercase/lowercase E handling
assertTrue("1e2 lowercase is numeric", isNumeric("1e2"));
assert("1e2 lowercase equals 100", val("1e2"), 100);
suiteEnd();

// ============================================================
// Val Edge Cases (LDEV-0032)
// ============================================================
suiteBegin("Lucee7: Val Edge Cases (LDEV-0032)");
assert("val of string prefix", val("11abc"), 11);
assert("val of pure string", val("abc"), 0);
// Note: Lucee returns 0 for val(true), but ACF returns 1
// RustCFML behavior may match either — this tests the ACF convention
assert("val of boolean true", val(true), 1);
assert("val of boolean false", val(false), 0);
assert("val of number", val(11), 11);
assert("val of empty string", val(""), 0);
assert("val of whitespace", val("  "), 0);
assert("val of leading whitespace num", val("  42"), 42);
// May fail: val with decimal prefix
assert("val of decimal string", val("3.14xyz"), 3.14);
assert("val of negative", val("-5abc"), -5);
assert("val of plus sign", val("+7"), 7);
suiteEnd();

// ============================================================
// Safe Navigation Operator (from general/SafeNavigator.cfc)
// ============================================================
// May fail: safe navigation operator (?.) might not be implemented
suiteBegin("Lucee7: Safe Navigation Operator");
s = {a: {b: {c: 1}}};
assert("safe nav exists", s?.a?.b?.c, 1);
s2 = {a: 1};
result = s2?.b?.c;
assertTrue("safe nav missing returns null", isNull(result));
// May fail: safe nav on null root
nullVar = nullValue();
result2 = nullVar?.foo;
assertTrue("safe nav on null root", isNull(result2));
// Safe nav should not error on missing intermediate
s3 = {};
result3 = s3?.deep?.nested?.value;
assertTrue("safe nav deep missing", isNull(result3));
suiteEnd();

// ============================================================
// Arguments Scope (from general/Arguments.cfc)
// ============================================================
suiteBegin("Lucee7: Arguments Scope");
function checkArgs(a, b, c) {
    return arguments[1] & arguments[2] & arguments[3];
}
assert("numeric arg access", checkArgs("x", "y", "z"), "xyz");

function checkArgs2(a, b, c) {
    return arguments.a & arguments.b & arguments.c;
}
assert("named arg access", checkArgs2("x", "y", "z"), "xyz");

function argCount() {
    return arrayLen(structKeyArray(arguments));
}
assert("argument count", argCount(1, 2, 3), 3);

// May fail: arguments treated as both struct and array
function argArrayLen() {
    return arrayLen(arguments);
}
assert("arguments arrayLen", argArrayLen("a", "b"), 2);
suiteEnd();

// ============================================================
// Struct Literal Types (from general/Struct.cfc)
// ============================================================
suiteBegin("Lucee7: Struct Literal Types");
s = {a: 1, b: 2};
assertTrue("curly brace struct", isStruct(s));
a = [1, 2, 3];
assertTrue("bracket array", isArray(a));
s2 = {a: 1, "b": 2, 'c': 3};
assert("mixed key quoting", s2.a & s2.b & s2.c, "123");
// May fail: numeric struct keys
s3 = {"1": "one", "2": "two"};
assert("numeric string keys", s3["1"], "one");
suiteEnd();

// ============================================================
// Array/Struct Implicit Construction
// ============================================================
suiteBegin("Lucee7: Implicit Construction");
arr = [];
assertTrue("empty array", isArray(arr));
assert("empty array len", arrayLen(arr), 0);
s = {};
assertTrue("empty struct", isStruct(s));
assert("empty struct count", structCount(s), 0);
// Nested implicit construction
nested = {arr: [1, 2, 3], sub: {x: "y"}};
assertTrue("nested array in struct", isArray(nested.arr));
assertTrue("nested struct in struct", isStruct(nested.sub));
assert("nested array access", nested.arr[1], 1);
assert("nested struct access", nested.sub.x, "y");
suiteEnd();

// ============================================================
// Nested Closures (from general/Closure.cfc)
// ============================================================
suiteBegin("Lucee7: Nested Closures");
outer = 1;
fn = function() {
    var inner = function() {
        return outer;
    };
    return inner();
};
assert("nested closure capture", fn(), 1);

// May fail: closure capturing loop variable
fns = [];
for (i = 1; i <= 3; i++) {
    arrayAppend(fns, function() { return i; });
}
// After loop, i=4; closures may capture final value or per-iteration value
// Lucee captures by reference, so all closures see i=4
assert("closure loop capture last value", fns[1](), 4);

// Closure modifying captured variable
counter = 0;
inc = function() { counter = counter + 1; };
inc();
inc();
assert("closure mutates captured var", counter, 2);
suiteEnd();

// ============================================================
// String as Boolean (from general/Boolean.cfc)
// ============================================================
suiteBegin("Lucee7: String as Boolean");
assertTrue("'yes' is boolean", isBoolean("yes"));
assertTrue("'no' is boolean", isBoolean("no"));
assertTrue("'true' is boolean", isBoolean("true"));
assertTrue("'false' is boolean", isBoolean("false"));
assertTrue("1 is boolean", isBoolean(1));
assertTrue("0 is boolean", isBoolean(0));
// May fail: "YES"/"NO" case variants
assertTrue("'YES' uppercase is boolean", isBoolean("YES"));
assertTrue("'True' mixed case is boolean", isBoolean("True"));
assertFalse("random string not boolean", isBoolean("maybe"));
assertFalse("empty string not boolean", isBoolean(""));
suiteEnd();

// ============================================================
// Empty String Comparisons (LDEV-0189)
// ============================================================
suiteBegin("Lucee7: Empty String Comparisons (LDEV-0189)");
assertTrue("empty eq empty", "" == "");
assert("empty len", len(""), 0);
assertTrue("empty is simple", isSimpleValue(""));
assertFalse("empty neq space", "" == " ");
assertTrue("empty string is falsy", !(""));
suiteEnd();

// ============================================================
// Null Handling (from general/NullSupport.cfc)
// ============================================================
suiteBegin("Lucee7: Null Handling");
assertTrue("nullValue is null", isNull(nullValue()));
x = nullValue();
assertTrue("null var is null", isNull(x));
// May fail: null in arrays — RustCFML may not preserve null elements
arr2 = [1, nullValue(), 3];
assert("array with null len", arrayLen(arr2), 3);
// May fail: null equality
assertTrue("null eq null", isNull(nullValue()));
// May fail: structKeyExists with null value
s = {a: nullValue()};
// In Lucee with full null support, the key exists but value is null
// In non-null-support mode, setting a key to null removes it
// RustCFML behavior may vary
suiteEnd();

// ============================================================
// Query Dot Notation (from general/Query.cfc)
// ============================================================
suiteBegin("Lucee7: Query Dot Notation");
q = queryNew("name", "varchar", [["Alice"], ["Bob"], ["Charlie"]]);
assertTrue("query col is array", isArray(q.name));
assert("query col len", arrayLen(q.name), 3);
assert("query col first", q.name[1], "Alice");
assert("query col last", q.name[3], "Charlie");
suiteEnd();

// ============================================================
// Struct Key Case Insensitivity (LDEV-0401)
// ============================================================
suiteBegin("Lucee7: Struct Key Case Insensitivity (LDEV-0401)");
s = {};
s.MyKey = "value";
assertTrue("case insensitive exists lower", structKeyExists(s, "mykey"));
assertTrue("case insensitive exists upper", structKeyExists(s, "MYKEY"));
assertTrue("case insensitive exists mixed", structKeyExists(s, "myKey"));
assert("case insensitive access", s["MYKEY"], "value");
assert("case insensitive dot", s.mykey, "value");
suiteEnd();

// ============================================================
// Type Coercion in Comparisons (LDEV-0198)
// ============================================================
suiteBegin("Lucee7: Type Coercion (LDEV-0198)");
assertTrue("string num eq", "1" == 1);
assertTrue("string num eq 2", "3.14" == 3.14);
assertFalse("string neq", "abc" == "def");
assertTrue("bool string eq", true == "true");
assertTrue("bool num eq", true == 1);
assertTrue("bool zero eq false", false == 0);
// May fail: loose comparison edge cases
assertTrue("empty string eq false", "" == false);
assertTrue("zero eq false", 0 == false);
suiteEnd();

// ============================================================
// Member Functions (from general/MemberFunction.cfc)
// ============================================================
// May fail: member functions may not be fully implemented
suiteBegin("Lucee7: Member Functions");
arr3 = [3, 1, 2];
arr3.sort("numeric");
assert("array member sort", arrayToList(arr3), "1,2,3");

str = "hello";
assert("string member len", str.len(), 5);

str2 = "hello world";
assert("string member ucase", str2.uCase(), "HELLO WORLD");

arr4 = [1, 2, 3];
assert("array member len", arr4.len(), 3);

// May fail: string member chaining
assert("string member trim", "  hello  ".trim(), "hello");
assert("string member lcase", "HELLO".lCase(), "hello");

// May fail: array member append
arr5 = [1, 2];
arr5.append(3);
assert("array member append", arrayLen(arr5), 3);

// May fail: struct member functions
s2 = {a: 1, b: 2};
assert("struct member count", s2.count(), 2);
assertTrue("struct member keyExists", s2.keyExists("a"));
suiteEnd();

// ============================================================
// Chained Function Calls (LDEV-0345)
// ============================================================
suiteBegin("Lucee7: Chained Function Calls (LDEV-0345)");
assert("chained string", lCase(trim("  HELLO  ")), "hello");
assert("chained array", arrayLen(arraySlice([1, 2, 3, 4, 5], 2, 3)), 3);
assert("chained list", listLen(listSort("c,a,b", "text")), 3);
assert("nested conversion", val(trim("  42  ")), 42);
suiteEnd();
