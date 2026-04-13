<cfscript>
suiteBegin("callStackGet / callStackDump");

// --- callStackGet returns array ---
stack = callStackGet();
assertTrue("callStackGet returns array", isArray(stack));
assertTrue("callStackGet has frames", arrayLen(stack) > 0);

// --- Each frame has correct keys ---
frame = stack[1];
assertTrue("frame has Function key", structKeyExists(frame, "Function"));
assertTrue("frame has Template key", structKeyExists(frame, "Template"));
assertTrue("frame has LineNumber key", structKeyExists(frame, "LineNumber"));

// --- Nested function calls show stack ---
function innerFunc() {
    return callStackGet();
}
function outerFunc() {
    return innerFunc();
}
nestedStack = outerFunc();
assertTrue("nested stack has multiple frames", arrayLen(nestedStack) >= 3);
assert("innermost frame is innerFunc", nestedStack[1].Function, "innerFunc");
assert("next frame is outerFunc", nestedStack[2].Function, "outerFunc");

// --- callStackGet("array") returns array format ---
function getStackAsArray() {
    return callStackGet("array");
}
arrayStack = getStackAsArray();
assertTrue("callStackGet array format", isArray(arrayStack));
assertTrue("callStackGet array has frames", arrayLen(arrayStack) >= 1);

suiteEnd();
</cfscript>
