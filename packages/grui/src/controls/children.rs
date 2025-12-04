use crate::renderer::{IntoRender, Render};
use godot::{classes::Control, obj::Gd};

pub trait HasChild<NewChild: IntoRender> {
    type Output;
    fn child(self, child: NewChild) -> Self::Output;
}

impl Render for () {
    fn to_controls(self) -> Vec<Gd<Control>> {
        vec![]
    }

    fn to_json(self) -> String {
        "[]".to_string()
    }
}

impl<A: Render> Render for (A,) {
    fn to_controls(self) -> Vec<Gd<Control>> {
        let (a,) = self;
        a.to_controls()
    }

    fn to_json(self) -> String {
        format!("[{}]", self.0.to_json())
    }
}

// Macro for larger tuples
macro_rules! impl_into_control_for_tuples {
    ($first:ident, $($ty:ident),*) => {
        impl<$first, $($ty),*> Render for ($first, $($ty,)*)
        where
            $first: Render,
            $($ty: Render ),*
        {
          fn to_controls(self) -> Vec<Gd<Control>> {
                #[allow(non_snake_case)]
                let ($first, $($ty,)*) = self;

                let mut controls = Vec::new();
                controls.extend($first.to_controls());
                $(controls.extend($ty.to_controls());)*

                controls
            }

            fn to_json(self) -> String {
                #[allow(non_snake_case)]
                let ($first, $($ty,)*) = self;

                let mut parts = Vec::new();
                parts.push($first.to_json());
                $(parts.push($ty.to_json());)*

                format!("[{}]", parts.join(","))
            }
        }
    };
}

impl_into_control_for_tuples!(A, B);
impl_into_control_for_tuples!(A, B, C);
impl_into_control_for_tuples!(A, B, C, D);
impl_into_control_for_tuples!(A, B, C, D, E);
impl_into_control_for_tuples!(A, B, C, D, E, F);
impl_into_control_for_tuples!(A, B, C, D, E, F, G);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_into_control_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_into_control_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_into_control_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_into_control_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
