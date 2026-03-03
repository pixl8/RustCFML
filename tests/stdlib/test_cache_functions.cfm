suiteBegin("Cache Functions");

// --- cachePut / cacheGet ---
cachePut("myKey", "myValue");
assert("cacheGet basic", cacheGet("myKey"), "myValue");

// --- cacheGet missing key returns null ---
result = cacheGet("nonExistent");
assertNull("cacheGet missing key", result);

// --- cachePut overwrites ---
cachePut("myKey", "newValue");
assert("cacheGet overwrite", cacheGet("myKey"), "newValue");

// --- cachePut with struct value ---
cachePut("structKey", { name: "test", count: 42 });
s = cacheGet("structKey");
assert("cacheGet struct name", s.name, "test");
assert("cacheGet struct count", s.count, 42);

// --- cacheKeyExists ---
assertTrue("cacheKeyExists true", cacheKeyExists("myKey"));
assertFalse("cacheKeyExists false", cacheKeyExists("doesNotExist"));

// --- cacheCount ---
cacheClear();
cachePut("a", 1);
cachePut("b", 2);
cachePut("c", 3);
assert("cacheCount", cacheCount(), 3);

// --- cacheGetAllIds ---
ids = cacheGetAllIds();
assertTrue("cacheGetAllIds is array", isArray(ids));
assert("cacheGetAllIds count", arrayLen(ids), 3);

// --- cacheGetAll ---
all = cacheGetAll();
assertTrue("cacheGetAll is struct", isStruct(all));
assert("cacheGetAll count", structCount(all), 3);

// --- cacheDelete ---
cacheDelete("a");
assertFalse("cacheDelete removes key", cacheKeyExists("a"));
assert("cacheCount after delete", cacheCount(), 2);

// --- cacheDelete with throwOnError ---
assertThrows("cacheDelete throw on missing", function() {
    cacheDelete("nonExistent", true);
});

// --- cacheClear removes all ---
cacheClear();
assert("cacheClear empties cache", cacheCount(), 0);

// --- cachePut with expiry ---
cachePut("expiring", "value", createTimespan(0, 0, 0, 1));
assertTrue("cacheKeyExists before expiry", cacheKeyExists("expiring"));

// --- cacheClear with filter ---
cacheClear();
cachePut("user_1", "Alice");
cachePut("user_2", "Bob");
cachePut("order_1", "Item");
cacheClear("user_*");
assertFalse("cacheClear filter removed user_1", cacheKeyExists("user_1"));
assertFalse("cacheClear filter removed user_2", cacheKeyExists("user_2"));
assertTrue("cacheClear filter kept order_1", cacheKeyExists("order_1"));

cacheClear();

suiteEnd();
