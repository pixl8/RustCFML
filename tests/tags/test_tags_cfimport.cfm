<cfscript>
suiteBegin("cfimport Tag");
</cfscript>

<!--- Import a CFML taglib directory with prefix "my" --->
<cfimport taglib="mytaglib" prefix="my">

<!--- Test 1: Use imported prefix tag for output (self-closing) --->
<cfsavecontent variable="greetResult"><my:greet name="World"></cfsavecontent>
<cfscript>
assert("cfimport prefix tag output", trim(greetResult), "Hi, World!");
</cfscript>

<!--- Test 2: Imported tag can write back to caller scope --->
<my:shout msg="hello">
<cfscript>
assert("cfimport prefix tag caller writeback", shoutResult, "HELLO");

// Test 3: cfimport without taglib (e.g. Java import) throws a sensible error
javaImportErrored = false;
try {
</cfscript>
<cfimport prefix="java" name="java.util.HashMap">
<cfscript>
} catch (any e) {
    javaImportErrored = true;
}
assert("cfimport java throws not-implemented", javaImportErrored, true);

suiteEnd();
</cfscript>
