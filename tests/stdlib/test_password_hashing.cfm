<cfscript>
suiteBegin("Password Hashing Functions");

// =========================================================
// generatePBKDFKey
// =========================================================

// PBKDF2 with SHA256 - deterministic output for known inputs
pbkdfKey = generatePBKDFKey("PBKDF2WithHmacSHA256", "password", "salt", 1000, 256);
assert("PBKDF2 SHA256 returns 64 hex chars (256 bits)", len(pbkdfKey), 64);
assertTrue("PBKDF2 SHA256 is valid hex", reFind("^[0-9A-Fa-f]+$", pbkdfKey) > 0);

// Same inputs should produce same output (deterministic)
pbkdfKey2 = generatePBKDFKey("PBKDF2WithHmacSHA256", "password", "salt", 1000, 256);
assert("PBKDF2 is deterministic", pbkdfKey, pbkdfKey2);

// Different key sizes
pbkdfKey128 = generatePBKDFKey("PBKDF2WithHmacSHA256", "password", "salt", 1000, 128);
assert("PBKDF2 128-bit key is 32 hex chars", len(pbkdfKey128), 32);

// SHA1 variant
pbkdfKeySHA1 = generatePBKDFKey("PBKDF2WithHmacSHA1", "password", "salt", 1000, 128);
assert("PBKDF2 SHA1 returns 32 hex chars", len(pbkdfKeySHA1), 32);

// SHA512 variant
pbkdfKeySHA512 = generatePBKDFKey("PBKDF2WithHmacSHA512", "password", "salt", 1000, 256);
assert("PBKDF2 SHA512 returns 64 hex chars", len(pbkdfKeySHA512), 64);

// Different algorithms produce different results
assertTrue("PBKDF2 SHA1 != SHA256 for same input", pbkdfKeySHA1 != pbkdfKey128);

// =========================================================
// generateBCryptHash / verifyBCryptHash
// =========================================================

bcryptHash = generateBCryptHash("testPassword123");
assertTrue("bcrypt hash is non-empty", len(bcryptHash) > 0);
assertTrue("bcrypt hash starts with $2", left(bcryptHash, 2) == "$2");

// Verify correct password
bcryptValid = verifyBCryptHash("testPassword123", bcryptHash);
assertTrue("bcrypt verify correct password", bcryptValid);

// Verify wrong password
bcryptInvalid = verifyBCryptHash("wrongPassword", bcryptHash);
assertFalse("bcrypt verify wrong password", bcryptInvalid);

// Custom rounds
bcryptHash4 = generateBCryptHash("mypass", 4);
assertTrue("bcrypt with 4 rounds produces hash", len(bcryptHash4) > 0);
assertTrue("bcrypt 4 rounds verifies", verifyBCryptHash("mypass", bcryptHash4));

// Two hashes of same password should differ (different salts)
bcryptHash2 = generateBCryptHash("testPassword123");
assertTrue("bcrypt hashes differ (different salts)", bcryptHash != bcryptHash2);

// =========================================================
// generateSCryptHash / verifySCryptHash
// =========================================================

scryptHash = generateSCryptHash("testPassword123");
assertTrue("scrypt hash is non-empty", len(scryptHash) > 0);
assertTrue("scrypt hash contains $scrypt$", find("scrypt", scryptHash) > 0);

// Verify correct password
scryptValid = verifySCryptHash("testPassword123", scryptHash);
assertTrue("scrypt verify correct password", scryptValid);

// Verify wrong password
scryptInvalid = verifySCryptHash("wrongPassword", scryptHash);
assertFalse("scrypt verify wrong password", scryptInvalid);

// =========================================================
// generateArgon2Hash / argon2CheckHash
// =========================================================

argon2Hash = generateArgon2Hash("testPassword123");
assertTrue("argon2 hash is non-empty", len(argon2Hash) > 0);
assertTrue("argon2 hash contains $argon2", find("argon2", argon2Hash) > 0);

// Verify correct password - note argon2CheckHash has reversed params (hash, password)
argon2Valid = argon2CheckHash(argon2Hash, "testPassword123");
assertTrue("argon2 verify correct password", argon2Valid);

// Verify wrong password
argon2Invalid = argon2CheckHash(argon2Hash, "wrongPassword");
assertFalse("argon2 verify wrong password", argon2Invalid);

// =========================================================
// csrfGenerateToken / csrfVerifyToken
// =========================================================

csrfToken = csrfGenerateToken();
assert("CSRF token is 64 hex chars", len(csrfToken), 64);
assertTrue("CSRF token is valid hex", reFind("^[0-9A-Fa-f]+$", csrfToken) > 0);

// Verify valid token
csrfValid = csrfVerifyToken(csrfToken);
assertTrue("csrfVerifyToken accepts valid token", csrfValid);

// Verify invalid token (wrong length)
csrfInvalidShort = csrfVerifyToken("abcdef");
assertFalse("csrfVerifyToken rejects short token", csrfInvalidShort);

// Verify invalid token (non-hex)
csrfInvalidHex = csrfVerifyToken("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
assertFalse("csrfVerifyToken rejects non-hex", csrfInvalidHex);

// Two tokens should be unique
csrfToken2 = csrfGenerateToken();
assertTrue("CSRF tokens are unique", csrfToken != csrfToken2);

// Token with key param (accepted but key ignored)
csrfTokenKeyed = csrfGenerateToken("mykey");
assert("CSRF token with key is still 64 chars", len(csrfTokenKeyed), 64);

suiteEnd();
</cfscript>
