// Based on https://github.com/not-fl3/macroquad/blob/97a99d00155cb7531f4432a2eb5f3c587e22f9b3/examples/3d.rs

use dolly::{driver::RigDriver, prelude::*};
use macroquad::prelude::*;

/// A custom camera rig which combines smoothed movement with a look-at driver.
#[derive(Debug)]
pub struct MovableLookAt<H: Handedness>(CameraRig<H>);

// Turn the nested rig into a driver, so it can be used in another rig.
impl<H: Handedness> RigDriver<H> for MovableLookAt<H> {
    fn update(&mut self, params: dolly::rig::RigUpdateParams<H>) -> dolly::transform::Transform<H> {
        self.0.update(params.delta_time_seconds)
    }
}

impl<H: Handedness> MovableLookAt<H> {
    pub fn from_position_target(
        camera_position: dolly::glam::Vec3,
        target_position: dolly::glam::Vec3,
    ) -> Self {
        Self(
            CameraRig::builder()
                // Allow moving the camera
                .with(Position::new(camera_position))
                // Predict camera movement to make the subsequent smoothing reactive
                .with(Smooth::new_position(1.25).predictive(true))
                // Smooth the predicted movement
                .with(Smooth::new_position(2.5))
                .with(MaintainDistance {
                    focal: target_position,
                    plane_normal: dolly::glam::Vec3::Y,
                    min_distance: 4.0,
                    max_distance: 10.0,
                })
                .with(LookAt::new(target_position + dolly::glam::Vec3::Y).tracking_smoothness(1.25))
                .build(),
        )
    }

    pub fn set_position_target(
        &mut self,
        camera_position: dolly::glam::Vec3,
        target_position: dolly::glam::Vec3,
    ) {
        self.0.driver_mut::<Position>().position = camera_position;
        self.0.driver_mut::<LookAt>().target = target_position;
        self.0.driver_mut::<MaintainDistance>().focal = target_position;
    }
}

#[macroquad::main("dolly nested_driver example")]
async fn main() {
    info!("{}", "WASD to move");
    info!("{}", "Spacebar and LShift to go up and down");

    let mut camera_position = dolly::glam::Vec3::new(4., 3., 8.);
    let mut player_position = dolly::glam::Vec3::new(2., 1.01, 2.);

    // Create a camera rig with our custom nested `MovableLookAt` driver within.
    let mut camera = CameraRig::builder()
        .with(MovableLookAt::from_position_target(
            camera_position,
            player_position,
        ))
        .build();

    loop {
        // Move either the player or the camera
        let forward = dolly::glam::Vec3::Y
            .cross(camera.final_transform.right())
            .normalize();
        let right = forward.cross(dolly::glam::Vec3::Y);
        let delta_pos = get_move_input(forward, right);
        player_position += delta_pos;
        camera_position = camera.final_transform.position;

        // Update the camera driver
        camera
            .driver_mut::<MovableLookAt<RightHanded>>()
            .set_position_target(camera_position, player_position);

        // Update the camera rig, and get the interpolated transform
        let camera_xform = camera.update(get_frame_time());

        clear_background(LIGHTGRAY);

        // Pass the camera to macroquad, doing some gymnastics to convince
        // the two different `glam` versions to talk to each other.
        set_camera(&Camera3D {
            position: camera_xform.position.d2m(),
            up: camera_xform.up().d2m(),
            target: (camera_xform.position + camera_xform.forward()).d2m(),
            ..Default::default()
        });

        draw_grid(20, 1., BLACK, GRAY);

        draw_cube_wires(vec3(0., 1.01, -6.), vec3(2., 2., 2.), DARKGREEN);
        draw_cube_wires(vec3(0., 1.01, 6.), vec3(2., 2., 2.), DARKBLUE);
        draw_cube_wires(player_position.d2m(), vec3(2., 2., 2.), YELLOW);

        draw_cube(vec3(-5., 1., -2.), vec3(2., 2., 2.), None, WHITE);
        draw_cube(vec3(-5., 1., 2.), vec3(2., 2., 2.), None, WHITE);
        draw_cube(vec3(2., 0., -2.), vec3(0.4, 0.4, 0.4), None, BLACK);

        draw_sphere(vec3(-8., 0., 0.), 1., None, BLUE);

        set_default_camera();

        next_frame().await
    }
}

trait DollyToMacroquad {
    type Target;
    fn d2m(self) -> Self::Target;
}

impl DollyToMacroquad for dolly::glam::Vec3 {
    type Target = Vec3;

    fn d2m(self) -> Self::Target {
        <[f32; 3]>::from(self).into()
    }
}

fn get_move_input(forward: dolly::glam::Vec3, right: dolly::glam::Vec3) -> dolly::glam::Vec3 {
    const SPEED: f32 = 0.05;

    let mut delta_pos = dolly::glam::Vec3::ZERO;

    if is_key_down(KeyCode::D) {
        delta_pos += SPEED * right;
    }
    if is_key_down(KeyCode::A) {
        delta_pos -= SPEED * right;
    }
    if is_key_down(KeyCode::S) {
        delta_pos -= SPEED * forward;
    }
    if is_key_down(KeyCode::W) {
        delta_pos += SPEED * forward;
    }
    if is_key_down(KeyCode::LeftShift) {
        delta_pos.y -= SPEED;
    }
    if is_key_down(KeyCode::Space) {
        delta_pos.y += SPEED;
    }

    delta_pos
}
