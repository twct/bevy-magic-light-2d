use bevy::{
    color::palettes::css::{MIDNIGHT_BLUE, ORANGE_RED},
    math::vec3,
    prelude::*,
    render::texture::{ImageFilterMode, ImageSamplerDescriptor},
    window::PrimaryWindow,
};
use bevy_magic_light_2d::gi::{
    post_process::LightPostProcessSettings,
    types::{LightOccluder2D, OmniLightSource2D, SkylightLight2D, SkylightMask2D},
    BevyMagicLight2DPlugin,
};

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct MouseLight;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    log::info!("Setup new example");

    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
        LightPostProcessSettings { time: 0.0 },
    ));

    commands.spawn(SpriteBundle {
        texture: asset_server.load("art/atlas_decoration.png"),
        ..Default::default()
    });

    commands.spawn(SkylightLight2D {
        color: MIDNIGHT_BLUE.into(),
        // intensity: 0.1,
        intensity: 0.8,
    });

    commands.spawn((
        SpatialBundle::default(),
        OmniLightSource2D {
            color: ORANGE_RED.into(),
            intensity: 0.5,
            falloff: vec3(80.0, 20.0, 0.05),
            jitter_intensity: 0.1,
            jitter_translation: 0.1,
            ..Default::default()
        },
        MouseLight,
    ));

    commands.spawn((
        SpatialBundle::default(),
        LightOccluder2D {
            h_size: Vec2::splat(100.0),
        },
    ));

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(200.0, 0.0, 0.0)),
        LightOccluder2D {
            h_size: Vec2::splat(100.0),
        },
    ));

    commands.spawn((
        SpatialBundle::default(),
        SkylightMask2D {
            h_size: Vec2::splat(100.0),
        },
    ));

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(200.0, 0.0, 0.0)),
        SkylightMask2D {
            h_size: Vec2::splat(100.0),
        },
    ));
}

fn mouse_light_move_system(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query: Query<&mut Transform, With<MouseLight>>,
) {
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        for mut transform in &mut query {
            transform.translation.x = cursor_position.x;
            transform.translation.y = cursor_position.y;
        }
    }
}

fn camera_move_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_target_position: Local<Vec2>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let speed = 150.0;
    let lerp_factor = 5.0;

    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyA) {
        direction.x = -1.0;
    } else if keys.pressed(KeyCode::KeyD) {
        direction.x = 1.0;
    }
    if keys.pressed(KeyCode::KeyW) {
        direction.y = 1.0;
    } else if keys.pressed(KeyCode::KeyS) {
        direction.y = -1.0;
    }

    *camera_target_position += direction * speed * time.delta_seconds();

    if let Ok(mut transform) = query.get_single_mut() {
        let current_position = transform.translation.xy();
        let new_position =
            current_position.lerp(*camera_target_position, time.delta_seconds() * lerp_factor);

        // transform.translation.x = new_position.x.round();
        // transform.translation.y = new_position.y.round();

        transform.translation.x = new_position.x;
        transform.translation.y = new_position.y;
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor {
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Nearest,
                        ..default()
                    },
                }),
            BevyMagicLight2DPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_move_system, mouse_light_move_system))
        .run();
}
