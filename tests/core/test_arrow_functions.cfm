<cfscript>
// Test arrow functions with multiple parameters
include "../harness.cfm";

suiteBegin("Arrow Functions");

// Test 1: Single parameter arrow function
double = (x) => x * 2;
assertTrue("single param arrow", double(5) eq 10);

// Test 2: Two parameter arrow function
add = (a, b) => a + b;
assertTrue("two param arrow", add(3, 4) eq 7);

// Test 3: Three parameter arrow function
sum3 = (a, b, c) => a + b + c;
assertTrue("three param arrow", sum3(1, 2, 3) eq 6);

// Test 4: Arrow function with method chaining
arr = [1, 2, 3, 4, 5];
result = arr.map((x) => x * 2).filter((x) => x gt 5);
assertTrue("chained arrow map/filter", arrayToList(result) eq "6,8,10");

// Test 5: Arrow function with reduce
total = arr.reduce((acc, x) => acc + x, 0);
assertTrue("arrow reduce", total eq 15);

// Test 6: Arrow function in array operations
squared = arr.map((n) => n * n);
assertTrue("arrow map square", arrayToList(squared) eq "1,4,9,16,25");

suiteEnd();
</cfscript>
