<cfscript>
suiteBegin("Misc Functions");

// soundex
assert("soundex Robert", soundex("Robert"), "R163");
assert("soundex Rupert", soundex("Rupert"), "R163");
assert("soundex Ashcraft", soundex("Ashcraft"), "A261");
assert("soundex Tymczak", soundex("Tymczak"), "T522");
assert("soundex Smith", soundex("Smith"), "S530");
assert("soundex empty", soundex(""), "");

// metaphone
assert("metaphone Smith", metaphone("Smith"), "SM0");
assert("metaphone Schmidt", metaphone("Schmidt"), "SKMT");
assert("metaphone phone", metaphone("phone"), "FN");
assert("metaphone Thompson", metaphone("Thompson"), "0MPS");

// toScript
assert("toScript string", toScript("hello", "x"), 'x="hello";');
assert("toScript number", toScript(42, "n"), 'n=42;');
assert("toScript boolean true", toScript(true, "b"), 'b="true";');
assert("toScript boolean false", toScript(false, "b"), 'b="false";');

// toScript array
arr = [1, 2, 3];
result = toScript(arr, "a");
assert("toScript array contains new Array", find("new Array", result) > 0, true);

// toScript struct
s = { name: "test" };
result = toScript(s, "obj");
assert("toScript struct contains new Object", find("new Object", result) > 0, true);

// htmlParse
doc = htmlParse("<html><body><p>Hello</p></body></html>");
assert("htmlParse returns struct", isStruct(doc), true);

suiteEnd();
</cfscript>
