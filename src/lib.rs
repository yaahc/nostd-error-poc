#![feature(min_specialization)]
#![feature(track_caller)]
use fakecore::any::{ProvideResult, Request};
use fakecore::error::Error;
use fakecore::track::Track;
use std::fmt;
use std::panic::Location;

#[derive(Debug)]
pub struct ExampleError {
    frames: Vec<&'static Location<'static>>,
}

impl Default for ExampleError {
    #[track_caller]
    fn default() -> Self {
        Self {
            frames: vec![Location::caller()],
        }
    }
}

impl Track for ExampleError {
    fn track(&mut self, location: &'static Location<'static>) {
        self.frames.push(location);
    }
}

impl fmt::Display for ExampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExampleError")
    }
}

impl Error for ExampleError {
    fn provide_context<'r, 'a>(&'a self, request: Request<'r, 'a>) -> ProvideResult<'r, 'a> {
        request
            .provide::<Vec<&'static Location<'static>>>(&self.frames)?
            .provide::<[&'static Location<'static>]>(&self.frames)
    }
}

#[derive(Debug)]
pub struct ExampleWrappingError {
    frames: Vec<&'static Location<'static>>,
    source: ExampleError,
}

impl From<ExampleError> for ExampleWrappingError {
    fn from(source: ExampleError) -> Self {
        Self {
            frames: vec![],
            source,
        }
    }
}

impl Track for ExampleWrappingError {
    fn track(&mut self, location: &'static Location<'static>) {
        self.frames.push(location);
    }
}

impl fmt::Display for ExampleWrappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExampleWrappingError")
    }
}

impl Error for ExampleWrappingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }

    fn provide_context<'r, 'a>(&'a self, request: Request<'r, 'a>) -> ProvideResult<'r, 'a> {
        request
            .provide::<Vec<&'static Location<'static>>>(&self.frames)?
            .provide::<[&'static Location<'static>]>(&self.frames)
    }
}
