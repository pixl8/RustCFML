<cfscript>
suiteBegin("XML DOM Functions");

// xmlNew()
doc = xmlNew();
assert("xmlNew returns struct", isStruct(doc), true);
// xmlNew() creates empty doc — xmlRoot may not exist until set
assertTrue("xmlNew is XML doc", isXMLDoc(doc));
assert("xmlNew has xmlChildren", structKeyExists(doc, "xmlChildren"), true);

// xmlElemNew()
elem = xmlElemNew(doc, "book");
assert("xmlElemNew returns struct", isStruct(elem), true);
assert("xmlElemNew xmlName", elem.xmlName, "book");
assert("xmlElemNew has xmlChildren", isArray(elem.xmlChildren), true);
assert("xmlElemNew has xmlAttributes", isStruct(elem.xmlAttributes), true);
assert("xmlElemNew has xmlText", structKeyExists(elem, "xmlText"), true);

// xmlElemNew with namespace
nsElem = xmlElemNew(doc, "http://example.com", "ns:item");
assert("xmlElemNew ns xmlName", nsElem.xmlName, "ns:item");
assert("xmlElemNew ns xmlNsURI", nsElem.xmlNsURI, "http://example.com");

// isXMLDoc
assert("isXMLDoc with doc", isXMLDoc(doc), true);
assert("isXMLDoc with elem", isXMLDoc(elem), false);
assert("isXMLDoc with string", isXMLDoc("hello"), false);

// isXMLElem
assert("isXMLElem with elem", isXMLElem(elem), true);
assert("isXMLElem with doc", isXMLElem(doc), false);
assert("isXMLElem with string", isXMLElem("hello"), false);

// isXMLNode
assert("isXMLNode with doc", isXMLNode(doc), true);
assert("isXMLNode with elem", isXMLNode(elem), true);
assert("isXMLNode with string", isXMLNode("hello"), false);

// isXMLRoot — empty doc has no root, elements are not root
assert("isXMLRoot with empty doc", isXMLRoot(doc), false);
assert("isXMLRoot with elem", isXMLRoot(elem), false);

// isXMLAttribute
assert("isXMLAttribute with elem", isXMLAttribute(elem), false);
assert("isXMLAttribute with string", isXMLAttribute("hello"), false);

// Check children via xmlChildren array length (standard approach)
assert("empty elem has no children", arrayLen(elem.xmlChildren), 0);
elem2 = xmlElemNew(doc, "chapter");
arrayAppend(elem.xmlChildren, elem2);
assert("elem has child after append", arrayLen(elem.xmlChildren), 1);

// xmlGetNodeType
assert("xmlGetNodeType doc", xmlGetNodeType(doc), "DOCUMENT_NODE");
assert("xmlGetNodeType elem", xmlGetNodeType(elem), "ELEMENT_NODE");

// xmlChildPos
parent = xmlElemNew(doc, "library");
child1 = xmlElemNew(doc, "book");
child2 = xmlElemNew(doc, "dvd");
child3 = xmlElemNew(doc, "book");
arrayAppend(parent.xmlChildren, child1);
arrayAppend(parent.xmlChildren, child2);
arrayAppend(parent.xmlChildren, child3);
assert("xmlChildPos first book", xmlChildPos(parent, "book", 1), 1);
assert("xmlChildPos second book", xmlChildPos(parent, "book", 2), 3);
assert("xmlChildPos dvd", xmlChildPos(parent, "dvd", 1), 2);
assert("xmlChildPos not found", xmlChildPos(parent, "cd", 1), -1);

suiteEnd();
</cfscript>
