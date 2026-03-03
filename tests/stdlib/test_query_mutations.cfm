<cfscript>
suiteBegin("Query Mutation Functions");

// === Helper: build a simple query ===
function makeQuery() {
    var q = queryNew("id,name,age", "integer,varchar,integer");
    queryAddRow(q);
    querySetCell(q, "id", 1);
    querySetCell(q, "name", "Alice");
    querySetCell(q, "age", 30);
    queryAddRow(q);
    querySetCell(q, "id", 2, 2);
    querySetCell(q, "name", "Bob", 2);
    querySetCell(q, "age", 25, 2);
    queryAddRow(q);
    querySetCell(q, "id", 3, 3);
    querySetCell(q, "name", "Charlie", 3);
    querySetCell(q, "age", 35, 3);
    return q;
}

// =========================================
// queryAppend
// =========================================
q1 = makeQuery();
q2 = queryNew("id,name,age", "integer,varchar,integer");
queryAddRow(q2);
querySetCell(q2, "id", 4);
querySetCell(q2, "name", "Dave");
querySetCell(q2, "age", 40);
queryAddRow(q2);
querySetCell(q2, "id", 5, 2);
querySetCell(q2, "name", "Eve", 2);
querySetCell(q2, "age", 28, 2);

result = queryAppend(q1, q2);
assert("queryAppend total rows", result.recordCount, 5);
row4 = queryGetRow(result, 4);
assert("queryAppend row 4 name", row4.name, "Dave");
row5 = queryGetRow(result, 5);
assert("queryAppend row 5 name", row5.name, "Eve");

// queryAppend merges columns
q3 = queryNew("id,name", "integer,varchar");
queryAddRow(q3);
querySetCell(q3, "id", 1);
querySetCell(q3, "name", "Alice");
q4 = queryNew("id,email", "integer,varchar");
queryAddRow(q4);
querySetCell(q4, "id", 2);
querySetCell(q4, "email", "bob@test.com");
merged = queryAppend(q3, q4);
assert("queryAppend merged rows", merged.recordCount, 2);
assertTrue("queryAppend merges columns", listFindNoCase(merged.columnList, "email") > 0);

// =========================================
// queryInsertAt
// =========================================
q = makeQuery();
newRow = { "id": 99, "name": "Inserted", "age": 50 };
result = queryInsertAt(q, newRow, 2);
assert("queryInsertAt total rows", result.recordCount, 4);
row2 = queryGetRow(result, 2);
assert("queryInsertAt inserted name", row2.name, "Inserted");
row3 = queryGetRow(result, 3);
assert("queryInsertAt shifted row", row3.name, "Bob");

// Insert at position 1 (beginning)
q = makeQuery();
result = queryInsertAt(q, { "id": 0, "name": "First", "age": 1 }, 1);
assert("queryInsertAt at beginning", result.recordCount, 4);
first = queryGetRow(result, 1);
assert("queryInsertAt first row name", first.name, "First");

// =========================================
// queryPrepend
// =========================================
q1 = makeQuery();
q2 = queryNew("id,name,age", "integer,varchar,integer");
queryAddRow(q2);
querySetCell(q2, "id", 10);
querySetCell(q2, "name", "Prepended");
querySetCell(q2, "age", 99);

result = queryPrepend(q1, q2);
assert("queryPrepend total rows", result.recordCount, 4);
first = queryGetRow(result, 1);
assert("queryPrepend first row is from q2", first.name, "Prepended");
second = queryGetRow(result, 2);
assert("queryPrepend second row is original first", second.name, "Alice");

// =========================================
// queryReverse
// =========================================
q = makeQuery();
result = queryReverse(q);
assert("queryReverse same count", result.recordCount, 3);
first = queryGetRow(result, 1);
assert("queryReverse first was last", first.name, "Charlie");
last = queryGetRow(result, 3);
assert("queryReverse last was first", last.name, "Alice");

// =========================================
// queryRowSwap
// =========================================
q = makeQuery();
result = queryRowSwap(q, 1, 3);
first = queryGetRow(result, 1);
assert("queryRowSwap row 1 now Charlie", first.name, "Charlie");
third = queryGetRow(result, 3);
assert("queryRowSwap row 3 now Alice", third.name, "Alice");
middle = queryGetRow(result, 2);
assert("queryRowSwap row 2 unchanged", middle.name, "Bob");

// =========================================
// querySetRow
// =========================================
q = makeQuery();
newData = { "id": 999, "name": "Replaced", "age": 77 };
result = querySetRow(q, 2, newData);
assert("querySetRow same count", result.recordCount, 3);
row2 = queryGetRow(result, 2);
assert("querySetRow replaced name", row2.name, "Replaced");
assert("querySetRow replaced age", row2.age, 77);
row1 = queryGetRow(result, 1);
assert("querySetRow row 1 unchanged", row1.name, "Alice");

suiteEnd();
</cfscript>
