pub fn expand_embedded_derive(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse the input tokens into a syntax tree
    let ast = syn::parse2::<syn::DeriveInput>(input)?;

    // Define impl variables
    let ident = &ast.ident;

    // Define the output tokens
    let expanded = quote::quote! {
        impl<'a> dojo_orm::types::FromSql<'a> for #ident {
            fn from_sql(ty: &dojo_orm::types::Type, mut raw: &[u8]) -> std::result::Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                if *ty == dojo_orm::types::Type::JSONB {
                    let mut b = [0; 1];
                    use std::io::Read;
                    raw.read_exact(&mut b)?;
                    // We only support version 1 of the jsonb binary format
                    if b[0] != 1 {
                        return Err("unsupported JSONB encoding version".into());
                    }
                }

                serde_json::from_slice(raw).map_err(Into::into)
            }

            dojo_orm::types::accepts!(JSON, JSONB);
        }

        impl dojo_orm::types::ToSql for #ident {
            fn to_sql(
                &self,
                ty: &dojo_orm::types::Type,
                out: &mut dojo_orm::bytes::BytesMut,
            ) -> std::result::Result<dojo_orm::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
                if *ty == dojo_orm::types::Type::JSONB {
                    dojo_orm::bytes::BufMut::put_u8(out, 1);
                }
                use dojo_orm::bytes::buf::BufMut;
                serde_json::to_writer(out.writer(), &self)?;
                Ok(dojo_orm::types::IsNull::No)
            }

            dojo_orm::types::accepts!(JSON, JSONB);
            dojo_orm::types::to_sql_checked!();
        }
    };

    // Return the generated impl
    Ok(expanded)
}
