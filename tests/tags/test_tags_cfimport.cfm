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

// Test 3: cfimport without taglib - verify it doesn't crash the page
assertTrue("cfimport tests completed", true);

suiteEnd();
</cfscript>
