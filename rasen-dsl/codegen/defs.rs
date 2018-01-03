//! Type metadata providers

use quote::Ident;

const INTS: [(&'static str, &'static str, &'static str); 3] = [
    ("Bool", "b", "bool"),
    ("Int", "i", "i32"),
    ("UInt", "u", "u32"),
];
const FLOATS: [(&'static str, &'static str, &'static str); 2] = [
    ("Float", "", "f32"),
    ("Double", "d", "f64"),
];

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Category {
    SCALAR,
    VECTOR,
    MATRIX,
}

#[derive(Clone)]
pub struct Type {
    pub name: Ident,
    pub category: Category,
    pub component: Option<Box<Type>>,
    pub size: Option<u32>,
    pub ty: &'static str,
}

fn scalar_type(name: &str, ty: &'static str) -> Type {
    Type {
        name: Ident::from(name),
        category: Category::SCALAR,
        component: None,
        size: None,
        ty: ty,
    }
}

fn vector_type(name: &str, scalar: &str, ty: &'static str, size: u32) -> Type {
    Type {
        name: Ident::from(name),
        category: Category::VECTOR,
        component: Some(box scalar_type(scalar, ty)),
        size: Some(size),
        ty: ty,
    }
}

fn matrix_type(name: &str, vec: &str, scalar: &str, ty: &'static str, size: u32) -> Type {
    Type {
        name: Ident::from(name),
        category: Category::MATRIX,
        component: Some(box vector_type(vec, scalar, ty, size)),
        size: Some(size),
        ty: ty,
    }
}

pub fn all_types() -> Vec<Type> {
    let mut res = Vec::new();

    for &(res_name, _, ty) in INTS.iter().chain(FLOATS.iter()) {
        res.push(scalar_type(res_name, ty));
    }

    for size in 2u32..=4u32 {
        for &(scalar_name, prefix, ty) in &INTS {
            let vec_name = format!("{}Vec{}", prefix.to_string().to_uppercase(), size);
            res.push(vector_type(&vec_name, scalar_name, ty, size));
        }

        for &(scalar_name, prefix, ty) in &FLOATS {
            let vec_name = format!("{}Vec{}", prefix.to_string().to_uppercase(), size);
            res.push(vector_type(&vec_name, scalar_name, ty, size));

            let mat_name = format!("{}Mat{}", prefix.to_string().to_uppercase(), size);
            res.push(matrix_type(&mat_name, &vec_name, scalar_name, ty, size));
        }
    }

    res
}

#[derive(Clone)]
pub struct Node {
    pub name: Ident,
    pub args: Option<Vec<Type>>,
    pub result: Type,
}

pub fn all_nodes() -> Vec<Node> {
    let mut res = Vec::new();

    for result in all_types() {
        res.push(Node {
            name: result.name.clone(),
            args: None,
            result: result.clone(),
        });

        res.push(Node {
            name: Ident::from("Value"),
            args: Some(vec![
                result.clone(),
            ]),
            result: result.clone(),
        });
    }

    res
}
