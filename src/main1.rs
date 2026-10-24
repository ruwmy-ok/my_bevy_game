use bevy::prelude::*;
use bevy_symbios_ground::{FbmNoise, HeightMap, HeightMapMeshBuilder, TerrainGenerator};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ---- 原有地形生成部分 ----
    let mut heightmap = HeightMap::new(128, 128, 1.0);
    FbmNoise::new(1337).generate(&mut heightmap);
    heightmap.normalize();

    let mesh = HeightMapMeshBuilder::new()
        .with_uv_tile_size(4.0)
        .build(&heightmap);

    // 地形实体：加一个 Transform 把地形居中（默认网格从原点延伸到正方向）
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.55, 0.35),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(-64.0, 0.0, -64.0),
    ));

    // ---- 补充：相机 ----
    // 放在地形斜上方，看向地形中心附近
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(80.0, 60.0, 80.0)
            .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
    ));

    // ---- 补充：平行光（模拟太阳） ----
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 100.0, 50.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}