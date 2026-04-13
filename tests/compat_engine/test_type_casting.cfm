<cfscript>
// Lucee 7 Compatibility Tests: Type Casting
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness

// ============================================================
// Implicit casting (from Lucee general/Casting.cfc)
// ============================================================
suiteBegin("Lucee7: Implicit casting");
zero = 0;
assert("string .1 + 0 = 0.1", ".1" + zero, 0.1);
assert("string 1 + 0 = 1", "1" + zero, 1);
assert("string 1.1 + 0 = 1.1", "1.1" + zero, 1.1);
assert("string 100 + 0", "100" + zero, 100);
assert("string -5 + 0", "-5" + zero, -5);
assert("true + 0 = 1", true + zero, 1);
assert("false + 0 = 0", false + zero, 0);
suiteEnd();

// ============================================================
// Val (from Lucee Val.cfc)
// ============================================================
suiteBegin("Lucee7: val()");
assert("val integer string", val("123"), 123);
assert("val alpha string", val("abc"), 0);
assert("val empty string", val(""), 0);
assert("val leading number", val("123abc"), 123);
assert("val float string", val("1.5"), 1.5);
assert("val negative", val("-3.5"), -3.5);
assert("val positive sign", val("+5"), 5);
assert("val 100", val("100"), 100);
assert("val zero string", val("0"), 0);
assert("val whitespace prefix", val("  42"), 42);
assert("val just whitespace", val("   "), 0);
suiteEnd();

// ============================================================
// toNumeric (from Lucee ToNumeric.cfc)
// ============================================================
suiteBegin("Lucee7: toNumeric()");
assert("toNumeric integer string", toNumeric("42"), 42);
assert("toNumeric float string", toNumeric("3.14"), 3.14);
assert("toNumeric negative", toNumeric("-10"), -10);
assert("toNumeric int passthrough", toNumeric(99), 99);
assert("toNumeric bool true", toNumeric(true), 1);
assert("toNumeric bool false", toNumeric(false), 0);
assertThrows("toNumeric non-numeric throws", function(){
    toNumeric("abc");
});
suiteEnd();

// ============================================================
// toBoolean
// ============================================================
suiteBegin("Lucee7: Boolean Casting");
assertTrue("boolean cast 1", !!1);
assertFalse("boolean cast 0", !!0);
assertTrue("boolean cast yes", !!"yes");
assertFalse("boolean cast no", !!"no");
assertTrue("boolean cast true string", !!"true");
assertFalse("boolean cast false string", !!"false");
assertTrue("boolean cast true literal", !!true);
assertFalse("boolean cast false literal", !!false);
suiteEnd();

// ============================================================
// toString (from Lucee ToString.cfc)
// ============================================================
suiteBegin("Lucee7: toString()");
assert("toString integer", toString(123), "123");
assert("toString zero", toString(0), "0");
assert("toString negative", toString(-5), "-5");
assert("toString string passthrough", toString("hello"), "hello");
// toString on binary
b64 = toBase64("test");
bin = toBinary(b64);
assert("toString binary", toString(bin), "test");
suiteEnd();

// ============================================================
// yesNoFormat (from Lucee YesNoFormat.cfc)
// ============================================================
suiteBegin("Lucee7: yesNoFormat()");
assert("yesNoFormat true", yesNoFormat(true), "Yes");
assert("yesNoFormat false", yesNoFormat(false), "No");
assert("yesNoFormat 1", yesNoFormat(1), "Yes");
assert("yesNoFormat 0", yesNoFormat(0), "No");
assert("yesNoFormat yes string", yesNoFormat("yes"), "Yes");
assert("yesNoFormat no string", yesNoFormat("no"), "No");
suiteEnd();

// ============================================================
// trueFalseFormat
// ============================================================
suiteBegin("Lucee7: trueFalseFormat()");
assert("trueFalseFormat true", trueFalseFormat(true), true);
assert("trueFalseFormat false", trueFalseFormat(false), false);
assert("trueFalseFormat 1", trueFalseFormat(1), true);
assert("trueFalseFormat 0", trueFalseFormat(0), false);
suiteEnd();

// ============================================================
// numberFormat (from Lucee NumberFormat.cfc)
// ============================================================
suiteBegin("Lucee7: numberFormat()");
// basic call should not error
nf1 = numberFormat(1234);
assertTrue("numberFormat basic returns string", len(nf1) > 0);

nf2 = numberFormat(1234.5, "9,999.99");
assertTrue("numberFormat with mask returns string", len(nf2) > 0);
assertTrue("numberFormat with mask contains decimal", find(".", nf2) > 0);

nf3 = numberFormat(0);
assertTrue("numberFormat zero", len(nf3) > 0);
suiteEnd();

// ============================================================
// decimalFormat (from Lucee DecimalFormat.cfc)
// ============================================================
suiteBegin("Lucee7: decimalFormat()");
df1 = decimalFormat(1234.5);
assertTrue("decimalFormat returns string", len(df1) > 0);
assertTrue("decimalFormat contains decimal", find(".", df1) > 0);

df2 = decimalFormat(42);
assertTrue("decimalFormat integer", len(df2) > 0);
suiteEnd();

// ============================================================
// dollarFormat (from Lucee DollarFormat.cfc)
// ============================================================
suiteBegin("Lucee7: dollarFormat()");
d1 = dollarFormat(1234.56);
assertTrue("dollarFormat contains dollar sign", find("$", d1) > 0);
assertTrue("dollarFormat has length", len(d1) > 0);

d2 = dollarFormat(0);
assertTrue("dollarFormat zero contains dollar", find("$", d2) > 0);

d3 = dollarFormat(99.99);
assertTrue("dollarFormat 99.99", find("$", d3) > 0);
suiteEnd();

// ============================================================
// createUUID / createGUID
// ============================================================
suiteBegin("Lucee7: createUUID / createGUID");
uuid1 = createUUID();
assertTrue("createUUID has length", len(uuid1) > 0);
// CFML UUID format is 8-4-4-16 (4 segments)
assert("createUUID has dashes", listLen(uuid1, "-"), 4);

uuid2 = createUUID();
assertTrue("createUUID unique", uuid1 != uuid2);

guid1 = createGUID();
assertTrue("createGUID has length", len(guid1) > 0);
suiteEnd();

// ============================================================
// duplicate deep copy (from Lucee Duplicate.cfc)
// ============================================================
suiteBegin("Lucee7: duplicate()");
// struct with nested array
a = {x:[1,2,3]};
b = duplicate(a);
arrayAppend(b.x, 4);
assert("duplicate struct - original unchanged", arrayLen(a.x), 3);
assert("duplicate struct - copy modified", arrayLen(b.x), 4);

// array copy
arr1 = [1, 2, 3];
arr2 = duplicate(arr1);
arrayAppend(arr2, 4);
assert("duplicate array - original unchanged", arrayLen(arr1), 3);
assert("duplicate array - copy modified", arrayLen(arr2), 4);

// simple values
s1 = "hello";
s2 = duplicate(s1);
assert("duplicate string", s2, "hello");

n1 = 42;
n2 = duplicate(n1);
assert("duplicate number", n2, 42);
suiteEnd();

// ============================================================
// de() function - delayed evaluation
// ============================================================
suiteBegin("Lucee7: de()");
assert("de wraps string in quotes", de("hello"), '"hello"');
assert("de empty string", de(""), '""');
assert("de with expression-like string", de("1+1"), '"1+1"');
suiteEnd();
</cfscript>
