mod block;
mod item;
mod enchant;

macro_rules! generate {
    ($data:ident, $i:ident, $( ($property:ident, $q:expr $(, $value:ident, $e:expr)?) ),+) => {
        $(let mut $property = proc_macro2::TokenStream::new();)+

        for $i in $data {
            use convert_case::{Case, Casing};

            $($(let $value = $e;)?)+

            $($q.to_tokens(&mut $property);)+
        }
    };
}

pub(crate) use generate;

#[proc_macro]
pub fn block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    block::block(input)
}

#[proc_macro]
pub fn item(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    item::item(input)
}

#[proc_macro]
pub fn enchant(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enchant::enchant(input)
}
