<cfscript>
suiteBegin("Interfaces");

// Create Circle with radius 5
c = createObject("component", "oop.Circle").init(5);

// Check radius property
assert("circle radius", c.radius, 5);

// Check area: pi * r^2 = 3.14159265 * 25 = 78.5398...
assert("circle area rounded", round(c.area()), 79);

// Check perimeter: 2 * pi * r = 2 * 3.14159265 * 5 = 31.4159...
assert("circle perimeter rounded", round(c.perimeter()), 31);

// Create a second circle with radius 10
c2 = createObject("component", "oop.Circle").init(10);
assert("circle2 radius", c2.radius, 10);
assert("circle2 area rounded", round(c2.area()), 314);
assert("circle2 perimeter rounded", round(c2.perimeter()), 63);

// Verify area scales with radius squared: c2.area / c.area should be 4
// (radius 10 vs radius 5, so area ratio = (10/5)^2 = 4)
assert("area scales with r squared", round(c2.area() / c.area()), 4);

suiteEnd();
</cfscript>
