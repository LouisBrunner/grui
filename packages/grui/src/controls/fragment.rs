use crate::{
    controls::{HasChild, IntoControl},
    renderer::{IntoRender, Render},
};
use godot::{classes::Control, obj::Gd};
use next_tuple::NextTuple;

pub(crate) struct FragmentControl<Ch> {
    children: Ch,
}

pub fn fragment() -> impl IntoControl {
    FragmentControl { children: () }
}

impl Render for FragmentControl<()> {
    fn to_controls(self) -> Vec<Gd<Control>> {
        Vec::new()
    }

    fn to_json(self) -> String {
        "".to_string()
    }
}

impl<Ch, NewChild> HasChild<NewChild> for FragmentControl<Ch>
where
    Ch: NextTuple + Render,
    <Ch as NextTuple>::Output<NewChild>: Render,

    NewChild: IntoRender,
    NewChild::Output: Render,
{
    type Output = FragmentControl<<Ch as NextTuple>::Output<NewChild::Output>>;

    fn child(self, child: NewChild) -> Self::Output {
        FragmentControl {
            children: self.children.next_tuple(child.into_render()),
        }
    }
}
