// Lucee 7 Compatibility Tests: Encoding/Hashing/Encryption Functions
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness
<cfscript>
suiteBegin("Lucee7: Encoding/Hashing/Encryption Functions");

// ============================================================
// Hash (from Lucee hash.cfc)
// ============================================================
assertTrue("hash returns string", len(hash("test")) > 0);
assert("md5 hash consistent", hash("test", "MD5"), hash("test", "MD5"));
assertTrue("sha256 hash", len(hash("test", "SHA-256")) > 0);
assertTrue("sha512 hash", len(hash("test", "SHA-512")) > 0);
assert("md5 known value empty string", uCase(hash("", "MD5")), "D41D8CD98F00B204E9800998ECF8427E");

// ============================================================
// HMAC (from Lucee HMAC.cfc)
// ============================================================
assertTrue("hmac returns string", len(hmac("test", "key")) > 0);
assert("hmac consistent", hmac("test", "key"), hmac("test", "key"));
assertTrue("hmac with algo", len(hmac("test", "key", "HMACSHA256")) > 0);

// ============================================================
// Encrypt / Decrypt (from Lucee Encrypt.cfc, Decrypt.cfc)
// ============================================================
key1 = generateSecretKey("AES");
encrypted1 = encrypt("hello", key1, "AES");
decrypted1 = decrypt(encrypted1, key1, "AES");
assert("encrypt/decrypt roundtrip", decrypted1, "hello");

key2 = generateSecretKey("AES", 256);
encrypted2 = encrypt("secret data", key2, "AES");
assert("decrypt 256", decrypt(encrypted2, key2, "AES"), "secret data");

// ============================================================
// ToBase64 / ToBinary (from Lucee toBase64.cfc, ToBinary.cfc)
// ============================================================
encoded = toBase64("hello");
assertTrue("toBase64 not empty", len(encoded) > 0);
assert("roundtrip base64", toString(toBinary(toBase64("hello"))), "hello");

// ============================================================
// BinaryEncode / BinaryDecode (from Lucee BinaryEncode.cfc, BinaryDecode.cfc)
// ============================================================
bin = toBinary(toBase64("test"));
hex = binaryEncode(bin, "hex");
assertTrue("hex encode", len(hex) > 0);
decoded = binaryDecode(hex, "hex");
assertTrue("binary decode", isBinary(decoded));

// ============================================================
// URLEncodedFormat / URLDecode (from Lucee URLEncode.cfc, URLDecode.cfc)
// ============================================================
assert("url roundtrip", urlDecode(urlEncodedFormat("hello world")), "hello world");
assertTrue("url encode has plus or %20", find("+", urlEncodedFormat("hello world")) > 0 || find("%20", urlEncodedFormat("hello world")) > 0);

// ============================================================
// CharsetEncode / CharsetDecode (from Lucee CharsetEncode.cfc, CharsetDecode.cfc)
// ============================================================
binUtf8 = charsetDecode("hello", "utf-8");
assertTrue("charsetDecode returns binary", isBinary(binUtf8));
assert("charset roundtrip", charsetEncode(charsetDecode("hello", "utf-8"), "utf-8"), "hello");

suiteEnd();
</cfscript>
