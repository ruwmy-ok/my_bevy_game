use bevy::{
    prelude::*,
    post_process::bloom::Bloom,
    pbr::{MaterialPipeline,MaterialPipelineKey},
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{AsBindGroup,RenderPipelineDescriptor,SpecializedMeshPipelineError}
    },
    shader::ShaderRef,
    input::mouse::{MouseButton,MouseWheel,MouseMotion},

};
use rand::RngExt;

const FRAGMENT_SHADER:&str="shaders/energy_pillar.wesl";
fn main(){
    App::new()
    .add_plugins((
    DefaultPlugins,
    MaterialPlugin::<EnergyPillarMaterial>::default(),
    UtilityShaderPlugin,))
    .add_systems(Startup, setup)
    .add_systems(Update,(update_time,orbit_camera_control) )
    .run();
}
#[derive(Resource)]
struct UtilityShader(Handle<Shader>);

struct UtilityShaderPlugin;

impl Plugin for UtilityShaderPlugin{
    fn build(&self,app:&mut App){
        let handle=app
        .world_mut()
        .resource_mut::<AssetServer>()
        .load::<Shader>("shaders/effect_utils.wesl");
        app.insert_resource(UtilityShader(handle));
    }
}

#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)]
struct EnergyPillarMaterial{
    #[uniform(0)]
    time:Vec4,
    #[uniform(0)]
    base_color:LinearRgba,
}
impl Material for EnergyPillarMaterial{
    fn fragment_shader()->ShaderRef{
        FRAGMENT_SHADER.into()
    }
    fn alpha_mode(&self)->AlphaMode{
        AlphaMode::Blend
    }
}
#[derive(Component)]
struct OrbitCamera{
    focus:Vec3,
    radius:f32,
    yaw:f32,
    pitch:f32,
    roll:f32,
}
impl  Default for OrbitCamera {
    fn default()->Self{
        Self { focus: Vec3::ZERO, radius: 10.0, yaw: 0.0, pitch: 0.3, roll: 0.0 }

    }
}
fn setup(
    mut commands:Commands,
    mut meshes:ResMut<Assets<Mesh>>,
    mut materials:ResMut<Assets<EnergyPillarMaterial>>,
){
     // 先把共用的网格和材质创建好
    let cylinder_mesh = meshes.add(Cylinder::new(1.2, 0.5));
    let pillar_material = materials.add(EnergyPillarMaterial {
        time: Vec4::ZERO,
        base_color: LinearRgba::rgb(0.5, 2.0, 8.0),
    });
     // 预先创建几种颜色材质
    let palette_colors = [
        LinearRgba::rgb(0.5, 2.0, 8.0),  // 蓝
        LinearRgba::rgb(8.0, 0.5, 0.5),  // 红
        LinearRgba::rgb(0.5, 8.0, 0.5),  // 绿
        LinearRgba::rgb(8.0, 8.0, 0.5),  // 黄
    ];
     let palette: Vec<Handle<EnergyPillarMaterial>> = palette_colors
        .iter()
        .map(|c| materials.add(EnergyPillarMaterial {
            time: Vec4::ZERO,
            base_color: *c,
        }))
        .collect();
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.2,0.5,1.2))),
        MeshMaterial3d(materials.add(EnergyPillarMaterial{
            time:Vec4::ZERO,
            base_color:LinearRgba::rgb(0.5,2.0,8.0),
        })),
        Transform::from_xyz(0.0,3.0,0.0),
       
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.2,0.5))),
        MeshMaterial3d(materials.add(EnergyPillarMaterial{
            time:Vec4::ZERO,
            base_color:LinearRgba::rgb(0.5,2.0,8.0),
        })),
        Transform::from_xyz(5.0,3.0,0.0),
    ));
    let mut rng = rand::rng();
    // 2. 循环批量生成
    for _ in 0..20 {
        // 3. 随机位置（和你要随机的其他属性）
        let x = rng.random_range(-10.0..10.0);
        let z = rng.random_range(-10.0..10.0);
        let y = 3.0; // 高度固定，或也随机
        let mat = palette[rng.random_range(0..palette.len())].clone();

        commands.spawn((
            Mesh3d(cylinder_mesh.clone()),          // 共用网格句柄
            MeshMaterial3d(mat.clone()), // 共用材质句柄
            Transform::from_xyz(x, y, z),
        ));
    }


    commands.spawn((
        Camera3d::default(),
        Camera{
            clear_color:ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        OrbitCamera{
            focus:Vec3::new(0.0,3.0,0.0),
            radius:10.0,
            yaw:0.0,
            pitch:0.3,
            roll:0.0,
        },
        AmbientLight{
            color:Color::srgb(0.2,0.2,0.3),
            brightness:200.0,
            ..default()
        },
        PointLight{
            intensity:800000.0,
            color:Color::srgb(0.5,8.0,1.0),
            ..default()
        },
        Bloom::NATURAL,
        Transform::from_xyz(4.0,5.0,8.0).looking_at(Vec3::new(0.0,3.0,0.0),Vec3::Y),
    ));
}
fn update_time(
    time:Res<Time>,
    mut materials:ResMut<Assets<EnergyPillarMaterial>>,
){
    for(_,material) in materials.iter_mut(){
        material.time.x=time.elapsed_secs();
    }
}

fn orbit_camera_control( mut mouse_motion:MessageReader<MouseMotion>,
   time:Res<Time>,
    mut mouse_wheel:MessageReader<MouseWheel>,
    mouse_button:Res<ButtonInput<MouseButton>>,
    mut query:Query<(&mut OrbitCamera,&mut Transform)>,)
   {
    let Ok((mut orbit,mut transform))=query.single_mut() else{return;};
    if !mouse_button.pressed(MouseButton::Left){
        //orbit.yaw+=time.delta_secs()*0.3;
        orbit.roll+=time.delta_secs().sin();
    }
    if mouse_button.pressed(MouseButton::Left){
        for ev in mouse_motion.read(){
            orbit.yaw-=ev.delta.x*0.01;
            orbit.pitch+=ev.delta.y*0.01;
            orbit.pitch=orbit.pitch.clamp(-1.5,1.5);
        }
    }
    for ev in mouse_wheel.read(){
        orbit.radius-=ev.y*0.5;
        orbit.radius=orbit.radius.clamp(2.0,30.0);
    }
    let rotation=Quat::from_euler(EulerRot::YXZ,orbit.yaw,-orbit.pitch,orbit.roll);
    transform.translation=orbit.focus+rotation*Vec3::new(0.0,0.0,orbit.radius);
    transform.rotation=rotation;
}

