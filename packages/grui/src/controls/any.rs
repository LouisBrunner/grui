use crate::core::renderer::Render;
use erased::ErasedBox;

fn check(id_1: &std::any::TypeId, id_2: &std::any::TypeId) {
    if id_1 != id_2 {
        panic!("Erased: type mismatch")
    }
}

pub struct Erased {
    type_id: std::any::TypeId,
    value: Option<ErasedBox>,
    drop: fn(ErasedBox),
}

impl Erased {
    pub fn new<T: 'static>(item: T) -> Self {
        Self {
            type_id: std::any::TypeId::of::<T>(),
            value: Some(ErasedBox::new(Box::new(item))),
            drop: |value| {
                let _ = unsafe { value.into_inner::<T>() };
            },
        }
    }

    pub fn into_inner<T: 'static>(mut self) -> T {
        check(&self.type_id, &std::any::TypeId::of::<T>());
        *unsafe { self.value.take().unwrap().into_inner::<T>() }
    }
}

impl Drop for Erased {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            (self.drop)(value);
        }
    }
}

pub struct AnyControl {
    value: Erased,
    to_controls: fn(Erased) -> Vec<godot::prelude::Gd<godot::classes::Control>>,
    to_json: fn(Erased) -> String,
}

impl Render for AnyControl {
    fn to_controls(self) -> Vec<godot::prelude::Gd<godot::classes::Control>> {
        (self.to_controls)(self.value)
    }

    fn to_json(self) -> String {
        (self.to_json)(self.value)
    }
}

pub trait IntoAny {
    fn into_any(self) -> AnyControl;
}

impl<T> IntoAny for T
where
    T: Render + 'static,
{
    fn into_any(self) -> AnyControl {
        fn to_controls<T: Render + 'static>(
            value: Erased,
        ) -> Vec<godot::prelude::Gd<godot::classes::Control>> {
            value.into_inner::<T>().to_controls()
        }

        fn to_json<T: Render + 'static>(value: Erased) -> String {
            value.into_inner::<T>().to_json()
        }

        AnyControl {
            value: Erased::new(self),
            to_controls: to_controls::<T>,
            to_json: to_json::<T>,
        }
    }
}
