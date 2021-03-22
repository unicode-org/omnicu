// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Collection of traits for providers that support type erasure of data structs.
//!
//! There are two traits for data structs, `ErasedDataStruct` and `SerdeSeDataStruct`.
//! Both of these traits are compatible with DataProvider such that they can be returned
//! from a call to load_payload.
//!
//! There are corresponding traits that a data provider can implement if it is capable of
//! upcasting data into either of the data struct traits.

use crate::error::Error;
use crate::prelude::*;
use std::any::Any;
use std::any::TypeId;
use std::borrow::Cow;
use std::fmt::Debug;

/// Auto-implemented trait allowing for type erasure of data provider structs. Requires the
/// static lifetime in order to be convertible to Any.
pub trait ErasedDataStruct: 'static + Debug {
    /// Clone this trait object reference, returning a boxed trait object.
    fn clone_into_box(&self) -> Box<dyn ErasedDataStruct>;

    /// Return this boxed trait object as Box<dyn Any>.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_provider::erased::ErasedDataStruct;
    /// use icu_provider::hello_world::HelloWorldV1;
    ///
    /// // Create type-erased box
    /// let erased: Box<dyn ErasedDataStruct> = Box::new(HelloWorldV1::default());
    ///
    /// // Convert to typed box
    /// let boxed: Box<HelloWorldV1> = erased.into_any().downcast().expect("Types should match");
    /// ```
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    /// Return this trait object reference as &dyn Any.
    ///
    /// Also see associated method downcast_ref().
    ///
    /// # Example
    ///
    /// ```
    /// use icu_provider::erased::ErasedDataStruct;
    /// use icu_provider::hello_world::HelloWorldV1;
    ///
    /// // Create type-erased reference
    /// let data = HelloWorldV1::default();
    /// let erased: &dyn ErasedDataStruct = &data;
    ///
    /// // Borrow as typed reference
    /// let borrowed: &HelloWorldV1 = erased.as_any().downcast_ref().expect("Types should match");
    /// ```
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "provider_serde")]
pub trait SerdeSeDataStruct<'s>: 's + Debug {
    /// Clone this trait object reference, returning a boxed trait object.
    fn clone_into_box(&self) -> Box<dyn SerdeSeDataStruct<'s> + 's>;

    /// Return this trait object reference for Serde serialization.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_provider::erased::SerdeSeDataStruct;
    /// use icu_provider::hello_world::HelloWorldV1;
    ///
    /// // Create type-erased reference
    /// let data = HelloWorldV1::default();
    /// let erased: &dyn SerdeSeDataStruct = &data;
    ///
    /// // Borrow as serialize trait object
    /// let serialize: &dyn erased_serde::Serialize = erased.as_serialize();
    ///
    /// // Serialize the object to a JSON string
    /// let mut buffer: Vec<u8> = vec![];
    /// serialize.erased_serialize(
    ///     &mut erased_serde::Serializer::erase(
    ///         &mut serde_json::Serializer::new(&mut buffer)
    ///     )
    /// ).expect("Serialization should succeed");
    /// assert_eq!("{\"message\":\"(und) Hello World\"}".as_bytes(), buffer);
    /// ```
    fn as_serialize(&self) -> &dyn erased_serde::Serialize;
}

impl_dyn_clone!(ErasedDataStruct);

#[cfg(feature = "provider_serde")]
impl_dyn_clone!(SerdeSeDataStruct<'s>, 's);

impl_dyn_from_payload!(ErasedDataStruct, 'd, 's);

#[cfg(feature = "provider_serde")]
impl_dyn_from_payload!(SerdeSeDataStruct<'s>, 'd, 's);

impl dyn ErasedDataStruct {
    /// Convenience function: Return a downcast reference, or an error if mismatched types.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_provider::erased::ErasedDataStruct;
    /// use icu_provider::hello_world::HelloWorldV1;
    ///
    /// // Create type-erased reference
    /// let data = HelloWorldV1::default();
    /// let erased: &dyn ErasedDataStruct = &data;
    ///
    /// // Borrow as typed reference
    /// let borrowed: &HelloWorldV1 = erased.downcast_ref().expect("Types should match");
    /// ```
    pub fn downcast_ref<T: Any>(&self) -> Result<&T, Error> {
        self.as_any()
            .downcast_ref()
            .ok_or_else(|| Error::MismatchedType {
                actual: Some(self.as_any().type_id()),
                generic: Some(TypeId::of::<T>()),
            })
    }
}

impl<'d, T> DataPayload<'d, T>
where
    T: ErasedDataStruct + Clone,
{
    /// Convert this DataPayload of a Sized type into a DataPayload of an ErasedDataStruct.
    ///
    /// Can be used to implement ErasedDataProvider on types implementing DataProvider.
    pub fn into_erased(self) -> DataPayload<'d, dyn ErasedDataStruct> {
        self.into()
    }
}

#[cfg(feature = "provider_serde")]
impl<'d, 's: 'd, T> DataPayload<'d, T>
where
    T: SerdeSeDataStruct<'s> + Clone,
{
    /// Convert this DataPayload of a Sized type into a DataPayload of a SerdeSeDataStruct.
    pub fn into_serde_se(self) -> DataPayload<'d, dyn SerdeSeDataStruct<'s> + 's> {
        self.into()
    }
}

impl<'d> DataPayload<'d, dyn ErasedDataStruct> {
    /// Convert this DataPayload of an ErasedDataStruct into a DataPayload of a Sized type. Returns
    /// an error if the type is not compatible.
    ///
    /// Can be used to implement DataProvider on types implementing ErasedDataProvider.
    pub fn downcast<T>(self) -> Result<DataPayload<'d, T>, Error>
    where
        T: Clone + Debug + Any,
    {
        let old_cow = match self.cow {
            Some(cow) => cow,
            None => return Ok(DataPayload { cow: None }),
        };
        let new_cow = match old_cow {
            Cow::Borrowed(erased) => {
                let borrowed: &'d T =
                    erased
                        .as_any()
                        .downcast_ref()
                        .ok_or_else(|| Error::MismatchedType {
                            actual: Some(erased.type_id()),
                            generic: Some(TypeId::of::<T>()),
                        })?;
                Cow::Borrowed(borrowed)
            }
            Cow::Owned(erased) => {
                let boxed: Box<T> =
                    erased
                        .into_any()
                        .downcast()
                        .map_err(|any| Error::MismatchedType {
                            actual: Some(any.type_id()),
                            generic: Some(TypeId::of::<T>()),
                        })?;
                Cow::Owned(*boxed)
            }
        };
        Ok(DataPayload { cow: Some(new_cow) })
    }
}

impl<T> ErasedDataStruct for T
where
    T: Clone + Debug + Any,
{
    fn clone_into_box(&self) -> Box<dyn ErasedDataStruct> {
        Box::new(self.clone())
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(feature = "provider_serde")]
impl<'s, T> SerdeSeDataStruct<'s> for T
where
    T: 's + serde::Serialize + Clone + Debug,
{
    fn clone_into_box(&self) -> Box<dyn SerdeSeDataStruct<'s> + 's> {
        Box::new(self.clone())
    }
    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        self
    }
}

/// Note: This trait is redundant with `DataProvider<dyn ErasedDataStruct>` and auto-implemented
/// for all types implementing that trait. This trait may eventually be removed when the following
/// Rust issues are resolved:
///
/// - https://github.com/rust-lang/rust/issues/41517 (trait aliases are not supported)
/// - https://github.com/rust-lang/rust/issues/68636 (identical traits can't be auto-implemented)

/// A type-erased data provider that loads a payload of types implementing Any.
pub trait ErasedDataProvider<'d> {
    /// Query the provider for data, returning the result as an ErasedDataStruct trait object.
    ///
    /// Returns Ok if the request successfully loaded data. If data failed to load, returns an
    /// Error with more information.
    fn load_erased(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'d, dyn ErasedDataStruct>, Error>;
}

// Auto-implement `ErasedDataProvider` on types implementing `DataProvider<dyn ErasedDataStruct>`
impl<'d, T> ErasedDataProvider<'d> for T
where
    T: DataProvider<'d, dyn ErasedDataStruct>,
{
    fn load_erased(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'d, dyn ErasedDataStruct>, Error> {
        DataProvider::<dyn ErasedDataStruct>::load_payload(self, req)
    }
}

/// Serve `Sized` objects from an `ErasedDataProvider` via downcasting.
impl<'a, 'd, 'de, T> DataProvider<'d, T> for dyn ErasedDataProvider<'d> + 'a
where
    T: Clone + Debug + Any,
{
    fn load_payload(&self, req: &DataRequest) -> Result<DataResponse<'d, T>, Error> {
        let result = ErasedDataProvider::load_erased(self, req)?;
        Ok(DataResponse {
            metadata: result.metadata,
            payload: result.payload.downcast()?,
        })
    }
}
