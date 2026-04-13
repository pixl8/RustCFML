<cfscript>
// Lucee 7 Compatibility Tests: Control Flow
// Synthesized from https://github.com/lucee/Lucee/tree/7.0/test
// Original tests Copyright (c) 2014, the Railo Company LLC / Copyright (c) 2015-2016, Lucee Association Switzerland
// Licensed under the GNU Lesser General Public License v2.1
// Adapted for RustCFML test harness

// ============================================================
// If/Else
// ============================================================
suiteBegin("Lucee7: If/Else");
result = "";
if (true) {
    result = "yes";
} else {
    result = "no";
}
assert("if true", result, "yes");

result = "";
if (false) {
    result = "yes";
} else {
    result = "no";
}
assert("if false goes to else", result, "no");

x = 5;
if (x > 3) {
    result = "big";
} else if (x > 1) {
    result = "medium";
} else {
    result = "small";
}
assert("if/else if/else", result, "big");

x = 2;
if (x > 3) {
    result = "big";
} else if (x > 1) {
    result = "medium";
} else {
    result = "small";
}
assert("else if branch", result, "medium");

x = 0;
if (x > 3) {
    result = "big";
} else if (x > 1) {
    result = "medium";
} else {
    result = "small";
}
assert("else branch", result, "small");
suiteEnd();

// ============================================================
// Switch/Case (comprehensive)
// ============================================================
suiteBegin("Lucee7: Switch/Case");
x = "b";
result = "";
switch(x) {
    case "a":
        result = "alpha";
        break;
    case "b":
        result = "beta";
        break;
    default:
        result = "other";
}
assert("switch basic match", result, "beta");

x = "b";
result = "";
switch(x) {
    case "a":
        result = "alpha";
        break;
    case "b":
        result = "bravo";
        break;
    case "c":
        result = "charlie";
        break;
    default:
        result = "other";
}
assert("switch multiple cases", result, "bravo");

x = "z";
result = "";
switch(x) {
    case "a":
        result = "alpha";
        break;
    default:
        result = "default";
}
assert("switch default", result, "default");

x = 2;
result = "";
switch(x) {
    case 1:
        result = "one";
        break;
    case 2:
        result = "two";
        break;
    case 3:
        result = "three";
        break;
}
assert("switch numeric", result, "two");

result = "";
switch("HELLO") {
    case "hello":
        result = "matched";
        break;
    default:
        result = "no match";
}
assert("switch case insensitive", result, "matched");
suiteEnd();

// ============================================================
// For Loop
// ============================================================
suiteBegin("Lucee7: For Loop");
sum = 0;
for (i = 1; i <= 5; i++) {
    sum += i;
}
assert("for loop sum", sum, 15);

arr = [10, 20, 30];
total = 0;
for (item in arr) {
    total += item;
}
assert("array for-in", total, 60);

keys = structKeyList({a: 1, b: 2, c: 3});
assert("struct has 3 keys", listLen(keys), 3);

collected = "";
for (i = 1; i <= 3; i++) {
    collected &= i;
}
assert("for loop string concat", collected, "123");

// Nested for
result = 0;
for (i = 1; i <= 3; i++) {
    for (j = 1; j <= 3; j++) {
        result++;
    }
}
assert("nested for loops", result, 9);
suiteEnd();

// ============================================================
// While / Do-While
// ============================================================
suiteBegin("Lucee7: While / Do-While");
i = 0;
while (i < 5) {
    i++;
}
assert("while loop", i, 5);

i = 0;
do {
    i++;
} while (i < 5);
assert("do-while loop", i, 5);

// do-while always runs at least once
i = 10;
do {
    i++;
} while (i < 5);
assert("do-while runs once even if false", i, 11);

// while with compound condition
i = 0;
j = 10;
while (i < 5 && j > 5) {
    i++;
    j--;
}
assert("while compound condition i", i, 5);
assert("while compound condition j", j, 5);
suiteEnd();

// ============================================================
// Break / Continue
// ============================================================
suiteBegin("Lucee7: Break / Continue");
sum = 0;
for (i = 1; i <= 10; i++) {
    if (i == 6) break;
    sum += i;
}
assert("break in for loop", sum, 15);

sum = 0;
for (i = 1; i <= 5; i++) {
    if (i == 3) continue;
    sum += i;
}
assert("continue in for loop", sum, 12);

// break in while
count = 0;
while (true) {
    count++;
    if (count == 7) break;
}
assert("break in while", count, 7);

// continue in while
sum = 0;
idx = 0;
while (idx < 10) {
    idx++;
    if (idx % 2 == 0) continue;
    sum += idx;
}
assert("continue in while (odd sum)", sum, 25);
suiteEnd();

// ============================================================
// Try/Catch/Finally (from Lucee tags/Try.cfc)
// ============================================================
suiteBegin("Lucee7: Try/Catch/Finally");
result = "";
try {
    throw(message="test error");
} catch (any e) {
    result = e.message;
}
assert("basic try/catch", result, "test error");

result = "";
try {
    result = "try";
} finally {
    result &= " finally";
}
assert("try/finally", result, "try finally");

result = "";
try {
    throw(message="err");
} catch (any e) {
    result = "caught";
} finally {
    result &= " finally";
}
assert("catch with finally", result, "caught finally");

result = "";
try {
    try {
        throw(message="inner");
    } catch (any e) {
        result = "inner caught";
        throw(message="rethrow");
    }
} catch (any e) {
    result &= " outer caught";
}
assert("nested try/catch rethrow", result, "inner caught outer caught");

// finally runs even when no exception
result = "";
try {
    result = "ok";
} catch (any e) {
    result = "caught";
} finally {
    result &= " finally";
}
assert("finally without exception", result, "ok finally");

// catch preserves exception details
caughtType = "";
caughtMsg = "";
try {
    throw(type="CustomType", message="custom error");
} catch (any e) {
    caughtType = e.type;
    caughtMsg = e.message;
}
assert("exception message", caughtMsg, "custom error");
assert("exception type", caughtType, "CustomType");
suiteEnd();

// ============================================================
// Throw / assertThrows
// ============================================================
suiteBegin("Lucee7: Throw");
assertThrows("basic throw", function(){
    throw(message="test");
});
assertThrows("throw with type", function(){
    throw(type="CustomType", message="custom error");
});
assertThrows("throw with detail", function(){
    throw(message="err", detail="some detail");
});
suiteEnd();
</cfscript>
