component implements="IShape" {
    property name="radius" type="numeric";

    function init(required numeric radius) {
        this.radius = arguments.radius;
        return this;
    }

    function area() {
        return 3.14159265 * this.radius * this.radius;
    }

    function perimeter() {
        return 2 * 3.14159265 * this.radius;
    }
}
