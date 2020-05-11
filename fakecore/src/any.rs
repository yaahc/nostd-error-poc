use core::any::TypeId;
use core::fmt;
use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;

/// A dynamic request for an object based on its type.
#[repr(C)]
pub struct Request<'a> {
    type_id: TypeId,
    _pinned: PhantomPinned,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Request<'a> {
    /// Provides an object of type `T` in response to this request.
    ///
    /// If an object of type `T` has already been provided for this request, the
    /// existing value will be replaced by the newly provided value.
    ///
    /// This method can be chained within `provide` implementations to concisely
    /// provide multiple objects.
    pub fn provide<T: ?Sized + 'static>(self: Pin<&mut Self>, value: &'a T) -> Pin<&mut Self> {
        self.provide_with(|| value)
    }

    /// Lazily provides an object of type `T` in response to this request.
    ///
    /// If an object of type `T` has already been provided for this request, the
    /// existing value will be replaced by the newly provided value.
    ///
    /// The passed closure is only called if the value will be successfully
    /// provided.
    ///
    /// This method can be chained within `provide` implementations to concisely
    /// provide multiple objects.
    pub fn provide_with<T: ?Sized + 'static, F>(mut self: Pin<&mut Self>, cb: F) -> Pin<&mut Self>
    where
        F: FnOnce() -> &'a T,
    {
        if let Some(buf) = self.as_mut().downcast_buf::<T>() {
            // NOTE: We could've already provided a value here of type `T`,
            // which will be clobbered in this case.
            *buf = Some(cb());
        }
        self
    }

    /// Get the `TypeId` of the requested type.
    fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns `true` if the requested type is the same as `T`
    pub fn is<T: ?Sized + 'static>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    /// Try to downcast this `Request` into a reference to the typed
    /// `RequestBuf` object, and access the trailing `Option<&'a T>`.
    ///
    /// This method will return `None` if `self` is not the prefix of a
    /// `RequestBuf<'_, T>`.
    fn downcast_buf<T: ?Sized + 'static>(self: Pin<&mut Self>) -> Option<&mut Option<&'a T>> {
        if self.is::<T>() {
            // Safety: `self` is pinned, meaning it exists as the first
            // field within our `RequestBuf`. As the type matches, and
            // `RequestBuf` has a known in-memory layout, this downcast is
            // sound.
            unsafe {
                let ptr = self.get_unchecked_mut() as *mut Self as *mut RequestBuf<'a, T>;
                Some(&mut (*ptr).value)
            }
        } else {
            None
        }
    }

    /// Calls the provided closure with a request for the the type `T`, returning
    /// `Some(&T)` if the request was fulfilled, and `None` otherwise.
    pub fn with<T: ?Sized + 'static, F>(f: F) -> Option<&'a T>
    where
        F: FnOnce(Pin<&mut Self>),
    {
        let mut buf = RequestBuf::new();
        // safety: We never move `buf` after creating `pinned`.
        let mut pinned = unsafe { Pin::new_unchecked(&mut buf) };
        f(pinned.as_mut().request());
        pinned.take()
    }
}

impl<'a> fmt::Debug for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Request")
            .field("type_id", &self.type_id())
            .finish()
    }
}

// Needs to have a known layout so we can do unsafe pointer shenanigans.
#[repr(C)]
struct RequestBuf<'a, T: ?Sized + 'static> {
    request: Request<'a>,
    value: Option<&'a T>,
}

impl<'a, T: ?Sized + 'static> RequestBuf<'a, T> {
    fn new() -> Self {
        RequestBuf {
            request: Request {
                type_id: TypeId::of::<T>(),
                _pinned: PhantomPinned,
                _marker: PhantomData,
            },
            value: None,
        }
    }

    fn request(self: Pin<&mut Self>) -> Pin<&mut Request<'a>> {
        // safety: projecting Pin onto our `request` field.
        unsafe { self.map_unchecked_mut(|this| &mut this.request) }
    }

    fn take(self: Pin<&mut Self>) -> Option<&'a T> {
        // safety: `Option<&'a T>` is `Unpin`
        unsafe { self.get_unchecked_mut().value.take() }
    }
}
