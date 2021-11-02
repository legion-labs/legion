use generic_data_runtime::TestEntity;
use legion_math::prelude::*;

#[test]
fn test_default_implementation() {
    let entity = TestEntity {
        ..Default::default()
    };

    assert_eq!(entity.test_string, "string literal");
    assert_eq!(entity.test_position, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(entity.test_rotation, Quat::IDENTITY);
    assert!(!entity.test_bool);
    assert!((entity.test_float32 - 32.32f32).abs() < f32::EPSILON);
    assert_eq!(entity.test_int, 123);
    assert_eq!(entity.test_blob, vec![0, 1, 2, 3]);
}
