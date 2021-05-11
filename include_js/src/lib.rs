pub use include_js_core::{JSStr, JSString};
pub use include_js_codegen::{JSTemplate, include_js};

#[cfg(feature = "template")]
pub use handlebars::Handlebars as TemplateEngine;
