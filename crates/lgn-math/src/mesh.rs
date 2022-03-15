use crate::{Vec2, Vec3};

pub fn calculate_tangents(
    positions: &[Vec3],
    tex_coords: &[Vec2],
    indices: &Option<Vec<u16>>,
) -> Vec<Vec3> {
    let length = positions.len();
    let mut tangents = Vec::with_capacity(length);
    // let mut bitangents = Vec::with_capacity(length);
    tangents.resize(length, Vec3::default());
    // bitangents.resize(length, Vec3::default());

    let num_triangles = if let Some(indices) = &indices {
        indices.len() / 3
    } else {
        length / 3
    };

    for i in 0..num_triangles {
        let idx0 = if let Some(indices) = &indices {
            indices[i * 3] as usize
        } else {
            i * 3
        };
        let idx1 = if let Some(indices) = &indices {
            indices[i * 3 + 1] as usize
        } else {
            i * 3 + 1
        };
        let idx2 = if let Some(indices) = &indices {
            indices[i * 3 + 2] as usize
        } else {
            i * 3 + 2
        };
        let v0 = positions[idx0];
        let v1 = positions[idx1];
        let v2 = positions[idx2];

        let uv0 = tex_coords[idx0];
        let uv1 = tex_coords[idx1];
        let uv2 = tex_coords[idx2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let f = delta_uv1.y * delta_uv2.x - delta_uv1.x * delta_uv2.y;
        //let b = (delta_uv2.x * edge1 - delta_uv1.x * edge2) / f;
        let t = if f != 0_f32 {
            (delta_uv1.y * edge2 - delta_uv2.y * edge1) / f
        } else {
            Vec3::ZERO
        };

        tangents[idx0] = t;
        tangents[idx1] = t;
        tangents[idx2] = t;

        //bitangents[idx0] = b;
        //bitangents[idx1] = b;
        //bitangents[idx2] = b;
    }

    tangents
}
