<cfscript>
// Test: createObject inside a function called from a for-in loop
function doLoad(name) {
    obj = createObject("component", "oop.EmptyCFC");
    return obj;
}

// Test 1: createObject directly in loop (top-level)
names = ["alpha", "beta", "gamma"];
results = [];
for (var name in names) {
    arrayAppend(results, name);
    obj = createObject("component", "oop.EmptyCFC");
}
writeOutput("direct: " & arrayLen(results) & "/3" & chr(10));

// Test 2: createObject via function call in loop
results2 = [];
for (var name in names) {
    arrayAppend(results2, name);
    doLoad(name);
}
writeOutput("via function: " & arrayLen(results2) & "/3" & chr(10));

// Test 3: createObject in a function that iterates
function loadAll() {
    var items = ["x", "y", "z"];
    var loaded = [];
    for (var item in items) {
        arrayAppend(loaded, item);
        var obj = createObject("component", "oop.EmptyCFC");
    }
    return loaded;
}
var result3 = loadAll();
writeOutput("function loop: " & arrayLen(result3) & "/3" & chr(10));

// Test 4: nested functions
function innerLoad(path) {
    return createObject("component", "oop.EmptyCFC");
}
function outerLoadAll() {
    var items = ["a", "b", "c"];
    var loaded = [];
    for (var item in items) {
        arrayAppend(loaded, item);
        innerLoad(item);
    }
    return loaded;
}
var result4 = outerLoadAll();
writeOutput("nested functions: " & arrayLen(result4) & "/3" & chr(10));
</cfscript>
