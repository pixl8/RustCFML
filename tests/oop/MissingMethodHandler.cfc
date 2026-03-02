component {
    function onMissingMethod(required string missingMethodName, required struct missingMethodArguments) {
        return "called: " & arguments.missingMethodName;
    }
}
