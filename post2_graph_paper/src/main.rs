use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
};

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
        mut grid_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GridMaterial>>>,
    ) {
        // Camera
        commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(400.0, 100.0, 400.0)
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

        // Graph Paper
        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(
                GridPlane(shape::Plane {
                    size: 3000.0,
                    subdivisions: 100,
                })
                .into(),
            ),
            material: grid_materials.add(ExtendedMaterial {
                base: Color::BLUE.into(),
                extension: GridMaterial {
                    color: Color::ORANGE,
                    subdivisions: Vec2::new(1.0, 1.0),
                    line_widths: Vec2::new(0.01, 0.01),
                },
            }),
            ..Default::default()
        });
    }
}

impl Plugin for TextGem {
    fn build(&self, app: &mut App) {
        let mut material_plugin =
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, GridMaterial>>::default();
        material_plugin.prepass_enabled = false;

        app.add_plugins(material_plugin)
            .add_systems(Startup, Self::hello_world)
            .add_systems(Startup, Self::startup);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(100)]
    color: Color,
    #[uniform(101)]
    subdivisions: Vec2,
    #[uniform(102)]
    line_widths: Vec2,
}

impl MaterialExtension for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "grid_material.wgsl".into()
    }
}

pub struct GridPlane(shape::Plane);

impl From<GridPlane> for Mesh {
    fn from(value: GridPlane) -> Self {
        let plane = value.0;

        // here this is split in the z and x directions if one ever needs asymmetrical subdivision
        // two Plane struct fields would need to be added instead of the single subdivisions field
        let z_vertex_count = plane.subdivisions + 2;
        let x_vertex_count = plane.subdivisions + 2;
        let num_vertices = (z_vertex_count * x_vertex_count) as usize;
        let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;
        let up = Vec3::Y.to_array();

        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        for z in 0..z_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;
                let ux = (x % 2) as f32;
                let uz = (z % 2) as f32;
                positions.push([(-0.5 + tx) * plane.size, 0.0, (-0.5 + tz) * plane.size]);
                normals.push(up);
                uvs.push([ux, uz]);
            }
        }

        for y in 0..z_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = y * x_vertex_count + x;
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad);
                indices.push(quad + x_vertex_count);
                indices.push(quad + 1);
            }
        }

        Mesh::new(PrimitiveTopology::TriangleList)
            .with_indices(Some(Indices::U32(indices)))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

pub struct GridBox {
    size: Vec3,
    subdivisions: UVec3,
}

impl From<GridBox> for Mesh {
    fn from(value: GridBox) -> Self {
        let x_vertex_count = value.subdivisions.x + 2;
        let y_vertex_count = value.subdivisions.y + 2;
        let z_vertex_count = value.subdivisions.z + 2;

        let num_vertices = (((z_vertex_count * x_vertex_count)
            + (z_vertex_count * y_vertex_count)
            + (x_vertex_count * y_vertex_count))
            * 2) as usize;
        let num_indices = ((((z_vertex_count - 1) * (x_vertex_count - 1))
            + ((z_vertex_count - 1) * (y_vertex_count - 1))
            + ((x_vertex_count - 1) * (y_vertex_count - 1)))
            * 6
            * 2) as usize;
        let x_up = Vec3::X.to_array();
        let x_down = (Vec3::X * -1.0).to_array();
        let y_up = Vec3::Y.to_array();
        let y_down = (Vec3::Y * -1.0).to_array();
        let z_up = Vec3::Z.to_array();
        let z_down = (Vec3::Z * -1.0).to_array();

        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);
        let mut index_offset: u32 = 0;

        // Front Mesh
        for z in 0..z_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let ty = 1.0 as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;
                let ux = (x % 2) as f32;
                let uz = (z % 2) as f32;
                positions.push([
                    (-0.5 + tx) * value.size.x,
                    (-0.5 + ty) * value.size.y,
                    (-0.5 + tz) * value.size.z,
                ]);
                normals.push(y_up);
                uvs.push([ux, uz]);
            }
        }

        // Front Indices
        for z in 0..z_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = z * x_vertex_count + x;
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad);
                indices.push(quad + x_vertex_count);
                indices.push(quad + 1);
            }
        }

        // Back Mesh
        index_offset = positions.len() as u32;
        for z in 0..z_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let ty = 0.0 as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;
                let ux = (x % 2) as f32;
                let uz = (z % 2) as f32;
                positions.push([
                    (-0.5 + tx) * value.size.x,
                    (-0.5 + ty) * value.size.y,
                    (-0.5 + tz) * value.size.z,
                ]);
                normals.push(y_down);
                uvs.push([ux, uz]);
            }
        }

        // Back Indices
        for z in 0..z_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = index_offset + z * x_vertex_count + x;
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad + x_vertex_count);
                indices.push(quad);
                indices.push(quad + 1);
            }
        }

        // Top Mesh
        index_offset = positions.len() as u32;
        for y in 0..y_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let ty = y as f32 / (y_vertex_count - 1) as f32;
                let tz = 1.0 as f32;
                let ux = (x % 2) as f32;
                let uy = (y % 2) as f32;
                positions.push([
                    (-0.5 + tx) * value.size.x,
                    (-0.5 + ty) * value.size.y,
                    (-0.5 + tz) * value.size.z,
                ]);
                normals.push(z_up);
                uvs.push([ux, uy]);
            }
        }

        // Top Indices
        for y in 0..y_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = index_offset + y * x_vertex_count + x;
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad + x_vertex_count);
                indices.push(quad);
                indices.push(quad + 1);
            }
        }

        // Bottom Mesh
        index_offset = positions.len() as u32;
        for y in 0..y_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let ty = y as f32 / (y_vertex_count - 1) as f32;
                let tz = 0.0 as f32;
                let ux = (x % 2) as f32;
                let uy = (y % 2) as f32;
                positions.push([
                    (-0.5 + tx) * value.size.x,
                    (-0.5 + ty) * value.size.y,
                    (-0.5 + tz) * value.size.z,
                ]);
                normals.push(z_down);
                uvs.push([ux, uy]);
            }
        }

        // Bottom Indices
        for y in 0..y_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = index_offset + y * x_vertex_count + x;
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad + x_vertex_count);
                indices.push(quad + 1);
                indices.push(quad);
            }
        }

        // Right Mesh
        index_offset = positions.len() as u32;
        for y in 0..y_vertex_count {
            for z in 0..z_vertex_count {
                let tx = 1.0 as f32;
                let ty = y as f32 / (y_vertex_count - 1) as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;
                let uz = (z % 2) as f32;
                let uy = (y % 2) as f32;
                positions.push([
                    (-0.5 + tx) * value.size.x,
                    (-0.5 + ty) * value.size.y,
                    (-0.5 + tz) * value.size.z,
                ]);
                normals.push(x_up);
                uvs.push([uz, uy]);
            }
        }

        // Right Indices
        for y in 0..y_vertex_count - 1 {
            for z in 0..z_vertex_count - 1 {
                let quad = index_offset + y * x_vertex_count + z;
                indices.push(quad + z_vertex_count + 1);
                indices.push(quad + 1);
                indices.push(quad + z_vertex_count);
                indices.push(quad + z_vertex_count);
                indices.push(quad + 1);
                indices.push(quad);
            }
        }

        // Left Mesh
        index_offset = positions.len() as u32;
        for y in 0..y_vertex_count {
            for z in 0..z_vertex_count {
                let tx = 0.0 as f32;
                let ty = y as f32 / (y_vertex_count - 1) as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;
                let uz = (z % 2) as f32;
                let uy = (y % 2) as f32;
                positions.push([
                    (-0.5 + tx) * value.size.x,
                    (-0.5 + ty) * value.size.y,
                    (-0.5 + tz) * value.size.z,
                ]);
                normals.push(x_down);
                uvs.push([uz, uy]);
            }
        }

        // Left Indices
        for y in 0..y_vertex_count - 1 {
            for z in 0..z_vertex_count - 1 {
                let quad = index_offset + y * x_vertex_count + z;
                indices.push(quad + 1);
                indices.push(quad + z_vertex_count + 1);
                indices.push(quad + z_vertex_count);
                indices.push(quad + 1);
                indices.push(quad + z_vertex_count);
                indices.push(quad);
            }
        }

        Mesh::new(PrimitiveTopology::TriangleList)
            .with_indices(Some(Indices::U32(indices)))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}
