use syn::{LitStr, Type};

pub fn enchant(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    let path = input.path;
    let _repr = input.repr;
    let path = format!("{}/{}", std::env::var("CARGO_MANIFEST_DIR").unwrap(), path);
    let _file = std::fs::File::open(path).unwrap();

    todo!()
}

struct Input {
    pub path: String,
    pub repr: Type,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: (input.parse() as syn::Result<LitStr>)?.value(),
            repr: input.parse()?,
        })
    }
}
