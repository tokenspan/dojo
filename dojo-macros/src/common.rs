use proc_macro2::Ident;
use syn::{Fields, GenericArgument, Type};

pub fn derive_get_fields_with_tys(fields: Fields) -> Vec<(Ident, Ident, Option<GenericArgument>)> {
    fields
        .into_iter()
        .filter_map(|f| match f.ty {
            Type::Path(tp) => Some((f.ident.unwrap(), tp)),
            _ => None,
        })
        .map(|(ident, tp)| {
            let outer_ty = tp.path.segments.first().unwrap().ident.clone();

            if outer_ty == *"Option" {
                let args = tp.path.segments.last().unwrap().clone().arguments;
                let inner_ty = match args {
                    syn::PathArguments::AngleBracketed(args) => args.args.first().unwrap().clone(),
                    _ => panic!("Option must have a generic type"),
                };

                (ident, outer_ty, Some(inner_ty))
            } else {
                (ident, outer_ty, None)
            }
        })
        .collect::<Vec<_>>()
}
