use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use serde_derive::{Serialize, Deserialize};
use syn::{Type, LitStr, LitInt};

pub fn item(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    let path = input.path;
    let repr = input.repr;
    let path = format!("{}/{}", std::env::var("CARGO_MANIFEST_DIR").unwrap(), path);
    let file = std::fs::File::open(path).unwrap();
    
    let data: Vec<Item> = serde_json::from_reader(file).unwrap();

    crate::generate!(
        data,
        item,
        (
            item_enum,
            quote!(
                #ident = #id,
            ),
            ident,
            Ident::new(&item.name.to_case(Case::Pascal), Span::call_site())
        ),
        (
            item_from_id,
            quote!(
                #id => Some(Self::#ident),
            ),
            id,
            LitInt::new(&item.id.to_string(), Span::call_site())
        ),
        (
            item_name,
            quote!(
                Self::#ident => #name,
            ),
            name,
            LitStr::new(&item.name, Span::call_site())
        ),
        (
            item_from_name,
            quote!(
                #name => Some(Self::#ident),
            )
        ),
        (
            item_display_name,
            quote!(
                Self::#ident => #display_name,
            ),
            display_name,
            LitStr::new(&item.display_name, Span::call_site())
        ),
        (
            item_stack_size,
            quote!(
                Self::#ident => #stack_size,
            ),
            stack_size,
            LitInt::new(&item.stack_size.to_string(), Span::call_site())
        ),
        (
            item_max_durability,
            quote!(
                Self::#ident => #max_durability,
            ),
            max_durability,
            {
                if let Some(max_durability) = item.max_durability {
                    quote!(Some(#max_durability))
                } else {
                    quote!(None)
                }
            }
        )
    );
    
    quote!(
        #[derive(Debug)]
        #[repr(#repr)]
        pub enum Item {
            #item_enum
        }

        impl ::miners_data::inv::item::Item for Item {
            fn id(self) -> u16 {
                self as #repr as u16
            }

            fn from_id(id: u16) -> Option<Self> {
                match id {
                    #item_from_id
                    _ => None,
                }
            }

            fn name(self) -> &'static str {
                match self {
                    #item_name
                }
            }

            fn from_name(name: &str) -> Option<Self> {
                match name {
                    #item_from_name
                    _ => None,
                }
            }

            fn display_name(self) -> &'static str {
                match self {
                    #item_display_name
                }
            }

            fn stack_size(self) -> u8 {
                match self {
                    #item_stack_size
                }
            }

            fn max_durability(self) -> Option<u16> {
                match self {
                    #item_max_durability
                }
            }
        }

    ).into_token_stream().into()
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    pub id: i64,
    pub display_name: String,
    pub name: String,
    pub stack_size: i64,
    pub max_durability: Option<u16>,
    #[serde(default)]
    pub enchant_categories: Vec<String>,
    #[serde(default)]
    pub repair_with: Vec<String>,
    #[serde(default)]
    pub variations: Option<Vec<Variation>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variation {
    pub metadata: i64,
    pub display_name: String,
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
