<cfscript>
suiteBegin("Password Hashing Functions");

// =========================================================
// generatePBKDFKey (standard CFML function)
// =========================================================

// PBKDF2 with SHA256 - deterministic output for known inputs
pbkdfKey = generatePBKDFKey("PBKDF2WithHmacSHA256", "password", "salt", 1000, 256);
assert("PBKDF2 SHA256 returns 44 base64 chars (256 bits)", len(pbkdfKey), 44);
assertTrue("PBKDF2 SHA256 is valid base64", reFind("^[A-Za-z0-9+/=]+$", pbkdfKey) > 0);

// Same inputs should produce same output (deterministic)
pbkdfKey2 = generatePBKDFKey("PBKDF2WithHmacSHA256", "password", "salt", 1000, 256);
assert("PBKDF2 is deterministic", pbkdfKey, pbkdfKey2);

// Different key sizes
pbkdfKey128 = generatePBKDFKey("PBKDF2WithHmacSHA256", "password", "salt", 1000, 128);
assert("PBKDF2 128-bit key is 24 base64 chars", len(pbkdfKey128), 24);

// SHA1 variant
pbkdfKeySHA1 = generatePBKDFKey("PBKDF2WithHmacSHA1", "password", "salt", 1000, 128);
assert("PBKDF2 SHA1 returns 24 base64 chars", len(pbkdfKeySHA1), 24);

// SHA512 variant
pbkdfKeySHA512 = generatePBKDFKey("PBKDF2WithHmacSHA512", "password", "salt", 1000, 256);
assert("PBKDF2 SHA512 returns 44 base64 chars", len(pbkdfKeySHA512), 44);

// Different algorithms produce different results
assertTrue("PBKDF2 SHA1 != SHA256 for same input", pbkdfKeySHA1 != pbkdfKey128);

suiteEnd();
</cfscript>
