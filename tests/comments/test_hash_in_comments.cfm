suiteBegin("Hash in Comments");

// # in single-line comments should not break anything
// github.com/user/repo#tag
var x = 1; // value is #1
// ##escaped
assert("code after hash in comment works", x, 1);

/* # in multi-line comments */
/* user/repo#tag */
var y = 2;
assert("code after hash in block comment works", y, 2);

suiteEnd();
