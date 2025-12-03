# grui-macros

Provides Leptos-like procedural macros for building Godot UI components in Rust.

Usage examples

Additional Leptos-parallel patterns:

- signal

```rust
use std::cell::RefCell;
use std::rc::Rc;

// A very naive signal implementation for illustration.
pub struct Signal<T>(Rc<RefCell<T>>);
impl<T> Signal<T> {
    pub fn new(value: T) -> Self { Self(Rc::new(RefCell::new(value))) }
    pub fn get(&self) -> T where T: Clone { self.0.borrow().clone() }
    pub fn set(&self, value: T) { *self.0.borrow_mut() = value; }
}

component! {
    fn Counter(count: Signal<i32>) -> impl grui::node::IntoControl {
        let current = count.get();
        control!(<label>{format!("Count: {}", current)}</label>)
    }
}
```

- For (iterating to build children)

```rust
let items = vec!["Alpha", "Beta", "Gamma"];
let list = control!(
    <vbox>{
        items.into_iter().map(|name| {
            control!(<label>{name}</label>)
        }).collect::<Vec<_>>()
    }</vbox>
);
```

- Setting Godot attributes directly

```rust
// Assuming `button_color` maps to a Godot exported property.
control!(<button button_color="red">Click</button>);
```

- Connecting Godot signals (using 0.4.0 crate API)

```rust
use godot::prelude::Callable;
let on_pressed = Callable::from_fn(|_| { println!("Pressed"); });
control!(<button on:pressed=on_pressed>Run</button>);
```

- Conditional rendering using functions

```rust
fn maybe_label(show: bool, text: &str) -> impl grui::node::IntoControl {
    if show { control!(<label>{text}</label>) } else { grui::node::Node::empty() }
}

let content = control!(<vbox>{ maybe_label(true, "Visible") }</vbox>);
```

- Effect::new (side-effect after creation)

```rust
struct Effect<F: Fn()>(F);
impl<F: Fn()> Effect<F> { fn new(f: F) -> Self { f(); Self(f) } }

Effect::new(|| println!("Effect ran after build"));

let ui = control!(<label>{"Ready"}</label>);
```


- component

```rust
use grui_macros::component;

component! {
    fn Button(label: String, disabled: bool) -> grui::node::Node {
        control!(<button>{label}</button>)
    }
}
```

This macro transforms a function `fn Button(label: String)` into:

- a `ButtonProps` struct with `label` and `children` fields
- a function `fn Button(props: ButtonProps)` which destructures `props` into local variables

- control!

```rust
control!(
    <button on:pressed=on_pressed>{label}</button>
)
```

The `control!` macro lets you write a JSX-like template that compiles to Godot control construction code. Attributes like `on:pressed` attach callbacks.

Tests

See `packages/grui-macros/src/*.rs` for many examples which the test-suite asserts exactly against generated tokenstreams.

License: MIT