use crate::track::Track;
use core::panic::Location;

pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    pub fn unwrap_err(self) -> E {
        match self {
            Self::Ok(_) => panic!(),
            Self::Err(e) => e,
        }
    }
}

// impl<T, E> core::ops::Try for Result<T, E> {
//     type Ok = T;
//     type Error = E;

//     fn into_result(self) -> core::result::Result<T, Self::Error> {
//         match self {
//             Self::Ok(t) => Ok(t),
//             Self::Err(e) => Err(e),
//         }
//     }

//     fn from_ok(v: T) -> Self {
//         Self::Ok(v)
//     }

//     fn from_error(mut v: Self::Error) -> Self {
//         Self::Err(v)
//     }
// }

impl<T, E> core::ops::Try for Result<T, E>
where
    E: Track,
{
    type Ok = T;
    type Error = E;

    fn into_result(self) -> core::result::Result<T, Self::Error> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(e) => Err(e),
        }
    }

    fn from_ok(v: T) -> Self {
        Self::Ok(v)
    }

    #[track_caller]
    fn from_error(mut v: Self::Error) -> Self {
        v.track(Location::caller());
        Self::Err(v)
    }
}
