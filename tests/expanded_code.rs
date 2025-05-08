#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use bincode_trait_derive::{BorrowDecodeFromDecode, Decode, Encode};
pub struct Particle {
    pub id: isize,
    pub name: String,
}
#[automatically_derived]
impl ::core::clone::Clone for Particle {
    #[inline]
    fn clone(&self) -> Particle {
        Particle {
            id: ::core::clone::Clone::clone(&self.id),
            name: ::core::clone::Clone::clone(&self.name),
        }
    }
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
pub struct Fish {
    pub id: isize,
    pub name: String,
}
#[automatically_derived]
impl ::core::clone::Clone for Fish {
    #[inline]
    fn clone(&self) -> Fish {
        Fish {
            id: ::core::clone::Clone::clone(&self.id),
            name: ::core::clone::Clone::clone(&self.name),
        }
    }
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
#[trait_decode()]
pub struct Cow {
    pub id: usize,
}
impl ::bincode::Encode for Cow {
    fn encode<E: ::bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> std::result::Result<(), ::bincode::error::EncodeError> {
        ::bincode::Encode::encode(&self.id, encoder)?;
        Ok(())
    }
}
impl<__Context> ::bincode::Decode<__Context> for Cow {
    fn decode<D: ::bincode::de::Decoder<Context = __Context>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, ::bincode::error::DecodeError> {
        Ok(Self {
            id: ::bincode::Decode::decode(decoder)?,
        })
    }
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
#[trait_decode(trait = ParticleFishTrait)]
pub struct Test {
    pub particle: Particle,
    pub fish: Fish,
    pub cow: Cow,
    pub fish_or_cow: FishOrCow,
}
impl ::bincode::Encode for Test {
    fn encode<E: ::bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> std::result::Result<(), ::bincode::error::EncodeError> {
        ::bincode::Encode::encode(&self.particle, encoder)?;
        ::bincode::Encode::encode(&self.fish, encoder)?;
        ::bincode::Encode::encode(&self.cow, encoder)?;
        ::bincode::Encode::encode(&self.fish_or_cow, encoder)?;
        Ok(())
    }
}
impl<__Context> ::bincode::Decode<__Context> for Test
where
    __Context: ParticleFishTrait,
{
    fn decode<D: ::bincode::de::Decoder<Context = __Context>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, ::bincode::error::DecodeError> {
        Ok(Self {
            particle: ::bincode::Decode::decode(decoder)?,
            fish: ::bincode::Decode::decode(decoder)?,
            cow: ::bincode::Decode::decode(decoder)?,
            fish_or_cow: ::bincode::Decode::decode(decoder)?,
        })
    }
}
impl<'_de, __Context> ::bincode::BorrowDecode<'_de, __Context> for Test
where
    __Context: ParticleFishTrait,
{
    fn borrow_decode<D: ::bincode::de::BorrowDecoder<'_de, Context = __Context>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, ::bincode::error::DecodeError> {
        <Self as ::bincode::Decode<__Context>>::decode(decoder)
    }
}
#[trait_decode(trait = FishListTrait)]
pub enum FishOrCow {
    Fish(Fish),
    Cow(Cow),
}
impl ::bincode::Encode for FishOrCow {
    fn encode<E: ::bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> std::result::Result<(), ::bincode::error::EncodeError> {
        match self {
            Self::Fish(field0) => {
                ::bincode::Encode::encode(&0, encoder)?;
                ::bincode::Encode::encode(field0, encoder)?;
                Ok(())
            }
            Self::Cow(field0) => {
                ::bincode::Encode::encode(&1, encoder)?;
                ::bincode::Encode::encode(field0, encoder)?;
                Ok(())
            }
        }
    }
}
impl<__Context> ::bincode::Decode<__Context> for FishOrCow
where
    __Context: FishListTrait,
{
    fn decode<D: ::bincode::de::Decoder<Context = __Context>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, ::bincode::error::DecodeError> {
        let discriminant: usize = ::bincode::Decode::decode(decoder)?;
        match discriminant {
            0usize => Ok(Self::Fish(::bincode::Decode::decode(decoder)?)),
            1usize => Ok(Self::Cow(::bincode::Decode::decode(decoder)?)),
            _other => {
                Err(
                    ::bincode::error::DecodeError::OtherString(
                        ::alloc::__export::must_use({
                            let res = ::alloc::fmt::format(
                                format_args!("unexpected enum variant discriminant"),
                            );
                            res
                        }),
                    ),
                )
            }
        }
    }
}
impl<'_de, __Context> ::bincode::BorrowDecode<'_de, __Context> for FishOrCow
where
    __Context: FishListTrait,
{
    fn borrow_decode<D: ::bincode::de::BorrowDecoder<'_de, Context = __Context>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, ::bincode::error::DecodeError> {
        <Self as ::bincode::Decode<__Context>>::decode(decoder)
    }
}
#[trait_decode(trait = ParticleFishTrait)]
pub struct TestGeneric<T = Fish> {
    pub particle: Particle,
    pub fish: Fish,
    pub generic: T,
}
impl<T> ::bincode::Encode for TestGeneric<T>
where
    T: ::bincode::Encode,
{
    fn encode<E: ::bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> std::result::Result<(), ::bincode::error::EncodeError> {
        ::bincode::Encode::encode(&self.particle, encoder)?;
        ::bincode::Encode::encode(&self.fish, encoder)?;
        ::bincode::Encode::encode(&self.generic, encoder)?;
        Ok(())
    }
}
impl<T, __Context> ::bincode::Decode<__Context> for TestGeneric<T>
where
    __Context: ParticleFishTrait,
    T: ::bincode::Decode<__Context>,
{
    fn decode<D: ::bincode::de::Decoder<Context = __Context>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, ::bincode::error::DecodeError> {
        Ok(Self {
            particle: ::bincode::Decode::decode(decoder)?,
            fish: ::bincode::Decode::decode(decoder)?,
            generic: ::bincode::Decode::decode(decoder)?,
        })
    }
}
impl<'_de, T, __Context> ::bincode::BorrowDecode<'_de, __Context> for TestGeneric<T>
where
    __Context: ParticleFishTrait,
    T: ::bincode::Decode<__Context>,
{
    fn borrow_decode<D: ::bincode::de::BorrowDecoder<'_de, Context = __Context>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, ::bincode::error::DecodeError> {
        <Self as ::bincode::Decode<__Context>>::decode(decoder)
    }
}
pub trait Field {
    type Element;
}
pub struct TestAssociatedGeneric<F: Field, T = Fish> {
    pub particle: Particle,
    pub fish: Fish,
    pub generic: T,
    pub field: F,
    pub el: F::Element,
}
impl<F: Field, T> ::bincode::Encode for TestAssociatedGeneric<F, T>
where
    F: ::bincode::Encode,
    T: ::bincode::Encode,
{
    fn encode<E: ::bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> std::result::Result<(), ::bincode::error::EncodeError> {
        ::bincode::Encode::encode(&self.particle, encoder)?;
        ::bincode::Encode::encode(&self.fish, encoder)?;
        ::bincode::Encode::encode(&self.generic, encoder)?;
        ::bincode::Encode::encode(&self.field, encoder)?;
        ::bincode::Encode::encode(&self.el, encoder)?;
        Ok(())
    }
}
pub struct SpecificContext {
    pub version: u32,
}
#[automatically_derived]
impl ::core::fmt::Debug for SpecificContext {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "SpecificContext",
            "version",
            &&self.version,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for SpecificContext {
    #[inline]
    fn clone(&self) -> SpecificContext {
        SpecificContext {
            version: ::core::clone::Clone::clone(&self.version),
        }
    }
}
