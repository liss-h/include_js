use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, LitStr};

#[cfg(feature = "template")]
use handlebars::Handlebars;

#[cfg(feature = "template")]
mod template;

fn read_to_string_relative(rel_path: &Path) -> String {
    let crate_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = PathBuf::from(crate_root).join(rel_path);

    if !path.exists() {
        panic!("'{}' does not exist", path.display());
    }

    std::fs::read_to_string(path).expect("could not read file")
}

/// Simmilar to `include_str!` from the stdlib but instead of including arbitrary files as `&str`
/// it includes sytactically valid javascript from files as `&JSStr`. If the file contains invalid
/// Javascript you will get a compiletime error.
/// 
/// **Note:** The path must be relative to $CARGO_MANIFEST_DIR.
///
/// # Examples
/// 
/// ```no_run
/// use include_js::{JSStr, include_js};
///
/// const JS: &JSStr = include_js!("src/js/some_script.js");
/// ```
#[proc_macro]
pub fn include_js(item: TokenStream) -> TokenStream {
    let input_path = parse_macro_input!(item as LitStr).value();

    let content = read_to_string_relative(Path::new(&input_path));
    let _ = boa::parse(&content, false).expect("syntax error");

    TokenStream::from(quote! {
        unsafe { JSStr::new_unchecked(#content) }
    })
}

/// Derives the `JSTemplate` trait for a struct with named fields.
/// This is simmilar to plain `include_js!` with the difference that
/// the Javascript is not yet fully filled in, so a template engine (in this case `Handlebars`)
/// to fill in the values at runtime.
///
/// **Note:** Currently the only supported attribute is `#[include_js(template = "SOME/PATH")]` and it is
/// required to specify it. The capabilities may be expanded in the future.
/// 
/// **Warning:** The ability of this macro to actually prove that the file contains valid Javascript once filled
/// in is kind of limited. It assumes that you will only fill-in expressions via the template engine; so to be able to
/// atleast do some kind of check it will use `[]` as a placeholder for every expression.
/// I might add the ability to disable the compiletime check or to enable an optional runtime check at some point, but this is
/// not implemented yet.
/// 
/// # Examples
///
/// Let this be your JS template script.
/// 
/// `src/js/move_window.js.handlebars`
/// ```javascript
/// let w = global
//      .get_window_actors()
///     .map(a => a.meta_window)
///     .filter(w => w.wm_class == "{{window_class}}")
///     .reduce((acc, x) => (acc && acc.id > x.id) ? acc : x, null);
///
/// w.move_resize_frame(true, {{x}}, {{y}}, {{width}}, {{height}});
/// ```
///
/// You can then do the following. 
///
/// ```no_run
/// use include_js::{JSString, JSTemplate};
///
/// #[derive(JSTemplate)]
/// #[include_js(template = "src/js/move_window.js.handlebars")]
/// struct MoveWindowCommand {
///     x: u32,
///     y: u32,
///     width: u32,
///     height: u32,
///     window_class: String,
/// }
///
/// let js: JSString = MoveWindowCommand { 
///     x: 0,
///     y: 5,
///     width: 100,
///     height: 200,
///     window_class: "org.gnome.Nautilus".to_owned(),
/// }.render_template();
/// 
/// let expected = r#"
/// let w = global
//      .get_window_actors()
///     .map(a => a.meta_window)
///     .filter(w => w.wm_class == "org.gnome.Nautilus")
///     .reduce((acc, x) => (acc && acc.id > x.id) ? acc : x, null);
///
/// w.move_resize_frame(true, 0, 5, 100, 200);
/// "#;
///
/// assert_eq!(expected, &*js);
/// ```
#[cfg(feature = "template")]
#[proc_macro_derive(JSTemplate, attributes(include_js))]
pub fn derive_js_template(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let template_path = {
        let template_attr = template::get_attr(&input);
        let template = template_attr.parse_args::<template::TemplatePathInput>().unwrap();

        template.path.value()
    };

    let struct_name = &input.ident;
    let content = read_to_string_relative(Path::new(&template_path));

    let data: HashMap<String, [(); 0]> = {
        let field_names = match &input.data {
            Data::Struct(ds) => template::struct_field_names(&ds),
            _ => panic!("only structs supported"),
        };

        field_names.into_iter().zip(std::iter::repeat([])).collect()
    };

    let expanded = {
        let mut h = Handlebars::new();
        h.set_strict_mode(true);
        h.render_template(&content, &data)
            .expect("error rendering template")
    };
    let _ = boa::parse(&expanded, false).expect("syntax error");

    TokenStream::from(quote! {
        impl JSTemplate for #struct_name {
            fn render_template(&self) -> ::include_js::JSString {                
                let mut h = ::include_js::TemplateEngine::new();
                h.set_strict_mode(true);
                let s = h.render_template(#content, self).unwrap();
                
                // safety: in the macro invocation it was made sure that the resulting string is js
                unsafe {
                    ::include_js::JSString::new_unchecked(s)
                }
            }
        }
    })
}
