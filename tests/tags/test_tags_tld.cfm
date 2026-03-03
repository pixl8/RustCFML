<cfscript>
suiteBegin("TLD Tag Library Descriptor");
</cfscript>

<!--- Import a taglib directory that contains a .tld file --->
<cfimport taglib="tldlib" prefix="tl">

<!--- Test 1: Use tag defined in TLD (self-closing) --->
<cfsavecontent variable="helloResult"><tl:hello name="TLD"></cfsavecontent>
<cfscript>
assert("TLD tag output", trim(helloResult), "Hello, TLD!");
</cfscript>

<!--- Test 2: TLD tag with caller writeback --->
<tl:upper text="lowercase">
<cfscript>
assert("TLD tag caller writeback", result, "LOWERCASE");

suiteEnd();
</cfscript>
