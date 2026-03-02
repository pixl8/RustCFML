<cfscript>
suiteBegin("Query Functions");

// --- queryNew with column list ---
q1 = queryNew("id,name");
assert("queryNew creates 0 rows", q1.recordCount, 0);

// --- queryNew with types ---
q2 = queryNew("id,name,age", "integer,varchar,integer");
assert("queryNew with types has 0 rows", q2.recordCount, 0);

// --- queryAddRow ---
queryAddRow(q2);

// --- querySetCell ---
querySetCell(q2, "id", 1);
querySetCell(q2, "name", "Alice");
querySetCell(q2, "age", 30);

// --- recordCount ---
assert("recordCount after 1 add", q2.recordCount, 1);

// --- add more rows ---
queryAddRow(q2);
querySetCell(q2, "id", 2, 2);
querySetCell(q2, "name", "Bob", 2);
querySetCell(q2, "age", 25, 2);

queryAddRow(q2);
querySetCell(q2, "id", 3, 3);
querySetCell(q2, "name", "Charlie", 3);
querySetCell(q2, "age", 35, 3);

assert("recordCount after 3 adds", q2.recordCount, 3);

// --- columnList ---
cols = q2.columnList;
assertTrue("columnList contains ID", listFindNoCase(cols, "id") > 0);
assertTrue("columnList contains NAME", listFindNoCase(cols, "name") > 0);
assertTrue("columnList contains AGE", listFindNoCase(cols, "age") > 0);

// --- queryGetRow ---
row1 = queryGetRow(q2, 1);
assert("queryGetRow name", row1.name, "Alice");
assert("queryGetRow age", row1.age, 30);

// --- queryColumnExists ---
assertTrue("queryColumnExists name", queryColumnExists(q2, "name"));
assertFalse("queryColumnExists nope", queryColumnExists(q2, "nope"));

// --- queryDeleteRow ---
q3 = queryNew("id,name", "integer,varchar");
queryAddRow(q3);
querySetCell(q3, "id", 1);
querySetCell(q3, "name", "X");
queryAddRow(q3);
querySetCell(q3, "id", 2, 2);
querySetCell(q3, "name", "Y", 2);
queryDeleteRow(q3, 1);
assert("queryDeleteRow reduces count", q3.recordCount, 1);

// --- queryAddColumn ---
q4 = queryNew("id", "integer");
queryAddRow(q4);
querySetCell(q4, "id", 1);
queryAddColumn(q4, "email", "varchar", ["test@example.com"]);
assertTrue("queryAddColumn adds column", queryColumnExists(q4, "email"));

// --- querySlice ---
sliced = querySlice(q2, 1, 2);
assert("querySlice returns 2 rows", sliced.recordCount, 2);

// --- queryColumnData ---
ages = queryColumnData(q2, "age");
assert("queryColumnData returns array", arrayLen(ages), 3);
assert("queryColumnData first value", ages[1], 30);

suiteEnd();
</cfscript>
