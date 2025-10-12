use syn::{Data, Fields, TypeArray, TypePath};

macro_rules! push_strs {
    ( $result:ident, $( $x:expr ),* $(,)? ) => (
        {
            $(
                $result.push_str($x);
            )*
        }
    )
}

struct StructField {
    ident: String,
    ty: String,
}

impl StructField {
    pub fn new(ident: String, ty: String) -> Option<Self> {
        if ident.starts_with("_cpu_padding") {
            return None;
        }
        Some(Self { ident, ty })
    }
}

fn rust_to_wgsl_path(type_path: &TypePath) -> String {
    let name_seg = type_path.path.segments.last().unwrap();
    match name_seg.ident.to_string().as_str() {
        "u32" => "u32".into(),
        "i32" => "i32".into(),
        "f32" => "f32".into(),
        "bool" => "bool".into(),
        "Vec3" | "Vec3A" => "vec3<f32>".into(),
        "IVec3" | "IVec3A" => "vec3<i32>".into(),
        "Vec4" => "vec4<f32>".into(),
        "IVec4" => "vec4<i32>".into(),
        "Mat4" => "mat4x4<f32>".into(),
        other => other.into(),
    }
}

fn rust_to_wgsl_array(array: &TypeArray) -> String {
    let ty = rust_to_wgsl_type(&array.elem);
    let len_expr = &array.len;
    let len = quote::quote! { #len_expr }.to_string();
    format!("array<{}, {}>", ty, len)
}

fn rust_to_wgsl_type(ty: &syn::Type) -> String {
    let wgsl_code = match ty {
        syn::Type::Path(path) => rust_to_wgsl_path(path),
        syn::Type::Array(array) => rust_to_wgsl_array(array),
        _ => panic!("unsupported type: {:?}", ty),
    };
    wgsl_code
}

fn wgsl_struct(ident: String, fields: Vec<StructField>) -> String {
    let mut result = String::new();
    push_strs!(result, "struct ", &ident, " {\n");
    for field in fields {
        push_strs!(result, "\t", &field.ident, ": ", &field.ty, ",\n");
    }
    result.push('}');
    result
}

pub(crate) fn rust_to_wgsl_code(struct_input: &syn::DeriveInput) -> String {
    let struct_name = &struct_input.ident;
    let Data::Struct(data_struct) = &struct_input.data else {
        panic!("unsupported ShaderType struct");
    };
    let mut struct_fields: Vec<StructField> = Vec::new();

    match &data_struct.fields {
        Fields::Named(fields_named) => {
            for field in fields_named.named.iter() {
                let field_opt = StructField::new(
                    field.ident.as_ref().unwrap().to_string(),
                    rust_to_wgsl_type(&field.ty),
                );
                if let Some(field) = field_opt {
                    struct_fields.push(field);
                }
            }
        }
        _ => panic!("ShaderType must be a named struct"),
    }
    let wgsl_expanded = wgsl_struct(struct_name.to_string(), struct_fields);
    wgsl_expanded
}
