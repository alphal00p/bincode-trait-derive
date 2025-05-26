use bincode_trait_derive::{BorrowDecodeFromDecode, Decode, Encode};

#[derive(Debug, Clone)]
pub struct TestContext {
    pub version: u32,
}

// Simple struct with context_type
#[derive(Debug, Encode, Decode, BorrowDecodeFromDecode, PartialEq)]
#[trait_decode(context_type = TestContext)]
pub struct TestStruct {
    pub id: u32,
    pub data: String,
}

// Simple enum with context_type
#[derive(Debug, Encode, Decode, BorrowDecodeFromDecode, PartialEq)]
#[trait_decode(context_type = TestContext)]
pub enum TestEnum {
    Simple(u32),
    Complex { name: String },
}

#[test]
fn test_basic_struct() {
    let test_context = TestContext { version: 1 };

    let test = TestStruct {
        id: 42,
        data: "Hello".to_string(),
    };

    // Just test that we can encode
    let encoded: Vec<u8> = bincode::encode_to_vec(&test, bincode::config::standard()).unwrap();
    assert!(!encoded.is_empty());

    let decoded: TestStruct = bincode::decode_from_slice_with_context(
        &encoded,
        bincode::config::standard(),
        test_context,
    )
    .unwrap()
    .0;

    assert_eq!(decoded, test);
}

#[test]
fn test_basic_enum() {
    let test_context = TestContext { version: 1 };

    let test = TestEnum::Simple(42);

    // Just test that we can encode
    let encoded: Vec<u8> = bincode::encode_to_vec(&test, bincode::config::standard()).unwrap();
    assert!(!encoded.is_empty());

    let decoded: TestEnum = bincode::decode_from_slice_with_context(
        &encoded,
        bincode::config::standard(),
        test_context.clone(),
    )
    .unwrap()
    .0;

    assert_eq!(decoded, test);

    let test = TestEnum::Complex {
        name: "desalniettemin".to_string(),
    };

    let encoded: Vec<u8> = bincode::encode_to_vec(&test, bincode::config::standard()).unwrap();
    assert!(!encoded.is_empty());

    let decoded: TestEnum = bincode::decode_from_slice_with_context(
        &encoded,
        bincode::config::standard(),
        test_context,
    )
    .unwrap()
    .0;

    assert_eq!(decoded, test);
}
