<cfscript>
suiteBegin("Utility Functions");

// --- duplicate struct ---
original = {a: 1, b: [2, 3]};
copy = duplicate(original);
assert("duplicate struct key a", copy.a, 1);
assertTrue("duplicate struct key b is array", isArray(copy.b));
assert("duplicate struct array length", arrayLen(copy.b), 2);

// modify original, verify copy is independent
original.a = 99;
arrayAppend(original.b, 4);
assert("duplicate struct is independent", copy.a, 1);
assert("duplicate array is independent", arrayLen(copy.b), 2);

// --- duplicate array ---
origArr = [1, 2, 3];
copyArr = duplicate(origArr);
assert("duplicate array length", arrayLen(copyArr), 3);
arrayAppend(origArr, 4);
assert("duplicate array independent", arrayLen(copyArr), 3);

// --- sleep ---
sleep(1);
assertTrue("sleep completes without error", true);

// --- getTickCount ---
tick1 = getTickCount();
assertTrue("getTickCount > 0", tick1 > 0);
tick2 = getTickCount();
assertTrue("getTickCount monotonic", tick2 >= tick1);

// --- setVariable / getVariable ---
setVariable("variables.dynamicVar", 42);
assert("setVariable sets value", variables.dynamicVar, 42);

// --- newLine ---
nl = chr(10);
assertTrue("chr(10) is not empty", len(nl) > 0);

suiteEnd();
</cfscript>
