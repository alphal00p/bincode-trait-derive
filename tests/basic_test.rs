use bincode_trait_derive::{BorrowDecodeFromDecode, Decode, Encode};

#[derive(Debug, Clone)]
pub struct TestContext {
    pub version: u32,
}

// Simple struct with context_type
#[derive(Encode, Decode, BorrowDecodeFromDecode)]
#[trait_decode(context_type = TestContext)]
pub struct TestStruct {
    pub id: u32,
    pub data: String,
}

// Simple enum with context_type
#[derive(Encode, Decode, BorrowDecodeFromDecode)]
#[trait_decode(context_type = TestContext)]
pub enum TestEnum {
    Simple(u32),
    Complex { name: String },
}

#[test]
fn test_basic_struct() {
    let test = TestStruct {
        id: 42,
        data: "Hello".to_string(),
    };
    
    // Just test that we can encode
    let encoded: Vec<u8> = bincode::encode_to_vec(test, bincode::config::standard()).unwrap();
    assert!(!encoded.is_empty());
}

#[test]
fn test_basic_enum() {
    let test = TestEnum::Simple(42);
    
    // Just test that we can encode
    let encoded: Vec<u8> = bincode::encode_to_vec(test, bincode::config::standard()).unwrap();
    assert!(!encoded.is_empty());
}