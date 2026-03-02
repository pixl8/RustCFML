component {
    property name="greeting" type="string" default="Hello";

    function init(string greeting="Hello") {
        this.greeting = arguments.greeting;
        return this;
    }

    function greet(required string name) {
        return this.greeting & ", " & arguments.name & "!";
    }

    function getGreeting() {
        return this.greeting;
    }

    function setGreeting(required string greeting) {
        this.greeting = arguments.greeting;
    }
}
