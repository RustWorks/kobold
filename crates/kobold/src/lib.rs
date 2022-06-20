pub use kobold_macros::html;

use wasm_bindgen::JsValue;
use web_sys::Node;

mod render_fn;
mod util;
mod value;

pub mod attribute;
pub mod branch;
pub mod list;
pub mod stateful;

pub use stateful::Stateful;

pub mod prelude {
    pub use crate::{html, Html, IntoHtml, ShouldRender, Stateful};
}

/// Re-exports for the [`html!`](html) macro to use
pub mod reexport {
    pub use wasm_bindgen;
    pub use web_sys;
}

pub enum ShouldRender {
    No,
    Yes,
}

impl From<()> for ShouldRender {
    fn from(_: ()) -> ShouldRender {
        ShouldRender::Yes
    }
}

impl ShouldRender {
    fn should_render(self) -> bool {
        match self {
            ShouldRender::Yes => true,
            ShouldRender::No => false,
        }
    }
}

pub trait Html: Sized {
    type Product: Mountable;

    fn build(self) -> Self::Product;

    fn update(self, p: &mut Self::Product);

    /// This is a no-op method that returns self, you souldn't override the default
    /// implementation. For details see [`IntoHtml`](IntoHtml).
    #[inline]
    fn into_html(self) -> Self {
        self
    }
}

/// Types that cannot implement [`Html`](Html) can instead implement `IntoHtml` and
/// still be usable within the `html!` macro.
///
/// This works as a trait specialization of sorts, allowing for `IntoHtml` to be
/// implemented for iterators without running into potential future conflict with
/// `std` foreign types like `&str`.
pub trait IntoHtml {
    type Html: Html;

    fn into_html(self) -> Self::Html;
}

pub trait Mountable: 'static {
    fn js(&self) -> &JsValue;

    fn mount(&self, parent: &Node) {
        util::__kobold_mount(parent, self.js());
    }

    fn unmount(&self) {
        util::__kobold_unmount(self.js());
    }

    fn mount_replace<M: Mountable>(&self, old: &M) {
        util::__kobold_replace(old.js(), self.js());
    }
}

pub fn start(html: impl Html) {
    use std::cell::Cell;

    thread_local! {
        static INIT: Cell<bool> = Cell::new(false);
    }

    if !INIT.with(|init| init.get()) {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        INIT.with(|init| init.set(true));
    }

    use std::mem::ManuallyDrop;

    let built = ManuallyDrop::new(html.build());

    util::__kobold_start(built.js());
}
