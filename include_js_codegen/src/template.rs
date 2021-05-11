use syn::{AttrStyle, Attribute, DataStruct, DeriveInput, Fields, Ident, LitStr, Token, parse::Parse};

mod kw {
    syn::custom_keyword!(template);
}

pub(super) struct TemplatePathInput {
    pub(super) attr_name: kw::template,
    pub(super) eq: Token![=],
    pub(super) path: LitStr,
}

impl Parse for TemplatePathInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr_name: kw::template = input.parse()?;
        let eq: Token![=] = input.parse()?;
        let path: LitStr = input.parse()?;
        Ok(TemplatePathInput { attr_name, eq, path })
    }
}

pub(super) fn struct_field_names(st: &DataStruct) -> Vec<String> {
    let fields = match &st.fields {
        Fields::Named(fields) => fields,
        _ => panic!("only normal struct supported"),
    };

    fields
        .named
        .iter()
        .map(|f| format!("{}", f.ident.as_ref().unwrap()))
        .collect()
}

pub(super) fn get_attr(input: &DeriveInput) -> Attribute {
    input
        .attrs
        .iter()
        .filter(|a| matches!(a.style, AttrStyle::Outer))
        .find(|a| {
            a.path
                .get_ident()
                .map(|id| id == &Ident::new("include_js", id.span()))
                .unwrap_or(false)
        })
        .expect("missing template path specification")
        .clone()
}
