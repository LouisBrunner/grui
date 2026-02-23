mod any;
mod builtin;
mod empty;
mod fragment;
mod functions;
mod owned;
mod std;
pub mod visitors;

use crate::core::renderer::Render;
pub use any::{AnyControl, IntoAny};
pub(crate) use builtin::Builtin;
pub use empty::empty;
pub use fragment::fragment;
pub use functions::*;
use godot::{classes::Control, obj::Gd};
pub use owned::OwnedControl;
pub use std::CollectControl;
pub use visitors::*;

pub trait IntoControl: Sized + Render {
    fn into_control(self) -> GControl<Self>;
}

impl<T> IntoControl for T
where
    T: Sized + Render,
{
    fn into_control(self) -> GControl<Self> {
        GControl { inner: self }
    }
}

pub struct GControl<T> {
    inner: T,
}

impl<T> GControl<T> {
    pub fn new(inner: T) -> Self {
        GControl { inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Render for GControl<T>
where
    T: Render,
{
    fn mount(self, parent: Gd<Control>) {
        self.inner.mount(parent);
    }

    fn to_json(self) -> String {
        self.inner.to_json()
    }
}
