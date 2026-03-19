suiteBegin("Hash in Strings");

// ## produces a single # in double-quoted strings
assert("double hash produces single hash", "##", chr(35));
assert("hello ##world", "hello ##world", "hello " & chr(35) & "world");
assert("##foo##", "##foo##", chr(35) & "foo" & chr(35));

// String interpolation still works
var x = "test";
assert("interpolation still works", "#x#", "test");
assert("mixed interp and hash", "#x###", "test" & chr(35));

// Lone # before closing quote
assert("hash before closing quote", "#", chr(35));

// #### produces ##
assert("quadruple hash", "####", chr(35) & chr(35));

suiteEnd();
