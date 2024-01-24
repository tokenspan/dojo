use quote::quote;
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields};

const SUPPORTED_TYPES: &[&str] = &["i32", "i64", "Uuid", "String", "NaiveDateTime"];

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(dojo))]
struct ModelStructAttrs {
    name: String,
    #[deluxe(default)]
    sort_keys: Vec<String>,
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(dojo))]
struct ModelFieldAttributes {
    #[deluxe(default = false)]
    skip: bool,
}

fn extract_model_field_attributes(
    ast: &mut DeriveInput,
) -> deluxe::Result<HashMap<String, crate::expand_model_derive::ModelFieldAttributes>> {
    let mut field_attrs = HashMap::new();

    if let Data::Struct(s) = &mut ast.data {
        for field in s.fields.iter_mut() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let attrs = deluxe::extract_attributes(field)?;
            field_attrs.insert(field_name, attrs);
        }
    }

    Ok(field_attrs)
}

pub fn expand_model_derive(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse the input tokens into a syntax tree
    let mut ast = syn::parse2::<syn::DeriveInput>(input)?;

    // Extract the attributes from the input
    let ModelStructAttrs { name, sort_keys } = deluxe::extract_attributes(&mut ast)?;
    let field_attrs = extract_model_field_attributes(&mut ast)?;

    // Define impl variables
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let fields: Fields = match ast.data.clone() {
        Data::Struct(data) => data.fields,
        _ => panic!("Table can only be derived for structs"),
    };

    // Get the field idents
    let field_idents = fields
        .clone()
        .into_iter()
        .filter_map(|f| f.ident)
        .collect::<Vec<_>>();

    let ident_columns = field_idents
        .iter()
        .filter_map(|ident| {
            let skip = field_attrs
                .get(&ident.to_string())
                .map(|attrs| attrs.skip)
                .unwrap_or(false);

            if !skip {
                Some(ident)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let supported_values = fields
        .iter()
        .map(|f| {
            let ty = &f.ty;
            let ident = f.ident.clone().unwrap();

            if SUPPORTED_TYPES.contains(&quote::quote!(#ty).to_string().as_str()) {
                quote::quote! {
                    stringify!(#ident) => Some(dojo_orm::Value::from(&self.#ident)),
                }
            } else {
                quote::quote! {
                    stringify!(#ident) => None,
                }
            }
        })
        .collect::<Vec<_>>();

    let struct_fields_idents = field_idents
        .iter()
        .map(|ident| {
            let skip = field_attrs
                .get(&ident.to_string())
                .map(|attrs| attrs.skip)
                .unwrap_or(false);

            if !skip {
                quote! {
                    #ident: row.try_get(stringify!(#ident))?,
                }
            } else {
                quote! {
                    #ident: Default::default(),
                }
            }
        })
        .collect::<Vec<_>>();

    let columns = ident_columns
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<_>>();

    // Define the output tokens
    let expanded = quote::quote! {
        #[async_trait::async_trait]
        impl #impl_generics dojo_orm::Model for #ident #ty_generics #where_clause {
            const NAME: &'static str = #name;

            const COLUMNS: &'static [&'static str] = &[
                #(#columns),*
            ];

            fn params(&self) -> Vec<&(dyn dojo_orm::types::ToSql + Sync)> {
                vec![#(&self.#ident_columns),*]
            }

            fn from_row(row: tokio_postgres::Row) -> anyhow::Result<Self> {
                Ok(#ident {
                    #(#struct_fields_idents)*
                })
            }

            fn get_value(&self, column: &str) -> Option<dojo_orm::Value> {
                match column {
                    #(#supported_values)*
                    _ => None,
                }
            }

            fn sort_keys() -> Vec<String> {
                vec![#(#sort_keys.to_string()),*]
            }
        }
    };

    // Return the generated impl
    Ok(expanded)
}
