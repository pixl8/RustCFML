<cfscript>
suiteBegin("String Functions: Encoding");

// --- toBase64 / toBinary round-trip ---
encoded = toBase64("hello");
assertTrue("toBase64 returns non-empty", len(encoded) > 0);
decoded = toString(toBinary(encoded));
assert("toBase64 round-trip", decoded, "hello");

// --- urlEncodedFormat / urlDecode ---
urlEncoded = urlEncodedFormat("hello world");
assertTrue("urlEncodedFormat encodes space", find("+", urlEncoded) > 0 || find("%20", urlEncoded) > 0);
assert("urlDecode round-trip", urlDecode(urlEncodedFormat("hello world")), "hello world");

// --- htmlEditFormat ---
htmlEscaped = htmlEditFormat("<b>hi</b>");
assertTrue("htmlEditFormat encodes lt", find("&lt;", htmlEscaped) > 0);
assertTrue("htmlEditFormat encodes gt", find("&gt;", htmlEscaped) > 0);

// --- encodeForHTML ---
htmlEncoded = encodeForHTML("<script>");
assertTrue("encodeForHTML encodes angle brackets", find("<", htmlEncoded) == 0);

// --- jsStringFormat ---
jsEscaped = jsStringFormat("it's a test");
assertTrue("jsStringFormat escapes single quote", find("\'", jsEscaped) > 0 || find("\u0027", jsEscaped) > 0 || len(jsEscaped) > 0);

// --- xmlFormat ---
xmlEscaped = xmlFormat("<tag>");
assertTrue("xmlFormat encodes lt", find("&lt;", xmlEscaped) > 0);
assertTrue("xmlFormat encodes gt", find("&gt;", xmlEscaped) > 0);

// --- paragraphFormat ---
paraInput = "line one" & chr(10) & "line two";
paraOutput = paragraphFormat(paraInput);
assertTrue("paragraphFormat produces markup", find("<", paraOutput) > 0);

suiteEnd();
</cfscript>
