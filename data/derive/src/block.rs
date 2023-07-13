use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use syn::LitBool;
use syn::LitFloat;
use syn::{LitInt, LitStr, Type};

pub fn block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    let path = input.path;
    let repr = input.repr;
    let path = format!("{}/{}", std::env::var("CARGO_MANIFEST_DIR").unwrap(), path);
    let file = std::fs::File::open(path).unwrap();

    let data: Vec<Block> =
        serde_json::from_reader(file).expect("json should be valid");

    let mut output = proc_macro2::TokenStream::new();

    crate::generate!(
        data,
        block,
        (
            block_enum,
            quote!(#ident = #id,),
            ident,
            Ident::new(&block.name.to_case(Case::Pascal), Span::call_site(),)
        ),
        (
            block_from_id,
            quote!(#id => Some(Self::#ident),),
            id,
            LitInt::new(&block.id.to_string(), Span::call_site())
        ),
        (
            block_name,
            quote!(Self::#ident => #name,),
            name,
            LitStr::new(&block.name, Span::call_site())
        ),
        (block_from_name, quote!(#name => Some(Self::#ident),)),
        (
            block_display_name,
            quote!(Self::#ident => #display_name,),
            display_name,
            LitStr::new(&block.display_name, Span::call_site())
        ),
        (
            block_hardness,
            quote!(Self::#ident => #hardness,),
            hardness,
            {
                if let Some(hardness) = &block.hardness {
                    let mut hardness = hardness.to_string();
                    if let None = hardness.find('.') {
                        hardness.push_str(".0")
                    }
                    let hardness = LitFloat::new(&hardness.to_string(), Span::call_site());
                    quote!(Some(#hardness))
                } else {
                    quote!(None)
                }
            }
        ),
        (
            block_stack_size,
            quote!(Self::#ident => #stack_size,),
            stack_size,
            LitInt::new(&block.stack_size.to_string(), Span::call_site())
        ),
        (
            block_diggable,
            quote!(Self::#ident => #diggable,),
            diggable,
            LitBool::new(block.diggable, Span::call_site())
        ),
        // TODO: add bounding box and drops
        (
            block_transparent,
            quote!(Self::#ident => #transparent,),
            transparent,
            LitBool::new(block.transparent, Span::call_site())
        ),
        (
            block_emit_light,
            quote!(Self::#ident => #emit_light,),
            emit_light,
            LitInt::new(&block.emit_light.to_string(), Span::call_site())
        ),
        (
            block_filter_light,
            quote!(Self::#ident => #filter_light,),
            filter_light,
            LitInt::new(&block.filter_light.to_string(), Span::call_site())
        ),
        (
            block_material,
            quote!(Self::#ident => #material,),
            material,
            {
                if let Some(material) = block.material {
                    let material = LitStr::new(&material, Span::call_site());
                    quote!(Some(#material))
                } else {
                    quote!(None)
                }
            }
        )
    );

    quote!(
        extern crate miners_data;

        #[repr(#repr)]
        #[derive(Copy, Clone)]
        pub enum Block {
            #block_enum
        }

        impl ::miners_data::block::Block for Block {
            fn id(self) -> u16 {
                self as #repr as u16
            }

            fn from_id(id: u16) -> Option<Self> {
                match id {
                    #block_from_id
                    _ => None
                }
            }

            fn name(self) -> &'static str {
                match self {
                    #block_name
                }
            }

            fn from_name(name: &str) -> Option<Self> {
                match name {
                    #block_from_name
                    _ => None
                }
            }

            fn display_name(self) -> &'static str {
                match self {
                    #block_display_name
                }
            }

            fn hardness(self) -> Option<f64> {
                match self {
                    #block_hardness
                }
            }

            fn stack_size(self) -> u8 {
                match self {
                    #block_stack_size
                }
            }

            fn diggable(self) -> bool {
                match self {
                    #block_diggable
                }
            }

            fn bounding_box(self) {}
            fn drops(self) {}

            fn transparent(self) -> bool {
                match self {
                    #block_transparent
                }
            }

            fn emit_light(self) -> u8 {
                match self {
                    #block_emit_light
                }
            }

            fn filter_light(self) -> u8 {
                match self {
                    #block_filter_light
                }
            }

            fn material(self) -> Option<&'static str> {
                match self {
                    #block_material
                }
            }

            fn harvest_tools(self) -> Option<()> {
                todo!()
            } 
            fn variations(self) -> Option<()> {
                todo!()
            }
            fn states(self) -> Option<()> {
                todo!()
            }
            fn min_state_id(self) -> Option<u16> {
                todo!()
            }
            fn max_state_id(self) -> Option<u16> {
                todo!()
            }
            fn default_state(self) -> Option<u16> {
                todo!()
            }
            fn resistance(self) -> Option<f32> {
                todo!()
            }
        }
    )
    .to_tokens(&mut output);
    output.into()
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Block {
    pub id: i64,
    pub display_name: String,
    pub name: String,
    pub hardness: Option<f64>,
    pub resistance: f64,
    pub min_state_id: Option<i64>,
    pub max_state_id: Option<i64>,
    pub states: Option<Vec<State>>,
    //pub drops: Vec<i64>, this causes an error for some reason
    pub diggable: bool,
    pub transparent: bool,
    pub filter_light: u8,
    pub emit_light: u8,
    pub bounding_box: String,
    pub stack_size: i64,
    pub material: Option<String>,
    pub default_state: Option<i64>,
    pub harvest_tools: Option<Value>, // TODO: figure out what is going on with harvest tools
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "num_values")]
    pub num_values: i64,
    #[serde(default)]
    pub values: Vec<String>,
}
