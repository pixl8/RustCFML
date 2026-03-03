<cfscript>
suiteBegin("Encoding/Decoding Functions");

// ========================================
// binaryDecode - UTF-8/US-ASCII support
// ========================================
binData = binaryDecode("hello", "utf-8");
assertTrue("binaryDecode utf-8 returns binary", isBinary(binData));
assert("binaryDecode utf-8 round-trip", toString(binData), "hello");

binAscii = binaryDecode("test", "us-ascii");
assertTrue("binaryDecode us-ascii returns binary", isBinary(binAscii));
assert("binaryDecode us-ascii round-trip", toString(binAscii), "test");

// binaryDecode hex (existing)
binHex = binaryDecode("48656C6C6F", "hex");
assertTrue("binaryDecode hex returns binary", isBinary(binHex));
assert("binaryDecode hex round-trip", toString(binHex), "Hello");

// binaryDecode base64 (existing)
binB64 = binaryDecode("SGVsbG8=", "base64");
assertTrue("binaryDecode base64 returns binary", isBinary(binB64));
assert("binaryDecode base64 round-trip", toString(binB64), "Hello");

// ========================================
// binaryEncode
// ========================================
binInput = binaryDecode("Hello", "utf-8");
hexEncoded = binaryEncode(binInput, "hex");
assert("binaryEncode hex", hexEncoded, "48656C6C6F");

b64Encoded = binaryEncode(binInput, "base64");
assert("binaryEncode base64", b64Encoded, "SGVsbG8=");

// ========================================
// charsetDecode / charsetEncode
// ========================================
csBin = charsetDecode("Hello World", "utf-8");
assertTrue("charsetDecode returns binary", isBinary(csBin));

csStr = charsetEncode(csBin, "utf-8");
assert("charsetEncode round-trip", csStr, "Hello World");

csBinIso = charsetDecode("test", "iso-8859-1");
assertTrue("charsetDecode iso-8859-1 returns binary", isBinary(csBinIso));
csStrIso = charsetEncode(csBinIso, "iso-8859-1");
assert("charsetEncode iso round-trip", csStrIso, "test");

// ========================================
// encodeForHTMLAttribute
// ========================================
htmlAttr = encodeForHTMLAttribute('<div class="test">');
assertTrue("encodeForHTMLAttribute encodes lt", find("&lt;", htmlAttr) > 0);
assertTrue("encodeForHTMLAttribute encodes gt", find("&gt;", htmlAttr) > 0);
assertTrue("encodeForHTMLAttribute encodes quote", find("&quot;", htmlAttr) > 0);

htmlAttrQuote = encodeForHTMLAttribute("it's");
assertTrue("encodeForHTMLAttribute encodes apos", find('&#x27;', htmlAttrQuote) > 0);

htmlAttrSlash = encodeForHTMLAttribute("a/b");
assertTrue("encodeForHTMLAttribute encodes slash", find('&#x2f;', htmlAttrSlash) > 0);

// ========================================
// encodeForXML
// ========================================
xmlEnc = encodeForXML('<tag attr="val">');
assertTrue("encodeForXML encodes lt", find("&lt;", xmlEnc) > 0);
assertTrue("encodeForXML encodes gt", find("&gt;", xmlEnc) > 0);
assertTrue("encodeForXML encodes quot", find("&quot;", xmlEnc) > 0);

xmlApos = encodeForXML("it's");
assertTrue("encodeForXML encodes apos", find("&apos;", xmlApos) > 0);

// ========================================
// encodeForXMLAttribute
// ========================================
xmlAttr = encodeForXMLAttribute('<tag>' & chr(9) & chr(10));
assertTrue("encodeForXMLAttribute encodes lt", find("&lt;", xmlAttr) > 0);
assertTrue("encodeForXMLAttribute encodes tab", find('&#x9;', xmlAttr) > 0);
assertTrue("encodeForXMLAttribute encodes newline", find("&##xA;", xmlAttr) > 0);

// ========================================
// encodeFor (generic dispatcher)
// ========================================
efHtml = encodeFor("html", "<b>bold</b>");
assertTrue("encodeFor html encodes lt", find("&lt;", efHtml) > 0);

efUrl = encodeFor("url", "hello world");
assertTrue("encodeFor url encodes space", find("%20", efUrl) > 0);

efXml = encodeFor("xml", "<tag>");
assertTrue("encodeFor xml encodes lt", find("&lt;", efXml) > 0);

efJs = encodeFor("javascript", "alert()");
assertTrue("encodeFor javascript returns string", len(efJs) > 0);

efCss = encodeFor("css", "<div>");
assertTrue("encodeFor css returns string", len(efCss) > 0);

efHtmlAttr = encodeFor("htmlAttribute", "it's");
assertTrue("encodeFor htmlAttribute encodes apos", find('&#x27;', efHtmlAttr) > 0);

// ========================================
// decodeForHTML
// ========================================
decoded = decodeForHTML("&lt;b&gt;bold&lt;/b&gt;");
assert("decodeForHTML basic entities", decoded, "<b>bold</b>");

decoded2 = decodeForHTML("&amp;");
assert("decodeForHTML amp", decoded2, "&");

decoded3 = decodeForHTML("&quot;");
assert("decodeForHTML quot", decoded3, chr(34));

decoded4 = decodeForHTML("&##39;");
assert("decodeForHTML numeric apos", decoded4, chr(39));

decodedNum = decodeForHTML("&##65;&##66;&##67;");
assert("decodeForHTML numeric entities", decodedNum, "ABC");

decodedHex = decodeForHTML("&##x41;&##x42;&##x43;");
assert("decodeForHTML hex entities", decodedHex, "ABC");

// ========================================
// decodeFromURL
// ========================================
urlDecoded = decodeFromURL("hello%20world");
assert("decodeFromURL basic", urlDecoded, "hello world");

urlDecoded2 = decodeFromURL("hello+world");
assert("decodeFromURL plus sign", urlDecoded2, "hello world");

urlDecoded3 = decodeFromURL("%3C%3E%26");
assert("decodeFromURL special chars", urlDecoded3, "<>&");

// ========================================
// urlEncode
// ========================================
urlEnc = urlEncode("hello world");
assertTrue("urlEncode encodes space", find("%20", urlEnc) > 0);

urlEnc2 = urlEncode("a&b=c");
assertTrue("urlEncode encodes amp", find("%26", urlEnc2) > 0);

// urlEncode should match urlEncodedFormat behavior
assert("urlEncode matches urlEncodedFormat", urlEncode("test 123"), urlEncodedFormat("test 123"));

// ========================================
// canonicalize
// ========================================
canon1 = canonicalize("%3Cscript%3E", false, false);
assert("canonicalize URL-encoded", canon1, "<script>");

canon2 = canonicalize("&lt;script&gt;", false, false);
assert("canonicalize HTML-encoded", canon2, "<script>");

canon3 = canonicalize("hello", false, false);
assert("canonicalize plain text unchanged", canon3, "hello");

// Double-encoded
canon4 = canonicalize("%26lt%3B", false, false);
assert("canonicalize double-encoded", canon4, "<");

suiteEnd();
</cfscript>
