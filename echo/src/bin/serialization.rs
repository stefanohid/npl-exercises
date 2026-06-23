use serde::{Serialize, Deserialize};
use serde_json::json;

/// When you send a number through a channel, you need to understand how to represent it first.
/// Endianness problem: the order of bytes in a multi-byte number can differ between systems (big-endian vs little-endian).
/// To solve this problem, we can use serialization and deserialization techniques to convert the number into 
/// a standardized format before sending it through the channel, and then convert it back to its original form on the receiving end.
/// Convention: we send numbers in big-endian format, which means the most significant byte is sent first.
/// Endpoints then translate the number to their native endianness if necessary.
fn main() {
    let _num: u16 = 0x1234;
    // println!("Is big-endian: {}", is_big_endian(num));
    //is_big_endian(num);
    serde();
}

#[allow(dead_code)]
fn is_big_endian(num: u16) {
    // let bytes = num.to_be_bytes(); // Convert to big-endian byte array
    // bytes[0] == 0x12 // Check if the first byte is the most significant byte
    let nn = num.to_ne_bytes(); // Convert to native-endian byte array
    println!("Native-endian bytes: {:?}", nn);
    println!("Big-endian bytes: {:?}", num.to_be_bytes());
}

/// Serde: Serialize-Deserialize framework.
fn serde() {
    #[derive(Serialize, Deserialize)]
    struct MyStruct {
        id: u32,
        name: String,
    }

    let my_struct = MyStruct { id: 1, name: "Alice".to_string() };

    let _my_struct_json = json!({
        "id": 1,
        "name": "Alice"
    });
    
    // Serialize to JSON
    let json_str = serde_json::to_string(&my_struct).unwrap();
    println!("Serialized JSON: {}", json_str);
    
    // Deserialize from JSON
    let deserialized_struct: MyStruct = serde_json::from_str(&json_str).unwrap();
    println!("Deserialized struct: id={}, name={}", deserialized_struct.id, deserialized_struct.name);
}