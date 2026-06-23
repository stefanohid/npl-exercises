pub fn hello_world(_x: String) -> String {
    ownership();
    _x
    // assert!(true, "This is a test assertion that always passes");
    //     //the char ! indicates that it's a macro, not a function call
    //     //throws a panic message if false
    // let x = 5;
    // let y: i32 = 10;
    // println!("Hello, world! The value of x is {} and the value of y is {}", x, y);

    // let tuple: (i32, i64, u32) = (2, 5, 3);
    // let i64_from_tuple = tuple.1; //this way you point to the second element of the tuple, which is 5
    // let (_a, b, _) = tuple; //this way you can destructure the tuple into its components
    //     //the underscore prefix tells rust not to warn about unused variables, since we only care about b in this case
    // println!("The value of b is {}", b);

    // //Strings
    // let s: String = String::from("hello"); //s owns "hello"
    // let slice: &str = &s[0..2]; // slice is a reference to part of hello

    // //Arrays - they are not pointers like in C, but they are fixed-size collections of elements of the same type
    // let v = [1, 2, 3, 4, 5]; //implicit
    // let v2: [i32; 5] = [1, 2, 3, 4, 5]; //explicit

    // s //this is the standard way to return a value from a function, but you can also use "return" with a semicolon
    //     //typical behavior of expressions; statements do end up with semicolons and return no value
}

fn ownership() {
    let s1 = String::from("pippo");
    let s2 = print_string(s1);
    println!("Second print {s2}");
    // let s2 = s1;
    // println!("{}", s1); //this will throw an error since s1 has been moved to s2, and we cannot use it anymore
    //     //this is the ownership system of Rust, which ensures memory safety without a garbage collector
    //     //when we assign s1 to s2, we are moving the ownership of the string
    //     //In Java we have the garbage collector that automatically manages memory, 
    //         //so s1 and s2 would both be references to the same string, and we could use both of them without any issues


    // //This, instead, creates an actual copy of the string, so s1 and s2 are independent
    // let s1 = String::from("pippo");
    // let s2 = s1.clone();
    // println!("{}", s1); // OK
    // println!("{}", s2); // OK

}

fn print_string (s: String) -> String{
    println!("Print string {s}");
    s //this function had taken ownership of the string, but we return it at the end, so the ownership is transferred back to the caller, and we can use it again
}