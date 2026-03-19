// Lucee 7 Compatibility Tests: String Functions
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test/functions
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness
<cfscript>
suiteBegin("Lucee7: String Functions");

// ============================================================
// Find / FindNoCase (from Lucee Find.cfc)
// ============================================================
assert("find x not in string", find("x", "Susi Sorglos"), 0);
assert("find s in Susi Sorglos", find("s", "Susi Sorglos"), 3);
assert("find s from pos 3", find("s", "Susi Sorglos", 3), 3);
assert("find s from pos 4", find("s", "Susi Sorglos", 4), 12);
assert("findNoCase S in susi", findNoCase("S", "susi"), 1);

// ============================================================
// FindOneOf
// ============================================================
assert("findOneOf abc in xbyz", findOneOf("abc", "xbyz"), 2);

// ============================================================
// Len / UCase / LCase / Trim / LTrim / RTrim
// ============================================================
assert("len hello", len("hello"), 5);
assert("len empty", len(""), 0);
assert("ucase hello", ucase("hello"), "HELLO");
assert("lcase HELLO", lcase("HELLO"), "hello");
assert("trim spaces", trim("  hi  "), "hi");
assert("ltrim spaces", ltrim("  hi  "), "hi  ");
assert("rtrim spaces", rtrim("  hi  "), "  hi");

// ============================================================
// Mid / Left / Right
// ============================================================
assert("mid Hello World 1 5", mid("Hello World", 1, 5), "Hello");
assert("left Hello 3", left("Hello", 3), "Hel");
assert("right Hello 3", right("Hello", 3), "llo");

// ============================================================
// Replace / ReplaceNoCase (from Lucee Replace.cfc)
// ============================================================
assert("replace World with CFML", replace("Hello World", "World", "CFML"), "Hello CFML");
assert("replace all a with b", replace("aaa", "a", "b", "all"), "bbb");
assert("replaceNoCase WORLD", replaceNoCase("Hello WORLD", "world", "CFML"), "Hello CFML");

// ============================================================
// ReplaceList / ReplaceListNoCase (from Lucee ReplaceList.cfc)
// ============================================================
assert("replaceList a,b,c to x,y,z", replaceList("a,b,c", "a,b,c", "x,y,z"), "x,y,z");

// ============================================================
// Reverse
// ============================================================
assert("reverse Hello", reverse("Hello"), "olleH");

// ============================================================
// RepeatString
// ============================================================
assert("repeatString ab 3", repeatString("ab", 3), "ababab");

// ============================================================
// RemoveChars (from Lucee removeChars.cfc)
// ============================================================
assert("removeChars space", removeChars("hello world", 6, 1), "helloworld");

// ============================================================
// Insert
// ============================================================
assert("insert dash at 3", insert("-", "hello", 3), "hel-lo");

// ============================================================
// SpanIncluding / SpanExcluding (from Lucee)
// ============================================================
assert("spanIncluding letters", spanIncluding("hello123", "abcdefghijklmnopqrstuvwxyz"), "hello");
assert("spanExcluding digits", spanExcluding("hello123", "0123456789"), "hello");

// ============================================================
// Compare / CompareNoCase (from Lucee Compare.cfc)
// ============================================================
assertTrue("compare a < b", compare("a", "b") < 0);
assertTrue("compare b > a", compare("b", "a") > 0);
assert("compare a == a", compare("a", "a"), 0);
assert("compareNoCase A == a", compareNoCase("A", "a"), 0);

// ============================================================
// Asc / Chr (from Lucee Asc.cfc, Chr.cfc)
// ============================================================
assert("asc A", asc("A"), 65);
assert("chr 65", chr(65), "A");
assert("asc a", asc("a"), 97);
assert("chr 97", chr(97), "a");

// ============================================================
// LJustify / RJustify / CJustify
// ============================================================
assert("lJustify length", len(lJustify("hi", 10)), 10);
assert("rJustify length", len(rJustify("hi", 10)), 10);
assert("lJustify starts with hi", left(lJustify("hi", 10), 2), "hi");
assert("rJustify ends with hi", right(rJustify("hi", 10), 2), "hi");

// ============================================================
// REFind / REFindNoCase (from Lucee refind.cfc)
// ============================================================
assert("reFind digits", reFind("[0-9]+", "abc123def"), 4);
assert("reFindNoCase letters in uppercase", reFindNoCase("[a-z]+", "ABC123"), 1);

// ============================================================
// REReplace / REReplaceNoCase
// ============================================================
assert("reReplace digits one", reReplace("abc123def", "[0-9]+", "NUM"), "abcNUMdef");
assert("reReplace digits all", reReplace("abc123def456", "[0-9]+", "NUM", "all"), "abcNUMdefNUM");

// ============================================================
// REMatch / REMatchNoCase (from Lucee rematch.cfc)
// ============================================================
result = reMatch("[0-9]+", "abc123def456");
assert("reMatch count", arrayLen(result), 2);
assert("reMatch first", result[1], "123");
assert("reMatch second", result[2], "456");

resultNC = reMatchNoCase("[a-z]+", "ABC123DEF");
assert("reMatchNoCase count", arrayLen(resultNC), 2);
assert("reMatchNoCase first", resultNC[1], "ABC");
assert("reMatchNoCase second", resultNC[2], "DEF");

// ============================================================
// REEscape (RustCFML-specific — Lucee does not have reEscape)
// ============================================================
try {
    escaped = reEscape("a.b");
    assertTrue("reEscape escapes dot", find("\.", escaped) > 0);
} catch (any e) {
    // Expected on Lucee — reEscape is not a standard CFML function
}

// ============================================================
// Wrap / StripCr
// ============================================================
testStr = "hello" & chr(13) & chr(10) & "world";
stripped = stripCr(testStr);
assertFalse("stripCr removes CR", find(chr(13), stripped) > 0);
assertTrue("stripCr keeps LF", find(chr(10), stripped) > 0);

// ============================================================
// JSStringFormat
// ============================================================
assertTrue("jsStringFormat escapes quote", find("\'", jsStringFormat("it's")) > 0);

// ============================================================
// URLEncodedFormat / URLDecode
// ============================================================
assert("urlEncode/Decode roundtrip", urlDecode(urlEncodedFormat("hello world")), "hello world");

// ============================================================
// HTMLEditFormat / EncodeForHTML
// ============================================================
assertTrue("htmlEditFormat lt", find("&lt;", htmlEditFormat("<b>")) > 0);
assertTrue("encodeForHTML lt", find("&lt;", encodeForHTML("<b>")) > 0);

// ============================================================
// UCFirst
// ============================================================
assert("ucFirst hello world", ucFirst("hello world"), "Hello world");

// ============================================================
// GetToken
// ============================================================
assert("getToken second", getToken("one,two,three", 2, ","), "two");

// ============================================================
// ToBase64 / ToBinary roundtrip
// ============================================================
assert("toBase64/toBinary roundtrip", toString(toBinary(toBase64("hello"))), "hello");

suiteEnd();
</cfscript>
