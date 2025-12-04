mod builtin;
mod empty;
mod std;
pub mod tuples;

use crate::renderer::Render;
pub(crate) use builtin::BuiltinControl;
pub use empty::empty;
use godot::{classes::Control, obj::Gd};
pub use std::CollectControl;
pub use tuples::CompatibleFn;

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
    fn to_controls(self) -> Vec<Gd<Control>> {
        self.inner.to_controls()
    }

    fn to_json(self) -> String {
        self.inner.to_json()
    }
}
