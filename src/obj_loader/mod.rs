use tobj;
use find_folder::Search;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32)
}

impl_vertex!(Vertex, position);

#[derive(Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32)
}

impl_vertex!(Normal, normal);

pub struct VertexIndicesNormals {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub normals: Vec<Normal>
}

pub fn load_model(name: &str) -> VertexIndicesNormals {
    let mut path = Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    path.push(name);

    let asset = tobj::load_obj(&path);
    assert!(asset.is_ok());

    let (models, _) = asset.unwrap();

    let mesh = &models[0].mesh;

    println!("model.name = \'{}\'", models[0].name);

    let indices = mesh.indices.clone();
    let mut vertices: Vec<Vertex> = Vec::with_capacity(mesh.positions.len() / 3);
    for i in 0..mesh.positions.len() / 3 {
        vertices.push(Vertex{ position: (mesh.positions[3 * i], mesh.positions[3 * i + 1], mesh.positions[3 * i + 2]) });
    }

    let mut normals: Vec<Normal> = Vec::with_capacity(mesh.normals.len() / 3);
    for i in 0..mesh.normals.len() / 3 {
        normals.push(Normal{ normal: (mesh.normals[3 * i], mesh.normals[3 * i + 1], mesh.normals[3 * i + 2]) });
    }

    println!("#Vertices {}, #Indices {}, #Normals {}", vertices.len(), indices.len(), normals.len());

    VertexIndicesNormals {indices: indices, normals: normals, vertices: vertices }
}