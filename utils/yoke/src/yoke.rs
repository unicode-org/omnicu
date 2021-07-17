// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::Yokeable;
use core::marker::PhantomData;
use core::ops::Deref;
use stable_deref_trait::StableDeref;

#[cfg(feature = "alloc")]
use alloc::rc::Rc;
#[cfg(feature = "alloc")]
use alloc::sync::Arc;

/// A Cow-like borrowed object "yoked" to its backing data.
///
/// This allows things like zero copy deserialized data to carry around
/// shared references to their backing buffer.
///
/// `Y` (the [`Yokeable`]) is the object containing the references,
/// and will typically be of the form `Foo<'static>`. The `'static` is
/// not the actual lifetime of the data, rather it is a convenient way to erase
/// the lifetime and make it dynamic.
///
/// `C` is the "cart", which `Y` may contain references to.
///
/// The primary constructor for [`Yoke`] is [`Yoke::attach_to_cart()`]. Several variants of that
/// constructor are provided to serve numerous types of call sites and `Yoke` signatures.
///
/// # Example
///
/// For example, we can use this to store zero-copy deserialized data in a cache:
///
/// ```rust
/// # use yoke::{Yoke, Yokeable};
/// # use std::rc::Rc;
/// # use std::borrow::Cow;
/// # fn load_from_cache(_filename: &str) -> Rc<[u8]> {
/// #     // dummy implementation
/// #     Rc::new([0x5, 0, 0, 0, 0, 0, 0, 0, 0x68, 0x65, 0x6c, 0x6c, 0x6f])
/// # }
///
/// fn load_object(filename: &str) -> Yoke<Cow<'static, str>, Rc<[u8]>> {
///     let rc: Rc<[u8]> = load_from_cache(filename);
///     Yoke::<Cow<'static, str>, Rc<[u8]>>::attach_to_cart_badly(rc, |data: &[u8]| {
///         // essentially forcing a #[serde(borrow)]
///         Cow::Borrowed(bincode::deserialize(data).unwrap())
///     })
/// }
///
/// let yoke = load_object("filename.bincode");
/// assert_eq!(&**yoke.get(), "hello");
/// assert!(matches!(yoke.get(), &Cow::Borrowed(_)));
/// ```
///
pub struct Yoke<Y: for<'a> Yokeable<'a>, C> {
    // must be the first field for drop order
    // this will have a 'static lifetime parameter, that parameter is a lie
    yokeable: Y,
    cart: C,
}

impl<Y: for<'a> Yokeable<'a>, C: StableDeref> Yoke<Y, C> {
    /// Construct a [`Yoke`] by yokeing an object to a cart in a closure.
    ///
    /// See also [`Yoke::try_attach_to_cart()`] to return a `Result` from the closure.
    ///
    /// Due to [compiler bug #84937](https://github.com/rust-lang/rust/issues/84937), call sites
    /// for this function may not compile; if this happens, use
    /// [`Yoke::attach_to_cart_badly()`] instead.
    pub fn attach_to_cart<F>(cart: C, f: F) -> Self
    where
        F: for<'de> FnOnce(&'de <C as Deref>::Target) -> <Y as Yokeable<'de>>::Output,
    {
        let deserialized = f(cart.deref());
        Self {
            yokeable: unsafe { Y::make(deserialized) },
            cart,
        }
    }

    /// Construct a [`Yoke`] by yokeing an object to a cart. If an error occurs in the
    /// deserializer function, the error is passed up to the caller.
    ///
    /// Due to [compiler bug #84937](https://github.com/rust-lang/rust/issues/84937), call sites
    /// for this function may not compile; if this happens, use
    /// [`Yoke::try_attach_to_cart_badly()`] instead.
    pub fn try_attach_to_cart<E, F>(cart: C, f: F) -> Result<Self, E>
    where
        F: for<'de> FnOnce(&'de <C as Deref>::Target) -> Result<<Y as Yokeable<'de>>::Output, E>,
    {
        let deserialized = f(cart.deref())?;
        Ok(Self {
            yokeable: unsafe { Y::make(deserialized) },
            cart,
        })
    }

    /// Construct a [`Yoke`] by yokeing an object to a cart in a closure.
    ///
    /// For a version of this function that takes a `FnOnce` instead of a raw function pointer,
    /// see [`Yoke::attach_to_cart()`].
    ///
    /// # Example
    ///
    /// For example, we can use this to store zero-copy deserialized data in a cache:
    ///
    /// ```rust
    /// # use yoke::{Yoke, Yokeable};
    /// # use std::rc::Rc;
    /// # use std::borrow::Cow;
    /// # fn load_from_cache(_filename: &str) -> Rc<[u8]> {
    /// #     // dummy implementation
    /// #     Rc::new([0x5, 0, 0, 0, 0, 0, 0, 0, 0x68, 0x65, 0x6c, 0x6c, 0x6f])
    /// # }
    ///
    /// fn load_object(filename: &str) -> Yoke<Cow<'static, str>, Rc<[u8]>> {
    ///     let rc: Rc<[u8]> = load_from_cache(filename);
    ///     Yoke::<Cow<'static, str>, Rc<[u8]>>::attach_to_cart_badly(rc, |data: &[u8]| {
    ///         // essentially forcing a #[serde(borrow)]
    ///         Cow::Borrowed(bincode::deserialize(data).unwrap())
    ///     })
    /// }
    ///
    /// let yoke: Yoke<Cow<str>, _> = load_object("filename.bincode");
    /// assert_eq!(&**yoke.get(), "hello");
    /// assert!(matches!(yoke.get(), &Cow::Borrowed(_)));
    /// ```
    pub fn attach_to_cart_badly(
        cart: C,
        f: for<'de> fn(&'de <C as Deref>::Target) -> <Y as Yokeable<'de>>::Output,
    ) -> Self {
        let deserialized = f(cart.deref());
        Self {
            yokeable: unsafe { Y::make(deserialized) },
            cart,
        }
    }

    /// Construct a [`Yoke`] by yokeing an object to a cart. If an error occurs in the
    /// deserializer function, the error is passed up to the caller.
    ///
    /// For a version of this function that takes a `FnOnce` instead of a raw function pointer,
    /// see [`Yoke::try_attach_to_cart()`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use yoke::{Yoke, Yokeable};
    /// # use std::rc::Rc;
    /// # use std::borrow::Cow;
    /// let rc = Rc::new([0xb, 0xa, 0xd]);
    ///
    /// let yoke_result: Result<Yoke<Cow<str>, Rc<[u8]>>, _> =
    ///     Yoke::try_attach_to_cart_badly(rc, |data: &[u8]| {
    ///         bincode::deserialize(data)
    ///     });
    ///
    /// assert!(matches!(yoke_result, Err(_)));
    /// ```
    pub fn try_attach_to_cart_badly<E>(
        cart: C,
        f: for<'de> fn(&'de <C as Deref>::Target) -> Result<<Y as Yokeable<'de>>::Output, E>,
    ) -> Result<Self, E> {
        let deserialized = f(cart.deref())?;
        Ok(Self {
            yokeable: unsafe { Y::make(deserialized) },
            cart,
        })
    }
}

impl<Y: for<'a> Yokeable<'a>, C> Yoke<Y, C> {
    /// Obtain a valid reference to the yokeable data
    ///
    /// This essentially transforms the lifetime of the internal yokeable data to
    /// be valid.
    /// For example, if you're working with a `Yoke<Cow<'static, T>, C>`, this
    /// will return an `&'a Cow<'a, T>`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use yoke::{Yoke, Yokeable};
    /// # use std::rc::Rc;
    /// # use std::borrow::Cow;
    /// # fn load_from_cache(_filename: &str) -> Rc<[u8]> {
    /// #     // dummy implementation
    /// #     Rc::new([0x5, 0, 0, 0, 0, 0, 0, 0, 0x68, 0x65, 0x6c, 0x6c, 0x6f])
    /// # }
    /// #
    /// # fn load_object(filename: &str) -> Yoke<Cow<'static, str>, Rc<[u8]>> {
    /// #     let rc: Rc<[u8]> = load_from_cache(filename);
    /// #     Yoke::<Cow<'static, str>, Rc<[u8]>>::attach_to_cart_badly(rc, |data: &[u8]| {
    /// #         Cow::Borrowed(bincode::deserialize(data).unwrap())
    /// #     })
    /// # }
    ///
    /// // load_object() defined in the example at the top of this page
    /// let yoke: Yoke<Cow<str>, _> = load_object("filename.bincode");
    /// assert_eq!(yoke.get(), "hello");
    /// ```
    pub fn get<'a>(&'a self) -> &'a <Y as Yokeable<'a>>::Output {
        self.yokeable.transform()
    }

    /// Get a reference to the backing cart.
    ///
    /// This can be useful when building caches, etc. However, if you plan to store the cart
    /// separately from the yoke, read the note of caution below in [`Yoke::into_backing_cart`].
    pub fn backing_cart(&self) -> &C {
        &self.cart
    }

    /// Get the backing cart by value, dropping the yokeable object.
    ///
    /// **Caution:** Calling this method could cause information saved in the yokeable object but
    /// not the cart to be lost. Use this method only if the yokeable object cannot contain its
    /// own information.
    ///
    /// # Example
    ///
    /// Good example: the yokeable object is only a reference, so no information can be lost.
    ///
    /// ```
    /// use yoke::Yoke;
    ///
    /// let local_data = "foo".to_string();
    /// let yoke = Yoke::<
    ///     &'static str,
    ///     Box<String>
    /// >::attach_to_box_cart(Box::new(local_data));
    /// assert_eq!(*yoke.get(), "foo");
    ///
    /// // Get back the cart
    /// let cart = yoke.into_backing_cart();
    /// assert_eq!(&*cart, "foo");
    /// ```
    ///
    /// Bad example: information specified in `.with_mut()` is lost.
    ///
    /// ```
    /// use yoke::Yoke;
    /// use std::borrow::Cow;
    ///
    /// let local_data = "foo".to_string();
    /// let mut yoke = Yoke::<
    ///     Cow<'static, str>,
    ///     Box<String>
    /// >::attach_to_box_cart(Box::new(local_data));
    /// assert_eq!(yoke.get(), "foo");
    ///
    /// // Override data in the cart
    /// yoke.with_mut(|cow| {
    ///     let mut_str = cow.to_mut();
    ///     mut_str.clear();
    ///     mut_str.push_str("bar");
    /// });
    /// assert_eq!(yoke.get(), "bar");
    ///
    /// // Get back the cart
    /// let cart = yoke.into_backing_cart();
    /// assert_eq!(&*cart, "foo"); // WHOOPS!
    /// ```
    pub fn into_backing_cart(self) -> C {
        self.cart
    }

    /// Mutate the stored [`Yokeable`] data.
    ///
    /// See [`Yokeable::transform_mut()`] for why this operation is safe.
    ///
    /// # Example
    ///
    /// This can be used to partially mutate the stored data, provided
    /// no _new_ borrowed data is introduced.
    ///
    /// ```rust
    /// # use yoke::{Yoke, Yokeable};
    /// # use std::rc::Rc;
    /// # use std::borrow::Cow;
    /// # use std::mem;
    /// # fn load_from_cache(_filename: &str) -> Rc<[u8]> {
    /// #     // dummy implementation
    /// #     Rc::new([0x5, 0, 0, 0, 0, 0, 0, 0, 0x68, 0x65, 0x6c, 0x6c, 0x6f])
    /// # }
    /// #
    /// # fn load_object(filename: &str) -> Yoke<Bar<'static>, Rc<[u8]>> {
    /// #     let rc: Rc<[u8]> = load_from_cache(filename);
    /// #     Yoke::<Bar<'static>, Rc<[u8]>>::attach_to_cart_badly(rc, |data: &[u8]| {
    /// #         // A real implementation would properly deserialize `Bar` as a whole
    /// #         Bar {
    /// #             numbers: Cow::Borrowed(bincode::deserialize(data).unwrap()),
    /// #             string: Cow::Borrowed(bincode::deserialize(data).unwrap()),
    /// #             owned: Vec::new(),
    /// #         }
    /// #     })
    /// # }
    ///
    /// // also implements Yokeable
    /// struct Bar<'a> {
    ///     numbers: Cow<'a, [u8]>,
    ///     string: Cow<'a, str>,
    ///     owned: Vec<u8>,
    /// }
    ///
    /// // `load_object()` deserializes an object from a file
    /// let mut bar: Yoke<Bar, _> = load_object("filename.bincode");
    /// assert_eq!(bar.get().string, "hello");
    /// assert!(matches!(bar.get().string, Cow::Borrowed(_)));
    /// assert_eq!(&*bar.get().numbers, &[0x68, 0x65, 0x6c, 0x6c, 0x6f]);
    /// assert!(matches!(bar.get().numbers, Cow::Borrowed(_)));
    /// assert_eq!(&*bar.get().owned, &[]);
    ///
    /// bar.with_mut(|bar| {
    ///     bar.string.to_mut().push_str(" world");
    ///     bar.owned.extend_from_slice(&[1, 4, 1, 5, 9]);   
    /// });
    ///
    /// assert_eq!(bar.get().string, "hello world");
    /// assert!(matches!(bar.get().string, Cow::Owned(_)));
    /// assert_eq!(&*bar.get().owned, &[1, 4, 1, 5, 9]);
    /// // Unchanged and still Cow::Borrowed
    /// assert_eq!(&*bar.get().numbers, &[0x68, 0x65, 0x6c, 0x6c, 0x6f]);
    /// assert!(matches!(bar.get().numbers, Cow::Borrowed(_)));
    ///
    /// # unsafe impl<'a> Yokeable<'a> for Bar<'static> {
    /// #     type Output = Bar<'a>;
    /// #     fn transform(&'a self) -> &'a Bar<'a> {
    /// #         self
    /// #     }
    /// #
    /// #     fn transform_owned(self) -> Bar<'a> {
    /// #         // covariant lifetime cast, can be done safely
    /// #         self
    /// #     }
    /// #
    /// #     unsafe fn make(from: Bar<'a>) -> Self {
    /// #         let ret = mem::transmute_copy(&from);
    /// #         mem::forget(from);
    /// #         ret
    /// #     }
    /// #
    /// #     fn transform_mut<F>(&'a mut self, f: F)
    /// #     where
    /// #         F: 'static + FnOnce(&'a mut Self::Output),
    /// #     {
    /// #         unsafe { f(mem::transmute(self)) }
    /// #     }
    /// # }
    /// ```
    pub fn with_mut<'a, F>(&'a mut self, f: F)
    where
        F: 'static + for<'b> FnOnce(&'b mut <Y as Yokeable<'a>>::Output),
    {
        self.yokeable.transform_mut(f)
    }
}

impl<Y: for<'a> Yokeable<'a>> Yoke<Y, ()> {
    /// Construct a new [`Yoke`] from static data. There will be no
    /// references to `cart` here since [`Yokeable`]s are `'static`,
    /// this is good for e.g. constructing fully owned
    /// [`Yoke`]s with no internal borrowing.
    ///
    /// This is similar to [`Yoke::new_owned()`] but it does not allow you to
    /// mix the [`Yoke`] with borrowed data. This is primarily useful
    /// for using [`Yoke`] in generic scenarios.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use yoke::Yoke;
    /// # use std::borrow::Cow;
    /// # use std::rc::Rc;
    ///
    /// let owned: Cow<str> = "hello".to_owned().into();
    /// // this yoke can be intermingled with actually-borrowed Yokes
    /// let yoke: Yoke<Cow<str>, ()> = Yoke::new_always_owned(owned);
    ///
    /// assert_eq!(yoke.get(), "hello");
    /// ```
    pub fn new_always_owned(yokeable: Y) -> Self {
        Self { yokeable, cart: () }
    }
}

impl<Y: for<'a> Yokeable<'a>, C: StableDeref> Yoke<Y, Option<C>> {
    /// Construct a new [`Yoke`] from static data. There will be no
    /// references to `cart` here since [`Yokeable`]s are `'static`,
    /// this is good for e.g. constructing fully owned
    /// [`Yoke`]s with no internal borrowing.
    ///
    /// This can be paired with [`Yoke::attach_to_option_cart()`] to mix owned
    /// and borrowed data.
    ///
    /// If you do not wish to pair this with borrowed data, [`Yoke::new_always_owned()`] can
    /// be used to get a [`Yoke`] API on always-owned data.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use yoke::Yoke;
    /// # use std::borrow::Cow;
    /// # use std::rc::Rc;
    ///
    /// let owned: Cow<str> = "hello".to_owned().into();
    /// // this yoke can be intermingled with actually-borrowed Yokes
    /// let yoke: Yoke<Cow<str>, Option<Rc<[u8]>>> = Yoke::new_owned(owned);
    ///
    /// assert_eq!(yoke.get(), "hello");
    /// ```
    pub fn new_owned(yokeable: Y) -> Self {
        Self {
            yokeable,
            cart: None,
        }
    }

    /// Similar to [`Yoke::attach_to_cart()`], except it constructs a `Yoke<Y, Option<C>>`
    /// instead, where the cart is `Some(..)`.
    ///
    /// This allows mixing [`Yoke`]s constructed from owned and borrowed data, when
    /// paired with [`Yoke::new_owned()`].
    ///
    /// This method is currently unusable due to a [compiler bug](https://github.com/rust-lang/rust/issues/84937),
    /// use [`Yoke::attach_to_option_cart_badly()`] instead
    pub fn attach_to_option_cart<F>(cart: C, f: F) -> Self
    where
        F: for<'de> FnOnce(&'de <C as Deref>::Target) -> <Y as Yokeable<'de>>::Output,
    {
        let deserialized = f(cart.deref());
        Self {
            yokeable: unsafe { Y::make(deserialized) },
            cart: Some(cart),
        }
    }
    /// Temporary version of [`Yoke::attach_to_option_cart()`]
    /// that doesn't hit https://github.com/rust-lang/rust/issues/84937
    ///
    /// See its docs for more details
    pub fn attach_to_option_cart_badly(
        cart: C,
        f: for<'de> fn(&'de <C as Deref>::Target) -> <Y as Yokeable<'de>>::Output,
    ) -> Self {
        let deserialized = f(cart.deref());
        Self {
            yokeable: unsafe { Y::make(deserialized) },
            cart: Some(cart),
        }
    }
}

/// This trait marks cart types that do not change source on cloning
///
/// This is conceptually similar to [`stable_deref_trait::CloneStableDeref`],
/// however [`stable_deref_trait::CloneStableDeref`] is not (and should not) be
/// implemented on [`Option`] (since it's not [`Deref`]). [`CloneableCart`] essentially is
/// "if there _is_ data to borrow from here, cloning the cart gives you an additional
/// handle to the same data".
pub unsafe trait CloneableCart: Clone {}

unsafe impl<T: ?Sized> CloneableCart for Rc<T> {}
unsafe impl<T: ?Sized> CloneableCart for Arc<T> {}
unsafe impl<T: CloneableCart> CloneableCart for Option<T> {}
unsafe impl<'a, T: ?Sized> CloneableCart for &'a T {}
unsafe impl CloneableCart for () {}

/// Clone requires that the cart derefs to the same address after it is cloned. This works for Rc, Arc, and &'a T.
/// For all other cart types, clone `.backing_cart()` and re-use `attach_to_cart()`.
impl<Y: for<'a> Yokeable<'a>, C: CloneableCart> Clone for Yoke<Y, C>
where
    for<'a> <Y as Yokeable<'a>>::Output: Clone,
{
    fn clone(&self) -> Self {
        Yoke {
            yokeable: unsafe { Y::make(self.get().clone()) },
            cart: self.cart.clone(),
        }
    }
}

impl<Y: for<'a> Yokeable<'a>, C> Yoke<Y, C> {
    /// Allows one to "project" a yoke to perform a transformation on the data, potentially
    /// looking at a subfield, and producing a new yoke. This will move cart, and the provided
    /// transformation is only allowed to use data known to be borrowed from the cart.
    ///
    /// This takes an additional `PhantomData<&()>` parameter as a workaround to the issue
    /// described in [#86702](https://github.com/rust-lang/rust/issues/86702). This parameter
    /// should just be ignored in the function.
    ///
    /// Furthermore,
    /// [compiler bug #84937](https://github.com/rust-lang/rust/issues/84937) prevents
    /// this from taking a capturing closure, however [`Yoke::project_with_capture()`]
    /// can be used for the same use cases.
    ///
    ///
    /// This can be used, for example, to transform data from one format to another:
    ///
    /// ```rust,ignore
    /// # // This doctest is temporarily ignored because of https://github.com/rust-lang/rust/issues/86703
    /// # use std::rc::Rc;
    /// # use yoke::Yoke;
    /// #
    /// fn slice(y: Yoke<&'static str, Rc<[u8]>>) -> Yoke<&'static [u8], Rc<[u8]>> {
    ///    y.project(move |yk, _| yk.as_bytes())
    /// }
    ///
    /// ```
    ///
    /// This can also be used to create a yoke for a subfield
    ///
    /// ```rust,ignore
    /// # // This doctest is temporarily ignored because of https://github.com/rust-lang/rust/issues/86703
    /// # use std::borrow::Cow;
    /// # use yoke::{Yoke, Yokeable};
    /// # use std::mem;
    /// # use std::rc::Rc;
    /// #
    /// // also safely implements Yokeable<'a>
    /// struct Bar<'a> {
    ///     string_1: &'a str,
    ///     string_2: &'a str,
    /// }
    ///
    /// fn project_string_1(bar: Yoke<Bar<'static>, Rc<[u8]>>) -> Yoke<&'static str, Rc<[u8]>> {
    ///     bar.project(|bar, _| bar.string_1)   
    /// }
    ///
    /// #
    /// # unsafe impl<'a> Yokeable<'a> for Bar<'static> {
    /// #     type Output = Bar<'a>;
    /// #     fn transform(&'a self) -> &'a Bar<'a> {
    /// #         self
    /// #     }
    /// #
    /// #     fn transform_owned(self) -> Bar<'a> {
    /// #         // covariant lifetime cast, can be done safely
    /// #         self
    /// #     }
    /// #
    /// #     unsafe fn make(from: Bar<'a>) -> Self {
    /// #         let ret = mem::transmute_copy(&from);
    /// #         mem::forget(from);
    /// #         ret
    /// #     }
    /// #
    /// #     fn transform_mut<F>(&'a mut self, f: F)
    /// #     where
    /// #         F: 'static + FnOnce(&'a mut Self::Output),
    /// #     {
    /// #         unsafe { f(mem::transmute(self)) }
    /// #     }
    /// # }
    /// ```
    //
    // Safety docs can be found below on `__project_safety_docs()`
    pub fn project<P>(
        self,
        f: for<'a> fn(
            <Y as Yokeable<'a>>::Output,
            PhantomData<&'a ()>,
        ) -> <P as Yokeable<'a>>::Output,
    ) -> Yoke<P, C>
    where
        P: for<'a> Yokeable<'a>,
    {
        let p = f(self.yokeable.transform_owned(), PhantomData);
        Yoke {
            yokeable: unsafe { P::make(p) },
            cart: self.cart,
        }
    }

    /// This is similar to [`Yoke::project`], however it does not move
    /// [`Self`] and instead clones the cart (only if the cart is a [`CloneableCart`])
    ///
    /// This is a bit more efficient than cloning the [`Yoke`] and then calling [`Yoke::project`]
    /// because then it will not clone fields that are going to be discarded.
    pub fn project_cloned<'this, P>(
        &'this self,
        f: for<'a> fn(
            &'this <Y as Yokeable<'a>>::Output,
            PhantomData<&'a ()>,
        ) -> <P as Yokeable<'a>>::Output,
    ) -> Yoke<P, C>
    where
        P: for<'a> Yokeable<'a>,
        C: CloneableCart,
    {
        let p = f(self.get(), PhantomData);
        Yoke {
            yokeable: unsafe { P::make(p) },
            cart: self.cart.clone(),
        }
    }

    /// This is similar to [`Yoke::project`], however it works around it not being able to
    /// use `FnOnce` by using an explicit capture input, until [compiler bug #84937](https://github.com/rust-lang/rust/issues/84937)
    /// is fixed.
    ///
    /// See the docs of [`Yoke::project`] for how this works.
    pub fn project_with_capture<P, T>(
        self,
        capture: T,
        f: for<'a> fn(
            <Y as Yokeable<'a>>::Output,
            capture: T,
            PhantomData<&'a ()>,
        ) -> <P as Yokeable<'a>>::Output,
    ) -> Yoke<P, C>
    where
        P: for<'a> Yokeable<'a>,
    {
        let p = f(self.yokeable.transform_owned(), capture, PhantomData);
        Yoke {
            yokeable: unsafe { P::make(p) },
            cart: self.cart,
        }
    }

    /// This is similar to [`Yoke::project_cloned`], however it works around it not being able to
    /// use `FnOnce` by using an explicit capture input, until [compiler bug #84937](https://github.com/rust-lang/rust/issues/84937)
    /// is fixed.
    ///
    /// See the docs of [`Yoke::project_cloned`] for how this works.
    pub fn project_cloned_with_capture<'this, P, T>(
        &'this self,
        capture: T,
        f: for<'a> fn(
            &'this <Y as Yokeable<'a>>::Output,
            capture: T,
            PhantomData<&'a ()>,
        ) -> <P as Yokeable<'a>>::Output,
    ) -> Yoke<P, C>
    where
        P: for<'a> Yokeable<'a>,
        C: CloneableCart,
    {
        let p = f(self.get(), capture, PhantomData);
        Yoke {
            yokeable: unsafe { P::make(p) },
            cart: self.cart.clone(),
        }
    }
}

/// Safety docs for project()
///
/// (Docs are on a private const to allow the use of compile_fail doctests)
///
/// This is safe to perform because of the choice of lifetimes on `f`, that is,
/// `for<a> fn(<Y as Yokeable<'a>>::Output, &'a ()) -> <P as Yokeable<'a>>::Output`.
///
/// What we want this function to do is take a Yokeable (`Y`) that is borrowing from the cart, and
/// produce another Yokeable (`P`) that also borrows from the same cart. There are a couple potential
/// hazards here:
///
/// - `P` ends up borrowing data from `Y` (or elsewhere) that did _not_ come from the cart,
///   for example `P` could borrow owned data from a `Cow`. This would make the `Yoke<P>` dependent
///   on data owned only by the `Yoke<Y>`.
/// - Borrowed data from `Y` escapes with the wrong lifetime
///
/// Let's walk through these and see how they're prevented.
///
/// ```rust, compile_fail
/// # use std::rc::Rc;
/// # use yoke::Yoke;
/// # use std::borrow::Cow;
/// fn borrow_potentially_owned(y: &Yoke<Cow<'static, str>, Rc<[u8]>>) -> Yoke<&'static str, Rc<[u8]>> {
///    y.project_cloned(|cow, _| &*cow)   
/// }
/// ```
///
/// In this case, the lifetime of `&*cow` is `&'this str`, however the function needs to be able to return
/// `&'a str` _for all `'a`_, which isn't possible.
///
///
/// ```rust, compile_fail
/// # use std::rc::Rc;
/// # use yoke::Yoke;
/// # use std::borrow::Cow;
/// fn borrow_potentially_owned(y: Yoke<Cow<'static, str>, Rc<[u8]>>) -> Yoke<&'static str, Rc<[u8]>> {
///    y.project(|cow, _| &*cow)   
/// }
/// ```
///
/// This has the same issue, `&*cow` is borrowing for a local lifetime.
///
/// Similarly, trying to project an owned field of a struct will produce similar errors:
///
/// ```rust,compile_fail
/// # use std::borrow::Cow;
/// # use yoke::{Yoke, Yokeable};
/// # use std::mem;
/// # use std::rc::Rc;
/// #
/// // also safely implements Yokeable<'a>
/// struct Bar<'a> {
///     owned: String,
///     string_2: &'a str,
/// }
///
/// fn project_owned(bar: &Yoke<Bar<'static>, Rc<[u8]>>) -> Yoke<&'static str, Rc<[u8]>> {
///     // ERROR (but works if you replace owned with string_2)
///     bar.project_cloned(|bar, _| &*bar.owned)   
/// }
///
/// #
/// # unsafe impl<'a> Yokeable<'a> for Bar<'static> {
/// #     type Output = Bar<'a>;
/// #     fn transform(&'a self) -> &'a Bar<'a> {
/// #         self
/// #     }
/// #
/// #     fn transform_owned(self) -> Bar<'a> {
/// #         // covariant lifetime cast, can be done safely
/// #         self
/// #     }
/// #
/// #     unsafe fn make(from: Bar<'a>) -> Self {
/// #         let ret = mem::transmute_copy(&from);
/// #         mem::forget(from);
/// #         ret
/// #     }
/// #
/// #     fn transform_mut<F>(&'a mut self, f: F)
/// #     where
/// #         F: 'static + FnOnce(&'a mut Self::Output),
/// #     {
/// #         unsafe { f(mem::transmute(self)) }
/// #     }
/// # }
/// ```
///
/// Borrowed data from `Y` similarly cannot escape with the wrong lifetime because of the `for<'a>`, since
/// it will never be valid for the borrowed data to escape for all lifetimes of 'a. Internally, `.project()`
/// uses `.get()`, however the signature forces the callers to be able to handle every lifetime.
///
///  `'a` is the only lifetime that matters here; `Yokeable`s must be `'static` and since
/// `Output` is an associated type it can only have one lifetime, `'a` (there's nowhere for it to get another from).
/// `Yoke`s can get additional lifetimes via the cart, and indeed, `project()` can operate on `Yoke<_, &'b [u8]>`,
/// however this lifetime is inaccessible to the closure, and even if it were accessible the `for<'a>` would force
/// it out of the output. All external lifetimes (from other found outside the yoke/closures
/// are similarly constrained here.
///
/// Essentially, safety is achieved by using `for<'a> fn(...)` with `'a` used in both `Yokeable`s to ensure that
/// the output yokeable can _only_ have borrowed data flow in to it from the input. All paths of unsoundness require the
/// unification of an existential and universal lifetime, which isn't possible.
const _: () = ();
