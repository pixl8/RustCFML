<cfscript>
function namedArgFunc(required string name, required string value) {
    return arguments.name & "=" & arguments.value;
}

function callWithNamedArgs() {
    var myName = "hello";
    var myValue = "world";
    return namedArgFunc(name=myName, value=myValue);
}
</cfscript>
