use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseWheel,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        let mut material_plugin =
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, GridMaterial>>::default();
        material_plugin.prepass_enabled = false;

        app.add_plugins(material_plugin)
            .add_systems(Update, CameraTarget::update);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(100)]
    pub color: Color,
    #[uniform(101)]
    pub subdivisions: UVec2,
    #[uniform(102)]
    pub line_widths: Vec2,
}

impl MaterialExtension for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "grid_material.wgsl".into()
    }
}

#[derive(Debug, Component)]
pub struct AsciiModel {
    lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    a: Vec3,
    b: Vec3,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            a: Vec3::new(f32::MIN, f32::MIN, f32::MIN),
            b: Vec3::new(f32::MAX, f32::MAX, f32::MAX),
        }
    }
}

impl BoundingBox {
    pub fn new(a: Vec3, b: Vec3) -> Self {
        Self {
            a: a.min(b),
            b: a.max(b),
        }
    }

    pub fn clamp(&self, position: Vec3) -> Vec3 {
        position.clamp(self.a, self.b)
    }
}

#[derive(Debug, Component)]
pub struct CameraTarget {
    /// value in range [0.0, 1.0] which determines which zoom_level_offsets to use
    zoom_level: f32,

    /// list of offsets to position the camera relative to the look_at point
    zoom_level_offsets: Vec<Vec3>,

    /// point in space the camera should look at
    look_at: Vec3,

    /// rotation angle around the up axis
    rotation: f32,

    /// normal vector representing up for the camera
    up: Vec3,

    /// bounding box for camera
    bounding_box: BoundingBox,

    /// true wheh zoom_level, look_at, or up change
    /// this let's our system know to update the camera transform in the scene
    is_dirty: bool,
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self {
            zoom_level: 0.0,
            zoom_level_offsets: vec![],
            look_at: Vec3::default(),
            rotation: 0.0,
            up: Vec3::Y,
            bounding_box: BoundingBox::default(),
            is_dirty: true,
        }
    }
}

impl CameraTarget {
    pub fn update(
        mut scroll_evr: EventReader<MouseWheel>,
        keys: Res<Input<KeyCode>>,
        time: Res<Time>,
        mut camera_query: Query<(&mut CameraTarget, &mut Transform), With<Camera>>,
    ) {
        let (mut target, mut camera_transform) = camera_query.single_mut();

        let delta_y: f32 = scroll_evr.read().map(|ev| ev.y).sum();
        let mut delta_zoom_level: f32 = if delta_y < 0.0 {
            -1.0
        } else if delta_y > 0.0 {
            1.0
        } else {
            0.0
        };

        if keys.pressed(KeyCode::Equals) || keys.pressed(KeyCode::Plus) {
            delta_zoom_level = -1.0;
        }
        if keys.pressed(KeyCode::Minus) {
            delta_zoom_level = 1.0;
        }

        let mut delta_rotation: f32 = 0.0;
        if keys.pressed(KeyCode::Q) {
            delta_rotation = -1.0;
        }
        if keys.pressed(KeyCode::E) {
            delta_rotation = 1.0;
        }

        let mut delta_x: f32 = 0.0;
        let mut delta_z: f32 = 0.0;
        if keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up) {
            delta_x = 1.0;
        }
        if keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down) {
            delta_x = -1.0;
        }
        if keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right) {
            delta_z = -1.0;
        }
        if keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left) {
            delta_z = 1.0;
        }

        if delta_zoom_level != 0.0 {
            target.change_zoom_to(delta_zoom_level * 0.1 * time.delta_seconds().clamp(0.0, 1.0));
        }
        if delta_rotation != 0.0 {
            target.change_rotation(delta_rotation * 0.8 * time.delta_seconds().clamp(0.0, 1.0));
        }

        target.update_transform(&mut camera_transform);
        if delta_x != 0.0 || delta_z != 0.0 {
            let mask = Vec3::new(1.0, 1.0, 1.0) - target.get_up();
            let look_at = target.get_look_at() * mask; // multiply out the up component
            let camera_at = camera_transform.translation * mask; // multiply out the up component
            let forward = (look_at - camera_at).normalize();
            let mut right_rotation = Transform::from_xyz(0.0, 0.0, 0.0);
            right_rotation
                .rotate_around(Vec3::default(), Quat::from_axis_angle(target.up, PI / 2.0));
            let right = right_rotation * forward;

            let scale_factor = 200.0 * time.delta_seconds().clamp(0.0, 1.0);
            let new_look_at =
                look_at + (forward * delta_x * scale_factor) + (right * delta_z * scale_factor);
            target.look_at(new_look_at);
            target.update_transform(&mut camera_transform);
        }
    }

    pub fn update_transform(&mut self, transform: &mut Transform) {
        if self.is_dirty {
            let zoom_level_a = self.zoom_level_offsets
                [(self.zoom_level * (self.zoom_level_offsets.len() - 1) as f32).floor() as usize];
            let zoom_level_b = self.zoom_level_offsets
                [(self.zoom_level * (self.zoom_level_offsets.len() - 1) as f32).ceil() as usize];
            let mut mix = self.zoom_level * (self.zoom_level_offsets.len() - 1) as f32;
            mix -= mix as u32 as f32;
            let offset = zoom_level_a.lerp(zoom_level_b, mix);
            transform.translation = self.look_at + offset;
            transform.rotate_around(self.look_at, Quat::from_axis_angle(self.up, self.rotation));
            transform.look_at(self.look_at, self.up);
            self.is_dirty = false;
        }
    }

    pub fn change_zoom_to(&mut self, delta_zoom_level: f32) {
        self.zoom_to(self.zoom_level + delta_zoom_level)
    }

    pub fn zoom_to(&mut self, zoom_level: f32) {
        let old_zoom_level = self.zoom_level;
        self.zoom_level = zoom_level.clamp(0.0, 1.0);

        if old_zoom_level != self.zoom_level {
            self.is_dirty = true
        }
    }

    pub fn zooming_to(mut self, zoom_level: f32) -> Self {
        self.zoom_to(zoom_level);
        self
    }

    pub fn get_look_at(&self) -> Vec3 {
        self.look_at
    }

    pub fn look_at(&mut self, look_at: Vec3) {
        let old_look_at = self.look_at;
        self.look_at = self.bounding_box.clamp(look_at);

        if old_look_at != self.look_at {
            self.is_dirty = true
        }
    }

    pub fn looking_at(mut self, look_at: Vec3) -> CameraTarget {
        self.look_at(look_at);
        self
    }

    pub fn change_rotation(&mut self, delta_rotation: f32) {
        self.rotate(self.rotation + delta_rotation)
    }

    pub fn rotate(&mut self, rotation: f32) {
        let old_rotation = self.rotation;
        self.rotation = rotation;

        if old_rotation != self.rotation {
            self.is_dirty = true
        }
    }

    pub fn rotating(mut self, rotation: f32) -> Self {
        self.rotate(rotation);
        self
    }

    pub fn get_up(&self) -> Vec3 {
        self.up
    }

    pub fn set_up(&mut self, up: Vec3) {
        self.up = up;
    }

    pub fn with_up(mut self, up: Vec3) -> Self {
        self.set_up(up);
        self
    }

    pub fn set_bounding_box(&mut self, bounding_box: BoundingBox) {
        let old_bounding_box = self.bounding_box;
        self.bounding_box = bounding_box;

        if old_bounding_box != self.bounding_box {
            self.is_dirty = true
        }
    }

    pub fn with_bounding_box(mut self, bounding_box: BoundingBox) -> Self {
        self.set_bounding_box(bounding_box);
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
    pub size: Vec3,
    pub subdivisions: UVec3,
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
