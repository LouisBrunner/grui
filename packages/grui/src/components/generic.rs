use godot::{classes::StyleBoxFlat, prelude::*};
use godot_grui::prelude::*;

use crate::prelude::any::AnyControl;

#[derive(Debug)]
pub enum SubDisplay {
    Linear,
    Flow,
    Split,
}

#[derive(Debug)]
pub enum Display {
    Vertical(SubDisplay),
    Horizontal(SubDisplay),
    Center,
}

impl Display {
    fn wrap(&self, children: AnyControl) -> AnyControl {
        match self {
            Display::Vertical(sub_display) => match sub_display {
                SubDisplay::Linear => control! {
                    <v_box_container>
                        {children}
                    </v_box_container>
                }
                .into_any(),
                SubDisplay::Flow => control! {
                    <v_flow_container>
                        {children}
                    </v_flow_container>
                }
                .into_any(),
                SubDisplay::Split => control! {
                    <v_split_container>
                        {children}
                    </v_split_container>
                }
                .into_any(),
            },
            Display::Horizontal(sub_display) => match sub_display {
                SubDisplay::Linear => control! {
                    <h_box_container>
                        {children}
                    </h_box_container>
                }
                .into_any(),
                SubDisplay::Flow => control! {
                    <h_flow_container>
                        {children}
                    </h_flow_container>
                }
                .into_any(),
                SubDisplay::Split => control! {
                    <h_split_container>
                        {children}
                    </h_split_container>
                }
                .into_any(),
            },
            Display::Center => control! {
              <center_container>
                {children}
              </center_container>
            }
            .into_any(),
        }
    }
}

#[component]
pub fn Generic(
    children: ChildrenFn,
    #[prop(optional)] margin: f32,
    #[prop(optional)] margin_x: f32,
    #[prop(optional)] margin_y: f32,
    #[prop(optional)] margin_left: f32,
    #[prop(optional)] margin_top: f32,
    #[prop(optional)] margin_right: f32,
    #[prop(optional)] margin_bottom: f32,
    #[prop(optional)] padding: f32,
    #[prop(optional)] padding_x: f32,
    #[prop(optional)] padding_y: f32,
    #[prop(optional)] padding_left: f32,
    #[prop(optional)] padding_top: f32,
    #[prop(optional)] padding_right: f32,
    #[prop(optional)] padding_bottom: f32,
    #[prop(optional)] background: Color,
    #[prop(default=Display::Vertical(SubDisplay::Linear))] display: Display,
) -> impl IntoControl {
    let mut result = display.wrap(children());
    if padding.is_some()
        || padding_x.is_some()
        || padding_y.is_some()
        || padding_left.is_some()
        || padding_top.is_some()
        || padding_right.is_some()
        || padding_bottom.is_some()
    {
        result = control! {
          <margin_container
            theme_override_constants:margin_left=padding_left.or(padding_x).or(padding).unwrap_or(0.0)
            theme_override_constants:margin_top=padding_top.or(padding_y).or(padding).unwrap_or(0.0)
            theme_override_constants:margin_right=padding_right.or(padding_x).or(padding).unwrap_or(0.0)
            theme_override_constants:margin_bottom=padding_bottom.or(padding_y).or(padding).unwrap_or(0.0)
          >
            {result}
          </margin_container>
        }
        .into_any();
    }
    if let Some(color) = background {
        let mut sbox = StyleBoxFlat::new_gd();
        sbox.set_bg_color(color);
        result = control! {
          <panel
            theme_override_styles:panel={sbox.clone()}
            />
          {result}
        }
        .into_any();
    };
    // if margin.is_some()
    //     || margin_left.is_some()
    //     || margin_top.is_some()
    //     || margin_right.is_some()
    //     || margin_bottom.is_some()
    // {
    result = control! {
      <margin_container
        anchor_right=1.0
        anchor_bottom=1.0
        theme_override_constants:margin_left=margin_left.or(margin_x).or(margin).unwrap_or(0.0)
        theme_override_constants:margin_top=margin_top.or(margin_y).or(margin).unwrap_or(0.0)
        theme_override_constants:margin_right=margin_right.or(margin_x).or(margin).unwrap_or(0.0)
        theme_override_constants:margin_bottom=margin_bottom.or(margin_y).or(margin).unwrap_or(0.0)
      >
        {result}
      </margin_container>
    }
    .into_any();
    // }

    control! {
      {result}
    }
}
