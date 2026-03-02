<cfscript>
suiteBegin("Type: Binary");

// --- toBase64 / toBinary round-trip ---
original = "hello";
encoded = toBase64(original);
assert("toBase64 of 'hello'", encoded, "aGVsbG8=");

decoded = toString(toBinary(encoded));
assert("toBinary round-trip", decoded, "hello");

// --- isBinary checks ---
binData = toBinary(toBase64("test data"));
assertTrue("isBinary on binary data", isBinary(binData));
assertFalse("isBinary on string", isBinary("not binary"));
assertFalse("isBinary on number", isBinary(42));

// --- Binary length ---
binHello = toBinary(toBase64("hello"));
assert("binary len matches string len", len(binHello), 5);

// --- binaryEncode / binaryDecode ---
binVal = toBinary(toBase64("ABC"));
hexStr = binaryEncode(binVal, "hex");
assert("binaryEncode hex of ABC", uCase(hexStr), "414243");
roundTrip = binaryDecode(hexStr, "hex");
assertTrue("binaryDecode returns binary", isBinary(roundTrip));
assert("binaryDecode round-trip", toString(roundTrip), "ABC");

// --- Empty binary ---
emptyBin = toBinary(toBase64(""));
assertTrue("empty binary is binary", isBinary(emptyBin));
assert("empty binary length", len(emptyBin), 0);

suiteEnd();
</cfscript>
