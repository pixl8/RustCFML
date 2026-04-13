<cfscript>
// Standalone test of Taffy concepts - no application._taffy needed

writeOutput("Testing Taffy concepts on RustCFML<br><br>");

// Test 1: Native JSON Serializer
writeOutput("1. NativeJsonSerializer... ");
s = createObject("component", "taffy.core.nativeJsonSerializer");
s.setData({key: "value", number: 42, array: [1,2,3]});
data = s.getData();
writeOutput("data: " & serializeJson(data) & "<br>");

// Test 2: Native JSON Deserializer
writeOutput("2. NativeJsonDeserializer... ");
d = createObject("component", "taffy.core.nativeJsonDeserializer");
parsed = d.getFromJson('{"name":"test","active":true}');
writeOutput("parsed: " & serializeJson(parsed) & "<br>");

// Test 3: Base Serializer
writeOutput("3. BaseSerializer status/mime... ");
bs = createObject("component", "taffy.core.baseSerializer");
bs.setData({test: "value"});
bs.withStatus(201);
bs.withMime("text/plain");
writeOutput("status=" & bs.getStatusCode() & ", mime=" & bs.getFileMime() & "<br>");

// Test 4: Base Deserializer
writeOutput("4. BaseDeserializer getFromForm... ");
bd = createObject("component", "taffy.core.baseDeserializer");
form = bd.getFromForm("name=John&age=30&city=NYC");
writeOutput("form: " & serializeJson(form) & "<br>");

// Test 5: Test basic component inheritance without Taffy's getRepInstance
writeOutput("5. Basic component extends... ");
component extends="taffy.core.baseSerializer" {
    function getSomething() {
        return "works";
    }
}
testComp = createObject("component", "testExtends");
result = testComp.getSomething();
writeOutput("result: " & result & "<br>");

writeOutput("<br>All Taffy concept tests completed!");
</cfscript>