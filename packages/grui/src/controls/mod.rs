mod builtin;
mod children;
mod empty;
mod fragment;
mod std;

use crate::renderer::Render;
pub(crate) use builtin::BuiltinControl;
pub use children::HasChild;
pub use empty::empty;
pub use fragment::fragment;
pub use std::CollectControl;

use godot::{classes::Control, obj::Gd};

pub trait IntoControl: Sized + Send + Render {
    fn into_control(self) -> GControl<Self>;
}

impl<T> IntoControl for T
where
    T: Sized + Send + Render,
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
