use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Text Gem".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(TextGem)
        .run();
}

pub struct TextGem;

impl TextGem {
    fn hello_world() {
        println!("Hello, world!")
    }

    fn startup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // Camera
        commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(200.0, 600.0, 200.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        });

        // Light
        commands.spawn(DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                1.0,
                -3.14 / 4.,
            )),
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            ..default()
        });

        // Cube
        commands.spawn(PbrBundle {
            mesh: meshes.add(shape::Cube::new(100.0).into()),
            material: materials.add(Color::BLUE.into()),
            ..Default::default()
        });
    }
}

impl Plugin for TextGem {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::hello_world)
            .add_systems(Startup, Self::startup);
    }
}
