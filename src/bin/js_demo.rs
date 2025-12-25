//! JavaScript Engine Demo
//! 
//! Demonstrates the basic JavaScript engine capabilities.

fn main() {
    println!("=== Rust JavaScript Engine Demo ===\n");
    
    // Test 1: Simple arithmetic
    println!("Test 1: Simple arithmetic");
    run_js("1 + 2 * 3");
    
    // Test 2: Variables
    println!("\nTest 2: Variables");
    run_js("var x = 10; var y = 20; x + y");
    
    // Test 3: Strings
    println!("\nTest 3: String concatenation");
    run_js("'Hello' + ' ' + 'World!'");
    
    // Test 4: Console.log
    println!("\nTest 4: console.log");
    run_js("console.log('Hello from JavaScript!'); console.log(1 + 2 + 3);");
    
    // Test 5: Functions
    println!("\nTest 5: Functions");
    run_js(r#"
        function add(a, b) {
            return a + b;
        }
        console.log('add(5, 3) =', add(5, 3));
    "#);
    
    // Test 6: If statements
    println!("\nTest 6: If statements");
    run_js(r#"
        var x = 10;
        if (x > 5) {
            console.log('x is greater than 5');
        } else {
            console.log('x is 5 or less');
        }
    "#);
    
    // Test 7: Loops
    println!("\nTest 7: For loop");
    run_js(r#"
        var sum = 0;
        for (var i = 1; i <= 5; i = i + 1) {
            sum = sum + i;
        }
        console.log('Sum of 1 to 5:', sum);
    "#);
    
    // Test 8: Objects
    println!("\nTest 8: Objects");
    run_js(r#"
        var person = { name: 'John', age: 30 };
        console.log('Name:', person.name);
        console.log('Age:', person.age);
    "#);
    
    // Test 9: Arrays
    println!("\nTest 9: Arrays");
    run_js(r#"
        var arr = [1, 2, 3, 4, 5];
        console.log('Array length:', arr.length);
        console.log('First element:', arr[0]);
    "#);
    
    // Test 10: Math object
    println!("\nTest 10: Math object");
    run_js(r#"
        console.log('Math.PI:', Math.PI);
        console.log('Math.floor(3.7):', Math.floor(3.7));
        console.log('Math.sqrt(16):', Math.sqrt(16));
        console.log('Math.pow(2, 8):', Math.pow(2, 8));
    "#);
    
    // Test 11: Comparison and boolean logic
    println!("\nTest 11: Comparisons");
    run_js(r#"
        console.log('5 > 3:', 5 > 3);
        console.log('5 === 5:', 5 === 5);
        console.log('true && false:', true && false);
        console.log('true || false:', true || false);
    "#);
    
    // Test 12: Ternary operator
    println!("\nTest 12: Ternary operator");
    run_js(r#"
        var age = 20;
        var status = age >= 18 ? 'adult' : 'minor';
        console.log('Status:', status);
    "#);
    
    println!("\n=== Demo Complete ===");
}

fn run_js(code: &str) {
    println!("  Code: {}", code.lines().next().unwrap_or(code).trim());
    match js_engine::execute(code) {
        Ok(result) => println!("  Result: {:?}", result),
        Err(e) => println!("  Error: {}", e),
    }
}
