pub(crate) mod any;
pub(crate) mod builtin;
pub(crate) mod children;
mod fragment;
mod functions;
pub(crate) mod owned;
pub(crate) mod props;
pub(crate) mod signals;
mod std;

use crate::core::render::Render;
// pub use any::{AnyControl, IntoAny};
// pub(crate) use builtin::Builtin;
// pub use fragment::fragment;
// pub use functions::*;
// pub use owned::OwnedControl;
// pub use std::CollectControl;

pub trait IntoControl: Sized + Render {
    fn into_control(self) -> Self;
}

impl<T> IntoControl for T
where
    T: Sized + Render,
{
    fn into_control(self) -> Self {
        self
    }
}

// pub struct GControl<T> {
//     inner: T,
// }

// impl<T> GControl<T> {
//     pub fn new(inner: T) -> Self {
//         GControl { inner }
//     }
// }

// impl<T> Render for GControl<T>
// where
//     T: Render,
// {
//     type State = T::State;

//     fn build(self) -> Self::State {
//         self.inner.build()
//     }

//     fn rebuild(self, state: &mut Self::State) {
//         self.inner.rebuild(state);
//     }

//     fn to_json(self) -> String {
//         self.inner.to_json()
//     }
// }
