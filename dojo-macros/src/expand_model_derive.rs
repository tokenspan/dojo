use syn::{Data, Fields};

const SUPPORTED_TYPES: &[&str] = &["i32", "i64", "Uuid", "String", "NaiveDateTime"];

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(dojo))]
struct ModelStructAttrs {
    name: String,
    sort_keys: Vec<String>,
}

pub fn expand_model_derive(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse the input tokens into a syntax tree
    let mut ast = syn::parse2::<syn::DeriveInput>(input)?;

    // Extract the attributes from the input
    let ModelStructAttrs { name, sort_keys } = deluxe::extract_attributes(&mut ast)?;

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

    let field_idents_str = field_idents
        .iter()
        .map(|i| i.to_string())
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

    // Define the output tokens
    let expanded = quote::quote! {
        #[async_trait::async_trait]
        impl #impl_generics dojo_orm::Model for #ident #ty_generics #where_clause {
            const NAME: &'static str = #name;

            const COLUMNS: &'static [&'static str] = &[
                #(#field_idents_str),*
            ];

            fn params(&self) -> Vec<&(dyn dojo_orm::types::ToSql + Sync)> {
                vec![#(&self.#field_idents),*]
            }

            fn from_row(row: tokio_postgres::Row) -> anyhow::Result<Self> {
                Ok(#ident {
                    #(#field_idents: row.try_get(stringify!(#field_idents))?),*
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
