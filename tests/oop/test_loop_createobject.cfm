suiteBegin("For-in loop with createObject inside");

// Test: createObject inside a for-in loop should not break iteration
names = ["alpha", "beta", "gamma"];
results = [];
for (var name in names) {
    arrayAppend(results, name);
    obj = createObject("component", "oop.EmptyCFC");
}
assert("all 3 iterations run", arrayLen(results), 3);
assert("first item", results[1], "alpha");
assert("second item", results[2], "beta");
assert("third item", results[3], "gamma");

suiteEnd();
