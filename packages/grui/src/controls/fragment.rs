use super::children::ChildrenGatherer;
use crate::core::{render::Render, IntoRender};
use frunk::{hlist::HList, HCons, HNil};

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
    type State = Ch::State;

    fn build(self) -> Self::State {
        self.children.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.children.rebuild(state);
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
