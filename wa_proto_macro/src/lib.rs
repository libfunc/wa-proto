extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Ident, Lit, Meta, Type};

// TODO: impl for enum E { A(...), B(...) }

fn args_in(name: Ident) -> TokenStream {
    quote! {
        Incoming::args(&self.#name, args)?;
    }
}

fn args_out(name: Ident) -> TokenStream {
    quote! {
        Outcoming::args(&self.#name, args)?;
    }
}

fn fill_gen(name: Ident) -> TokenStream {
    quote! {
        Incoming::fill(&self.#name, heap, args)?;
    }
}

fn init_gen(name: Ident, typ: &Type) -> TokenStream {
    let stream = quote! {
        let (_, #name) = <#typ as Incoming>::init(args)?;
    };

    stream
}

fn read_gen(name: Ident, typ: &Type) -> TokenStream {
    let stream = quote! {
        let #name = <#typ as Outcoming>::read(heap, args)?;
    };

    stream
}

fn is_need_init_fill_gen(typ: &Type) -> TokenStream {
    quote! {
        <#typ as Incoming>::IS_NEED_INIT_FILL
    }
}

fn is_need_read_gen(typ: &Type) -> TokenStream {
    quote! {
        <#typ as Outcoming>::IS_NEED_READ
    }
}

fn get_primitive_name(ast: &DeriveInput) -> TokenStream {
    ast.attrs
        .iter()
        .find_map(|attr| {
            attr.path.segments.first().and_then(|segment| {
                if segment.ident != "coming" {
                    return None;
                }
                match attr.parse_args::<Meta>() {
                    Ok(Meta::NameValue(name_value)) => {
                        if name_value.path.to_token_stream().to_string() != "primitive" {
                            return None;
                        }
                        if let Lit::Str(litstr) = name_value.lit {
                            let s = litstr.parse::<Ident>().unwrap();
                            let value = s.to_token_stream();
                            Some(value)
                        } else {
                            None
                        }
                    }
                    Ok(_) => None,
                    Err(_) => None,
                }
            })
        })
        .expect("complex enums must include primitive type name!")
}

#[proc_macro_derive(Incoming, attributes(coming))]
pub fn derive_set_incoming(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(stream as DeriveInput);

    let name = &ast.ident;
    let data = &ast.data;

    match data {
        Data::Struct(s) => {
            let fields = &s.fields;
            match fields {
                Fields::Named(fields) => {
                    let named = &fields.named;
                    let named_len = named.len();

                    let mut init_streams: Vec<TokenStream> = Vec::with_capacity(named_len);
                    let mut init_names: Vec<String> = Vec::with_capacity(named_len);
                    let mut args_streams: Vec<TokenStream> = Vec::with_capacity(named_len);
                    let mut fill_streams: Vec<TokenStream> = Vec::with_capacity(named_len);
                    let mut is_need_init_fill_streams: Vec<TokenStream> =
                        Vec::with_capacity(named_len);

                    for field_name in named.iter() {
                        let ident = field_name.ident.clone().unwrap();
                        let typ = field_name.ty.clone();

                        let args_ge = args_in(ident.clone());
                        args_streams.push(args_ge.clone());

                        let fill_ge = fill_gen(ident.clone());
                        fill_streams.push(fill_ge.clone());

                        let stream = init_gen(ident.clone(), &typ);
                        init_streams.push(stream);
                        init_names.push(ident.to_string());

                        let is_need = is_need_init_fill_gen(&typ);
                        is_need_init_fill_streams.push(is_need);
                    }

                    let need_init_fill = is_need_init_fill_streams.into_iter().fold(
                        TokenStream::new(),
                        |mut acc, x| {
                            if acc.is_empty() {
                                acc.extend([x]);
                            } else {
                                acc.extend([TokenStream::from_str(" || ").unwrap(), x]);
                            }
                            acc
                        },
                    );

                    let names = init_names.iter().map(|s| Ident::new(&s, Span::call_site()));

                    let gen = quote! {
                        impl Incoming for #name {
                            const IS_NEED_INIT_FILL: bool = #need_init_fill;

                            #[cfg(not(feature = "std"))]
                            fn init(args: &mut core::slice::IterMut<u32>) -> Result<(u32, Self), wa_proto::ProtocolError> {
                                #(#init_streams)*
                                Ok((0, #name { #(#names),* }))
                            }

                            #[cfg(feature = "std")]
                            fn args(&self, args: &mut Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                                #(#args_streams)*
                                Ok(())
                            }

                            #[cfg(feature = "std")]
                            fn fill(&self, heap: &mut core::cell::RefMut<[u8]>, args: &mut core::slice::Iter<u32>) -> Result<(), wa_proto::ProtocolError> {
                                #(#fill_streams)*
                                Ok(())
                            }
                        }
                    };

                    proc_macro::TokenStream::from(gen)
                }
                Fields::Unnamed(fields) => {
                    let unnamed = &fields.unnamed;
                    let unnamed_len = unnamed.len();
                    let mut init_streams: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut init_names: Vec<String> = Vec::with_capacity(unnamed_len);
                    let mut args_streams: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut fill_streams: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut is_need_init_fill_streams: Vec<TokenStream> =
                        Vec::with_capacity(unnamed_len);

                    for (index, field_name) in unnamed.iter().enumerate() {
                        let typ = field_name.ty.clone();
                        let idx_str = index.to_string();
                        let idx_literal = Literal::usize_unsuffixed(index);
                        let arg_idx = format!("arg{}", &idx_str);
                        let arg_idx_ident = Ident::new(&arg_idx, Span::call_site());

                        let args_ge = quote! {
                            Incoming::args(&self.#idx_literal, args)?;
                        };
                        args_streams.push(args_ge.clone());

                        let fill_ge = quote! {
                            Incoming::fill(&self.#idx_literal, heap, args)?;
                        };
                        fill_streams.push(fill_ge.clone());

                        let stream = init_gen(arg_idx_ident.clone(), &typ);
                        init_streams.push(stream);
                        init_names.push(arg_idx);

                        let is_need = is_need_init_fill_gen(&typ);
                        is_need_init_fill_streams.push(is_need);
                    }

                    let need_init_fill = is_need_init_fill_streams.into_iter().fold(
                        TokenStream::new(),
                        |mut acc, x| {
                            if acc.is_empty() {
                                acc.extend([x]);
                            } else {
                                acc.extend([TokenStream::from_str(" || ").unwrap(), x]);
                            }
                            acc
                        },
                    );

                    let names = init_names.iter().map(|s| Ident::new(&s, Span::call_site()));

                    let gen = quote! {
                        impl Incoming for #name {
                            const IS_NEED_INIT_FILL: bool = #need_init_fill;

                            #[cfg(not(feature = "std"))]
                            fn init(args: &mut core::slice::IterMut<u32>) -> Result<(u32, Self), wa_proto::ProtocolError> {
                                #(#init_streams)*
                                Ok((0, #name ( #(#names),* )))
                            }

                            #[cfg(feature = "std")]
                            fn args(&self, args: &mut Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                                #(#args_streams)*
                                Ok(())
                            }

                            #[cfg(feature = "std")]
                            fn fill(&self, heap: &mut core::cell::RefMut<[u8]>, args: &mut core::slice::Iter<u32>) -> Result<(), wa_proto::ProtocolError> {
                                #(#fill_streams)*
                                Ok(())
                            }
                        }
                    };

                    proc_macro::TokenStream::from(gen)
                }
                Fields::Unit => {
                    let gen = quote! {
                        impl Incoming for #name {
                            #[cfg(not(feature = "std"))]
                            fn init(args: &mut core::slice::IterMut<u32>) -> Result<(u32, Self), wa_proto::ProtocolError> {
                                Ok((0, #name))
                            }

                            #[cfg(feature = "std")]
                            fn args(&self, args: &mut Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                                Ok(())
                            }

                            #[cfg(feature = "std")]
                            fn fill(&self, heap: &mut core::cell::RefMut<[u8]>, args: &mut core::slice::Iter<u32>) -> Result<(), wa_proto::ProtocolError> {
                                Ok(())
                            }
                        }
                    };
                    proc_macro::TokenStream::from(gen)
                }
            }
        }
        Data::Enum(data_enum) => {
            let is_simple_enum = data_enum.variants.iter().all(|item| item.fields.is_empty());

            if is_simple_enum {
                let gen = quote! {
                    impl Incoming for #name {
                        #[cfg(not(feature = "std"))]
                        fn init(args: &mut core::slice::IterMut<u32>) -> Result<(u32, Self), wa_proto::ProtocolError> {
                            let val = *args.next().ok_or(wa_proto::ProtocolError(ARGS_NEXT_ERROR))?;
                            let pt = #name::from_u32(val).ok_or(wa_proto::ProtocolError(ENUM_FROM_U32_ERROR))?;
                            Ok((val, pt))
                        }

                        #[cfg(feature = "std")]
                        fn args(&self, args: &mut Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                            args.push(*self as u32);
                            Ok(())
                        }

                        #[cfg(feature = "std")]
                        fn fill(&self, heap: &mut core::cell::RefMut<[u8]>, args: &mut core::slice::Iter<u32>) -> Result<(), wa_proto::ProtocolError> {
                            args.next().ok_or(wa_proto::ProtocolError("args is end".to_string()))?;
                            Ok(())
                        }
                    }
                };

                proc_macro::TokenStream::from(gen)
            } else {
                let primitive_name = get_primitive_name(&ast);
                let mut variants_names: Vec<(TokenStream, Option<Field>)> =
                    Vec::with_capacity(data_enum.variants.len());

                for variant in &data_enum.variants {
                    if variant.discriminant.is_some() {
                        // why? because discriminant number may not be equal to primitive number
                        panic!("enums variants with discriminant not support in current moment");
                    }
                    let fields = &variant.fields;
                    let fields = match fields {
                        Fields::Unit => None,
                        Fields::Unnamed(fields) => {
                            let len = fields.unnamed.len();
                            if len != 1 {
                                panic!("enums variants is currently support only with 1 unnamed fields");
                            }
                            let field = fields.unnamed.first().unwrap();
                            Some(field.clone())
                        }
                        Fields::Named(_) => {
                            panic!("enums named variants is currently not support");
                        }
                    };
                    let variant_name = &variant.ident;
                    variants_names.push((variant_name.to_token_stream(), fields));
                }

                let init_items: Vec<TokenStream> = variants_names
                    .iter()
                    .map(|(variant_name, inner)| {
                        if let Some(inner) = inner {
                            quote! {
                                #primitive_name::#variant_name => {
                                    let v = <#inner>::init(args)?.1;
                                    #name::#variant_name(v)
                                }
                            }
                        } else {
                            quote! {
                                #primitive_name::#variant_name => {
                                    #name::#variant_name
                                }
                            }
                        }
                    })
                    .collect();

                let args_in_items: Vec<TokenStream> = variants_names
                    .iter()
                    .map(|(variant_name, inner)| {
                        if inner.is_some() {
                            quote! {
                                #name::#variant_name(value) => {
                                    value.args(args)?;
                                }
                            }
                        } else {
                            quote! {
                                #name::#variant_name => {}
                            }
                        }
                    })
                    .collect();

                let fill_items: Vec<TokenStream> = variants_names
                    .iter()
                    .map(|(variant_name, inner)| {
                        if inner.is_some() {
                            quote! {
                                #name::#variant_name(value) => {
                                    value.fill(heap, args)?;
                                }
                            }
                        } else {
                            quote! {
                                #name::#variant_name => {}
                            }
                        }
                    })
                    .collect();

                let need_init_fill: Vec<TokenStream> = variants_names
                    .iter()
                    .filter_map(|(_, inner)| {
                        inner.as_ref().map(|inner| is_need_init_fill_gen(&inner.ty))
                    })
                    .collect();

                let need_init_fill =
                    need_init_fill
                        .into_iter()
                        .fold(TokenStream::new(), |mut acc, x| {
                            if acc.is_empty() {
                                acc.extend([x]);
                            } else {
                                acc.extend([TokenStream::from_str(" || ").unwrap(), x]);
                            }
                            acc
                        });

                let gen = quote! {
                    impl Incoming for #name {
                        const IS_NEED_INIT_FILL: bool = #need_init_fill;

                        #[cfg(not(feature = "std"))]
                        fn init(args: &mut core::slice::IterMut<u32>) -> Result<(u32, Self), wa_proto::ProtocolError> {
                            let val = *args.next().ok_or(wa_proto::ProtocolError(ARGS_NEXT_ERROR))?;
                            let pt: #primitive_name = FromPrimitive::from_u32(val).ok_or(wa_proto::ProtocolError(ENUM_FROM_U32_ERROR))?;
                            let item = match pt {
                                #(#init_items)*
                            };
                            Ok((val, item))
                        }

                        #[cfg(feature = "std")]
                        fn args(&self, args: &mut Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                            args.push(self.get_primitive_enum() as u32);
                            match self {
                                #(#args_in_items)*
                            }
                            Ok(())
                        }

                        #[cfg(feature = "std")]
                        fn fill(&self, heap: &mut core::cell::RefMut<[u8]>, args: &mut core::slice::Iter<u32>) -> Result<(), wa_proto::ProtocolError> {
                            args.next().ok_or(wa_proto::ProtocolError("args is end".to_string()))?;
                            match self {
                                #(#fill_items)*
                            }
                            Ok(())
                        }
                    }
                };

                proc_macro::TokenStream::from(gen)
            }
        }
        Data::Union(_) => {
            // raw C unions like Rust enums
            panic!("unions not supported, but Rust enums is implemented Incoming trait (use Enums instead)");
        }
    }
}

#[proc_macro_derive(Outcoming, attributes(coming))]
pub fn derive_set_outcoming(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(stream as DeriveInput);

    let name = &ast.ident;
    let data = &ast.data;

    match data {
        Data::Struct(data_struct) => {
            let fields = &data_struct.fields;
            match fields {
                Fields::Named(fields) => {
                    let named = &fields.named;
                    let named_len = named.len();

                    let mut args_streams: Vec<TokenStream> = Vec::with_capacity(named_len);
                    let mut read_streams: Vec<TokenStream> = Vec::with_capacity(named_len);
                    let mut init_names: Vec<String> = Vec::with_capacity(named_len);
                    let mut is_need_read_streams: Vec<TokenStream> = Vec::with_capacity(named_len);

                    for field_name in named.iter() {
                        let ident = field_name.ident.clone().unwrap();
                        let typ = field_name.ty.clone();

                        let args_ge = args_out(ident.clone());
                        args_streams.push(args_ge.clone());

                        let read_stream = read_gen(ident.clone(), &typ);
                        read_streams.push(read_stream);

                        init_names.push(ident.to_string());

                        let is_need = is_need_read_gen(&typ);
                        is_need_read_streams.push(is_need);
                    }

                    let is_need_read_stream =
                        is_need_read_streams
                            .into_iter()
                            .fold(TokenStream::new(), |mut acc, x| {
                                if acc.is_empty() {
                                    acc.extend([x]);
                                } else {
                                    acc.extend([TokenStream::from_str(" || ").unwrap(), x]);
                                }
                                acc
                            });

                    let names = init_names.iter().map(|s| Ident::new(&s, Span::call_site()));
                    let gen = quote! {
                        impl Outcoming for #name {
                            const IS_NEED_READ: bool = #is_need_read_stream;

                            #[cfg(not(feature = "std"))]
                            fn args(&self, args: &mut Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                                #(#args_streams)*
                                Ok(())
                            }

                            #[cfg(feature = "std")]
                            fn read(heap: &[u8], args: &mut core::slice::Iter<u32>) -> Result<Self, wa_proto::ProtocolError> where Self: Sized {
                                #(#read_streams)*

                                Ok(#name { #(#names),* })
                            }
                        }
                    };

                    proc_macro::TokenStream::from(gen)
                }
                Fields::Unnamed(fields) => {
                    let unnamed = &fields.unnamed;
                    let unnamed_len = unnamed.len();

                    let mut args_streams: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut read_streams: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
                    let mut init_names: Vec<String> = Vec::with_capacity(unnamed_len);
                    let mut is_need_read_streams: Vec<TokenStream> =
                        Vec::with_capacity(unnamed_len);

                    for (index, field_name) in unnamed.iter().enumerate() {
                        let typ = field_name.ty.clone();
                        let idx_str = index.to_string();
                        let idx_literal = Literal::usize_unsuffixed(index);
                        let arg_idx = format!("arg{}", &idx_str);
                        let arg_idx_ident = Ident::new(&arg_idx, Span::call_site());

                        let args_ge = quote! {
                            Outcoming::args(&self.#idx_literal, args)?;
                        };
                        args_streams.push(args_ge.clone());

                        let read_stream = read_gen(arg_idx_ident, &typ);
                        read_streams.push(read_stream);

                        init_names.push(arg_idx);

                        let is_need = is_need_read_gen(&typ);
                        is_need_read_streams.push(is_need);
                    }

                    let is_need_read_stream =
                        is_need_read_streams
                            .into_iter()
                            .fold(TokenStream::new(), |mut acc, x| {
                                if acc.is_empty() {
                                    acc.extend([x]);
                                } else {
                                    acc.extend([TokenStream::from_str(" || ").unwrap(), x]);
                                }
                                acc
                            });

                    let names = init_names.iter().map(|s| Ident::new(&s, Span::call_site()));
                    let gen = quote! {
                        impl Outcoming for #name {
                            const IS_NEED_READ: bool = #is_need_read_stream;

                            #[cfg(not(feature = "std"))]
                            fn args(&self, args: &mut alloc::vec::Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                                #(#args_streams)*
                                Ok(())
                            }

                            #[cfg(feature = "std")]
                            fn read(heap: &[u8], args: &mut core::slice::Iter<u32>) -> Result<Self, wa_proto::ProtocolError> {
                                #(#read_streams)*

                                Ok(#name ( #(#names),* ))
                            }
                        }
                    };

                    proc_macro::TokenStream::from(gen)
                }
                Fields::Unit => {
                    let gen = quote! {
                        impl Outcoming for #name {
                            #[cfg(not(feature = "std"))]
                            fn args(&self, args: &mut alloc::vec::Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                                Ok(())
                            }

                            #[cfg(feature = "std")]
                            fn read(heap: &[u8], args: &mut core::slice::Iter<u32>) -> Result<Self, wa_proto::ProtocolError> {
                                Ok(#name)
                            }
                        }
                    };

                    proc_macro::TokenStream::from(gen)
                }
            }
        }
        Data::Enum(data_enum) => {
            let is_simple_enum = data_enum.variants.iter().all(|item| item.fields.is_empty());

            if is_simple_enum {
                let gen = quote! {
                    impl Outcoming for #name {
                        #[cfg(not(feature = "std"))]
                        fn args(&self, args: &mut alloc::vec::Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                            args.push(*self as u32);
                            Ok(())
                        }

                        #[cfg(feature = "std")]
                        fn read(heap: &[u8], args: &mut core::slice::Iter<u32>) -> Result<Self, wa_proto::ProtocolError> {
                            let val = *args.next().ok_or(wa_proto::ProtocolError("args is end".to_string()))?;
                            let pt = #name::from_u32(val).ok_or(wa_proto::ProtocolError("#name enum type error".to_string()))?;
                            Ok(pt)
                        }
                    }
                };

                proc_macro::TokenStream::from(gen)
            } else {
                let primitive_name = get_primitive_name(&ast);
                let mut variants_names: Vec<(TokenStream, Option<Field>)> =
                    Vec::with_capacity(data_enum.variants.len());

                for variant in &data_enum.variants {
                    if variant.discriminant.is_some() {
                        // why? because discriminant number may not be equal to primitive number
                        panic!("enums variants with discriminant not support in current moment");
                    }
                    let fields = &variant.fields;
                    let fields = match fields {
                        Fields::Unit => None,
                        Fields::Unnamed(fields) => {
                            let len = fields.unnamed.len();
                            if len != 1 {
                                panic!("enums variants is currently support only with 1 unnamed fields");
                            }
                            let field = fields.unnamed.first().unwrap();
                            Some(field.clone())
                        }
                        Fields::Named(_) => {
                            panic!("enums named variants is currently not support");
                        }
                    };
                    let variant_name = &variant.ident;
                    variants_names.push((variant_name.to_token_stream(), fields));
                }

                let args_in_items: Vec<TokenStream> = variants_names
                    .iter()
                    .map(|(variant_name, inner)| {
                        if inner.is_some() {
                            quote! {
                                #name::#variant_name(value) => {
                                    value.args(args)?;
                                }
                            }
                        } else {
                            quote! {
                                #name::#variant_name => {}
                            }
                        }
                    })
                    .collect();

                let read_items: Vec<TokenStream> = variants_names
                    .iter()
                    .map(|(variant_name, inner)| {
                        if let Some(inner) = inner {
                            quote! {
                                #primitive_name::#variant_name => {
                                    let v = <#inner>::read(heap, args)?;
                                    #name::#variant_name(v)
                                }
                            }
                        } else {
                            quote! {
                                #primitive_name::#variant_name => {
                                    #name::#variant_name
                                }
                            }
                        }
                    })
                    .collect();

                let need_read: Vec<TokenStream> = variants_names
                    .iter()
                    .filter_map(|(_, inner)| {
                        inner.as_ref().map(|inner| is_need_read_gen(&inner.ty))
                    })
                    .collect();

                let need_read = need_read
                    .into_iter()
                    .fold(TokenStream::new(), |mut acc, x| {
                        if acc.is_empty() {
                            acc.extend([x]);
                        } else {
                            acc.extend([TokenStream::from_str(" || ").unwrap(), x]);
                        }
                        acc
                    });

                let gen = quote! {
                    impl Outcoming for #name {
                        const IS_NEED_READ: bool = #need_read;

                        #[cfg(not(feature = "std"))]
                        fn args(&self, args: &mut alloc::vec::Vec<u32>) -> Result<(), wa_proto::ProtocolError> {
                            args.push(self.get_primitive_enum() as u32);
                            match self {
                                #(#args_in_items)*
                            }
                            Ok(())
                        }

                        #[cfg(feature = "std")]
                        fn read(heap: &[u8], args: &mut core::slice::Iter<u32>) -> Result<Self, wa_proto::ProtocolError> {
                            let val = *args.next().ok_or(wa_proto::ProtocolError("args is end".to_string()))?;
                            let pt: #primitive_name = FromPrimitive::from_u32(val).ok_or(wa_proto::ProtocolError("#name enum type error".to_string()))?;
                            let t = match pt {
                                #(#read_items)*
                            };
                            Ok(t)
                        }
                    }
                };

                proc_macro::TokenStream::from(gen)
            }
        }
        Data::Union(_) => {
            panic!("unions not supported, but Rust enums is implemented Incoming trait (use Enums instead)")
        }
    }
}
