use convert_case::{Case, Casing};
use syn::Data;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(dojo))]
struct EnumStructAttrs {
    name: String,
    rename_all: String,
}

pub fn expand_enum_derive(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse the input tokens into a syntax tree
    let mut ast = syn::parse2::<syn::DeriveInput>(input)?;

    // Extract the attributes from the input
    let EnumStructAttrs { name, rename_all } = deluxe::extract_attributes(&mut ast)?;

    let ident = &ast.ident;
    let variants = match ast.data.clone() {
        Data::Enum(data) => data.variants,
        _ => panic!("Table can only be derived for structs"),
    };

    // Get the field idents
    let field_idents = variants
        .clone()
        .into_iter()
        .map(|f| f.ident)
        .collect::<Vec<_>>();

    let field_idents_str = field_idents
        .iter()
        .map(|i| {
            let s = i.to_string();
            match rename_all.as_str() {
                "lowercase" => s.to_case(Case::Lower),
                "UPPERCASE" => s.to_case(Case::Upper),
                "PascalCase" => s.to_case(Case::Pascal),
                "camelCase" => s.to_case(Case::Camel),
                "snake_case" => s.to_case(Case::Snake),
                "kebab-case" => s.to_case(Case::Kebab),
                "UPPER_SNAKE_CASE" => s.to_case(Case::ScreamingSnake),
                _ => s,
            }
        })
        .collect::<Vec<_>>();

    // Define the output tokens
    let expanded = quote::quote! {
        impl dojo_orm::types::ToSql for #ident {
            fn to_sql(
                &self,
                ty: &dojo_orm::types::Type,
                out: &mut dojo_orm::bytes::BytesMut,
            ) -> Result<dojo_orm::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
            where
                Self: Sized,
            {
                let s = match *self {
                    #(#ident::#field_idents => #field_idents_str),*
                };
                out.extend_from_slice(s.as_bytes());
                Ok(dojo_orm::types::IsNull::No)
            }

            fn accepts(ty: &dojo_orm::types::Type) -> bool
            where
                Self: Sized,
            {
                if ty.name() != #name {
                    return false;
                }

                match *ty.kind() {
                    dojo_orm::types::Kind::Enum(ref variants) => {
                        variants.iter().any(|v| #(v == #field_idents_str)||*)
                    }
                    _ => false,
                }
            }

            dojo_orm::types::to_sql_checked!();
        }

        impl<'a> dojo_orm::types::FromSql<'a> for #ident {
            fn from_sql(
                _ty: &dojo_orm::types::Type,
                raw: &[u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                match std::str::from_utf8(raw)? {
                    #(#field_idents_str => Ok(#ident::#field_idents)),*,
                    _ => Err("Unrecognized enum variant".into()),
                }
            }

            fn accepts(ty: &dojo_orm::types::Type) -> bool {
                if ty.name() != #name {
                    return false;
                }

                match *ty.kind() {
                    dojo_orm::types::Kind::Enum(ref variants) => {
                        variants.iter().any(|v| #(v == #field_idents_str)||* )
                    }
                    _ => false,
                }
            }
        }
    };

    // Return the generated impl
    Ok(expanded)
}
