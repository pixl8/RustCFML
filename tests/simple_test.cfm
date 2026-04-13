<cfscript>
// Copy Taffy files locally and test
writeOutput("Testing with local component files<br><br>");

// Just test with a simple local component in same directory
component localBase {
    variables.data = "";
    
    function setData(val) {
        variables.data = val;
        return this;
    }
    
    function getData() {
        return variables.data;
    }
    
    function withStatus(code) {
        return this;
    }
    
    function withMime(mime) {
        return this;
    }
}

writeOutput("1. Creating local component... ");
obj = createObject("component", "localBase");
writeOutput("isObject: " & isObject(obj) & "<br>");

writeOutput("2. setData: ");
obj.setData({test: "value"});
writeOutput("OK<br>");

writeOutput("3. getData: ");
result = obj.getData();
writeOutput(serializeJson(result) & "<br>");

writeOutput("4. withStatus: ");
obj.withStatus(201);
writeOutput("OK<br>");

writeOutput("5. withMime: ");
obj.withMime("application/json");
writeOutput("OK<br>");

// Now test native serializeJson
writeOutput("<br>6. Testing serializeJson... ");
jsonOut = serializeJson({name: "test", count: 42});
writeOutput(jsonOut & "<br>");

// Test deserializeJson  
writeOutput("<br>7. Testing deserializeJson... ");
parsed = deserializeJson('{"key":"value","num":123}');
writeOutput("key=" & parsed.key & ", num=" & parsed.num & "<br>");

writeOutput("<br>All basic tests passed!");
</cfscript>