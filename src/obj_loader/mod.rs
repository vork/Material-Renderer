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

type Coordinate = (f32, f32);
#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub x: Coordinate,
    pub y: Coordinate,
    pub z: Coordinate
}

pub fn load_model(name: &str) -> (VertexIndicesNormals, Bounds) {
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

    let mut x: Coordinate = (vertices[0].position.0, vertices[0].position.0);
    let mut y: Coordinate = (vertices[0].position.1, vertices[0].position.1);
    let mut z: Coordinate = (vertices[0].position.2, vertices[0].position.2);

    let vtx = vertices.clone();
    for v in vtx {
        if v.position.0 < x.0 {
            x.0 = v.position.0;
        }
        if v.position.0 > x.1 {
            x.1 = v.position.0;
        }

        if v.position.1 < y.0 {
            y.0 = v.position.1;
        }
        if v.position.1 > y.1 {
            y.1 = v.position.1;
        }

        if v.position.2 < z.0 {
            z.0 = v.position.2;
        }
        if v.position.2 > z.1 {
            z.1 = v.position.2;
        }
    }

    let bounds = Bounds{ x: x, y: y, z: z };

    println!("#Vertices {}, #Indices {}, #Normals {}", vertices.len(), indices.len(), normals.len());

    (VertexIndicesNormals {indices: indices, normals: normals, vertices: vertices }, bounds)
}