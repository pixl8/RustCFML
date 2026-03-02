<cfscript>
suiteBegin("String Functions");

// --- len ---
assert("len of hello", len("hello"), 5);

// --- ucase / lcase ---
assert("ucase", ucase("hello"), "HELLO");
assert("lcase", lcase("HELLO"), "hello");

// --- trim / ltrim / rtrim ---
assert("trim", trim("  hi  "), "hi");
assert("ltrim", ltrim("  hi"), "hi");
assert("rtrim", rtrim("hi  "), "hi");

// --- find / findNoCase ---
assert("find ll in hello", find("ll", "hello"), 3);
assert("findNoCase LL in hello", findNoCase("LL", "hello"), 3);

// --- mid / left / right ---
assert("mid(hello,2,3)", mid("hello", 2, 3), "ell");
assert("left(hello,3)", left("hello", 3), "hel");
assert("right(hello,3)", right("hello", 3), "llo");

// --- replace / replaceNoCase ---
assert("replace ll with r", replace("hello", "ll", "r"), "hero");
assert("replaceNoCase hello->world", replaceNoCase("Hello", "hello", "world"), "world");

// --- reverse ---
assert("reverse hello", reverse("hello"), "olleh");

// --- repeatString ---
assert("repeatString ab 3", repeatString("ab", 3), "ababab");

// --- insert ---
assert("insert X into hello at 3", insert("X", "hello", 3), "helXlo");

// --- removeChars ---
assert("removeChars hello 2 3", removeChars("hello", 2, 3), "ho");

// --- chr / asc ---
assert("chr(65)", chr(65), "A");
assert("asc(A)", asc("A"), 65);

// --- compare / compareNoCase ---
assert("compare equal", compare("abc", "abc"), 0);
assert("compareNoCase equal", compareNoCase("abc", "ABC"), 0);

// --- ucFirst ---
assert("ucFirst hello world", ucFirst("hello world"), "Hello world");

// --- wrap ---
wrapped = wrap("hello world test long text", 5);
assertTrue("wrap contains line break", find(chr(10), wrapped) > 0);

// --- spanIncluding ---
assert("spanIncluding", spanIncluding("hello123", "helo"), "hello");

// --- getToken ---
assert("getToken second", getToken("one,two,three", 2, ","), "two");

suiteEnd();
</cfscript>
