use crate::application::vertex::{BasicMesh, Vertex};

#[allow(unused_imports)]
use crate::utils::prelude::*;

// Thanks, Sebastian Lague. https://www.youtube.com/watch?v=sLqXFF8mlEU
pub fn generate_sphere(subdivisions: u32, points_per_subdivision: u32) -> Vec<BasicMesh> {
    generate_sphere_with_position(subdivisions, points_per_subdivision, nglm::zero())
}

pub fn generate_sphere_with_position(subdivisions: u32,
                                     points_per_subdivision: u32,
                                     center: nglm::Vec3) -> Vec<BasicMesh> {
    let face_normals = vec![
        nglm::Vec3::ith(1,  1.0), // up
        nglm::Vec3::ith(1, -1.0), // down
        nglm::Vec3::ith(0,  1.0), // left
        nglm::Vec3::ith(0, -1.0), // right
        nglm::Vec3::ith(2,  1.0), // front
        nglm::Vec3::ith(2, -1.0), // back
    ];

    face_normals.iter()
        .map(|n| generate_face(n, subdivisions, points_per_subdivision, center.clone()))
        .flatten()
        .collect()
}

fn generate_face(normal: &nglm::Vec3,
                 subdivisions: u32,
                 points_per_subdivision: u32,
                 sphere_center: nglm::Vec3) -> Vec<BasicMesh> {
    let mut meshes = Vec::new();

    let subdivision_size = 1.0 / (subdivisions as f32);

    for y in 0..subdivisions {
        for x in 0..subdivisions {
            let subdivision_x = (x as f32) * subdivision_size;
            let subdivision_y = (y as f32) * subdivision_size;
            let subdivision_start = nglm::vec2(subdivision_x, subdivision_y);

            meshes.push(generate_subface(normal,
                                         points_per_subdivision,
                                         subdivision_start,
                                         subdivision_size,
                                         sphere_center.clone()));
        }
    }

    meshes
}

fn generate_subface(normal: &nglm::Vec3,
                    points_per_side: u32,
                    subdivision_start: nglm::Vec2,
                    subdivision_side_length: f32,
                    sphere_center: nglm::Vec3) -> BasicMesh {
    let face_color = determine_face_color(normal);

    let axis_a = nglm::vec3(normal.y, normal.z, normal.x);
    let axis_b = nglm::cross(normal, &axis_a);

    let mut mesh = BasicMesh::with_capacities((points_per_side * points_per_side) as usize,
                                              ((points_per_side - 1) * (points_per_side - 1) * 6) as usize);

    for y in 0..points_per_side {
        for x in 0..points_per_side {
            let t: nglm::Vec2 = nglm::vec2(
                (x as f32) / ((points_per_side as f32) - 1.0) * subdivision_side_length + subdivision_start.x,
                (y as f32) / ((points_per_side as f32) - 1.0) * subdivision_side_length + subdivision_start.y);

            let point: nglm::Vec3 = (normal + axis_a * (2.0 * t.x - 1.0) + axis_b * (2.0 * t.y - 1.0)).into();

            let sphere_point = point_on_cube_to_point_on_sphere(point) + sphere_center;
            let normal: nglm::Vec3 = sphere_point.normalize();

            mesh.push_vertex(Vertex::from_vecs(sphere_point, normal, face_color.clone()));

            let vertex_index = y * points_per_side + x;
            if x != points_per_side - 1 && y != points_per_side - 1 {
                mesh.push_index(vertex_index);
                mesh.push_index(vertex_index + points_per_side + 1);
                mesh.push_index(vertex_index + points_per_side);
                mesh.push_index(vertex_index);
                mesh.push_index(vertex_index + 1);
                mesh.push_index(vertex_index + points_per_side + 1);
            }
        }
    }

    mesh
}

fn point_on_cube_to_point_on_sphere(p: nglm::Vec3) -> nglm::Vec3 {
    let x2 = p.x * p.x;
    let y2 = p.y * p.y;
    let z2 = p.z * p.z;
    let x = p.x * (1.0 - (y2 + z2) / 2.0 + (y2 * z2) / 3.0).sqrt();
    let y = p.y * (1.0 - (z2 + x2) / 2.0 + (z2 * x2) / 3.0).sqrt();
    let z = p.z * (1.0 - (x2 + y2) / 2.0 + (x2 * y2) / 3.0).sqrt();

    nglm::vec3(x, y, z)
}

fn determine_face_color(_normal: &nglm::Vec3) -> nglm::Vec4 {
    let r = js_sys::Math::random() as f32;
    let g = js_sys::Math::random() as f32;
    let b = js_sys::Math::random() as f32;
    nglm::vec4(r, g, b, 1.0)
}