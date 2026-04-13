<cfscript>suiteBegin("Tags: cfstoredproc");</cfscript>

<cfscript>
// Requires RUSTCFML_TEST_MYSQL_DSN env var, e.g. "mysql://root:pass@127.0.0.1:3306/rustcfml"
ds = "";
try { ds = createObject("java", "java.lang.System").getenv("RUSTCFML_TEST_MYSQL_DSN"); } catch (any e) {}
if (isNull(ds)) ds = "";
</cfscript>

<cfif len(ds) GT 0>

<!--- Test 1: Call stored procedure with no params --->
<cfstoredproc procedure="sp_get_users" datasource="#ds#">
    <cfprocresult name="qUsers">
</cfstoredproc>
<cfscript>
assertTrue("sp_get_users returns query", isQuery(qUsers));
assertTrue("sp_get_users has rows", qUsers.recordCount >= 3);
assert("sp_get_users first name", qUsers.name[1], "Alice");
assert("sp_get_users second name", qUsers.name[2], "Bob");
</cfscript>

<!--- Test 2: Call stored procedure with IN parameter --->
<cfstoredproc procedure="sp_get_user_by_id" datasource="#ds#">
    <cfprocparam value="2" cfsqltype="cf_sql_integer">
    <cfprocresult name="qSingle">
</cfstoredproc>
<cfscript>
assertTrue("sp_get_user_by_id returns query", isQuery(qSingle));
assert("sp_get_user_by_id record count", qSingle.recordCount, 1);
assert("sp_get_user_by_id name", qSingle.name[1], "Bob");
assert("sp_get_user_by_id email", qSingle.email[1], "bob@example.com");
</cfscript>

<!--- Test 3: Call stored procedure that inserts and returns --->
<cfstoredproc procedure="sp_add_user" datasource="#ds#">
    <cfprocparam value="Diana" cfsqltype="cf_sql_varchar">
    <cfprocparam value="diana@example.com" cfsqltype="cf_sql_varchar">
    <cfprocresult name="qInsert">
</cfstoredproc>
<cfscript>
assertTrue("sp_add_user returns result", isQuery(qInsert));
assertTrue("sp_add_user new_id > 0", qInsert.new_id[1] > 0);

// Clean up inserted test row
queryExecute("DELETE FROM sp_test_users WHERE name = ?", ["Diana"], { datasource: ds });
</cfscript>

<cfelse>
<cfscript>
// Skip: no MySQL DSN provided
assertTrue("cfstoredproc skipped (no RUSTCFML_TEST_MYSQL_DSN)", true);
</cfscript>
</cfif>

<cfscript>suiteEnd();</cfscript>
