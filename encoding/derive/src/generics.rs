use syn::{parse_quote, Generics, Lifetime, TypeParamBound};

pub fn prepare_generics(
    generics: &mut Generics,
    traid: TypeParamBound,
    lifetime: Option<Lifetime>,
) {
    if let Some(lt) = lifetime {
        if generics.lifetimes().next().is_some() {
            let lifetimes = generics.lifetimes();
            generics.params.push(parse_quote! {#lt: #(#lifetimes)+*});
        } else {
            generics.params.push(parse_quote! {#lt})
        }
    }
    for tp in generics.type_params_mut() {
        tp.bounds.push(traid.clone());
    }
}
