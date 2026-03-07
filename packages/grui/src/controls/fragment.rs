use super::{any::AnyState, children::ChildrenGatherer};
use crate::core::render::{BuildOptions, IntoRender, Render};
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
    type State = Vec<AnyState>;

    fn build(self, opts: &BuildOptions) -> Self::State {
        self.children.gather().build(opts)
    }

    fn rebuild(self, state: &mut Self::State, opts: &BuildOptions) {
        self.children.gather().rebuild(state, opts);
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
