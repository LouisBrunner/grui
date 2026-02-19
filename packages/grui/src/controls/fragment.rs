use crate::{
    core::renderer::Render,
    prelude::{visitors::ChildrenGatherer, IntoRender},
};
use frunk::{hlist::HList, HCons, HNil};
use godot::{classes::Control, obj::Gd};

pub struct Fragment<Ch> {
    children: Ch,
}

pub fn fragment() -> Fragment<HNil> {
    Fragment { children: HNil }
}

impl<Ch> Render for Fragment<Ch>
where
    Ch: HList + ChildrenGatherer,
{
    fn to_controls(self) -> Vec<Gd<Control>> {
        self.children.gather_controls()
    }

    fn to_json(self) -> String {
        let parts = self.children.gather_json();
        format!("{}", parts.join(", "))
    }
}

impl<Ch> Fragment<Ch>
where
    Ch: HList,
{
    pub fn child<NewChild>(self, child: NewChild) -> Fragment<HCons<NewChild::Output, Ch>>
    where
        NewChild: IntoRender,
    {
        Fragment {
            children: HCons {
                head: child.into_render(),
                tail: self.children,
            },
        }
    }
}
