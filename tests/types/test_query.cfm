<cfscript>
suiteBegin("Type: Query");

// --- queryNew with column list ---
q1 = queryNew("id,name,email");
assert("queryNew column count", q1.recordCount, 0);

// --- queryNew with column list and types ---
q2 = queryNew("id,name,active", "integer,varchar,bit");
assert("queryNew typed recordCount", q2.recordCount, 0);

// --- queryAddRow and querySetCell ---
q = queryNew("id,name,email");
queryAddRow(q);
querySetCell(q, "id", 1);
querySetCell(q, "name", "Alice");
querySetCell(q, "email", "alice@example.com");
assert("after addRow recordCount", q.recordCount, 1);

// --- Add second row ---
queryAddRow(q);
querySetCell(q, "id", 2, 2);
querySetCell(q, "name", "Bob", 2);
querySetCell(q, "email", "bob@example.com", 2);
assert("two rows recordCount", q.recordCount, 2);

// --- Column access returns value from first row by default ---
assert("column access name", q.name[1], "Alice");
assert("column access second row", q.name[2], "Bob");

// --- recordCount property ---
assert("recordCount is 2", q.recordCount, 2);

// --- columnList property ---
cols = q.columnList;
assertTrue("columnList contains ID", findNoCase("id", cols) > 0);
assertTrue("columnList contains NAME", findNoCase("name", cols) > 0);
assertTrue("columnList contains EMAIL", findNoCase("email", cols) > 0);

// --- queryAddRow with data struct ---
q3 = queryNew("id,name");
queryAddRow(q3, {id: 1, name: "Charlie"});
assert("addRow with struct", q3.name[1], "Charlie");
assert("addRow with struct id", q3.id[1], 1);

// --- queryGetRow returns struct ---
row = queryGetRow(q, 1);
assert("queryGetRow name", row.name, "Alice");
assert("queryGetRow id", row.id, 1);

suiteEnd();
</cfscript>
