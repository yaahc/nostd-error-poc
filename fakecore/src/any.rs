use core::any::TypeId;
use core::cell::Cell;
use core::fmt;
use core::marker::PhantomData;
use core::ptr::NonNull;

/// A dynamic request for an object based on its type.
///
/// `'r` is the lifetime of request, and `'out` is the lifetime of the requested
/// reference.
pub struct Request<'r, 'out> {
    buf: NonNull<TypeId>,
    _marker: PhantomData<&'r mut &'out Cell<()>>,
}

impl<'r, 'out> Request<'r, 'out> {
    /// Provides an object of type `T` in response to this request.
    ///
    /// Returns `Err(FulfilledRequest)` if the value was successfully provided,
    /// and `Ok(self)` if `T` was not the type being requested.
    ///
    /// This method can be chained within `provide` implementations using the
    /// `?` operator to concisely provide multiple objects.
    pub fn provide<T: ?Sized + 'static>(self, value: &'out T) -> ProvideResult<'r, 'out> {
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
    pub fn provide_with<T: ?Sized + 'static, F>(mut self, cb: F) -> ProvideResult<'r, 'out>
    where
        F: FnOnce() -> &'out T,
    {
        match self.downcast_buf::<T>() {
            Some(this) => {
                debug_assert!(
                    this.value.is_none(),
                    "Multiple requests to a `RequestBuf` were acquired?"
                );
                this.value = Some(cb());
                Err(FulfilledRequest(PhantomData))
            }
            None => Ok(self),
        }
    }

    /// Get the `TypeId` of the requested type.
    pub fn type_id(&self) -> TypeId {
        unsafe { *self.buf.as_ref() }
    }

    /// Returns `true` if the requested type is the same as `T`
    pub fn is<T: ?Sized + 'static>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    /// Try to downcast this `Request` into a reference to the typed
    /// `RequestBuf` object.
    ///
    /// This method will return `None` if `self` was not derived from a
    /// `RequestBuf<'_, T>`.
    fn downcast_buf<T: ?Sized + 'static>(&mut self) -> Option<&mut RequestBuf<'out, T>> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self.buf.as_ptr() as *mut RequestBuf<'out, T>)) }
        } else {
            None
        }
    }

    /// Calls the provided closure with a request for the the type `T`, returning
    /// `Some(&T)` if the request was fulfilled, and `None` otherwise.
    ///
    /// The `ObjectProviderExt` trait provides helper methods specifically for
    /// types implementing `ObjectProvider`.
    pub fn with<T: ?Sized + 'static, F>(f: F) -> Option<&'out T>
    where
        for<'a> F: FnOnce(Request<'a, 'out>) -> ProvideResult<'a, 'out>,
    {
        let mut buf = RequestBuf {
            type_id: TypeId::of::<T>(),
            value: None,
        };
        let _ = f(Request {
            buf: unsafe {
                NonNull::new_unchecked(&mut buf as *mut RequestBuf<'out, T> as *mut TypeId)
            },
            _marker: PhantomData,
        });
        buf.value
    }
}

impl<'r, 'out> fmt::Debug for Request<'r, 'out> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Request")
            .field("type_id", &self.type_id())
            .finish()
    }
}

// Needs to have a known layout so we can do unsafe pointer shenanigans.
#[repr(C)]
struct RequestBuf<'a, T: ?Sized> {
    type_id: TypeId,
    value: Option<&'a T>,
}

/// Marker type indicating a request has been fulfilled.
pub struct FulfilledRequest(PhantomData<&'static Cell<()>>);

/// Provider method return type.
///
/// Either `Ok(Request)` for an unfulfilled request, or `Err(FulfilledRequest)`
/// if the request was fulfilled.
pub type ProvideResult<'r, 'a> = Result<Request<'r, 'a>, FulfilledRequest>;
