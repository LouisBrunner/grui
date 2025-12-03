use godot::{classes::Control, obj::Gd};

pub trait IntoControl
where
    Self: Sized + Send,
{
    fn to_controls(self) -> Vec<Gd<Control>>;
    fn to_json(self) -> String;
}

pub(crate) struct BuiltinControl;

impl IntoControl for BuiltinControl {
    fn to_controls(self) -> Vec<Gd<Control>> {
        panic!("TODO: finish")
    }

    fn to_json(self) -> String {
        panic!("TODO: finish")
    }
}

struct EmptyControl;

impl IntoControl for EmptyControl {
    fn to_controls(self) -> Vec<Gd<Control>> {
        Vec::new()
    }

    fn to_json(self) -> String {
        "null".to_string()
    }
}

pub fn empty() -> impl IntoControl {
    EmptyControl {}
}

struct FragmentControl<C: IntoControl> {
    pub children: Vec<C>,
}

impl<C: IntoControl> IntoControl for FragmentControl<C> {
    fn to_controls(self) -> Vec<Gd<Control>> {
        let mut controls = Vec::with_capacity(self.children.len());
        for child in self.children {
            controls.extend(child.to_controls());
        }
        controls
    }

    fn to_json(self) -> String {
        let children_json: Vec<String> = self.children.into_iter().map(|c| c.to_json()).collect();
        format!("[{}]", children_json.join(","))
    }
}

pub fn fragment<C: IntoControl>(children: Vec<C>) -> impl IntoControl {
    FragmentControl { children }
}

impl<C: IntoControl> IntoControl for Vec<C> {
    fn to_controls(self) -> Vec<Gd<Control>> {
        FragmentControl { children: self }.to_controls()
    }

    fn to_json(self) -> String {
        FragmentControl { children: self }.to_json()
    }
}
