use crate::{core::renderer::Render, prelude::IntoControl};
use godot::{classes::Control, obj::Gd};
use reactive_graph::owner::Owner;

pub struct OwnedControl<T> {
    inner: T,
    #[allow(dead_code)]
    owner: Owner,
}

impl<T> OwnedControl<T> {
    pub fn new_with_owner(control: T, owner: Owner) -> Self
    where
        T: IntoControl,
    {
        OwnedControl {
            inner: control,
            owner,
        }
    }
}

impl<T> Render for OwnedControl<T>
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
