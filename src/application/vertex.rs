use memoffset::offset_of;
use crate::render_core::mesh::{ToMesh, VertexAttribute};

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

    fn get_position(&self) -> nglm::Vec3 {
        let position_data = std::ptr::addr_of!(self.position.data);
        let data = unsafe{ std::ptr::read_unaligned(position_data) };
        let slice: [f32; 3] = data.0[0];
        nglm::vec3(slice[0], slice[1], slice[2])
    }
}

pub struct BasicMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    bounding_box: nglm::Mat3x2,
}

impl BasicMesh {
    pub fn with_capacities(vertex_cap: usize, index_cap: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_cap),
            indices: Vec::with_capacity(index_cap),
            bounding_box: nglm::zero(),
        }
    }

    #[allow(dead_code)]
    pub fn with_contents(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let bounding_box = calculate_bounding_box(&vertices);
        Self {
            vertices,
            indices,
            bounding_box,
        }
    }

    pub fn push_vertex(&mut self, vertex: Vertex) {
        self.bounding_box = incorporate_into_bounding_box(self.bounding_box, &vertex);
        self.vertices.push(vertex);
    }

    pub fn push_index(&mut self, index: u32) {
        self.indices.push(index);
    }
}

#[allow(dead_code)]
fn calculate_bounding_box(vertices: &Vec<Vertex>) -> nglm::Mat3x2 {
    let min_max = vertices.iter()
        .map(|v| (v.position, v.position))
        .reduce(|(a_min, a_max), (b0, b1)| {
            (nglm::vec3(a_min.x.min(b0.x), a_min.y.min(b0.y), a_min.z.min(b0.z)),
             nglm::vec3(a_max.x.max(b1.x), a_max.y.max(b1.y), a_max.z.max(b1.z)))
        })
        .unwrap_or((nglm::zero(), nglm::zero()));

    let bounds = nglm::zero();
    nglm::set_column(&bounds, 0, &min_max.0);
    nglm::set_column(&bounds, 1, &min_max.1);
    bounds
}

fn incorporate_into_bounding_box(bounding_box: nglm::Mat3x2, vertex: &Vertex) -> nglm::Mat3x2 {
    let min_bounds = bounding_box.column(0);
    let max_bounds = bounding_box.column(1);

    let position = vertex.get_position();

    nglm::mat3x2(
        min_bounds.x.min(position.x), max_bounds.x.max(position.x),
        min_bounds.y.min(position.y), max_bounds.y.max(position.y),
        min_bounds.z.min(position.z), max_bounds.z.max(position.z),
    )
}

impl ToMesh for BasicMesh {
    type Vertex = Vertex;

    fn get_attributes(&self) -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::new("position".to_owned(), 3, offset_of!(Vertex, position)),
            VertexAttribute::new("normal".to_owned(), 3, offset_of!(Vertex, normal)),
            VertexAttribute::new("color".to_owned(), 4, offset_of!(Vertex, color)),
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

    fn get_bounding_box(&self) -> nglm::Mat3x2 {
        self.bounding_box
    }
}
