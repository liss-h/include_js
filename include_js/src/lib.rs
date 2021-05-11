pub use include_js_core::{JSStr, JSString, JSTemplate};
pub use include_js_codegen::include_js;

#[cfg(feature = "template")]
pub use handlebars::Handlebars as TemplateEngine;

#[cfg(feature = "template")]
pub use include_js_codegen::JSTemplate;
