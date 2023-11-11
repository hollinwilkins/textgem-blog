pub mod grid;
pub mod pbr_sprite;

use std::f32::consts::PI;

use bevy::{
    pbr::{self, ExtendedMaterial},
    prelude::*,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut grid_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GridMaterial>>>,
    mut pbr_sprite_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, pbr_sprite::PbrSpriteMaterial>>,
    >,
) {
    // Setup CameraTarget
    let mut camera_target = grid::CameraTarget::default()
        .looking_at(Vec3::new(0.0, 0.0, 0.0))
        .with_up(Vec3::Y)
        .with_bounding_box(grid::BoundingBox::new(
            Vec3::new(-3000.0, 15.0, -3000.0),
            Vec3::new(3000.0, 4000.0, 3000.0),
        ));

    let min_x = 500.0;
    let max_x = 1200.0;
    let min_y = 100.0;
    let max_y = 4000.0;
    let num_steps = 100;
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
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
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
            base: Color::BLUE.into(),
            extension: GridMaterial {
                color: Color::ORANGE,
                subdivisions: UVec2::new(0, 0),
                line_widths: Vec2::new(0.01, 0.01),
            },
        }),
        ..Default::default()
    });

    // PBR Sprite
    let pbr_sprite_transform = Transform::from_xyz(0.0, 100.0, 0.0);
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(pbr_sprite::QuadSprite::new(Vec2::new(32.0, 32.0)).into()),
        material: pbr_sprite_materials.add(ExtendedMaterial {
            base: Color::WHITE.into(),
            extension: PbrSpriteMaterial::default(),
        }),
        transform: pbr_sprite_transform,
        ..Default::default()
    });
}
