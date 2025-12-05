#[cfg(test)]
mod tests {
    use godot::prelude::*;
    use grui::{prelude::*, renderer::TestRenderer};
    use pretty_assertions::assert_eq;

    #[component]
    fn Simple(a: u32, b: String) -> impl IntoControl {
        control! {
          <label text=format!("a: {}, b: {}", a, b) />
        }
    }

    #[test]
    fn with_simple() {
        let props = SimpleProps {
            a: 42,
            b: "dauphin".to_string(),
        };
        let renderer = TestRenderer::mount(Simple, props);
        assert_eq!(
            renderer.snapshot(),
            r#"[{"type": "Label", "props": {"text": "a: 42, b: dauphin"}}]"#
        );
    }

    #[component]
    fn Builtins<F>(resume: F) -> impl IntoControl
    where
        F: CompatibleFn,
    {
        control! {
            <>
                <panel />
                <vboxcontainer>
                    <button on:click=resume text="Resume" />
                    <button text="Save" />
                    <>
                      <button text="Load" />
                    </>
                </vboxcontainer>
            </>
        }
    }

    #[test]
    fn with_builtins() {
        let props = BuiltinsProps {
            resume: move |_| {
                godot_print!("Resumed!");
                Ok(Variant::nil())
            },
        };
        let renderer = TestRenderer::mount(Builtins, props);
        assert_eq!(
            renderer.snapshot(),
            r#"[{"type": "Panel"}, {"type": "VBoxContainer", "children": [{"type": "Button", "props": {"text": "Resume"}, "signals": ["click"]}, {"type": "Button", "props": {"text": "Save"}}, {"type": "Button", "props": {"text": "Load"}}]}]"#
        );
    }

    #[component]
    fn StaticIter() -> impl IntoControl {
        let title = "Item";
        control! {
          <vboxcontainer>
              {
                (1..=10).map(|i| {
                    control! {
                        <label text=format!("{} {}", title, i) />
                    }
                }).collect_control()
              }
          </vboxcontainer>
        }
    }

    #[test]
    fn with_static_iter() {
        let props = StaticIterProps {};
        let renderer = TestRenderer::mount(StaticIter, props);
        assert_eq!(
            renderer.snapshot(),
            r#"[{"type": "VBoxContainer", "children": [{"type": "Label", "props": {"text": "Item 1"}}, {"type": "Label", "props": {"text": "Item 2"}}, {"type": "Label", "props": {"text": "Item 3"}}, {"type": "Label", "props": {"text": "Item 4"}}, {"type": "Label", "props": {"text": "Item 5"}}, {"type": "Label", "props": {"text": "Item 6"}}, {"type": "Label", "props": {"text": "Item 7"}}, {"type": "Label", "props": {"text": "Item 8"}}, {"type": "Label", "props": {"text": "Item 9"}}, {"type": "Label", "props": {"text": "Item 10"}}]}]"#
        );
    }

    #[component]
    fn Custom(label: String) -> impl IntoControl {
        control! {
          <panel>
            <>
              <label text=label />
              <Simple a=10 b="hello".to_string() />
            </>
          </panel>
        }
    }

    #[test]
    fn with_custom() {
        let props = CustomProps {
            label: "Custom Label".to_string(),
        };
        let renderer = TestRenderer::mount(Custom, props);
        assert_eq!(
            renderer.snapshot(),
            r#"[{"type": "Panel", "children": [{"type": "Label", "props": {"text": "Custom Label"}}, {"type": "Label", "props": {"text": "a: 10, b: hello"}}]}]"#
        );
    }
}
