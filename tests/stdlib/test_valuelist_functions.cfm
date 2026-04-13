<cfscript>
suiteBegin("valueList / quotedValueList");

// Create a test query
q = queryNew("name,age", "varchar,integer");
queryAddRow(q);
querySetCell(q, "name", "Alice", 1);
querySetCell(q, "age", 30, 1);
queryAddRow(q);
querySetCell(q, "name", "Bob", 2);
querySetCell(q, "age", 25, 2);
queryAddRow(q);
querySetCell(q, "name", "Charlie", 3);
querySetCell(q, "age", 35, 3);

// valueList basic (query.column resolves to an array via dot notation)
assert("valueList basic", valueList(q.name), "Alice,Bob,Charlie");

// valueList with custom delimiter
assert("valueList custom delimiter", valueList(q.name, "|"), "Alice|Bob|Charlie");

// valueList with numeric column
assert("valueList numeric", valueList(q.age), "30,25,35");

// quotedValueList basic
assert("quotedValueList basic", quotedValueList(q.name), "'Alice','Bob','Charlie'");

// quotedValueList with custom delimiter
assert("quotedValueList custom delimiter", quotedValueList(q.name, "|"), "'Alice'|'Bob'|'Charlie'");

// Empty query
emptyQ = queryNew("name", "varchar");
assert("valueList empty query", valueList(emptyQ.name), "");

suiteEnd();
</cfscript>
