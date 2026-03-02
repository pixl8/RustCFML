<cfscript>
suiteBegin("Variables");

// --- var declaration and assignment ---
myNum = 42;
assert("var declaration numeric", myNum, 42);

myStr = "hello";
assert("var declaration string", myStr, "hello");

myBool = true;
assert("var declaration boolean", myBool, true);

// --- Unscoped assignment (variables scope) ---
unscoped = "I am in variables scope";
assert("unscoped assignment", unscoped, "I am in variables scope");
assert("unscoped lives in variables scope", variables.unscoped, "I am in variables scope");

// --- Compound operator: += ---
counter = 10;
counter += 5;
assert("compound +=", counter, 15);

// --- Compound operator: -= ---
counter -= 3;
assert("compound -=", counter, 12);

// --- Compound operator: *= ---
counter *= 2;
assert("compound *=", counter, 24);

// --- Compound operator: /= ---
counter /= 4;
assert("compound /=", counter, 6);

// --- Compound operator: %= ---
counter %= 4;
assert("compound %=", counter, 2);

// --- Compound operator: &= ---
greeting = "Hello";
greeting &= " World";
assert("compound &=", greeting, "Hello World");

// --- Multiple assignment in sequence ---
a = 1;
b = 2;
c = 3;
assert("sequential assignment a", a, 1);
assert("sequential assignment b", b, 2);
assert("sequential assignment c", c, 3);

// --- Variable reassignment / type change ---
mutable = 100;
assert("before type change", mutable, 100);
mutable = "now a string";
assert("after type change to string", mutable, "now a string");
mutable = [1, 2, 3];
assert("after type change to array", arrayLen(mutable), 3);

suiteEnd();
</cfscript>
