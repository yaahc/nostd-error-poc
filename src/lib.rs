#![feature(min_specialization)]
use fakecore::any::provider::Request;
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
    fn provide_context<'a>(&'a self, request: &mut Request<'a>) {
        request
            .provide_ref::<Vec<&'static Location<'static>>>(&self.frames)
            .provide_ref::<[&'static Location<'static>]>(&self.frames);
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

    fn provide_context<'a>(&'a self, request: &mut Request<'a>) {
        request
            .provide_ref::<Vec<&'static Location<'static>>>(&self.frames)
            .provide_ref::<[&'static Location<'static>]>(&self.frames);
    }
}
