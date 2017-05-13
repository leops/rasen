use quote::Ident;

// Metadata
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
    CONCRETE,
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

fn concrete_type(name: &'static str) -> Type {
    Type {
        name: Ident::from(name),
        category: Category::CONCRETE,
        component: None,
        size: None,
        ty: name,
    }
}

fn scalar_type(name: &str, ty: &'static str) -> Type {
    Type {
        name: Ident::from(name),
        category: Category::SCALAR,
        component: Some(box concrete_type(ty)),
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
        res.push(concrete_type(ty));
        res.push(scalar_type(res_name, ty));
    }

    for size in 2u32...4u32 {
        for &(scalar_name, prefix, ty) in INTS.iter() {
            let vec_name = format!("{}Vec{}", prefix.to_string().to_uppercase(), size);
            res.push(vector_type(&vec_name, scalar_name, ty, size));
        }

        for &(scalar_name, prefix, ty) in FLOATS.iter() {
            let vec_name = format!("{}Vec{}", prefix.to_string().to_uppercase(), size);
            res.push(vector_type(&vec_name, scalar_name, ty, size));

            let mat_name = format!("{}Mat{}", prefix.to_string().to_uppercase(), size);
            res.push(matrix_type(&mat_name, &vec_name, scalar_name, ty, size));
        }
    }

    res
}

pub const OPERATIONS: &[(&str, u32, &[&str], &str)] = &[
    ("Input", 0, &[], ""),
    ("Uniform", 0, &[], ""),
    ("Multiply", 4, &[], ""),
    ("Index", 2, &[], ""),
    ("Normalize", 1, &[ "S" ], "where T0: IntoValue<Output=R>, R: Vector<S>, S: Scalar"),
    ("Dot", 2, &[ "V" ], "where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R>, R: Scalar"),
    ("Clamp", 3, &[], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=R>"),
    ("Modulus", 2, &[], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar"),
    ("Cross", 2, &[ "S" ], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Vector<S>, S: Scalar"),
    ("Floor", 1, &[], "where T0: IntoValue<Output=R>, R: Scalar"),
    ("Ceil", 1, &[], "where T0: IntoValue<Output=R>, R: Scalar"),
    ("Round", 1, &[], "where T0: IntoValue<Output=R>, R: Scalar"),
    ("Sin", 1, &[], "where T0: IntoValue<Output=R>, R: Scalar"),
    ("Cos", 1, &[], "where T0: IntoValue<Output=R>, R: Scalar"),
    ("Tan", 1, &[], "where T0: IntoValue<Output=R>, R: Scalar"),
    ("Pow", 2, &[], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar"),
    ("Min", 2, &[], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar"),
    ("Max", 2, &[], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar"),
    ("Length", 1, &[ "V" ], "where T0: IntoValue<Output=V>, V: Vector<R>, R: Scalar"),
    ("Distance", 2, &[ "V" ], "where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R>, R: Scalar"),
    ("Reflect", 2, &[ "S" ], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Vector<S>, S: Scalar"),
    ("Refract", 3, &[ "S" ], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=S>, R: Vector<S>, S: Scalar"),
    ("Mix", 3, &[], "where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=R>")
];

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

        if result.category != Category::CONCRETE {
            res.push(Node {
                name: Ident::from("Value"),
                args: Some(vec![
                    result.clone(),
                ]),
                result: result.clone(),
            });
        }
    }

    res
}

pub fn single_node(name: &str) -> Vec<Node> {
    all_nodes().into_iter()
        .filter(|node| node.name == name)
        .collect()
}