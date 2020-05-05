use core::any::TypeId;
use core::cell::Cell;
use core::fmt;
use core::marker::PhantomData;

/// A dynamic request for an object based on its type.
///
/// `'out` is the lifetime of the requested reference.
#[repr(transparent)]
pub struct Request<'out>(RequestBuf<PhantomData<&'out Cell<()>>>);
// FIXME: The argument of the RequestBuf should be a thin unsized type,
// but `extern type` is impossible to use correctly right now
// (it cannot be placed at offset > 0, and it cannot be placed inside a union).
// Since miri doesn't complain we'll let it slide.

impl<'out> Request<'out> {
    /// Provides an object of type `T` in response to this request.
    ///
    /// Returns `Err(FulfilledRequest)` if the value was successfully provided,
    /// and `Ok(self)` if `T` was not the type being requested.
    ///
    /// This method can be chained within `provide` implementations using the
    /// `?` operator to concisely provide multiple objects.
    pub fn provide<T: ?Sized + 'static>(&mut self, value: &'out T) -> &mut Self {
        self.provide_with(|| value)
    }

    /// Lazily provides an object of type `T` in response to this request.
    ///
    /// Returns `Err(FulfilledRequest)` if the value was successfully provided,
    /// and `Ok(self)` if `T` was not the type being requested.
    ///
    /// The passed closure is only called if the value will be successfully
    /// provided.
    ///
    /// This method can be chained within `provide` implementations using the
    /// `?` operator to concisely provide multiple objects.
    pub fn provide_with<T: ?Sized + 'static, F>(&mut self, cb: F) -> &mut Self
    where
        F: FnOnce() -> &'out T,
    {
        if self.is::<T>() {
            let this = unsafe { &mut *(self as *mut _ as *mut RequestBuf<Option<&'out T>>) };
            debug_assert!(
                this.value.is_none(),
                "Multiple requests to a `RequestBuf` were acquired?"
            );
            this.value = Some(cb());
        }
        self
    }

    /// Get the `TypeId` of the requested type.
    pub fn type_id(&self) -> TypeId {
        self.0.type_id
    }

    /// Returns `true` if the requested type is the same as `T`
    pub fn is<T: ?Sized + 'static>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    /// Calls the provided closure with a request for the the type `T`, returning
    /// `Some(&T)` if the request was fulfilled, and `None` otherwise.
    ///
    /// The `ObjectProviderExt` trait provides helper methods specifically for
    /// types implementing `ObjectProvider`.
    pub fn with<T: ?Sized + 'static, F>(f: F) -> Option<&'out T>
    where
        F: FnOnce(&mut Self),
    {
        let mut buf = RequestBuf {
            type_id: TypeId::of::<T>(),
            value: None,
        };
        unsafe {
            let request = &mut *(&mut buf as *mut _ as *mut Request);
            f(request);
        }
        buf.value
    }
}

impl<'out> fmt::Debug for Request<'out> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Request")
            .field("type_id", &self.type_id())
            .finish()
    }
}

// Needs to have a known layout so we can do unsafe pointer shenanigans.
#[repr(C)]
struct RequestBuf<T: ?Sized> {
    type_id: TypeId,
    value: T,
}
