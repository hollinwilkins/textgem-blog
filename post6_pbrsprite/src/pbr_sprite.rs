use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
};

#[derive(Debug, Default, Clone, Asset, TypePath, AsBindGroup)]
pub struct PbrSpriteMaterial {
    #[uniform(200)]
    pub sprite_draw_rect: Vec4,
    #[texture(201)]
    #[sampler(202)]
    #[dependency]
    pub sprite_texture: Option<Handle<Image>>,
}

impl MaterialExtension for PbrSpriteMaterial {
    fn fragment_shader() -> ShaderRef {
        "pbr_sprite.wgsl".into()
    }
}

pub struct QuadSprite {
    size: Vec2,
    flip: bool,
}

impl QuadSprite {
    pub fn new(size: Vec2) -> Self {
        Self { size, flip: false }
    }

    pub fn flipped(mut self) -> Self {
        self.flip = true;
        self
    }
}

impl From<QuadSprite> for Mesh {
    fn from(quad: QuadSprite) -> Self {
        let extent_x = quad.size.x / 2.0;
        let extent_y = quad.size.y / 2.0;

        let (u_left, u_right) = if quad.flip { (1.0, 0.0) } else { (0.0, 1.0) };
        let vertices = [
            ([-extent_x, -extent_y, 0.0], [0.0, 0.0, 1.0], [u_left, 1.0]),
            ([-extent_x, extent_y, 0.0], [0.0, 0.0, 1.0], [u_left, 0.0]),
            ([extent_x, extent_y, 0.0], [0.0, 0.0, 1.0], [u_right, 0.0]),
            ([extent_x, -extent_y, 0.0], [0.0, 0.0, 1.0], [u_right, 1.0]),
        ];

        let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2, 2, 0, 1, 3, 0, 2]);

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        Mesh::new(PrimitiveTopology::TriangleList)
            .with_indices(Some(indices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

pub struct PbrSpritePlugin;

impl Plugin for PbrSpritePlugin {
    fn build(&self, app: &mut App) {
        let mut material_plugin =
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, PbrSpriteMaterial>>::default();
        material_plugin.prepass_enabled = false;

        app.add_plugins(material_plugin);
    }
}
