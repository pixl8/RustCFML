<cfscript>
suiteBegin("Null Coalescing (??) Operator");

// ?? with undefined variable
var result1 = undefinedVar ?? "fallback";
assert("?? with undefined var", result1, "fallback");

// ?? with defined variable
var definedVar = "hello";
var result2 = definedVar ?? "fallback";
assert("?? with defined var", result2, "hello");

// ?? with null value
var nullVar = null;
var result3 = nullVar ?? "default";
assert("?? with null value", result3, "default");

// ?? with expression on right side
var result4 = undefinedVar2 ?? ("fall" & "back");
assert("?? with expression", result4, "fallback");

// ?? chained
var result5 = undefinedA ?? undefinedB ?? "final";
assert("?? chained", result5, "final");

// ?? with numeric values
var num = 42;
var result6 = num ?? 0;
assert("?? with numeric defined", result6, 42);

// Verify ?: still works alongside ??
var result7 = undefinedVar3 ?: "elvis";
assert("?: still works", result7, "elvis");

suiteEnd();
</cfscript>
