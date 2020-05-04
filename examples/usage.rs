#![feature(error_iter)]
use core::panic::Location;
use etf::{ExampleError, ExampleWrappingError};
use fakecore::error::Error;
use fakecore::result::Result::{self, *};

fn main() {
    let e = four().unwrap_err();
    report(&e);
}

fn one() -> Result<(), ExampleError> {
    Err(ExampleError::default())
}

fn two() -> Result<(), ExampleError> {
    Ok(one()?)
}

fn three() -> Result<(), ExampleWrappingError> {
    Ok(two()?)
}

fn four() -> Result<(), ExampleWrappingError> {
    Ok(three()?)
}

pub fn report(error: &(dyn Error + 'static)) {
    let mut ind = 0;
    let mut source = Some(error);

    while let Some(error) = source {
        println!("{}: {}", ind, error);

        if let Some(locs) = error.context::<Vec<&'static Location<'static>>>() {
            for loc in locs.iter() {
                println!("  @ {}", loc);
            }
        }

        ind += 1;
        source = error.source();
    }

    let locations = error
        .chain()
        .filter_map(|e| e.context::<[&'static Location<'static>]>())
        .flat_map(|locs| locs.iter());

    println!("\nFull Return Trace:");
    for (i, loc) in locations.enumerate() {
        println!("    {}: {}", i, loc);
    }
}
