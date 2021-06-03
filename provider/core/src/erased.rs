// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Collection of traits for providers that support type erasure of data structs.

use crate::error::Error;
use crate::prelude::*;
use std::any::Any;
use std::any::TypeId;
use std::rc::Rc;

/// Auto-implemented trait allowing for type erasure of data provider structs.
///
/// Requires the static lifetime in order to be convertible to [`Any`].
pub trait ErasedDataStruct: 'static {
    /// Clone this trait object reference, returning a boxed trait object.
    fn clone_into_box(&self) -> Box<dyn ErasedDataStruct>;

    /// Return this boxed trait object as [`Box`]`<dyn `[`Any`]`>`.
    ///
    /// # Examples
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

    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any>;

    /// Return this trait object reference as `&dyn `[`Any`].
    ///
    /// Also see associated method [`downcast_ref()`](trait.ErasedDataStruct.html#method.downcast_ref).
    ///
    /// # Examples
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

impl_dyn_clone!(ErasedDataStruct);

// TODO: This could be moved to yoke as a blanket impl
impl<'s> ZeroCopyClone<dyn ErasedDataStruct> for &'static dyn ErasedDataStruct {
    fn zcc<'b>(this: &'b (dyn ErasedDataStruct)) -> &'b dyn ErasedDataStruct {
        this
    }
}

/// Marker type for [`ErasedDataStruct`].
#[allow(non_camel_case_types)]
pub struct ErasedDataStruct_M {}

impl<'s> DataMarker<'s> for ErasedDataStruct_M {
    type Yokeable = &'static dyn ErasedDataStruct;
    type Cart = dyn ErasedDataStruct;
}

impl<'d, M> crate::dynutil::UpcastDataPayload<'d, 'static, M> for ErasedDataStruct_M
where
    M: DataMarker<'static>,
    M::Cart: Sized,
{
    fn upcast(other: DataPayload<'d, 'static, M>) -> DataPayload<'d, 'static, ErasedDataStruct_M> {
        use crate::data_provider::DataPayloadInner::*;
        match other.inner {
            Borrowed(yoke) => {
                // TODO(#752): This is not completely sound, because calling `.into_backing_cart()`
                // throws away overrides stored in the Yokeable, such as those from `.with_mut()`.
                let cart: &'d dyn ErasedDataStruct = yoke.into_backing_cart();
                DataPayload::from_borrowed(cart)
            }
            RcStruct(yoke) => {
                let cart: Rc<dyn ErasedDataStruct> = Rc::from(yoke);
                DataPayload::from_partial_owned(cart)
            }
            Owned(yoke) => {
                let cart: Rc<dyn ErasedDataStruct> = Rc::from(yoke);
                DataPayload::from_partial_owned(cart)
            }
        }
    }
}

impl<'d> DataPayload<'d, 'static, ErasedDataStruct_M> {
    /// Convert this [`DataPayload`] of an [`ErasedDataStruct`] into a [`DataPayload`] of a concrete type.
    /// Returns an error if the type is not compatible.
    ///
    /// This is the main way to consume data returned from an [`ErasedDataProvider`].
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_provider::prelude::*;
    /// use icu_provider::erased::*;
    /// use icu_provider::hello_world::*;
    /// use icu_locid_macros::langid;
    ///
    /// let provider = HelloWorldProvider::new_with_placeholder_data();
    ///
    /// let erased_payload: DataPayload<ErasedDataStruct_M> = provider
    ///     .load_payload(&DataRequest {
    ///         resource_path: ResourcePath {
    ///             key: key::HELLO_WORLD_V1,
    ///             options: ResourceOptions {
    ///                 variant: None,
    ///                 langid: Some(langid!("de")),
    ///             }
    ///         }
    ///     })
    ///     .expect("Loading should succeed")
    ///     .take_payload()
    ///     .expect("Data should be present");
    ///
    /// let downcast_payload: DataPayload<HelloWorldV1_M> = erased_payload
    ///     .downcast()
    ///     .expect("Types should match");
    ///
    /// assert_eq!("Hallo Welt", downcast_payload.get().message);
    /// ```
    pub fn downcast<M>(self) -> Result<DataPayload<'d, 'static, M>, Error>
    where
        M: DataMarker<'static>,
        M::Cart: Sized,
        M::Yokeable: ZeroCopyClone<M::Cart>,
    {
        use crate::data_provider::DataPayloadInner::*;
        use yoke::Yoke;
        match self.inner {
            Borrowed(yoke) => {
                let any_ref: &dyn Any = yoke.into_backing_cart().as_any();
                let y1 = any_ref.downcast_ref::<M::Cart>();
                match y1 {
                    Some(t_ref) => return Ok(DataPayload::from_borrowed(t_ref)),
                    None => {
                        return Err(Error::MismatchedType {
                            actual: Some(any_ref.type_id()),
                            generic: Some(TypeId::of::<M::Cart>()),
                        })
                    }
                };
            }
            RcStruct(yoke) => {
                let any_rc: Rc<dyn Any> = yoke.into_backing_cart().into_any_rc();
                // `any_rc` is the Yoke that was converted into the `dyn ErasedDataStruct`. It could have
                // been any valid variant of Yoke, so we need to check each possibility.
                let y1 = any_rc.downcast::<Yoke<M::Yokeable, Rc<M::Cart>>>();
                let any_rc = match y1 {
                    Ok(rc_yoke) => match Rc::try_unwrap(rc_yoke) {
                        Ok(yoke) => {
                            return Ok(DataPayload {
                                inner: RcStruct(yoke),
                            })
                        }
                        Err(_) => return Err(Error::MultipleReferences),
                    },
                    Err(any_rc) => any_rc,
                };
                let y2 = any_rc.downcast::<Yoke<M::Yokeable, Option<&'static ()>>>();
                let any_rc = match y2 {
                    Ok(rc_yoke) => match Rc::try_unwrap(rc_yoke) {
                        Ok(yoke) => return Ok(DataPayload { inner: Owned(yoke) }),
                        Err(_) => return Err(Error::MultipleReferences),
                    },
                    Err(any_rc) => any_rc,
                };
                return Err(Error::MismatchedType {
                    actual: Some(any_rc.type_id()),
                    generic: Some(TypeId::of::<M::Cart>()),
                });
            }
            // This is unreachable because ErasedDataStruct_M cannot be fully owned, since it
            // contains a reference.
            Owned(_) => unreachable!(),
        };
    }
}

impl<T> ErasedDataStruct for T
where
    T: Any,
    for<'a> &'a T: Clone,
{
    fn clone_into_box(&self) -> Box<dyn ErasedDataStruct> {
        todo!()
        // Box::new(self.clone())
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A type-erased data provider that loads a payload of types implementing [`Any`].
///
/// Note: This trait is redundant with [`DataProvider`]`<dyn `[`ErasedDataStruct`]`>` and auto-implemented
/// for all types implementing that trait. This trait may eventually be removed when the following
/// Rust issues are resolved:
///
/// - [#41517](https://github.com/rust-lang/rust/issues/41517) (trait aliases are not supported)
/// - [#68636](https://github.com/rust-lang/rust/issues/68636) (identical traits can't be auto-implemented)
pub trait ErasedDataProvider<'d> {
    /// Query the provider for data, returning the result as an [`ErasedDataStruct`] trait object.
    ///
    /// Returns [`Ok`] if the request successfully loaded data. If data failed to load, returns an
    /// Error with more information.
    fn load_erased(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'d, 'static, ErasedDataStruct_M>, Error>;
}

// Auto-implement `ErasedDataProvider` on types implementing `DataProvider<dyn ErasedDataStruct>`
impl<'d, T> ErasedDataProvider<'d> for T
where
    T: DataProvider<'d, 'static, ErasedDataStruct_M>,
{
    fn load_erased(
        &self,
        req: &DataRequest,
    ) -> Result<DataResponse<'d, 'static, ErasedDataStruct_M>, Error> {
        DataProvider::<ErasedDataStruct_M>::load_payload(self, req)
    }
}

impl<'d, M> DataProvider<'d, 'static, M> for dyn ErasedDataProvider<'d> + 'd
where
    M: DataMarker<'static>,
    <M::Yokeable as yoke::Yokeable<'static>>::Output: Clone + Any,
    M::Yokeable: ZeroCopyClone<M::Cart>,
    M::Cart: Sized,
{
    /// Serve [`Sized`] objects from an [`ErasedDataProvider`] via downcasting.
    fn load_payload(&self, req: &DataRequest) -> Result<DataResponse<'d, 'static, M>, Error> {
        let result = ErasedDataProvider::load_erased(self, req)?;
        Ok(DataResponse {
            metadata: result.metadata,
            payload: result.payload.map(|p| p.downcast()).transpose()?,
        })
    }
}
