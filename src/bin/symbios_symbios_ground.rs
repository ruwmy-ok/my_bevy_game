use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_symbios_ground::{
    FbmNoise, HeightMap, HeightMapMeshBuilder, SplatMapper, SplatRule, TerrainGenerator,
    TerrainQuery, WeightMap,HydraulicErosion, LakeMeshBuilder,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(Color::srgb(0.53, 0.72, 0.92))) // 天空蓝
        .add_systems(Update, (mouse_look, wasd_move))
        .run();
}

#[derive(Component)]
struct FirstPersonCamera {
    yaw: f32,
    pitch: f32,
    vertical_velocity: f32,  // 新增：垂直速度
    is_grounded: bool,       // 新增：是否在地面上
}

const EYE_HEIGHT: f32 = 1.7;
const MOVE_SPEED: f32 = 30.0;
const MOUSE_SENSITIVITY: f32 = 0.002;
const MAP_SIZE: usize = 512;
const SCALE: f32 = 10.0;
const HEIGHT_SCALE: f32 = 120.0;
/// 水面高度（世界单位）。取高度范围的一个比例，让低洼处被水覆盖。
const WATER_LEVEL: f32 = HEIGHT_SCALE * 0.45;
const JUMP_SPEED: f32 = 8.0;     // 起跳初速度
const GRAVITY: f32 = 25.0;       // 重力加速度
/// 四层预览颜色：草、土、岩、雪（sRGB）
const LAYER_COLOURS: [[f32; 3]; 4] = [
    [0.28, 0.48, 0.18], // R — 草
    [0.52, 0.40, 0.26], // G — 土
    [0.45, 0.45, 0.47], // B — 岩
    [0.92, 0.94, 0.97], // A — 雪
];

/// 按高度和坡度定义四层样条规则（高度单位为世界单位）
fn demo_splat_mapper(height_scale: f32) -> SplatMapper {
    let alt = |lo: f32, hi: f32| (lo * height_scale, hi * height_scale);
    SplatMapper::new([
        // R — 草：低海拔、平缓
        SplatRule::new(alt(0.0, 0.4), (0.0, 0.3), 1.0),
        // G — 土：中海拔、中等坡度
        SplatRule::new(alt(0.2, 0.7), (0.0, 0.5), 1.0),
        // B — 岩：陡坡（任何高度）
        SplatRule::new(alt(0.0, 1.0), (0.3, 1.0), 1.0),
        // A — 雪：高海拔、平缓
        SplatRule::new(alt(0.7, 1.0), (0.0, 0.4), 1.0),
    ])
}

/// 把权重图解析成可直接用于 StandardMaterial 的彩色纹理
fn weights_to_colour_image(weight_map: &WeightMap) -> Image {
    let raw: Vec<u8> = weight_map
        .data
        .iter()
        .flat_map(|weights| {
            let mut rgb = [0.0f32; 3];
            for (layer, &w) in weights.iter().enumerate() {
                let w = w as f32 / 255.0;
                for (c, out) in rgb.iter_mut().enumerate() {
                    *out += LAYER_COLOURS[layer][c] * w;
                }
            }
            let to_byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            [to_byte(rgb[0]), to_byte(rgb[1]), to_byte(rgb[2]), 255]
        })
        .collect();

    let mut image = Image::new(
        Extent3d {
            width: weight_map.width as u32,
            height: weight_map.height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        raw,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    // 线性采样 + 边缘钳制，避免色带出现阶梯
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // ---- 地形生成 ----
    let mut heightmap = HeightMap::new(MAP_SIZE, MAP_SIZE, SCALE);
    FbmNoise {
        base_frequency: 4.0,
        ..FbmNoise::new(6000)
    }
    .generate(&mut heightmap);
    heightmap.normalize();

    for h in heightmap.data_mut() {
        *h *= HEIGHT_SCALE;
    }

    // ---- Splat 分色 ----
    let mapper = demo_splat_mapper(HEIGHT_SCALE);
    let weight_map = mapper.generate(&heightmap);
    let colour_handle = images.add(weights_to_colour_image(&weight_map));

    // ---- 网格构建（整张纹理铺满地形，不平铺） ----
    let vertex_extent = (MAP_SIZE - 1) as f32 * SCALE;
    let mesh = HeightMapMeshBuilder::new()
        .with_uv_tile_size(vertex_extent)
        .build(&heightmap);

    // 网格从原点向正方向延伸，平移使其居中
    let half = vertex_extent / 2.0;
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(colour_handle),
            perceptual_roughness: 0.95,
            metallic: 0.0,
            ..default()
        })),
        Transform::from_xyz(-half, 0.0, -half),
    ));
// ---- 湖泊 / 海面：一个半透明大平面 ----
    // 平面尺寸略大于地形，覆盖整个水平范围
    let water_size = vertex_extent * 1.2;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(water_size, water_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.15, 0.35, 0.65, 0.65),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.1,
            metallic: 0.0,
            ..default()
        })),
        Transform::from_xyz(0.0, WATER_LEVEL, 0.0),
    ));
    // ---- 插入 TerrainQuery（用同一个 mapper） ----
    let terrain_query = TerrainQuery::new(heightmap, mapper);
    // 用世界坐标 (0,0) 采样初始地面高度（加回 half 偏移）
    let start_ground = terrain_query.height_at(0.0 + half, 0.0 + half);
    commands.insert_resource(terrain_query);

    // ---- 第一人称相机 ----
    // 起点若低于水面，抬高到水面之上，避免出生在水下
    let start_y = (start_ground + EYE_HEIGHT).max(WATER_LEVEL + EYE_HEIGHT);
    let start_pos = Vec3::new(0.0, start_y, 0.0);
    let yaw = 0.0;
    let pitch = -0.1;
    commands.spawn((
        Camera3d::default(),
        FirstPersonCamera { yaw, pitch, vertical_velocity: 0.0,
    is_grounded: true, },
        Transform::from_translation(start_pos)
            .with_rotation(Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)),
    ));

    // ---- 光照 ----
    commands.spawn((
        DirectionalLight {
             color: Color::srgb(0.53, 0.72, 0.92),
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(100.0, 300.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn(AmbientLight {
        brightness: 200.0,
        ..default()
    });
}

fn mouse_look(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut FirstPersonCamera, &mut Transform)>,
) {
    let Ok((mut cam, mut transform)) = query.single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    for ev in mouse_motion.read() {
        delta += ev.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }

    cam.yaw -= delta.x * MOUSE_SENSITIVITY;
    cam.pitch = (cam.pitch - delta.y * MOUSE_SENSITIVITY).clamp(-1.5, 1.5);

    transform.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
}

fn wasd_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    terrain: Res<TerrainQuery>,
    mut query: Query<(&mut FirstPersonCamera, &mut Transform)>,
) {
    let Ok((mut cam, mut transform)) = query.single_mut() else {
        return;
    };
     let dt = time.delta_secs();
    let forward = Vec3::new(-cam.yaw.sin(), 0.0, -cam.yaw.cos());
    let right = Vec3::new(cam.yaw.cos(), 0.0, -cam.yaw.sin());

    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir -= right;
    }

    if dir.length_squared() > 0.0 {
        let dir = dir.normalize();
        transform.translation += dir * MOVE_SPEED * time.delta_secs();
    }

    // 关键：网格平移了 -half，采样时加回 half，回到高度图坐标空间
    /*let vertex_extent = (MAP_SIZE - 1) as f32 * SCALE;
    let half = vertex_extent / 2.0;
    let ground_y = terrain.height_at(
        transform.translation.x + half,
        transform.translation.z + half,
    );*/
      // 相机不会沉到水面以下
      // ---- 采样地面高度 ----
    let vertex_extent = (MAP_SIZE - 1) as f32 * SCALE;
    let half = vertex_extent / 2.0;
    let ground_y = terrain.height_at(
        transform.translation.x + half,
        transform.translation.z + half,
    );
    let floor_y = ground_y + EYE_HEIGHT;

    // ---- 跳跃与重力 ----
    // 空格起跳（只有在地面时才能跳）
    if keys.just_pressed(KeyCode::Space) && cam.is_grounded {
        cam.vertical_velocity = JUMP_SPEED;
        cam.is_grounded = false;
    }

    // 施加重力
    if !cam.is_grounded {
        cam.vertical_velocity -= GRAVITY * dt;
    }

    // 应用垂直速度
    transform.translation.y += cam.vertical_velocity * dt;

    // ---- 落地判定 ----
    if transform.translation.y <= floor_y {
        transform.translation.y = floor_y;
        cam.vertical_velocity = 0.0;
        cam.is_grounded = true;
    } else {
        cam.is_grounded = false;
    }
    //transform.translation.y = (ground_y + EYE_HEIGHT).max(WATER_LEVEL + EYE_HEIGHT * 0.5);
}