<cfscript>
suiteBegin("Encoding/Decoding Functions");

// ========================================
// charsetDecode for text-to-binary (UTF-8/US-ASCII)
// ========================================
binData = charsetDecode("hello", "utf-8");
assertTrue("charsetDecode utf-8 returns binary", isBinary(binData));
assert("charsetDecode utf-8 round-trip", charsetEncode(binData, "utf-8"), "hello");

binAscii = charsetDecode("test", "us-ascii");
assertTrue("charsetDecode us-ascii returns binary", isBinary(binAscii));
assert("charsetDecode us-ascii round-trip", charsetEncode(binAscii, "us-ascii"), "test");

// binaryDecode hex (standard encoding)
binHex = binaryDecode("48656C6C6F", "hex");
assertTrue("binaryDecode hex returns binary", isBinary(binHex));
assert("binaryDecode hex round-trip", charsetEncode(binHex, "utf-8"), "Hello");

// binaryDecode base64 (standard encoding)
binB64 = binaryDecode("SGVsbG8=", "base64");
assertTrue("binaryDecode base64 returns binary", isBinary(binB64));
assert("binaryDecode base64 round-trip", charsetEncode(binB64, "utf-8"), "Hello");

// ========================================
// binaryEncode
// ========================================
binInput = charsetDecode("Hello", "utf-8");
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

// ========================================
// encodeForXML
// ========================================
xmlEnc = encodeForXML('<tag attr="val">');
assertTrue("encodeForXML encodes lt", find("&lt;", xmlEnc) > 0);
assertTrue("encodeForXML encodes gt", find("&gt;", xmlEnc) > 0);

// ========================================
// encodeForXMLAttribute
// ========================================
xmlAttr = encodeForXMLAttribute('<tag>' & chr(9) & chr(10));
assertTrue("encodeForXMLAttribute encodes lt", find("&lt;", xmlAttr) > 0);

// ========================================
// encodeForHTML / encodeForURL / encodeForJavaScript / encodeForCSS
// ========================================
efHtml = encodeForHTML("<b>bold</b>");
assertTrue("encodeForHTML encodes lt", find("&lt;", efHtml) > 0);

efUrl = encodeForURL("hello world");
assertTrue("encodeForURL encodes space", find("%20", efUrl) > 0 || find("+", efUrl) > 0);

efJs = encodeForJavaScript("alert()");
assertTrue("encodeForJavaScript returns string", len(efJs) > 0);

efCss = encodeForCSS("<div>");
assertTrue("encodeForCSS returns string", len(efCss) > 0);

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
assertTrue("urlEncode encodes space", find("+", urlEnc) > 0);

urlEnc2 = urlEncode("a&b=c");
assertTrue("urlEncode encodes amp", find("%26", urlEnc2) > 0);

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
