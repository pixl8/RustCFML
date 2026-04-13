<cfscript>
suiteBegin("Struct Function Property Method Dispatch");

// Test 1: function ref stored in struct, called via dot notation
function myLine(text) { return arguments.text; }
function myGreen(text) { return "green:" & arguments.text; }

svc = {};
svc.line = myLine;
svc.greenLine = myGreen;

assert("direct function ref call via struct.line()", svc.line("test"), "test");
assert("direct function ref call via struct.greenLine()", svc.greenLine("test"), "green:test");

// Test 2: nested struct function call (this.svc.method())
obj = {};
obj.svc = svc;
assert("nested struct function call .line()", obj.svc.line("nested"), "nested");
assert("nested struct function call .greenLine()", obj.svc.greenLine("nested"), "green:nested");

// Test 3: function ref stored in struct via CFC invoke
TestCFC = createObject("component", "oop/StructMethodCFC");
TestCFC.svc = svc;
result = invoke(TestCFC, "callLine", {text: "invoked"});
assert("CFC invoke struct.line()", result, "invoked");
result2 = invoke(TestCFC, "callGreenLine", {text: "invoked"});
assert("CFC invoke struct.greenLine()", result2, "green:invoked");

// Test 4: getComponentMetadata on instance (this)
md = invoke(TestCFC, "getMeta", {});
assertTrue("getComponentMetadata(this) has functions", structKeyExists(md, "functions"));
assertTrue("getComponentMetadata(this) functions is array", isArray(md.functions));
funcNames = [];
for (f in md.functions) {
    if (isStruct(f)) arrayAppend(funcNames, lCase(f.name));
}
assertTrue("metadata includes callLine", arrayContains(funcNames, "callline"));
assertTrue("metadata includes callGreenLine", arrayContains(funcNames, "callgreenline"));

suiteEnd();
</cfscript>
