use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Component for #name {}
    };

    TokenStream::from(expanded)
}

// #[proc_macro_attribute]
// pub fn system_test_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let input_fn = parse_macro_input!(item as ItemFn);

//     // Retrieve the types of the function's arguments
//     let fn_args = input_fn.sig.inputs.iter().filter_map(|arg| {
//         if let FnArg::Typed(pat_type) = arg {
//             Some(&pat_type.ty)
//         } else {
//             None
//         }
//     });

//     let types: Vec<_> = fn_args.collect();
//     let fn_name = &input_fn.sig.ident;
//     let fn_block = &input_fn.block;
//     let fn_vis = &input_fn.vis;
//     let fn_sig = &input_fn.sig;

//     // Step 1: Check if the first type is a Query
//     if let Some(first_type) = types.get(0) {
//         // Step 2: Dereference the Box to get the syn::Type
//         if let syn::Type::Path(path) = &***first_type {
//             // Step 3: Check if the type is Query and extract the generic argument
//             if let Some(seg) = path.path.segments.last() {
//                 if seg.ident == "Query" {
//                     if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
//                         // Step 4: Handle the generic arguments
//                         if let Some(syn::GenericArgument::Type(syn::Type::Tuple(tuple))) =
//                             args.args.first()
//                         {
//                             // Step 5: Extract the types from the tuple
//                             let tuple_types: Vec<_> = tuple.elems.iter().collect();

//                             // Step 6: Generate code to print each type in the tuple
//                             let print_types_code = tuple_types.iter().map(|ty| {
//                                 quote! {
//                                     println!("Type: {:?}", stringify!(#ty));
//                                 }
//                             });

//                             // Step 7: Generate the final function code
//                             let expanded = quote! {
//                                 #fn_vis #fn_sig {
//                                     let _q = #first_type::new();
//                                     #(#print_types_code)* // Print each type in the tuple
//                                     #fn_block
//                                 }
//                             };

//                             // Return the expanded code
//                             return expanded.into();
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     // Fallback case: if not a Query with a tuple, handle as usual
//     let query_ty = types[0]; // support for one parameter
//     let expanded = quote! {
//         #fn_vis #fn_sig {
//             let _q = #query_ty::new();
//             #fn_block
//         }
//     };

//     expanded.into()
// }
