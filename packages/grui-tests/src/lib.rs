#[cfg(test)]
mod tests {
    use godot::prelude::*;
    use grui::{prelude::*, renderer::TestRenderer};

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
            r#"{"type":"label","props":{"text":"a: 42, b: dauphin"}}"#
        );
    }

    #[component]
    fn Builtins(resume: Callable) -> impl IntoControl {
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
            resume: Callable::invalid(),
        };
        let renderer = TestRenderer::mount(Builtins, props);
        assert_eq!(renderer.snapshot(), r#"{}"#);
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
                }).collect::<Vec<_>>()
              }
          </vboxcontainer>
        }
    }

    #[test]
    fn with_static_iter() {
        let props = StaticIterProps {};
        let renderer = TestRenderer::mount(StaticIter, props);
        assert_eq!(renderer.snapshot(), r#"{}"#);
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
        assert_eq!(renderer.snapshot(), r#"{}"#);
    }
}
