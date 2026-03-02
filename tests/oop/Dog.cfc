component extends="Animal" {
    function init(string sound="Woof") {
        super.init("Dog", arguments.sound);
        return this;
    }

    function fetch() {
        return "Fetching!";
    }
}
