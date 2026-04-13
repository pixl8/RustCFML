<cfscript>
suiteBegin("htmlParse");

// --- Parse simple HTML ---
doc = htmlParse("<html><head><title>Test</title></head><body><p>Hello</p></body></html>");
assertTrue("doc is struct", isStruct(doc));
assert("doc xmlType", doc.xmlType, "DOCUMENT");
assertTrue("doc has xmlRoot", structKeyExists(doc, "xmlRoot"));

// --- Root element ---
root = doc.xmlRoot;
assert("root xmlName", root.xmlName, "html");
assert("root xmlType", root.xmlType, "ELEMENT");
assertTrue("root has children", arrayLen(root.xmlChildren) > 0);

// --- Find body/head children ---
headFound = false;
bodyFound = false;
for (child in root.xmlChildren) {
    if (structKeyExists(child, "xmlName")) {
        if (child.xmlName == "head") headFound = true;
        if (child.xmlName == "body") bodyFound = true;
    }
}
assertTrue("found head element", headFound);
assertTrue("found body element", bodyFound);

// --- Attributes ---
doc2 = htmlParse('<html><body><div id="main" class="container">Content</div></body></html>');
root2 = doc2.xmlRoot;
// Navigate to body > div
body = "";
for (child in root2.xmlChildren) {
    if (structKeyExists(child, "xmlName") && child.xmlName == "body") {
        body = child;
        break;
    }
}
assertTrue("body is struct", isStruct(body));
div = body.xmlChildren[1];
assert("div xmlName", div.xmlName, "div");
assertTrue("div has attributes", structKeyExists(div, "xmlAttributes"));
assert("div id attr", div.xmlAttributes.id, "main");
assert("div class attr", div.xmlAttributes.class, "container");
assert("div text content", div.xmlText, "Content");

suiteEnd();
</cfscript>
