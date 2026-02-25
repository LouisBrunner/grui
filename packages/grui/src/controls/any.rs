use std::any::TypeId;

use crate::core::render::{Mountable, Render};
use erased::ErasedBox;
use godot::{classes::Control, obj::Gd};

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

    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        check(&self.type_id, &std::any::TypeId::of::<T>());
        unsafe { self.value.as_mut().unwrap().get_mut::<T>() }
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
    type_id: TypeId,
    value: Erased,
    build: fn(Erased) -> AnyState,
    rebuild: fn(Erased, &mut AnyState),
    to_json: fn(Erased) -> String,
}

impl Render for AnyControl {
    type State = AnyState;

    fn build(self) -> Self::State {
        (self.build)(self.value)
    }

    fn rebuild(self, state: &mut Self::State) {
        (self.rebuild)(self.value, state)
    }

    fn to_json(self) -> String {
        (self.to_json)(self.value)
    }
}

pub struct AnyState {
    type_id: TypeId,
    state: Erased,
    mount: fn(&mut Erased, parent: &Gd<Control>),
    unmount: fn(&mut Erased),
}

impl Mountable for AnyState {
    fn mount(&mut self, parent: &Gd<Control>) {
        (self.mount)(&mut self.state, parent);
    }

    fn unmount(&mut self) {
        (self.unmount)(&mut self.state);
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
        fn mount_any<T>(state: &mut Erased, parent: &Gd<Control>)
        where
            T: Render,
            T::State: 'static,
        {
            state.get_mut::<T::State>().mount(parent)
        }

        fn unmount_any<T>(state: &mut Erased)
        where
            T: Render,
            T::State: 'static,
        {
            state.get_mut::<T::State>().unmount();
        }

        fn build<T: Render + 'static>(value: Erased) -> AnyState {
            let state = Erased::new(value.into_inner::<T>().build());
            AnyState {
                type_id: TypeId::of::<T>(),
                state,
                mount: mount_any::<T>,
                unmount: unmount_any::<T>,
            }
        }

        fn rebuild<T: Render + 'static>(value: Erased, state: &mut AnyState) {
            let state = state.state.get_mut::<<T as Render>::State>();
            value.into_inner::<T>().rebuild(state)
        }

        fn to_json<T: Render + 'static>(value: Erased) -> String {
            value.into_inner::<T>().to_json()
        }

        AnyControl {
            type_id: TypeId::of::<T>(),
            value: Erased::new(self),
            build: build::<T>,
            rebuild: rebuild::<T>,
            to_json: to_json::<T>,
        }
    }
}
