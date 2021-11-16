struct StaticMesh {
    vertices: Vec<(f32, f32, f32)>, //TODO: vector type?
}

impl StaticMesh {
    fn new_cube(size: f32) {
        let cube = StaticMesh {
            vertices: Vec::with_capacity(36),
        };
        let half_size = size / 2.;
        cube.vertices.push((half_size, half_size, half_size));
    }
}
