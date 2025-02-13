#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Control<T>
where
    T: Sized,
{
    inner: T,
}

impl<T> Control<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Unwraps the view, returning the inner type.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

pub trait Render: Sized {
    type State: Sized; // TODO: unclear

    fn build(self) -> Self::State;

    fn rebuild(self, state: &mut Self::State);
}

pub trait IntoControl
where
    Self: Sized + Render + Send,
{
    fn into_control(self) -> Control<Self>;
}

impl<T> IntoControl for T
where
    T: Sized + Render + Send,
{
    fn into_control(self) -> Control<Self> {
        Control { inner: self }
    }
}
