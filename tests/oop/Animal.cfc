component {
    property name="species" type="string";
    property name="sound" type="string";

    function init(required string species, string sound="") {
        this.species = arguments.species;
        this.sound = arguments.sound;
        return this;
    }

    function speak() {
        return this.sound;
    }

    function getSpecies() {
        return this.species;
    }
}
