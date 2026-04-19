<cfscript>
suiteBegin("Control Flow");

// --- if/else/elseif ---
val = 10;
if (val > 20) {
    ifResult = "high";
} else if (val > 5) {
    ifResult = "mid";
} else {
    ifResult = "low";
}
assert("if/elseif/else", ifResult, "mid");

if (true) {
    simpleBranch = "yes";
} else {
    simpleBranch = "no";
}
assert("simple if/else true", simpleBranch, "yes");

if (false) {
    simpleBranch2 = "yes";
} else {
    simpleBranch2 = "no";
}
assert("simple if/else false", simpleBranch2, "no");

// --- for loop counting up ---
sum = 0;
for (i = 1; i <= 5; i++) {
    sum += i;
}
assert("for loop count up", sum, 15);

// --- for loop counting down ---
downResult = "";
for (j = 3; j >= 1; j--) {
    downResult &= j;
}
assert("for loop count down", downResult, "321");

// --- for-in loop over array ---
arr = ["a", "b", "c"];
joined = "";
for (item in arr) {
    joined &= item;
}
assert("for-in array", joined, "abc");

// --- for-in loop over struct ---
st = { x: 1, y: 2 };
keys = [];
for (key in st) {
    arrayAppend(keys, key);
}
arraySort(keys, "text");
assert("for-in struct key count", arrayLen(keys), 2);
assertTrue("for-in struct has x", arrayFind(keys, "X") > 0 || arrayFind(keys, "x") > 0);

// --- while loop ---
w = 0;
wSum = 0;
while (w < 5) {
    w++;
    wSum += w;
}
assert("while loop", wSum, 15);

// --- do-while loop ---
dw = 0;
do {
    dw++;
} while (dw < 3);
assert("do-while loop", dw, 3);

// do-while runs at least once
dwOnce = 0;
do {
    dwOnce++;
} while (false);
assert("do-while runs at least once", dwOnce, 1);

// --- switch/case with default ---
color = "green";
switch (color) {
    case "red":
        switchResult = "stop";
        break;
    case "green":
        switchResult = "go";
        break;
    case "yellow":
        switchResult = "caution";
        break;
    default:
        switchResult = "unknown";
        break;
}
assert("switch/case match", switchResult, "go");

unknown = "purple";
switch (unknown) {
    case "red":
        switchDefault = "stop";
        break;
    default:
        switchDefault = "unknown";
        break;
}
assert("switch/case default", switchDefault, "unknown");

// --- break in loops ---
breakSum = 0;
for (b = 1; b <= 10; b++) {
    if (b > 3) break;
    breakSum += b;
}
assert("break in for loop", breakSum, 6);

// --- continue in loops ---
contSum = 0;
for (c = 1; c <= 5; c++) {
    if (c == 3) continue;
    contSum += c;
}
assert("continue in for loop", contSum, 12);

// --- Nested loops with break ---
outerCount = 0;
for (o = 1; o <= 3; o++) {
    outerCount++;
    for (inner = 1; inner <= 10; inner++) {
        if (inner > 2) break;
    }
}
assert("nested loops - outer completes", outerCount, 3);

// --- while with break ---
whileBreak = 0;
while (true) {
    whileBreak++;
    if (whileBreak == 5) break;
}
assert("while with break", whileBreak, 5);

// --- while with continue ---
whileCont = 0;
whileContSum = 0;
while (whileCont < 5) {
    whileCont++;
    if (whileCont == 3) continue;
    whileContSum += whileCont;
}
assert("while with continue", whileContSum, 12);

// --- counted-while fusion: stride at end, various shapes ---
// Exercises the ForLoopStep-based fusion path in compile_while.
cwa = 1; cwaSum = 0;
while (cwa <= 10) {
    cwaSum += cwa;
    cwa++;
}
assert("while counted (i++)", cwaSum, 55);

cwb = 1; cwbSum = 0;
while (cwb <= 10) {
    cwbSum += cwb;
    cwb = cwb + 1;
}
assert("while counted (i = i + 1)", cwbSum, 55);

cwc = 0; cwcSum = 0;
while (cwc < 20) {
    cwcSum += cwc;
    cwc += 3;
}
assert("while counted (i += 3, non-unit step)", cwcSum, 63);

// Down-counting
cwd = 5; cwdSum = 0;
while (cwd > 0) {
    cwdSum += cwd;
    cwd--;
}
assert("while counted (i--)", cwdSum, 15);

// Empty body (stride only) — fusion should still apply
cwe = 0;
while (cwe < 100) { cwe++; }
assert("while counted empty body", cwe, 100);

// Entry-check: condition false on entry → zero iterations
cwf = 10; cwfCount = 0;
while (cwf < 5) {
    cwfCount++;
    cwf++;
}
assert("while counted zero iters", cwfCount, 0);

// Break inside body: hoisted stride must not change break behavior
cwg = 0; cwgSum = 0;
while (cwg < 100) {
    if (cwg >= 5) break;
    cwgSum += cwg;
    cwg++;
}
assert("while counted + break", cwgSum, 10);  // 0+1+2+3+4

// Continue BEFORE stride: fusion must bail out (otherwise continue would
// skip stride → infinite loop / wrong sum). We just check it terminates
// with the right sum — any regression here would hang or fail.
cwh = 0; cwhSum = 0;
while (cwh < 6) {
    cwh++;
    if (cwh == 3) continue;
    cwhSum += cwh;
}
assert("while + continue (no fusion)", cwhSum, 18);  // 1+2+4+5+6

// --- counted-do-while fusion ---
dwa = 1; dwaSum = 0;
do {
    dwaSum += dwa;
    dwa++;
} while (dwa <= 10);
assert("do-while counted", dwaSum, 55);

// do-while runs body at least once even if condition false
dwb = 100; dwbCount = 0;
do {
    dwbCount++;
    dwb++;
} while (dwb < 50);
assert("do-while counted body runs once", dwbCount, 1);

// do-while + continue: fusion must bail out
dwc = 0; dwcSum = 0;
do {
    dwc++;
    if (dwc == 3) continue;
    dwcSum += dwc;
} while (dwc < 6);
assert("do-while + continue (no fusion)", dwcSum, 18);  // 1+2+4+5+6

// --- for loop empty body (edge case) ---
emptyLoopCounter = 0;
for (e = 0; e < 0; e++) {
    emptyLoopCounter++;
}
assert("for loop zero iterations", emptyLoopCounter, 0);

// --- Nested if inside loop ---
nestedResult = 0;
for (n = 1; n <= 10; n++) {
    if (n MOD 2 == 0) {
        nestedResult += n;
    }
}
assert("nested if in loop (sum evens)", nestedResult, 30);

// --- if with numeric truthy ---
if (1) {
    numTruthy = "yes";
} else {
    numTruthy = "no";
}
assert("if numeric 1 is truthy", numTruthy, "yes");

if (0) {
    numFalsy = "yes";
} else {
    numFalsy = "no";
}
assert("if numeric 0 is falsy", numFalsy, "no");

// --- switch with string matching ---
fruit = "APPLE";
switch (fruit) {
    case "APPLE":
        fruitResult = "found apple";
        break;
    default:
        fruitResult = "not found";
        break;
}
assert("switch case match", fruitResult, "found apple");

// --- for loop with step > 1 ---
stepResult = "";
for (s = 0; s < 10; s = s + 3) {
    stepResult = stepResult & s;
}
assert("for loop step by 3", stepResult, "0369");

// --- Deeply nested conditions ---
depth = 0;
if (true) {
    if (true) {
        if (true) {
            depth = 3;
        }
    }
}
assert("deeply nested if", depth, 3);

suiteEnd();
</cfscript>
