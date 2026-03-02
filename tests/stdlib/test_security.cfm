<cfscript>
suiteBegin("Security Functions");

// --- hash ---
hashResult = hash("hello");
assertTrue("hash returns non-empty", len(hashResult) > 0);

md5 = hash("hello", "MD5");
assert("hash MD5 is 32 chars", len(md5), 32);

sha256 = hash("hello", "SHA-256");
assert("hash SHA-256 is 64 chars", len(sha256), 64);

sha512 = hash("hello", "SHA-512");
assert("hash SHA-512 is 128 chars", len(sha512), 128);

// --- createUUID ---
uuid1 = createUUID();
assert("createUUID length is 35", len(uuid1), 35);

uuid2 = createUUID();
assertTrue("createUUID returns unique values", uuid1 != uuid2);

// --- hmac ---
hmacResult = hmac("hello", "key");
assertTrue("hmac returns non-empty", len(hmacResult) > 0);

// --- encrypt / decrypt round-trip ---
secretKey = generateSecretKey("AES");
encrypted = encrypt("hello", secretKey, "AES", "Base64");
assertTrue("encrypt returns non-empty", len(encrypted) > 0);
decrypted = decrypt(encrypted, secretKey, "AES", "Base64");
assert("decrypt round-trip", decrypted, "hello");

// --- encodeForHTML ---
encoded = encodeForHTML("<script>");
assertFalse("encodeForHTML escapes angle bracket", find("<", encoded) > 0);

// --- encodeForURL ---
urlEncoded = encodeForURL("hello world");
assertTrue("encodeForURL encodes space", find("hello", urlEncoded) > 0);
assertFalse("encodeForURL no literal space", find(" ", urlEncoded) > 0);

suiteEnd();
</cfscript>
