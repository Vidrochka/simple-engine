use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
use darling::{ast::NestedMeta, FromMeta};

#[derive(FromMeta)]
struct TemplateArgs {
    name: String,
    template: String,
    #[darling(default, multiple)]
    styles: Vec<String>,
}

#[proc_macro_attribute]
pub fn template(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    let stream = proc_macro2::token_stream::TokenStream::from(attr);

    // Парсим атрибут как `Meta`
    // let meta = parse_macro_input!(attr as MetaList);

    // Используем `darling` для парсинга аргументов
    let args = match TemplateArgs::from_list(&NestedMeta::parse_meta_list(stream).unwrap()) {
        Ok(args) => args,
        Err(err) => return TokenStream::from(err.write_errors()),
    };

    let component_name = args.name;
    let template_file = args.template;
    let style_includes: Vec<_> = args.styles
        .iter()
        .map(|file| {
            quote! { include_str!(#file).to_string() }
        })
        .collect();

    let expanded = quote! {
        #input

        impl xui::component::IComponentTemplate for #struct_name {
            fn name(&self) -> String {
                #component_name.to_string()
            }

            fn template(&self) -> String {
                include_str!(#template_file).to_string()
            }

            fn styles(&self) -> Vec<String> {
                vec![#(#style_includes),*]
            }
        }
    };

    TokenStream::from(expanded)
}
