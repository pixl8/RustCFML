<cfscript>
suiteBegin("XML Functions");

// --- isXML ---
assertTrue("isXML valid document", isXML("<root><item>test</item></root>"));
assertFalse("isXML invalid string", isXML("not xml"));

// --- xmlParse ---
doc = xmlParse("<root><item>test</item></root>");
assertNotNull("xmlParse returns object", doc);

// --- xmlRoot access ---
rootNode = doc.xmlRoot;
assertNotNull("xmlRoot not null", rootNode);
assert("xmlRoot name", rootNode.xmlName, "root");

// --- child element access ---
children = rootNode.xmlChildren;
assertTrue("xmlChildren has elements", arrayLen(children) > 0);
assert("first child name", children[1].xmlName, "item");
assert("first child text", children[1].xmlText, "test");

// --- xmlSearch ---
results = xmlSearch(doc, "//item");
assertTrue("xmlSearch finds elements", arrayLen(results) > 0);

suiteEnd();
</cfscript>
