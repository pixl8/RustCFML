<cfscript>
suiteBegin("Inheritance");

// Create Dog with default sound
d = createObject("component", "oop.Dog").init();

// Test inherited speak method
assert("dog speak", d.speak(), "Woof");

// Test inherited getSpecies method
assert("dog getSpecies", d.getSpecies(), "Dog");

// Test Dog-specific method
assert("dog fetch", d.fetch(), "Fetching!");

// Test this scope properties inherited from Animal
assert("dog species property", d.species, "Dog");
assert("dog sound property", d.sound, "Woof");

// Create Cat
c = createObject("component", "oop.Cat").init();

// Test Cat inherited methods
assert("cat speak", c.speak(), "Meow");
assert("cat getSpecies", c.getSpecies(), "Cat");

// Test Cat-specific method
assert("cat purr", c.purr(), "Purrrr");

// Test Cat this scope properties
assert("cat species property", c.species, "Cat");
assert("cat sound property", c.sound, "Meow");

// Create Dog with custom sound
d2 = createObject("component", "oop.Dog").init("Bark");
assert("dog custom sound speak", d2.speak(), "Bark");
assert("dog custom sound property", d2.sound, "Bark");

suiteEnd();
</cfscript>
