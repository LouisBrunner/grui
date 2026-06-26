#[cfg(feature = "testing")]
#[cfg(test)]
mod tests {
    use godot_grui::{prelude::*, testing::*};
    use pretty_assertions::assert_eq;
    use std::sync::{atomic::AtomicUsize, Arc};

    #[component]
    fn Simple(a: u32, #[prop(into)] b: String) -> impl IntoControl {
        control! {
          <label text=format!("a: {}, b: {}", a, b) />
        }
    }

    #[test]
    fn with_simple() {
        TestRenderer::mount(
            || control! { <Simple a=42 b="dauphin" /> },
            |renderer| {
                assert_eq!(
                    renderer
                        .get_root()
                        .snapshot()
                        .expect("snapshot to be correct"),
                    r#"{"type":"Root","children":[{"type":"Label","props":{"text":"\"a: 42, b: dauphin\""}}]}"#
                );
            },
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
        let called_times = Arc::new(AtomicUsize::new(0));
        TestRenderer::mount(
            || {
                let resume = {
                    let called_times = called_times.clone();
                    SignalCallable::new(move |_| {
                        called_times.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    })
                };
                control! { <Builtins resume=resume /> }
            },
            |renderer| {
                assert_eq!(
                    renderer
                        .get_root()
                        .snapshot()
                        .expect("snapshot to be correct"),
                    r#"{"type":"Root","children":[{"type":"Panel"},{"type":"VBoxContainer","children":[{"type":"Button","props":{"text":"\"Resume\""},"signals":["click"]},{"type":"Button","props":{"text":"\"Save\""}},{"type":"Button","props":{"text":"\"Load\""}}]}]}"#
                );

                assert_eq!(called_times.load(std::sync::atomic::Ordering::SeqCst), 0);

                renderer
                    .get_root()
                    .select_by_indices("1.0")
                    .expect("to find resume")
                    .emit_signal("click", &[]);

                assert_eq!(called_times.load(std::sync::atomic::Ordering::SeqCst), 1);
            },
        );
    }

    #[component]
    fn Reactive() -> impl IntoControl {
        let (count, set_count) = signal(0);
        let increase = SignalCallable::new(move |_| {
            set_count.update(|prev| *prev += 1);
        });
        control! {
          <label text=format!("{}", count.get()) />
          <button on:click=increase text="+" />
        }
    }

    #[tokio::test]
    #[test_log::test]
    async fn with_reactive() {
        TestRenderer::mount_async(|| control! { <Reactive /> }, |renderer| async move {
            assert_eq!(
                renderer
                    .get_root()
                    .snapshot()
                    .expect("snapshot to be correct"),
                r#"{"type":"Root","children":[{"type":"Label","props":{"text":"\"0\""}},{"type":"Button","props":{"text":"\"+\""},"signals":["click"]}]}"#
            );
            renderer
                .get_root()
                .select_by_indices("1")
                .expect("to find button")
                .emit_signal("click", &[]);

            wait_for_async_changes();

            assert_eq!(
                renderer
                    .get_root()
                    .snapshot()
                    .expect("snapshot to be correct"),
                r#"{"type":"Root","children":[{"type":"Label","props":{"text":"\"1\""}},{"type":"Button","props":{"text":"\"+\""},"signals":["click"]}]}"#
            );
        }).await;
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
        TestRenderer::mount(
            || control! { <StaticIter /> },
            |renderer| {
                assert_eq!(
                    renderer
                        .get_root()
                        .snapshot()
                        .expect("snapshot to be correct"),
                    r#"{"type":"Root","children":[{"type":"VBoxContainer","children":[{"type":"Label","props":{"text":"\"Item 1\""}},{"type":"Label","props":{"text":"\"Item 2\""}},{"type":"Label","props":{"text":"\"Item 3\""}},{"type":"Label","props":{"text":"\"Item 4\""}},{"type":"Label","props":{"text":"\"Item 5\""}},{"type":"Label","props":{"text":"\"Item 6\""}},{"type":"Label","props":{"text":"\"Item 7\""}},{"type":"Label","props":{"text":"\"Item 8\""}},{"type":"Label","props":{"text":"\"Item 9\""}},{"type":"Label","props":{"text":"\"Item 10\""}}]}]}"#
                );
            },
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
        TestRenderer::mount(
            || control! { <Custom label="Custom Label" /> },
            |renderer| {
                assert_eq!(
                    renderer
                        .get_root()
                        .snapshot()
                        .expect("snapshot to be correct"),
                    r#"{"type":"Root","children":[{"type":"Panel","children":[{"type":"Label","props":{"text":"\"Custom Label\""}},{"type":"Label","props":{"text":"\"a: 10, b: hello\""}}]}]}"#
                );
            },
        );
    }
}
