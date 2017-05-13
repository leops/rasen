use quote::{Ident, Tokens};
use defs::OPERATIONS;

fn operation(name: &str, args: u32, adnl_generics: &[&str], constraints_str: &str) -> Tokens {
    let node = Ident::from(name);
    let fn_name = Ident::from(name.to_lowercase());
    let indices: Vec<u32> = (0..args).collect();
    let mut generics: Vec<_> = {
        indices.iter()
            .map(|i| Ident::from(format!("T{}", i)))
            .collect()
    };
    let args1: Vec<Ident> = {
        indices.iter()
            .map(|i| Ident::from(format!("arg_{}", i)))
            .collect()
    };
    let args2 = args1.clone();
    let arg_list: Vec<_> = {
        args1.iter()
            .zip(generics.iter())
            .map(|(arg, ty)| quote! { #arg: #ty })
            .collect()
    };
    let srcs1: Vec<_> = {
        args1.iter()
            .map(|arg| Ident::from(format!("{}_src", arg)))
            .collect()
    };
    let srcs2 = srcs1.clone();

    let mut constraints = Tokens::new();
    constraints.append(constraints_str);

    for gen in adnl_generics {
        generics.push(Ident::from(*gen));
    }

    let graph_opt = {
        args1.iter()
            .map(|arg| quote! { #arg.get_graph() })
            .fold(None, |root, item| match root {
                Some(tokens) => Some(quote! { #tokens.or(#item) }),
                None => Some(item),
            })
    };

    quote! {
        pub fn #fn_name< #( #generics , )* R >( #( #arg_list ),* ) -> Value<R> #constraints {
            if #( #args1.get_concrete().is_some() )&&* {
                println!("unimplemented {}", #name);
            }

            let graph_opt = #graph_opt;
            if let Some(graph_ref) = graph_opt {
                #( let #srcs1 = #args2.get_index(graph_ref.clone()); )*

                let index = {
                    let mut graph = graph_ref.borrow_mut();
                    let index = graph.add_node(Node::#node);
                    #( graph.add_edge(#srcs2, index, #indices); )*
                    index
                };

                return Value::Abstract {
                    graph: graph_ref.clone(),
                    index,
                    ty: PhantomData,
                };
            }

            unreachable!()
        }
    }
}

pub fn impl_operations() -> Vec<Tokens> {
    OPERATIONS.iter()
        .filter_map(|&(name, args, generics, constraints)| {
            if name == "Input" || name == "Uniform" || name == "Multiply" || name == "Index" {
                None
            } else {
                Some(operation(name, args, generics, constraints))
            }
        })
        .collect()
}