#![allow(unused_variables)]
use crate::any::provider::Request;
use core::fmt::{Debug, Display};
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    /// Provides type based access to context intended for error reports
    ///
    /// Used in conjunction with [`context`] to extract references to member variables from `dyn
    /// Error` trait objects.
    ///
    /// # Example
    ///
    /// ```rust
    /// use backtrace::Backtrace;
    /// use core::fmt;
    /// use fakecore::any::Request;
    ///
    /// #[derive(Debug)]
    /// struct Error {
    ///     backtrace: Backtrace,
    /// }
    ///
    /// impl fmt::Display for Error {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "Example Error")
    ///     }
    /// }
    ///
    /// impl fakecore::error::Error for Error {
    ///     fn provide_context<'a>(&'a self, mut request: Pin<&mut Request<'a>>) {
    ///         request.provide::<Backtrace>(&self.backtrace);
    ///     }
    /// }
    ///
    /// let backtrace = Backtrace::new();
    /// let error = Error { backtrace };
    /// let dyn_error = &error as &dyn fakecore::error::Error;
    /// let backtrace_ref = dyn_error.context::<Backtrace>().unwrap();
    ///
    /// assert!(core::ptr::eq(&error.backtrace, backtrace_ref));
    /// ```
    fn provide_context<'a>(&'a self, request: &mut Request<'a>) {}

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl dyn Error {
    pub fn context_ref<T: ?Sized + 'static>(&self) -> Option<&T> {
        use crate::any::tag::Ref;
        crate::any::provider::request::<Ref<T>, _>(|request| self.provide_context(request))
    }

    pub fn context<T: 'static>(&self) -> Option<T> {
        use crate::any::tag::Value;
        crate::any::provider::request::<Value<T>, _>(|request| self.provide_context(request))
    }

    pub fn chain(&self) -> Chain<'_> {
        Chain {
            current: Some(self),
        }
    }
}

pub struct Chain<'a> {
    current: Option<&'a (dyn Error + 'static)>,
}

impl<'a> Iterator for Chain<'a> {
    type Item = &'a (dyn Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        self.current = self.current.and_then(Error::source);
        current
    }
}
