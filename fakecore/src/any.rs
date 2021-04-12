use core::any::TypeId;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub mod provider {
    //! Tag-based value lookup API for trait objects.
    //!
    //! This provides a similar API to my `object_provider` crate, built on top of
    //! `dyno`

    use super::{Tag, Tagged};

    /// An untyped request for a value of a specific type.
    ///
    /// This type is generally used as an `&mut Request<'a>` outparameter.
    #[repr(transparent)]
    pub struct Request<'a> {
        tagged: dyn Tagged<'a> + 'a,
    }

    impl<'a> Request<'a> {
        /// Helper for performing transmutes as `Request<'a>` has the same layout as
        /// `dyn Tagged<'a> + 'a`, just with a different type!
        ///
        /// This is just to have our own methods on it, and less of the interface
        /// exposed on the `provide` implementation.
        fn wrap_tagged<'b>(t: &'b mut (dyn Tagged<'a> + 'a)) -> &'b mut Self {
            unsafe { &mut *(t as *mut (dyn Tagged<'a> + 'a) as *mut Request<'a>) }
        }

        pub fn is<I>(&self) -> bool
        where
            I: Tag<'a>,
        {
            self.tagged.is::<ReqTag<I>>()
        }

        pub fn provide<I>(&mut self, value: I::Type) -> &mut Self
        where
            I: Tag<'a>,
        {
            if let Some(res @ None) = self.tagged.downcast_mut::<ReqTag<I>>() {
                *res = Some(value);
            }
            self
        }

        pub fn provide_ref<I: ?Sized + 'static>(&mut self, value: &'a I) -> &mut Self
        {
            use crate::any::tag::Ref;
            if let Some(res @ None) = self.tagged.downcast_mut::<ReqTag<Ref<I>>>() {
                *res = Some(value);
            }
            self
        }

        pub fn provide_with<I, F>(&mut self, f: F) -> &mut Self
        where
            I: Tag<'a>,
            F: FnOnce() -> I::Type,
        {
            if let Some(res @ None) = self.tagged.downcast_mut::<ReqTag<I>>() {
                *res = Some(f());
            }
            self
        }
    }

    pub trait Provider {
        fn provide<'a>(&'a self, request: &mut Request<'a>);
    }

    impl dyn Provider {
        pub fn request<'a, I>(&'a self) -> Option<I::Type>
        where
            I: Tag<'a>,
        {
            request::<I, _>(|request| self.provide(request))
        }
    }

    pub fn request<'a, I, F>(f: F) -> Option<<I as Tag<'a>>::Type>
    where
        I: Tag<'a>,
        F: FnOnce(&mut Request<'a>),
    {
        let mut result: Option<<I as Tag<'a>>::Type> = None;
        f(Request::<'a>::wrap_tagged(<dyn Tagged>::tag_mut::<ReqTag<I>>(
            &mut result,
        )));
        result
    }

    /// Implementation detail: Specific `Tag` tag used by the `Request` code under
    /// the hood.
    ///
    /// Composition of `Tag` types!
    struct ReqTag<I>(I);
    impl<'a, I: Tag<'a>> Tag<'a> for ReqTag<I> {
        type Type = Option<I::Type>;
    }
}

pub mod tag {
    //! Simple type-based tag values for use in generic code.
    use super::Tag;
    use core::marker::PhantomData;

    /// Type-based `Tag` for `&'a T` types.
    pub struct Ref<T: ?Sized + 'static>(PhantomData<T>);

    impl<'a, T: ?Sized + 'static> Tag<'a> for Ref<T> {
        type Type = &'a T;
    }

    /// Type-based `Tag` for `&'a mut T` types.
    pub struct RefMut<T: ?Sized + 'static>(PhantomData<T>);

    impl<'a, T: ?Sized + 'static> Tag<'a> for RefMut<T> {
        type Type = &'a mut T;
    }

    /// Type-based `Tag` for concrete types.
    pub struct Value<T: 'static>(PhantomData<T>);

    impl<T: 'static> Tag<'_> for Value<T> {
        type Type = T;
    }
}

/// An identifier which may be used to tag a specific
pub trait Tag<'a>: Sized + 'static {
    /// The type of values which may be tagged by this `Tag`.
    type Type: 'a;
}

mod private {
    pub trait Sealed {}
}

/// Sealed trait representing a type-erased tagged object.
pub unsafe trait Tagged<'a>: private::Sealed + 'a {
    /// The `TypeId` of the `Tag` this value was tagged with.
    fn tag_id(&self) -> TypeId;
}

/// Internal wrapper type with the same representation as a known external type.
#[repr(transparent)]
struct TaggedImpl<'a, I>
where
    I: Tag<'a>,
{
    _value: I::Type,
}

impl<'a, I> private::Sealed for TaggedImpl<'a, I> where I: Tag<'a> {}

unsafe impl<'a, I> Tagged<'a> for TaggedImpl<'a, I>
where
    I: Tag<'a>,
{
    fn tag_id(&self) -> TypeId {
        TypeId::of::<I>()
    }
}

// FIXME: This should also handle the cases for `dyn Tagged<'a> + Send`,
// `dyn Tagged<'a> + Send + Sync` and `dyn Tagged<'a> + Sync`...
//
// Should be easy enough to do it with a macro...
impl<'a> dyn Tagged<'a> {
    /// Tag a reference to a concrete type with a given `Tag`.
    ///
    /// This is like an unsizing coercion, but must be performed explicitly to
    /// specify the specific tag.
    pub fn tag_ref<I>(value: &I::Type) -> &dyn Tagged<'a>
    where
        I: Tag<'a>,
    {
        // SAFETY: `TaggedImpl<'a, I>` has the same representation as `I::Type`
        // due to `#[repr(transparent)]`.
        unsafe { &*(value as *const I::Type as *const TaggedImpl<'a, I>) }
    }

    /// Tag a reference to a concrete type with a given `Tag`.
    ///
    /// This is like an unsizing coercion, but must be performed explicitly to
    /// specify the specific tag.
    pub fn tag_mut<I>(value: &mut I::Type) -> &mut dyn Tagged<'a>
    where
        I: Tag<'a>,
    {
        // SAFETY: `TaggedImpl<'a, I>` has the same representation as `I::Type`
        // due to `#[repr(transparent)]`.
        unsafe { &mut *(value as *mut I::Type as *mut TaggedImpl<'a, I>) }
    }

    /// Tag a Box of a concrete type with a given `Tag`.
    ///
    /// This is like an unsizing coercion, but must be performed explicitly to
    /// specify the specific tag.
    #[cfg(feature = "alloc")]
    pub fn tag_box<I>(value: Box<I::Type>) -> Box<dyn Tagged<'a>>
    where
        I: Tag<'a>,
    {
        // SAFETY: `TaggedImpl<'a, I>` has the same representation as `I::Type`
        // due to `#[repr(transparent)]`.
        unsafe { Box::from_raw(Box::into_raw(value) as *mut TaggedImpl<'a, I>) }
    }

    /// Returns `true` if the dynamic type is tagged with `I`.
    #[inline]
    pub fn is<I>(&self) -> bool
    where
        I: Tag<'a>,
    {
        self.tag_id() == TypeId::of::<I>()
    }

    /// Returns some reference to the dynamic value if it is tagged with `I`,
    /// or `None` if it isn't.
    #[inline]
    pub fn downcast_ref<I>(&self) -> Option<&I::Type>
    where
        I: Tag<'a>,
    {
        if self.is::<I>() {
            // SAFETY: Just checked whether we're pointing to a
            // `TaggedImpl<'a, I>`, which was cast to from an `I::Type`.
            unsafe { Some(&*(self as *const dyn Tagged<'a> as *const I::Type)) }
        } else {
            None
        }
    }

    /// Returns some reference to the dynamic value if it is tagged with `I`,
    /// or `None` if it isn't.
    #[inline]
    pub fn downcast_mut<I>(&mut self) -> Option<&mut I::Type>
    where
        I: Tag<'a>,
    {
        if self.is::<I>() {
            // SAFETY: Just checked whether we're pointing to a
            // `TaggedImpl<'a, I>`, which was cast to from an `I::Type`.
            unsafe { Some(&mut *(self as *mut dyn Tagged<'a> as *mut I::Type)) }
        } else {
            None
        }
    }

    #[inline]
    #[cfg(feature = "alloc")]
    pub fn downcast_box<I>(self: Box<Self>) -> Result<Box<I::Type>, Box<Self>>
    where
        I: Tag<'a>,
    {
        if self.is::<I>() {
            unsafe {
                // SAFETY: Just checked whether we're pointing to a
                // `TaggedImpl<'a, I>`, which was cast to from an `I::Type`.
                let raw: *mut dyn Tagged<'a> = Box::into_raw(self);
                Ok(Box::from_raw(raw as *mut I::Type))
            }
        } else {
            Err(self)
        }
    }
}
