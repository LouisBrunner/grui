pub(crate) mod any;
pub(crate) mod builtin;
pub(crate) mod children;
mod fragment;
pub(crate) mod functions;
pub(crate) mod owned;
pub(crate) mod props;
pub(crate) mod signals;
mod std;

use crate::core::render::Render;
pub use any::IntoAny;
pub use children::{ChildrenFn, ToChildren};
pub use fragment::fragment;
pub use functions::ControlFn;
pub use signals::{CompatibleFn, SignalCallable};
pub use std::CollectControl;

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
