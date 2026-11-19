use parser_lib::csv::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::utils::*;

pub(super) fn gen_impl_to_vec(
    struct_name: &Ident,
    file_name: &str,
    fields: &Vec<(Ident, (InferredType, bool), usize)>,
    column_names: &Vec<&str>,
    csv_file: &CSVFile,
) -> TokenStream {
    let fn_ident = Ident::new("new", struct_name.span());

    let mut fields2 = vec![];
    for (field_ident, (inferred_type, optional), _index) in fields.iter() {
        let field_type_str = rust_type(*inferred_type, *optional);
        let field_type = match string_to_type(&field_type_str) {
            Ok(t) => t,
            Err(err) => return err.into(),
        };
        fields2.push((field_ident.clone(), field_type));
    }

    let (key_name_function_name, key_type_name) = get_key_name_function_and_type(fields);

    let mut get_fields = vec![];
    for (field_ident, (inferred_type, optional), index) in fields {
        let func_name = convert_function(inferred_type);
        let index_argument = match create_litint(&index.to_string()) {
            Ok(n) => n,
            Err(e) => return e.into(),
        };

        if let Some(func_name) = func_name {
            let get_fun_ident = Ident::new("get_field", struct_name.span());
            let func_name_ident = Ident::new(&func_name, struct_name.span());
            let get_field = if !*optional {
                quote! {
                    #func_name_ident(record.#get_fun_ident(#index_argument)).unwrap()
                }
            } else {
                quote! {
                    #func_name_ident(record.#get_fun_ident(#index_argument))
                }
            };

            get_fields.push(get_field);
        } else {
            return quote! {
                compile_error!("Cannot find conversion function for field {}", stringify!(#field_ident));
            };
        }
    }

    let new_func = create_new_function(&fn_ident, &fields2);
    let getters = create_getters(&fields2);
    let setters = create_setters(&fields2);

    let getters_with_names = create_getters_with_names(&fields2, column_names);
    let setters_with_names = create_setters_with_names(&fields2, column_names);
    let typed_vec_func = get_to_typed_vec(struct_name, file_name, &get_fields);
    let hash_typed_vec_func = if is_key_option(fields) {
        TokenStream::new()
    } 
    else {
        let has_duplicated = col_has_duplication(csv_file);
        get_to_hash_typed_vec(struct_name, file_name, &get_fields, &key_name_function_name, &key_type_name, has_duplicated)
    };

    quote! {
        impl #struct_name {
            #new_func

            #getters
            #setters

            #getters_with_names
            #setters_with_names

            #typed_vec_func
            #hash_typed_vec_func           
        }
    }
}

fn is_key_option(fields: &Vec<(Ident, (InferredType, bool), usize)>) -> bool {
    let first_field = fields.first().unwrap();
    let (_, optional) = first_field.1;
    optional
}

fn get_key_name_function_and_type(fields: &Vec<(Ident, (InferredType, bool), usize)>) -> (String, String) {
    let key_name_function = get_key_name_function(fields);
    let key_type_name = get_key_type_name(fields);
    (key_name_function, key_type_name)
}

fn get_key_name_function(fields: &Vec<(Ident, (InferredType, bool), usize)>) -> String {
    let first_field = fields.first().unwrap();
    let field_name = first_field.0.to_string();
    format!("get_{}", field_name)
}

fn get_key_type_name(fields: &Vec<(Ident, (InferredType, bool), usize)>) -> String {
    let first_field = fields.first().unwrap();
    let (ty, optional) = first_field.1;
    rust_type(ty, optional)
}

fn col_has_duplication(csv_file: &CSVFile) -> bool {
    csv_file.has_duplicated_value(0)
}

fn get_to_typed_vec(struct_name: &Ident, file_name: &str, get_fields: &Vec<TokenStream>) -> TokenStream {
    quote! {
        pub fn to_typed_vec() -> Vec<Self> {
            let csv_file = parser_lib::csv::parse_csv_file(#file_name)
                .expect("Failed to parse CSV file");

            let mut result = vec![];
            for record in csv_file.get_data_records() {
                result.push(
                    #struct_name::new(
                    #(#get_fields),*
                ));
            }

            result
        }
    }
}

fn get_to_hash_typed_vec(struct_name: &Ident, file_name: &str, get_fields: &Vec<TokenStream>, key_name_function: &str, key_type_name:&str, has_duplicated: bool) -> TokenStream {
    let key_fn = ident_from_str(key_name_function);
    let key_type = ident_from_str(key_type_name);

    let warning = if has_duplicated { warning("keyword column has duplication") }
                else { TokenStream::new() };

    quote! {
        pub fn to_indexed_typed_vec() -> IndexedStore<Self, #key_type, fn(&#struct_name) -> #key_type> {
            let csv_file = parser_lib::csv::parse_csv_file(#file_name)
                .expect("Failed to parse CSV file");

            let mut result = IndexedStore::new((|n:&#struct_name| n.#key_fn().clone()) as fn(&#struct_name) -> #key_type);
            for record in csv_file.get_data_records() {
                result.push(
                    #struct_name::new(
                    #(#get_fields),*
                ));
            }

            result
        }

        #warning
    }
}