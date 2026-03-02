component extends="Animal" {
    function init() {
        super.init("Cat", "Meow");
        return this;
    }

    function purr() {
        return "Purrrr";
    }
}
