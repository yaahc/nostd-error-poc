use crate::any::{ProvideResult, Request};
use core::fmt::{Debug, Display};

pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn get_context<'r, 'a>(&'a self, request: Request<'r, 'a>) -> ProvideResult<'r, 'a> {
        Ok(request)
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl dyn Error {
    pub fn context<T: ?Sized + 'static>(&self) -> Option<&T> {
        Request::with::<T, _>(|req| self.get_context(req))
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
