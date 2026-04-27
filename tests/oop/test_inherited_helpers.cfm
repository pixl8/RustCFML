<cfscript>
suiteBegin("Inherited helpers visible during child cfc body (Bug G)");

c = new oop.InheritedHelpersChild();

// Child's pseudo-constructor referenced parent's variables.encode.string;
// before the fix, that threw silently, leaving __variables empty.
assert("dummyData survives", structKeyExists(c.__variables, "dummyData"), true);
assert("dummyData.whatever set", c.__variables.dummyData.whatever, true);
assert("dummyData.encoded carries function result", c.__variables.dummyData.encoded, "ENC:payload");

// Method that reads through variables.dummyData also returns the populated struct.
got = c.get();
assert("get() returns whatever", got.whatever, true);
assert("get() returns encoded", got.encoded, "ENC:payload");

// Multiple instances should be independent and reproducible.
c2 = new oop.InheritedHelpersChild();
assert("second instance encoded", c2.__variables.dummyData.encoded, "ENC:payload");

suiteEnd();
</cfscript>
