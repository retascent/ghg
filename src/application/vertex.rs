use memoffset::offset_of;
use crate::render_core::camera::Camera;
use crate::render_core::mesh::{ToMesh, VertexAttribute};

use crate::utils::prelude::*;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Vertex {
    position: nglm::Vec3,
    normal: nglm::Vec3,
    color: nglm::Vec4,
}

impl Vertex {
    pub fn from_vecs(position: nglm::Vec3, normal: nglm::Vec3, color: nglm::Vec4) -> Self {
        Self {
            position,
            normal,
            color,
        }
    }

    pub fn get_position(&self) -> nglm::Vec3 {
        let position_data = std::ptr::addr_of!(self.position.data);
        let data = unsafe { std::ptr::read_unaligned(position_data) };
        let slice: [f32; 3] = data.0[0];
        nglm::vec3(slice[0], slice[1], slice[2])
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, position: nglm::Vec3) {
        let position_data = std::ptr::addr_of_mut!(self.position.data);
        unsafe { std::ptr::write_unaligned(position_data, position.data) };
    }
}

pub struct BasicMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    bounding_box: Option<nglm::Mat3x2>,
    center: Option<nglm::Vec3>,
}

impl BasicMesh {
    pub fn with_capacities(vertex_cap: usize, index_cap: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_cap),
            indices: Vec::with_capacity(index_cap),
            bounding_box: None,
            center: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_contents(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let (bounding_box, center) = calculate_bounding_box_and_center(&vertices);
        Self {
            vertices,
            indices,
            bounding_box,
            center,
        }
    }

    pub fn push_vertex(&mut self, vertex: Vertex) {
        let (bounding_box, center) = incorporate_into_bounding_box(self.bounding_box, &vertex);
        self.bounding_box = Some(bounding_box);
        self.center = Some(center);
        self.vertices.push(vertex);
    }

    pub fn push_index(&mut self, index: u32) {
        self.indices.push(index);
    }

    // pub fn vertices_mut(&mut self) -> &mut Vec<Vertex> {
    //     &mut self.vertices
    // }
}

#[allow(dead_code)]
fn calculate_bounding_box_and_center(vertices: &Vec<Vertex>) -> (Option<nglm::Mat3x2>, Option<nglm::Vec3>) {
    if vertices.is_empty() {
        ghg_log!("Returning early");
        return (None, None);
    }

    let (min, max) = vertices.iter()
        .map(|v| (v.position, v.position))
        .reduce(|(a_min, a_max), (b0, b1)| {
            (nglm::vec3(a_min.x.min(b0.x), a_min.y.min(b0.y), a_min.z.min(b0.z)),
             nglm::vec3(a_max.x.max(b1.x), a_max.y.max(b1.y), a_max.z.max(b1.z)))
        })
        .expect("Bad assumptions!");

    ghg_log!("Min/max: {}/{}", min, max);

    let mut bounds: nglm::Mat3x2 = nglm::zero();
    bounds.set_column(0, &min);
    bounds.set_column(1, &max);

    ghg_log!("Bounds: {}", bounds);
    let center = center_of_bounding_box(&bounds);
    ghg_log!("Center: {}", center);
    (Some(bounds), Some(center))
}

fn incorporate_into_bounding_box(bounding_box: Option<nglm::Mat3x2>, vertex: &Vertex) -> (nglm::Mat3x2, nglm::Vec3) {
    if bounding_box.is_some() {
        let bounds = bounding_box.unwrap();
        let min_bounds = bounds.column(0);
        let max_bounds = bounds.column(1);

        let position = vertex.get_position();

        let new_bounds = nglm::mat3x2(
            min_bounds.x.min(position.x), max_bounds.x.max(position.x),
            min_bounds.y.min(position.y), max_bounds.y.max(position.y),
            min_bounds.z.min(position.z), max_bounds.z.max(position.z),
        );

        let center = center_of_bounding_box(&new_bounds);

        (new_bounds, center)
    } else {
        let vertex_position = vertex.get_position();
        let mut bounds: nglm::Mat3x2 = nglm::zero();
        bounds.set_column(0, &vertex_position);
        bounds.set_column(1, &vertex_position);

        (bounds, vertex_position)
    }
}

fn center_of_bounding_box(bounds: &nglm::Mat3x2) -> nglm::Vec3 {
    nglm::vec3(
        bounds.m11 + (bounds.m12 - bounds.m11) / 2.0,
        bounds.m21 + (bounds.m22 - bounds.m21) / 2.0,
        bounds.m31 + (bounds.m32 - bounds.m31) / 2.0,
    )
}

impl ToMesh for BasicMesh {
    type Vertex = Vertex;

    fn get_attributes(&self) -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::new("position", 3, offset_of!(Vertex, position)),
            VertexAttribute::new("normal", 3, offset_of!(Vertex, normal)),
            VertexAttribute::new("color", 4, offset_of!(Vertex, color)),
        ]
    }

    fn get_flat_vertex_buffer(&self) -> &[f32] {
        unsafe {
            let (prefix, floats, suffix) = self.vertices.align_to::<f32>();
            if !prefix.is_empty() || !suffix.is_empty() {
                panic!("Vertex type has bad alignment; fix it or clone it into a flat vector instead");
            }
            floats
        }
    }

    fn get_flat_index_buffer(&self) -> &[u32] {
        &self.indices
    }

    fn get_bounding_box(&self) -> Option<nglm::Mat3x2> {
        self.bounding_box
    }

    fn get_center(&self) -> Option<nglm::Vec3> {
        self.center
    }

    // TODO: This is sphere-specific code, so I need to implement Sphere as its own mesh collection.
    fn is_visible(&self, camera: &Camera) -> bool {
        // true
        let mesh_center = self.center;
        if mesh_center.is_none() {
            ghg_error!("Invalid mesh!");
            return false;
        }
        let camera_position = camera.position();
        let dist = camera_position.metric_distance(&mesh_center.unwrap());
        let camera_from_origin = camera_position.magnitude();

        dist <= camera_from_origin
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::prelude::*;
    use wasm_bindgen_test::*;
    use crate::application::sphere::*;
    use crate::application::vertex::{BasicMesh, Vertex};
    use crate::render_core::mesh::ToMesh;

    #[wasm_bindgen_test]
    fn origin_mesh() {
        let difference_epsilon = 0.001f32;
        let position = nglm::vec3(0.0, 0.0, 0.0);

        let origin = Vertex::from_vecs(position.clone(),
                                       nglm::vec3(0.0, 0.0, 0.0),
                                       nglm::vec4(0.0, 0.0, 0.0, 1.0));

        let mesh = BasicMesh::with_contents(vec![origin], vec![0]);

        assert!(mesh.center.unwrap().metric_distance(&position) <= difference_epsilon);
    }

    #[wasm_bindgen_test]
    fn non_origin_mesh() {
        let difference_epsilon = 0.001f32;
        let position = nglm::vec3(1.0, 1.0, 1.0);

        let origin = Vertex::from_vecs(position.clone(),
                                       nglm::vec3(0.0, 0.0, 0.0),
                                       nglm::vec4(0.0, 0.0, 0.0, 1.0));

        let mesh = BasicMesh::with_contents(vec![origin], vec![0]);

        ghg_log!("Center: {}", mesh.center.unwrap());

        assert!(mesh.center.unwrap().metric_distance(&position) <= difference_epsilon);
    }

    #[wasm_bindgen_test]
    fn center() {
        let cube_meshes = generate_sphere(1, 2); // Actually a cube

        for mesh in cube_meshes {
            let vertices = &mesh.vertices;
            let num_vertices = vertices.len() as f32;
            let difference_epsilon = 0.001f32;
            let sum_minimum = 1.0f32;

            let sum_of_x_coords: f32 = vertices.iter().map(|v| v.get_position().x).sum();
            let sum_of_y_coords: f32 = vertices.iter().map(|v| v.get_position().y).sum();
            let sum_of_z_coords: f32 = vertices.iter().map(|v| v.get_position().z).sum();

            ghg_log!("Sum vector: ({}, {}, {})", sum_of_x_coords, sum_of_y_coords, sum_of_z_coords);

            let x_comp = if sum_of_x_coords.abs() >= sum_minimum { 1.0 } else { 0.0 };
            let y_comp = if sum_of_y_coords.abs() >= sum_minimum { 1.0 } else { 0.0 };
            let z_comp = if sum_of_z_coords.abs() >= sum_minimum { 1.0 } else { 0.0 };

            let face_normal = nglm::vec3(x_comp, y_comp, z_comp);

            ghg_log!("Face normal {:?}, mesh center {:?}", face_normal, mesh.center.unwrap());

            assert!(face_normal.metric_distance(&mesh.center.unwrap()) <= difference_epsilon);
        }
    }
}
