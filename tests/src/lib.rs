use std::marker::PhantomData;

use bincode_trait_derive::{BorrowDecodeFromDecode, Decode, Encode};

#[derive(Clone)]
pub struct Particle {
    pub id: isize,
    pub name: String,
}

pub struct ParticleList {
    pub particles: Vec<Particle>,
}

impl ParticleList {
    fn get_particle_from_id(&self, id: isize) -> Option<Particle> {
        self.particles.iter().find(|p| p.id == id).cloned()
    }
}

pub trait ParticleListTrait {
    fn get_particle_list(&self) -> &ParticleList;
}

impl ParticleListTrait for ParticleList {
    fn get_particle_list(&self) -> &ParticleList {
        self
    }
}

impl bincode::Encode for Particle {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.id.encode(encoder)?;
        Ok(())
    }
}

impl<C> bincode::Decode<C> for Particle
where
    C: ParticleListTrait,
{
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id = isize::decode(decoder)?;
        let context = decoder.context();
        let particle_list = context.get_particle_list();
        Ok(particle_list.get_particle_from_id(id).unwrap())
    }
}

#[derive(Clone)]
pub struct Fish {
    pub id: isize,
    pub name: String,
}

impl bincode::Encode for Fish {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.id.encode(encoder)?;
        Ok(())
    }
}

pub struct FishList {
    pub fishes: Vec<Fish>,
}

impl FishList {
    fn get_fish_from_id(&self, id: isize) -> Option<Fish> {
        self.fishes.iter().find(|f| f.id == id).cloned()
    }
}

pub trait FishListTrait {
    fn get_fish_list(&self) -> &FishList;
}

impl FishListTrait for FishList {
    fn get_fish_list(&self) -> &FishList {
        self
    }
}

impl<C> bincode::Decode<C> for Fish
where
    C: FishListTrait,
{
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id = isize::decode(decoder)?;
        let context = decoder.context();
        let fish_list = context.get_fish_list();
        Ok(fish_list.get_fish_from_id(id).unwrap())
    }
}

#[derive(Encode, bincode_trait_derive::Decode)]
#[trait_decode()]
pub struct Cow {
    pub id: usize,
}

pub trait ParticleFishTrait: FishListTrait + ParticleListTrait {}

pub struct MyContext {
    pub particle_list: ParticleList,
    pub fish_list: FishList,
}

impl ParticleListTrait for MyContext {
    fn get_particle_list(&self) -> &ParticleList {
        &self.particle_list
    }
}

impl FishListTrait for MyContext {
    fn get_fish_list(&self) -> &FishList {
        &self.fish_list
    }
}

impl ParticleFishTrait for MyContext {}

#[derive(Encode, Decode, BorrowDecodeFromDecode)]
#[trait_decode(trait = ParticleFishTrait)]
pub struct Test {
    pub particle: Particle,
    pub fish: Fish,
    pub cow: Cow,
    pub fish_or_cow: FishOrCow,
}

#[derive(Encode, Decode, BorrowDecodeFromDecode)]
#[trait_decode(trait = FishListTrait)]
pub enum FishOrCow {
    Fish(Fish),
    Cow(Cow),
}

#[derive(bincode_trait_derive::Encode, Decode, BorrowDecodeFromDecode)]
#[trait_decode(trait = ParticleFishTrait)]
pub struct TestGeneric<T = Fish> {
    pub particle: Particle,
    pub fish: Fish,
    pub generic: T,
}

pub trait Ring {
    type Element;
}

pub trait Exponent {}
impl Exponent for u16 {}

pub trait MonomialOrder {}

impl MonomialOrder for LexOrder {}
pub struct LexOrder {}

#[derive(bincode_trait_derive::Encode)]
pub struct MultivariatePolynomial<F: Ring, E: Exponent = u16, O: MonomialOrder = LexOrder> {
    pub coefficients: Vec<F::Element>,
    pub exponents: Vec<E>,
    /// The coefficient ring.
    pub ring: F,
    // pub variables: Arc<Vec<Variable>>,
    pub(crate) _phantom: PhantomData<O>,
}

#[derive(bincode_trait_derive::Encode)]
pub struct TestAssociatedGeneric<T, F: Ring> {
    pub particle: Particle,
    pub fish: Fish,
    pub generic: T,
    pub field: F,
    pub el: F::Element,
}

#[derive(Debug, Clone)]
pub struct SpecificContext {
    pub version: u32,
    // This context could hold specific data needed for decoding,
    // though for these examples, the fields don't actively use it.
}

#[derive(
    bincode_trait_derive::Encode,
    bincode_trait_derive::Decode,
    bincode_trait_derive::BorrowDecodeFromDecode,
)]
#[trait_decode(context_type = SpecificContext)]
pub struct DataForSpecificContext {
    pub item_id: u32,
    pub description: String,
    pub related_cow: Cow, // Cow uses #[trait_decode()]
}

#[derive(
    bincode_trait_derive::Encode,
    bincode_trait_derive::Decode,
    bincode_trait_derive::BorrowDecodeFromDecode,
)]
#[trait_decode(context_type = SpecificContext)]
pub enum ItemVariantWithSpecificContext {
    Simple(u64),
    Described { name: String, value: i32 },
    ReferencedCow(Cow), // Add a variant with Cow
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_context() -> MyContext {
        let particle_list = ParticleList {
            particles: vec![
                Particle {
                    id: 0,
                    name: "squark".to_string(),
                },
                Particle {
                    id: 2,
                    name: "gluino".to_string(),
                },
            ],
        };

        let fish_list = FishList {
            fishes: vec![
                Fish {
                    id: 1,
                    name: "blobfish".to_string(),
                },
                Fish {
                    id: 3,
                    name: "starfish".to_string(),
                },
            ],
        };

        MyContext {
            particle_list,
            fish_list,
        }
    }

    #[test]
    fn test() {
        let context = build_test_context();

        let test_struct = Test {
            particle: Particle {
                id: 0,
                name: "squark".to_string(),
            },
            fish: Fish {
                id: 1,
                name: "blobfish".to_string(),
            },
            cow: Cow { id: 42 },
            fish_or_cow: FishOrCow::Fish(Fish {
                id: 3,
                name: "starfish".to_string(),
            }),
        };

        // Encode
        let encoded: Vec<u8> =
            bincode::encode_to_vec(test_struct, bincode::config::standard()).unwrap();

        // Decode
        let (decoded, _): (Test, usize) =
            bincode::decode_from_slice_with_context(&encoded, bincode::config::standard(), context)
                .unwrap();

        assert_eq!(decoded.particle.id, 0);
        assert_eq!(decoded.particle.name, "squark");
        assert_eq!(decoded.fish.id, 1);
        assert_eq!(decoded.fish.name, "blobfish");
        assert_eq!(decoded.cow.id, 42);
        match decoded.fish_or_cow {
            FishOrCow::Fish(fish) => {
                assert_eq!(fish.id, 3);
                assert_eq!(fish.name, "starfish");
            }
            FishOrCow::Cow(_) => panic!("Expected Fish variant"),
        }
    }

    #[test]
    fn test_generic_cow() {
        let context = build_test_context();

        let test_struct = TestGeneric {
            particle: Particle {
                id: 0,
                name: "squark".to_string(),
            },
            fish: Fish {
                id: 1,
                name: "blobfish".to_string(),
            },
            generic: Cow { id: 42 },
        };

        // Encode
        let encoded: Vec<u8> =
            bincode::encode_to_vec(test_struct, bincode::config::standard()).unwrap();

        // Decode
        let (decoded, _): (TestGeneric<Cow>, usize) =
            bincode::decode_from_slice_with_context(&encoded, bincode::config::standard(), context)
                .unwrap();

        assert_eq!(decoded.particle.id, 0);
        assert_eq!(decoded.particle.name, "squark");
        assert_eq!(decoded.fish.id, 1);
        assert_eq!(decoded.fish.name, "blobfish");
        assert_eq!(decoded.generic.id, 42);
    }

    #[test]
    fn test_generic_fish() {
        let context = build_test_context();

        let test_struct = TestGeneric {
            particle: Particle {
                id: 0,
                name: "squark".to_string(),
            },
            fish: Fish {
                id: 1,
                name: "blobfish".to_string(),
            },
            generic: Fish {
                id: 3,
                name: "starfish".to_string(),
            },
        };

        // Encode
        let encoded: Vec<u8> =
            bincode::encode_to_vec(test_struct, bincode::config::standard()).unwrap();

        // Decode
        let (decoded, _): (TestGeneric, usize) =
            bincode::decode_from_slice_with_context(&encoded, bincode::config::standard(), context)
                .unwrap();

        assert_eq!(decoded.particle.id, 0);
        assert_eq!(decoded.particle.name, "squark");
        assert_eq!(decoded.fish.id, 1);
        assert_eq!(decoded.fish.name, "blobfish");
        assert_eq!(decoded.generic.id, 3);
        assert_eq!(decoded.generic.name, "starfish");
    }
}
