use vulkano::image::{AttachmentImage, ImmutableImage};
use vulkano::format;

use obj_loader::Model;

struct Textures {
    environment_cube: ImmutableImage<format::R16G16B16A16Sfloat>,
    lut_brdf: AttachmentImage,
    irradience_cube: AttachmentImage,
    prefiltered_cube: AttachmentImage,
    albedo_map: ImmutableImage<format::R8G8B8A8Unorm>,
    normal_map: ImmutableImage<format::R8G8B8A8Unorm>,
    ao_map: ImmutableImage<format::R8Unorm>,
    metallic_map: ImmutableImage<format::R8Unorm>,
    roughness_map: ImmutableImage<format::R8Unorm>
}

struct Mesh {
    object: Model,
    skybox: Model
}

