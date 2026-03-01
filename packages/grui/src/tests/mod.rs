#[cfg(test)]
mod tests {
    use godot::prelude::*;
    use grui::{prelude::*, TestRenderer};
    use pretty_assertions::assert_eq;

    #[component]
    fn Simple(a: u32, #[prop(into)] b: String) -> impl IntoControl {
        control! {
          <label text=format!("a: {}, b: {}", a, b) />
        }
    }

    #[test]
    fn with_simple() {
        let renderer = TestRenderer::mount(control! { <Simple a=42 b="dauphin" /> });
        assert_eq!(
            renderer.snapshot(),
            r#"[{"type": "Label", "props": {"text": "a: 42, b: dauphin"}}]"#
        );
    }

    #[component]
    fn Builtins(resume: SignalCallable) -> impl IntoControl {
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
        let resume = SignalCallable::new(|_| {
            godot_print!("Resumed!");
        });
        let renderer = TestRenderer::mount(control! { <Builtins resume=resume /> });
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
        let renderer = TestRenderer::mount(control! { <StaticIter /> });
        assert_eq!(
            renderer.snapshot(),
            r#"[{"type": "VBoxContainer", "children": [{"type": "Label", "props": {"text": "Item 1"}}, {"type": "Label", "props": {"text": "Item 2"}}, {"type": "Label", "props": {"text": "Item 3"}}, {"type": "Label", "props": {"text": "Item 4"}}, {"type": "Label", "props": {"text": "Item 5"}}, {"type": "Label", "props": {"text": "Item 6"}}, {"type": "Label", "props": {"text": "Item 7"}}, {"type": "Label", "props": {"text": "Item 8"}}, {"type": "Label", "props": {"text": "Item 9"}}, {"type": "Label", "props": {"text": "Item 10"}}]}]"#
        );
    }

    #[component]
    fn Custom(#[prop(into)] label: String) -> impl IntoControl {
        control! {
          <panel>
            <>
              <label text=label.clone() />
              <Simple a=10 b="hello" />
            </>
          </panel>
        }
    }

    #[test]
    fn with_custom() {
        let renderer = TestRenderer::mount(control! { <Custom label="Custom Label" /> });
        assert_eq!(
            renderer.snapshot(),
            r#"[{"type": "Panel", "children": [{"type": "Label", "props": {"text": "Custom Label"}}, {"type": "Label", "props": {"text": "a: 10, b: hello"}}]}]"#
        );
    }
}
