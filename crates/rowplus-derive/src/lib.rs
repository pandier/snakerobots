use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, parse_macro_input};

#[derive(Default)]
struct Attributes {
    alias: Option<String>,
    nested: bool,
    flatten: bool,
}

#[proc_macro_derive(RowPlus, attributes(rowplus))]
pub fn devive_row_plus(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let table = parse_attributes(&input.attrs)
        .alias
        .expect("alias needs to be specified for struct");

    let output = match input.data {
        Data::Struct(s) => {
            let col_additions = s.fields.iter()
                .map(|field| {
                    let ident = field.ident.as_ref().expect("tuple structs are not supported");
                    let field_attrs = parse_attributes(&field.attrs);
                    let alias = field_attrs.alias.unwrap_or_else(|| ident.to_string());
                    let ty = &field.ty;

                    if field_attrs.nested {
                        quote! {
                            cols.nest::<#ty>((prefix.clone() + #alias).as_str());
                        }
                    } else if field_attrs.flatten {
                        quote! {
                            cols.flat::<#ty>(table, root);
                        }
                    } else {
                        quote! {
                            cols.add(table, #alias, (prefix.clone() + #alias).as_str());
                        }
                    }
                });

            let row_fields = s.fields.iter()
                .map(|field| {
                    let ident = field.ident.as_ref().expect("tuple structs are not supported");
                    let field_attrs = parse_attributes(&field.attrs);
                    let alias = field_attrs.alias.unwrap_or_else(|| ident.to_string());
                    let ty = &field.ty;

                    if field_attrs.nested {
                        quote! {
                            #ident: <#ty>::from_row_nested(row, (prefix.clone() + #alias).as_str())?,
                        }
                    } else if field_attrs.flatten {
                        quote! {
                            #ident: <#ty>::from_row_under(row, table, root)?,
                        }
                    } else {
                        quote! {
                            #ident: row.try_get((prefix.clone() + #alias).as_str())?,
                        }
                    }
                });

            // TODO: Make columns cached (lazy)
            quote! {
                impl rowplus::RowPlus for #name {
                    fn columns() -> rowplus::RowPlusColumns {
                        Self::columns_under(#table, true)
                    }

                    fn columns_under(table: &str, root: bool) -> rowplus::RowPlusColumns {
                        let prefix = if root { String::new() } else { String::from(table) + "." };
                        let mut cols = rowplus::RowPlusColumns::new();
                        #(
                            #col_additions
                        )*
                        cols
                    }

                    fn from_row_under(row: &sqlx::postgres::PgRow, table: &str, root: bool) -> Result<Self, sqlx::Error> {
                        use sqlx::Row;
                        use rowplus::RowPlusNested;
                        let prefix = if root { String::new() } else { String::from(table) + "." };
                        Ok(Self {
                            #(
                                #row_fields
                            )*
                        })
                    }
                }

                impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for #name {
                    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
                        rowplus::RowPlus::from_row_under(row, #table, true)
                    }
                }
            }
        },
        _ => panic!("only structs are supported"),
    };

    output.into()
}

fn parse_attributes(attrs: &Vec<Attribute>) -> Attributes {
    let mut result = Attributes::default();
    for attr in attrs {
        if attr.path().is_ident("rowplus") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("alias") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.alias = Some(value.value());
                } else if meta.path.is_ident("nested") {
                    result.nested = true;
                } else if meta.path.is_ident("flatten") {
                    result.flatten = true;
                }
                Ok(())
            }).unwrap();
        }
    }
    result
}
