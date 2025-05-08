use bincode_trait_derive::Encode;

// Create a Ring trait
pub trait Ring {
    type Element;
}

// Test Ring implementation 
pub struct TestRing {}
impl Ring for TestRing {
    type Element = i32;
}

// Implement Encode for TestRing
impl bincode::Encode for TestRing {
    fn encode<E: bincode::enc::Encoder>(&self, _encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct Fish {
    pub id: isize,
    pub name: String,
}

impl bincode::Encode for Fish {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.id.encode(encoder)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Particle {
    pub id: isize,
    pub name: String,
}

impl bincode::Encode for Particle {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.id.encode(encoder)?;
        Ok(())
    }
}

#[derive(Encode)]
pub struct TestAssociatedGeneric<T, F: Ring> {
    pub particle: Particle,
    pub fish: Fish,
    pub generic: T,
    pub field: F,
    pub el: F::Element,
}

#[test]
fn test_associated_generic() {
    let test = TestAssociatedGeneric {
        particle: Particle { id: 1, name: "particle".to_string() },
        fish: Fish { id: 2, name: "fish".to_string() },
        generic: "test".to_string(),
        field: TestRing {},
        el: 42,
    };
    
    // Try to encode
    let encoded: Vec<u8> = bincode::encode_to_vec(test, bincode::config::standard()).unwrap();
    assert!(!encoded.is_empty());
}