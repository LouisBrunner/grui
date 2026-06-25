use crate::core::render::{BuildOptions, MountPlace, Mountable, Render};
use erased::ErasedBox;
use std::any::TypeId;

fn check(id_1: &std::any::TypeId, id_2: &std::any::TypeId) {
    if id_1 != id_2 {
        panic!("Erased: type mismatch")
    }
}

pub(crate) struct Erased {
    type_id: std::any::TypeId,
    value: Option<ErasedBox>,
    drop: fn(ErasedBox),
}

impl Erased {
    pub(crate) fn new<T: 'static>(item: T) -> Self {
        Self {
            type_id: std::any::TypeId::of::<T>(),
            value: Some(ErasedBox::new(Box::new(item))),
            drop: |value| {
                let _ = unsafe { value.into_inner::<T>() };
            },
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_ref<T: 'static>(&self) -> &T {
        check(&self.type_id, &std::any::TypeId::of::<T>());
        unsafe { self.value.as_ref().unwrap().get_ref::<T>() }
    }

    pub(crate) fn get_mut<T: 'static>(&mut self) -> &mut T {
        check(&self.type_id, &std::any::TypeId::of::<T>());
        unsafe { self.value.as_mut().unwrap().get_mut::<T>() }
    }

    pub(crate) fn into_inner<T: 'static>(mut self) -> T {
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
    build: fn(Erased, &BuildOptions) -> AnyState,
    rebuild: fn(Erased, &mut AnyState, &BuildOptions),
}

impl Render for AnyControl {
    type State = AnyState;

    fn build(self, opts: &BuildOptions) -> Self::State {
        (self.build)(self.value, opts)
    }

    fn rebuild(self, state: &mut Self::State, opts: &BuildOptions) {
        if self.type_id == state.type_id {
            (self.rebuild)(self.value, state, opts)
        } else {
            let mut new = self.build(opts);
            state.mount_after(&mut new);
            state.unmount();
            *state = new;
        }
    }
}

pub struct AnyState {
    type_id: TypeId,
    state: Erased,
    mount: fn(&mut Erased, place: MountPlace),
    mount_after: fn(&mut Erased, &mut dyn Mountable),
    unmount: fn(&mut Erased),
}

impl AnyState {
    pub(crate) fn new<T>(state: T::State) -> Self
    where
        T: Render + 'static,
    {
        fn mount_any<T>(state: &mut Erased, place: MountPlace)
        where
            T: Render,
            T::State: 'static,
        {
            state.get_mut::<T::State>().mount(place)
        }

        fn mount_after_any<T>(state: &mut Erased, sibling: &mut dyn Mountable)
        where
            T: Render,
            T::State: 'static,
        {
            state.get_mut::<T::State>().mount_after(sibling);
        }

        fn unmount_any<T>(state: &mut Erased)
        where
            T: Render,
            T::State: 'static,
        {
            state.get_mut::<T::State>().unmount();
        }

        AnyState {
            type_id: TypeId::of::<T>(),
            state: Erased::new(state),
            mount: mount_any::<T>,
            mount_after: mount_after_any::<T>,
            unmount: unmount_any::<T>,
        }
    }
}

impl Mountable for AnyState {
    fn mount(&mut self, place: MountPlace) {
        (self.mount)(&mut self.state, place);
    }

    fn mount_after(&mut self, sibling: &mut dyn Mountable) {
        (self.mount_after)(&mut self.state, sibling);
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
        fn build<T: Render + 'static>(value: Erased, opts: &BuildOptions) -> AnyState {
            AnyState::new::<T>(value.into_inner::<T>().build(opts))
        }

        fn rebuild<T: Render + 'static>(value: Erased, state: &mut AnyState, opts: &BuildOptions) {
            let state = state.state.get_mut::<<T as Render>::State>();
            value.into_inner::<T>().rebuild(state, opts)
        }

        AnyControl {
            type_id: TypeId::of::<T>(),
            value: Erased::new(self),
            build: build::<T>,
            rebuild: rebuild::<T>,
        }
    }
}
