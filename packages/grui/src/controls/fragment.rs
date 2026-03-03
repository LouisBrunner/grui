use super::{any::AnyState, children::ChildrenGatherer};
use crate::core::render::{IntoRender, Render, TestSnapshot};
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

    fn build(self) -> Self::State {
        self.children.gather().build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.children.gather().rebuild(state);
    }

    fn get_test_snapshot(&self) -> TestSnapshot {
        let parts: Vec<TestSnapshot> = self
            .children
            .gather()
            .iter()
            .enumerate()
            .map(|(i, child)| child.get_test_snapshot().prefix_action(&i.to_string()))
            .collect();
        TestSnapshot {
            json: format!(
                "{}",
                parts
                    .iter()
                    .map(|s| s.json.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            actions: TestSnapshot::new().merge_actions(parts).actions,
        }
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
