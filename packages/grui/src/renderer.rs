use crate::control::IntoControl;
use godot::{classes::Control, obj::Gd};

pub struct Renderer {
    root: Gd<Control>,
}

impl Renderer {
    pub fn mount<P, C>(parent: Gd<Control>, component: C, props: P) -> Self
    where
        C: FnOnce(P) -> impl IntoControl,
    {
        let control = component(props).into_control();
        parent.add_child(control.get_gd());
        Renderer {
            root: control.get_gd(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grui_macros::{component, control};

    #[component]
    fn MyComp(a: u32, b: String) -> impl IntoControl {
        return control! {
          <label>{format!("a: {}, b: {}", a, b)}</label>
        };
    }

    #[test]
    fn it_works() {
        let props = MyCompProps {
            a: 42,
            b: "dauphin".to_string(),
        };
        let renderer = Renderer::mount(parent, MyComp, props);
    }
}
