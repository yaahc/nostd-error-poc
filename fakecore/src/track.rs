use core::panic::Location;

pub trait Track {
    #[allow(unused_variables)]
    fn track(&mut self, location: &'static Location<'static>) {}
}

default impl<T> Track for T {}
