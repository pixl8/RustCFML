<cfscript>
suiteBegin("Query Higher-Order Functions");

// --- Build test query ---
q = queryNew("id,name,age", "integer,varchar,integer");
queryAddRow(q);
querySetCell(q, "id", 1);
querySetCell(q, "name", "Charlie");
querySetCell(q, "age", 30);

queryAddRow(q);
querySetCell(q, "id", 2, 2);
querySetCell(q, "name", "Alice", 2);
querySetCell(q, "age", 22, 2);

queryAddRow(q);
querySetCell(q, "id", 3, 3);
querySetCell(q, "name", "Bob", 3);
querySetCell(q, "age", 35, 3);

queryAddRow(q);
querySetCell(q, "id", 4, 4);
querySetCell(q, "name", "Diana", 4);
querySetCell(q, "age", 28, 4);

// --- queryEach ---
count = 0;
queryEach(q, function(row) {
    count++;
});
assert("queryEach iterates all rows", count, 4);

// --- queryMap ---
mapped = queryMap(q, function(row) {
    row.name = uCase(row.name);
    return row;
});
assert("queryMap transforms data", queryGetRow(mapped, 1).name, "CHARLIE");
assert("queryMap preserves row count", mapped.recordCount, 4);

// --- queryFilter ---
older = queryFilter(q, function(row) {
    return row.age > 25;
});
assert("queryFilter keeps matching rows", older.recordCount, 3);

young = queryFilter(q, function(row) {
    return row.age < 25;
});
assert("queryFilter young", young.recordCount, 1);
assert("queryFilter young is Alice", queryGetRow(young, 1).name, "Alice");

// --- queryReduce ---
totalAge = queryReduce(q, function(acc, row) {
    return acc + row.age;
}, 0);
assert("queryReduce sum ages", totalAge, 115);

// --- querySort ---
sorted = querySort(q, function(row1, row2) {
    return compare(row1.name, row2.name);
});
assert("querySort first by name", queryGetRow(sorted, 1).name, "Alice");
assert("querySort last by name", queryGetRow(sorted, 4).name, "Diana");

// --- querySome ---
hasOlder = querySome(q, function(row) {
    return row.age > 30;
});
assertTrue("querySome finds age > 30", hasOlder);

hasAncient = querySome(q, function(row) {
    return row.age > 100;
});
assertFalse("querySome no age > 100", hasAncient);

// --- queryEvery ---
allAdults = queryEvery(q, function(row) {
    return row.age >= 18;
});
assertTrue("queryEvery all adults", allAdults);

allOlder = queryEvery(q, function(row) {
    return row.age > 25;
});
assertFalse("queryEvery not all > 25", allOlder);

suiteEnd();
</cfscript>
