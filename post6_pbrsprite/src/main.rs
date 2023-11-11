pub mod grid;
pub mod pbr_sprite;

use std::f32::consts::PI;

use bevy::{
    pbr::{self, ExtendedMaterial},
    prelude::*,
    render::{
        mesh::{MeshVertexAttributeId, VertexAttributeValues},
        texture::{ImageLoaderSettings, ImageSampler},
    },
};
use grid::GridMaterial;
use pbr_sprite::PbrSpriteMaterial;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Text Gem".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(grid::GridPlugin)
        .add_plugins(pbr_sprite::PbrSpritePlugin)
        .add_systems(Startup, init_scene)
        .run();
}

fn init_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut grid_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GridMaterial>>>,
    mut pbr_sprite_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, pbr_sprite::PbrSpriteMaterial>>,
    >,
) {
    let graph_blue = Color::rgba(0.19, 0.51, 1.0, 1.0);
    let light_grey = Color::rgba(0.9, 0.9, 0.92, 1.0);

    // Setup CameraTarget
    let mut camera_target = grid::CameraTarget::default()
        .looking_at(Vec3::new(0.0, 0.0, 0.0))
        .with_up(Vec3::Y)
        .with_bounding_box(grid::BoundingBox::new(
            Vec3::new(-3000.0, 15.0, -3000.0),
            Vec3::new(3000.0, 4000.0, 3000.0),
        ))
        .rotating(PI / -4.0);

    let min_x = 100.0;
    let max_x = 1000.0;
    let min_y = 30.0;
    let max_y = 4000.0;
    let num_steps = 300;
    for i in 0..num_steps {
        let progress = i as f32 / num_steps as f32;
        let y = min_y + (max_y - min_y) * progress;
        let x = min_x + (max_x - min_x) * progress;

        camera_target.add_zoom_level_offset(Vec3::new(x, y, 0.0))
    }

    // Camera
    commands.spawn((Camera3dBundle::default(), camera_target));

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(100.0, 100.0, 100.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 30000.0,
            ..default()
        },
        ..default()
    });

    // Grid Box
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(
            grid::GridBox {
                size: Vec3::new(6000.0, 30.0, 6000.0),
                subdivisions: UVec3::new(200, 0, 200),
            }
            .into(),
        ),
        material: grid_materials.add(ExtendedMaterial {
            base: light_grey.into(),
            extension: GridMaterial {
                color: graph_blue.into(),
                subdivisions: UVec2::new(0, 0),
                line_widths: Vec2::new(0.01, 0.01),
            },
        }),
        ..Default::default()
    });

    // PBR Sprite
    let image: Handle<Image> =
        asset_server.load_with_settings("goombah.png", |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(
            pbr_sprite::PaperSprite(pbr_sprite::QuadSprite::new(Vec2::new(32.0, 32.0))).into(),
        ),
        material: pbr_sprite_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(image),
                alpha_mode: AlphaMode::Mask(0.2),
                ..Default::default()
            },
            extension: pbr_sprite::PbrSpriteMaterial {
                uv_scale: Vec2::new(1.0, 1.0),
                uv_translate: Vec2::new(0.0, 0.0),
                outline_thickness: 0.05,
                outline_color: Color::WHITE,
            },
        }),
        transform: Transform::from_xyz(0.0, 30.0, 0.0),
        ..Default::default()
    });
}
