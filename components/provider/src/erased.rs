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
//!
//! There are convenience macros, `impl_erased!` and `impl_serde_se!`, to help implement
//! the data provider traits.

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

#[cfg(feature = "eserde")]
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

impl ToOwned for dyn ErasedDataStruct {
    type Owned = Box<dyn ErasedDataStruct>;

    fn to_owned(&self) -> Self::Owned {
        ErasedDataStruct::clone_into_box(self)
    }
}

#[cfg(feature = "eserde")]
impl<'s> ToOwned for dyn SerdeSeDataStruct<'s> + 's {
    type Owned = Box<dyn SerdeSeDataStruct<'s> + 's>;

    fn to_owned(&self) -> Self::Owned {
        SerdeSeDataStruct::clone_into_box(self)
    }
}

impl Clone for Box<dyn ErasedDataStruct> {
    fn clone(&self) -> Box<dyn ErasedDataStruct> {
        ErasedDataStruct::clone_into_box(self.as_ref())
    }
}

#[cfg(feature = "eserde")]
impl<'s> Clone for Box<dyn SerdeSeDataStruct<'s> + 's> {
    fn clone(&self) -> Box<dyn SerdeSeDataStruct<'s> + 's> {
        SerdeSeDataStruct::clone_into_box(self.as_ref())
    }
}

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

impl<'d, T> DataResponse<'d, T>
where
    T: ErasedDataStruct + Clone,
{
    /// Convert this DataResponse of a Sized type into a DataResponse of an ErasedDataStruct.
    ///
    /// Can be used to implement ErasedDataProvider on types implementing DataProvider.
    pub fn into_erased(self) -> DataResponse<'d, dyn ErasedDataStruct> {
        DataResponse {
            metadata: self.metadata,
            payload: self.payload.map(|p| match p {
                Cow::Borrowed(v) => Cow::Borrowed(v as &dyn ErasedDataStruct),
                Cow::Owned(v) => {
                    let boxed: Box<dyn ErasedDataStruct> = Box::new(v);
                    Cow::Owned(boxed)
                }
            }),
        }
    }
}

#[cfg(feature = "eserde")]
impl<'d, 's: 'd, T> DataResponse<'d, T>
where
    T: SerdeSeDataStruct<'s> + Clone,
{
    /// Convert this DataResponse of a Sized type into a DataResponse of a SerdeSeDataStruct.
    pub fn into_serde_se(self) -> DataResponse<'d, dyn SerdeSeDataStruct<'s> + 's> {
        DataResponse {
            metadata: self.metadata,
            payload: self.payload.map(|p| match p {
                Cow::Borrowed(v) => Cow::Borrowed(v as &dyn SerdeSeDataStruct),
                Cow::Owned(v) => {
                    let boxed: Box<dyn SerdeSeDataStruct> = Box::new(v);
                    Cow::Owned(boxed)
                }
            }),
        }
    }
}

impl<'d> DataResponse<'d, dyn ErasedDataStruct> {
    /// Convert this DataResponse of an ErasedDataStruct into a DataResponse of a Sized type.
    ///
    /// Can be used to implement DataProvider on types implementing ErasedDataProvider.
    pub fn downcast<T>(self) -> Result<DataResponse<'d, T>, Error>
    where
        T: Clone + Debug + Any,
    {
        let metadata = self.metadata;
        let cow = match self.payload {
            Some(cow) => cow,
            None => {
                return Ok(DataResponse {
                    metadata,
                    payload: None,
                })
            }
        };
        let payload = match cow {
            Cow::Borrowed(erased) => {
                let borrowed: &'d T =
                    erased
                        .as_any()
                        .downcast_ref()
                        .ok_or_else(|| Error::MismatchedType {
                            actual: Some(erased.type_id()),
                            generic: Some(TypeId::of::<T>()),
                        })?;
                Some(Cow::Borrowed(borrowed))
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
                Some(Cow::Owned(*boxed))
            }
        };
        Ok(DataResponse { metadata, payload })
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

#[cfg(feature = "eserde")]
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

// Note: Once trait aliases land, we could enable the following alias.
// https://github.com/rust-lang/rust/issues/41517
// pub trait ErasedDataProvider<'d> = DataProvider<'d, dyn ErasedDataStruct>;

/// A type-erased data provider that loads a payload of types implementing Any.
pub trait ErasedDataProvider<'d> {
    /// Query the provider for data, returning the result as an ErasedDataStruct trait object.
    ///
    /// Returns Ok if the request successfully loaded data. If data failed to load, returns an
    /// Error with more information.
    fn load_payload(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'d, dyn ErasedDataStruct>, Error>;
}

#[cfg(feature = "eserde")]
pub trait SerdeSeDataProvider<'d, 's: 'd> {
    fn load_payload(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'d, dyn SerdeSeDataStruct<'s> + 's>, Error>;
}

/// Helper macro to implement ErasedDataProvider on an object implementing DataProvider for a
/// single type. Calls to `self.load_to_receiver` delegate to `self.load_payload`.
#[macro_export]
macro_rules! impl_erased {
    ($provider:ty, $struct:ty, $lifetime:tt) => {
        impl<$lifetime> $crate::erased::ErasedDataProvider<$lifetime> for $provider {
            fn load_payload(
                &self,
                req: &$crate::prelude::DataRequest,
            ) -> Result<
                $crate::prelude::DataResponse<'d, dyn $crate::erased::ErasedDataStruct>,
                $crate::prelude::DataError,
            > {
                let result: $crate::prelude::DataResponse<$struct> =
                    $crate::prelude::DataProvider::load_payload(self, req)?;
                Ok(result.into_erased())
            }
        }
    };
}

/// Helper macro to implement ErasedDataProvider on an object implementing DataProvider for a
/// single type. Calls to `self.load_to_receiver` delegate to `self.load_payload`.
#[cfg(feature = "eserde")]
#[macro_export]
macro_rules! impl_serde_se {
    ($provider:ty, $struct:ty, $lifetime:tt) => {
        impl<$lifetime, 's: $lifetime> $crate::erased::SerdeSeDataProvider<$lifetime, 's> for $provider {
            fn load_payload(
                &self,
                req: &$crate::prelude::DataRequest,
            ) -> Result<
                $crate::prelude::DataResponse<'d, dyn $crate::erased::SerdeSeDataStruct<'s> + 's>,
                $crate::prelude::DataError,
            > {
                let result: $crate::prelude::DataResponse<$struct> =
                    $crate::prelude::DataProvider::load_payload(self, req)?;
                Ok(result.into_serde_se())
            }
        }
    };
}

/// Convenience implementation of DataProvider<T> given an ErasedDataProvider trait object.
impl<'a, 'd, 'de, T> DataProvider<'d, T> for dyn ErasedDataProvider<'d> + 'a
where
    T: Clone + Debug + Any,
{
    fn load_payload(&self, req: &DataRequest) -> Result<DataResponse<'d, T>, Error> {
        let result = ErasedDataProvider::load_payload(self, req)?;
        result.downcast()
    }
}

impl<'d, T> DataProvider<'d, dyn ErasedDataStruct> for T
where
    T: ErasedDataProvider<'d>,
{
    fn load_payload(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'d, dyn ErasedDataStruct>, Error> {
        ErasedDataProvider::load_payload(self, req)
    }
}

#[cfg(feature = "eserde")]
impl<'d, 's: 'd, T> DataProvider<'d, dyn SerdeSeDataStruct<'s> + 's> for T
where
    T: SerdeSeDataProvider<'d, 's>,
{
    fn load_payload(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'d, dyn SerdeSeDataStruct<'s> + 's>, Error> {
        SerdeSeDataProvider::load_payload(self, req)
    }
}
