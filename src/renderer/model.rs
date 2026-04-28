use gltf::{Gltf, json::material, mesh::util::indices};
use super::{Mesh, Texture2D};

pub struct Model {
    pub meshes: Vec<Mesh>,
}

impl Model {
    pub fn load(path: &str) -> Self {
        let (doc, buffers, images) = gltf::import(path)
            .expect("Failed to load glTF model");

    let mut meshes = Vec::new();

        for mesh in doc.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|b| Some(&buffers[b.index()]));

                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .expect("Mesh has no positions")
                    .collect();

                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .expect("Mesh has no normals")
                    .collect();

                let tex_coords: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .expect("Mesh has no texture coordinates")
                    .into_f32()
                    .collect();

                let indices : Vec<u32> = reader
                    .read_indices()
                    .expect("Mesh has no indices")
                    .into_u32()
                    .collect();

                let tangents = reader
                    .read_tangents()
                    .map(|tangent_iter| tangent_iter.collect());

                let tangents = tangents.unwrap_or_else(|| {
                    compute_tangents(&positions, &normals, &tex_coords, &indices)
                });

                let mut vertices = Vec::with_capacity(positions.len() * 12);
                for i in 0..positions.len() {
                    vertices.extend_from_slice(&positions[i]);
                    vertices.extend_from_slice(&normals[i]);
                    vertices.extend_from_slice(&tangents[i]);
                    vertices.extend_from_slice(&tex_coords[i]);
                }

                let material = primitive.material();
                let pbr = material.pbr_metallic_roughness();

                let albedo = Texture2D::from_gltf(
                    pbr.base_color_texture()
                        .map(|info| &images[info.texture().source().index()]),
                    true,
                );
                let normal = Texture2D::from_gltf(
                    material.normal_texture()
                        .map(|info| &images[info.texture().source().index()]),
                    false,
                );
                let orm = Texture2D::from_gltf(
                    pbr.metallic_roughness_texture()
                        .map(|info| &images[info.texture().source().index()]),
                    false,
                );

                meshes.push(Mesh::new(
                    &vertices,
                    &indices,
                    albedo,
                    normal,
                    orm,
                ));

                
            }
        }

        Self { meshes }
    }

    pub fn draw(&self) {
        for mesh in &self.meshes {
            mesh.draw();
        }
    }
}


fn compute_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> Vec<[f32; 4]> {
    let vertex_count = positions.len();

    let mut tan1 = vec![[0.0f32; 3]; vertex_count];
    let mut tan2 = vec![[0.0f32; 3]; vertex_count];

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];

        let uv0 = uvs[i0];
        let uv1 = uvs[i1];
        let uv2 = uvs[i2];

        let x1 = p1[0] - p0[0];
        let y1 = p1[1] - p0[1];
        let z1 = p1[2] - p0[2];

        let x2 = p2[0] - p0[0];
        let y2 = p2[1] - p0[1];
        let z2 = p2[2] - p0[2];

        let s1 = uv1[0] - uv0[0];
        let t1 = uv1[1] - uv0[1];

        let s2 = uv2[0] - uv0[0];
        let t2 = uv2[1] - uv0[1];

        let r = 1.0 / (s1 * t2 - s2 * t1);

        let sdir = [
            (t2 * x1 - t1 * x2) * r,
            (t2 * y1 - t1 * y2) * r,
            (t2 * z1 - t1 * z2) * r,
        ];

        let tdir = [
            (s1 * x2 - s2 * x1) * r,
            (s1 * y2 - s2 * y1) * r,
            (s1 * z2 - s2 * z1) * r,
        ];

        for &i in &[i0, i1, i2] {
            tan1[i][0] += sdir[0];
            tan1[i][1] += sdir[1];
            tan1[i][2] += sdir[2];

            tan2[i][0] += tdir[0];
            tan2[i][1] += tdir[1];
            tan2[i][2] += tdir[2];
        }
    }

    let mut tangents = vec![[0.0f32; 4]; vertex_count];

    for i in 0..vertex_count {
        let n = normals[i];
        let t = tan1[i];

        // Gram-Schmidt orthogonalization
        let dot_nt = n[0] * t[0] + n[1] * t[1] + n[2] * t[2];

        let mut tangent = [
            t[0] - n[0] * dot_nt,
            t[1] - n[1] * dot_nt,
            t[2] - n[2] * dot_nt,
        ];

        // Normalize
        let len = (tangent[0] * tangent[0]
            + tangent[1] * tangent[1]
            + tangent[2] * tangent[2])
            .sqrt();

        if len > 0.0 {
            tangent[0] /= len;
            tangent[1] /= len;
            tangent[2] /= len;
        }

        // Handedness
        let cross = [
            n[1] * tangent[2] - n[2] * tangent[1],
            n[2] * tangent[0] - n[0] * tangent[2],
            n[0] * tangent[1] - n[1] * tangent[0],
        ];

        let handedness = if (cross[0] * tan2[i][0]
            + cross[1] * tan2[i][1]
            + cross[2] * tan2[i][2])
            < 0.0
        {
            -1.0
        } else {
            1.0
        };

        tangents[i] = [tangent[0], tangent[1], tangent[2], handedness];
    }

    tangents
}