use bevy::{
    input::mouse::MouseWheel,
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
        // Setup CameraTarget
        let mut camera_target = CameraTarget::default()
            .looking_at(Vec3::new(0.0, 0.0, 0.0))
            .with_up(Vec3::Y);

        let min_x = 1000.0;
        let max_x = 1200.0;
        let min_y = 500.0;
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

        // Grid Box
        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(
                GridBox {
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
    }
}

impl Plugin for TextGem {
    fn build(&self, app: &mut App) {
        let mut material_plugin =
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, GridMaterial>>::default();
        material_plugin.prepass_enabled = false;

        app.add_plugins(material_plugin)
            .add_systems(Startup, Self::hello_world)
            .add_systems(Startup, Self::startup)
            .add_systems(Update, CameraTarget::update);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(100)]
    color: Color,
    #[uniform(101)]
    subdivisions: UVec2,
    #[uniform(102)]
    line_widths: Vec2,
}

impl MaterialExtension for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "grid_material.wgsl".into()
    }
}

#[derive(Debug, Component)]
pub struct CameraTarget {
    zoom_level: usize,
    zoom_level_offsets: Vec<Vec3>,
    look_at: Vec3,
    up: Vec3,
    is_dirty: bool,
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self {
            zoom_level: 0,
            zoom_level_offsets: vec![],
            look_at: Vec3::default(),
            up: Vec3::Y,
            is_dirty: true,
        }
    }
}

impl CameraTarget {
    pub fn update(
        mut scroll_evr: EventReader<MouseWheel>,
        keys: Res<Input<KeyCode>>,
        mut camera_query: Query<(&mut CameraTarget, &mut Transform), With<Camera>>,
    ) {
        let (mut target, mut camera_transform) = camera_query.single_mut();

        let delta_y: f32 = scroll_evr.read().map(|ev| ev.y).sum();
        let delta_zoom_level = if delta_y < 0.0 {
            -1
        } else if delta_y > 0.0 {
            1
        } else {
            0
        };
        target.change_zoom_to(delta_zoom_level);

        let mut delta_x: f32 = 0.0;
        let mut delta_z: f32 = 0.0;
        if keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up) {
            delta_x -= 1.0;
        }
        if keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down) {
            delta_x += 1.0;
        }
        if keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right) {
            delta_z -= 1.0;
        }
        if keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left) {
            delta_z += 1.0;
        }

        if delta_x != 0.0 || delta_z != 0.0 {
            let mut look_at = *target.get_look_at();
            look_at.x += delta_x * 10.0;
            look_at.z += delta_z * 10.0;
            target.look_at(look_at)
        }

        target.update_transform(&mut camera_transform)
    }

    pub fn update_transform(&mut self, transform: &mut Transform) {
        if self.is_dirty {
            transform.translation = self.look_at + self.zoom_level_offsets[self.zoom_level];
            transform.look_at(self.look_at, self.up);
            self.is_dirty = false;
        }
    }

    pub fn change_zoom_to(&mut self, delta_zoom_level: i32) {
        let new_zoom_level =
            (self.zoom_level as i32 + delta_zoom_level).clamp(0, i32::MAX) as usize;
        self.zoom_to(new_zoom_level);
    }

    pub fn zoom_to(&mut self, zoom_level: usize) {
        let old_zoom_level = self.zoom_level;
        self.zoom_level = zoom_level.clamp(0, self.zoom_level_offsets.len() - 1);

        if old_zoom_level != self.zoom_level {
            self.is_dirty = true
        }
    }

    pub fn zooming_to(mut self, zoom_level: usize) -> Self {
        self.zoom_to(zoom_level);
        self
    }

    pub fn get_look_at(&self) -> &Vec3 {
        &self.look_at
    }

    pub fn look_at(&mut self, look_at: Vec3) {
        let old_look_at = self.look_at;
        self.look_at = look_at;

        if old_look_at != self.look_at {
            self.is_dirty = true
        }
    }

    pub fn looking_at(mut self, look_at: Vec3) -> CameraTarget {
        self.look_at(look_at);
        self
    }

    pub fn set_up(&mut self, up: Vec3) {
        self.up = up;
    }

    pub fn with_up(mut self, up: Vec3) -> Self {
        self.set_up(up);
        self
    }

    pub fn add_zoom_level_offset(&mut self, zoom_level_offset: Vec3) {
        self.zoom_level_offsets.push(zoom_level_offset)
    }

    pub fn with_zoom_level_offset(mut self, zoom_level_offset: Vec3) -> Self {
        self.add_zoom_level_offset(zoom_level_offset);
        self
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
