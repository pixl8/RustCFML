<cfscript>
// Test component accessors attribute
include "../harness.cfm";

suiteBegin("Component Accessors");

// Test 1: Basic accessor generation - using separate CFC file (TestComponent.cfc)
tc = new TestComponent();
tc.setName("John");
tc.setEmail("john@example.com");
tc.setAge(30);

assertTrue("accessor setter/getter name", tc.getName() eq "John");
assertTrue("accessor setter/getter email", tc.getEmail() eq "john@example.com");
assertTrue("accessor setter/getter age", tc.getAge() eq 30);

// Test 2: Direct property access
tc.name = "Jane";
tc.age = 25;

assertTrue("direct property access name", tc.name eq "Jane");
assertTrue("direct property access age", tc.age eq 25);

// Test 3: accessors="false" should not generate accessors - using NoAccessorComponent.cfc
nac = new NoAccessorComponent();
nac.value = "test";
assertTrue("no accessors direct access", nac.value eq "test");

suiteEnd();
</cfscript>