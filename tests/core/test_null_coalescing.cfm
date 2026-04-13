<cfscript>
suiteBegin("Elvis (?:) Operator");

// ?: with undefined variable
result1 = undefinedVar ?: "fallback";
assert("?: with undefined var", result1, "fallback");

// ?: with defined variable
definedVar = "hello";
result2 = definedVar ?: "fallback";
assert("?: with defined var", result2, "hello");

// ?: with null value
nullVar = nullValue();
result3 = nullVar ?: "default";
assert("?: with null value", result3, "default");

// ?: with expression on right side
result4 = undefinedVar2 ?: ("fall" & "back");
assert("?: with expression", result4, "fallback");

// ?: chained
result5 = undefinedA ?: undefinedB ?: "final";
assert("?: chained", result5, "final");

// ?: with numeric values
num = 42;
result6 = num ?: 0;
assert("?: with numeric defined", result6, 42);

suiteEnd();
</cfscript>
