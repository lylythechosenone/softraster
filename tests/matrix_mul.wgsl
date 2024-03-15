fn test_matrix_matrix_mul(lhs: mat2x3<f32>, rhs: mat3x2<f32>) -> mat3x3<f32> {
    return lhs * rhs;
}

fn test_ptr_matrix_matrix_mul(lhs: mat2x3<f32>, rhs: mat3x2<f32>) -> mat3x3<f32> {
    var lhs_ptr = lhs;
    var rhs_ptr = rhs;
    return lhs_ptr * rhs_ptr;
}

fn test_matrix_vector_mul(lhs: mat3x3<f32>, rhs: vec3<f32>) -> vec3<f32> {
    return lhs * rhs;
}

fn test_vector_matrix_mul(lhs: vec3<f32>, rhs: mat3x3<f32>) -> vec3<f32> {
    return lhs * rhs;
}
