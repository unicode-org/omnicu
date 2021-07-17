// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::Yoke;
use crate::Yokeable;
#[cfg(feature = "alloc")]
use alloc::borrow::{Cow, ToOwned};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::rc::Rc;
#[cfg(feature = "alloc")]
use alloc::string::String;

/// Trait for types that can be crated from a reference to a cart type `C` with no allocations.
///
/// A type can be the `ZeroCopyFrom` target of multiple cart types.
///
/// The intention is for `ZeroCopyFrom` to produce a struct from a cart with as little work as
/// possible. Although it is technically possible to implement `ZeroCopyFrom` without being
/// zero-copy (using heap allocations), doing so defeats the purpose of `ZeroCopyFrom`.
///
/// For example, `impl ZeroCopyFrom<C> for Cow<str>` should return a `Cow::Borrowed` pointing at
/// data in the cart `C`, even if the cart is itself fully owned.
///
/// # Examples
///
/// Implementing `ZeroCopyFrom` on a custom data struct:
///
/// ```
/// use yoke::Yokeable;
/// use yoke::ZeroCopyFrom;
/// use std::borrow::Cow;
///
/// struct MyStruct<'s> {
///     message: Cow<'s, str>,
/// }
///
/// unsafe impl<'a> Yokeable<'a> for MyStruct<'static> {
///     // (not shown; see `Yokeable` for examples)
/// #    type Output = MyStruct<'a>;
/// #    fn transform(&'a self) -> &'a Self::Output {
/// #        self
/// #    }
/// #    fn transform_owned(self) -> MyStruct<'a> {
/// #        // covariant lifetime cast, can be done safely
/// #        self
/// #    }
/// #    unsafe fn make(from: Self::Output) -> Self {
/// #        std::mem::transmute(from)
/// #    }
/// #    fn transform_mut<F>(&'a mut self, f: F)
/// #    where
/// #        F: 'static + for<'b> FnOnce(&'b mut Self::Output),
/// #    {
/// #        unsafe {
/// #            f(std::mem::transmute::<&'a mut Self, &'a mut Self::Output>(
/// #                self,
/// #            ))
/// #        }
/// #    }
/// }
///
/// // Reference from a borrowed version of self
/// impl<'s> ZeroCopyFrom<MyStruct<'s>> for MyStruct<'static> {
///     fn zero_copy_from<'b>(cart: &'b MyStruct<'s>) -> MyStruct<'b> {
///         MyStruct {
///             message: Cow::Borrowed(&cart.message)
///         }
///     }
/// }
///
/// // Reference from a string slice directly
/// impl ZeroCopyFrom<str> for MyStruct<'static> {
///     fn zero_copy_from<'b>(cart: &'b str) -> MyStruct<'b> {
///         MyStruct {
///             message: Cow::Borrowed(cart)
///         }
///     }
/// }
/// ```
pub trait ZeroCopyFrom<C: ?Sized>: for<'a> Yokeable<'a> {
    /// Clone the cart `C` into a [`Yokeable`] struct, which may retain references into `C`.
    fn zero_copy_from<'b>(cart: &'b C) -> <Self as Yokeable<'b>>::Output;
}

impl<'b, 's, Y: ZeroCopyFrom<C> + for<'a> Yokeable<'a>, C: ?Sized> Yoke<Y, &'b C> {
    /// Construct a [`Yoke`]`<Y, &C>` from a borrowed cart by zero-copy cloning the cart to `Y` and
    /// then yokeing that object to the cart.
    ///
    /// This results in a [`Yoke`] bound to the lifetime of the reference to the borrowed cart.
    ///
    /// The type `Y` must implement [`ZeroCopyFrom`]`<C>`.
    ///
    /// # Example
    ///
    /// ```
    /// use yoke::Yoke;
    /// use std::borrow::Cow;
    ///  
    /// let yoke = Yoke::<
    ///     Cow<'static, str>,
    ///     &str
    /// >::attach_to_borrowed_cart("demo");
    ///
    /// assert_eq!("demo", yoke.get());
    /// ```
    pub fn attach_to_borrowed_cart(cart: &'b C) -> Self {
        Yoke::<Y, &'b C>::attach_to_cart_badly(cart, Y::zero_copy_from)
    }
}

#[cfg(feature = "alloc")]
impl<'b, 's, Y: ZeroCopyFrom<C> + for<'a> Yokeable<'a>, C: ?Sized> Yoke<Y, Box<C>> {
    /// Construct a [`Yoke`]`<Y, Box<C>>` from a boxed cart by zero-copy cloning the cart to `Y` and
    /// then yokeing that object to the cart.
    ///
    /// This results in a [`Yoke`] bound to the lifetime of data within the cart. If the cart is
    /// fully owned, then the resulting [`Yoke`] will be `'static`.
    ///
    /// The type `Y` must implement [`ZeroCopyFrom`]`<C>`.
    ///
    /// # Example
    ///
    /// ```
    /// use yoke::Yoke;
    /// use std::borrow::Cow;
    ///
    /// let box_cart = Box::new("demo".to_string());
    ///  
    /// let yoke = Yoke::<
    ///     Cow<'static, str>,
    ///     Box<String>
    /// >::attach_to_box_cart(box_cart);
    ///
    /// assert_eq!("demo", yoke.get());
    /// ```
    pub fn attach_to_box_cart(cart: Box<C>) -> Self {
        Yoke::<Y, Box<C>>::attach_to_cart_badly(cart, Y::zero_copy_from)
    }
}

#[cfg(feature = "alloc")]
impl<'b, 's, Y: ZeroCopyFrom<C> + for<'a> Yokeable<'a>, C: ?Sized> Yoke<Y, Rc<C>> {
    /// Construct a [`Yoke`]`<Y, Rc<C>>` from a reference-counted cart by zero-copy cloning the
    /// cart to `Y` and then yokeing that object to the cart.
    ///
    /// This results in a [`Yoke`] bound to the lifetime of data within the cart. If the cart is
    /// fully owned, then the resulting [`Yoke`] will be `'static`.
    ///
    /// The type `Y` must implement [`ZeroCopyFrom`]`<C>`.
    ///
    /// # Example
    ///
    /// ```
    /// use yoke::Yoke;
    /// use std::borrow::Cow;
    /// use std::rc::Rc;
    ///
    /// let rc_cart = Rc::from("demo".to_string());
    ///  
    /// let yoke = Yoke::<
    ///     Cow<'static, str>,
    ///     Rc<String>
    /// >::attach_to_rc_cart(rc_cart);
    ///
    /// assert_eq!("demo", yoke.get());
    /// ```
    pub fn attach_to_rc_cart(cart: Rc<C>) -> Self {
        Yoke::<Y, Rc<C>>::attach_to_cart_badly(cart, Y::zero_copy_from)
    }
}

// Note: The following could be blanket implementations, but that would require constraining the
// blanket `T` on `T: 'static`, which may not be desirable for all downstream users who may wish
// to customize their `ZeroCopyFrom` impl. The blanket implementation may be safe once Rust has
// specialization.

#[cfg(feature = "alloc")]
impl ZeroCopyFrom<str> for Cow<'static, str> {
    fn zero_copy_from<'b>(cart: &'b str) -> Cow<'b, str> {
        Cow::Borrowed(cart)
    }
}

#[cfg(feature = "alloc")]
impl ZeroCopyFrom<String> for Cow<'static, str> {
    fn zero_copy_from<'b>(cart: &'b String) -> Cow<'b, str> {
        Cow::Borrowed(cart)
    }
}

impl ZeroCopyFrom<str> for &'static str {
    fn zero_copy_from<'b>(cart: &'b str) -> &'b str {
        cart
    }
}

impl ZeroCopyFrom<String> for &'static str {
    fn zero_copy_from<'b>(cart: &'b String) -> &'b str {
        cart
    }
}

impl<C, T: ZeroCopyFrom<C>> ZeroCopyFrom<Option<C>> for Option<T> {
    fn zero_copy_from<'b>(cart: &'b Option<C>) -> Option<<T as Yokeable<'b>>::Output> {
        cart.as_ref()
            .map(|c| <T as ZeroCopyFrom<C>>::zero_copy_from(c))
    }
}

impl<C1, T1: ZeroCopyFrom<C1>, C2, T2: ZeroCopyFrom<C2>> ZeroCopyFrom<(C1, C2)> for (T1, T2) {
    fn zero_copy_from<'b>(
        cart: &'b (C1, C2),
    ) -> (<T1 as Yokeable<'b>>::Output, <T2 as Yokeable<'b>>::Output) {
        (
            <T1 as ZeroCopyFrom<C1>>::zero_copy_from(&cart.0),
            <T2 as ZeroCopyFrom<C2>>::zero_copy_from(&cart.1),
        )
    }
}

// These duplicate the functionality from above and aren't quite necessary due
// to deref coercions, however for the custom derive to work, there always needs
// to be `impl ZCF<T> for T`, otherwise it may fail to perform the necessary
// type inference. Deref coercions do not typically work when sufficient generics
// or inference are involved, and the proc macro does not necessarily have
// enough type information to figure this out on its own.
#[cfg(feature = "alloc")]
impl<B: ToOwned + ?Sized + 'static> ZeroCopyFrom<Cow<'_, B>> for Cow<'static, B> {
    fn zero_copy_from<'b>(cart: &'b Cow<'_, B>) -> Cow<'b, B> {
        Cow::Borrowed(cart)
    }
}

impl ZeroCopyFrom<&'_ str> for &'static str {
    fn zero_copy_from<'b>(cart: &'b &'_ str) -> &'b str {
        cart
    }
}
