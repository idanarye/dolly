use std::marker::PhantomData;

use glam::Vec3;

use crate::{
    driver::RigDriver, handedness::Handedness, rig::RigUpdateParams, transform::Transform,
};

#[derive(Default, Debug)]
pub struct MaintainDistance {
    pub focal: Vec3,
    pub plane_normal: Vec3,
    pub min_distance: f32,
    pub max_distance: f32,
}

impl MaintainDistance {}

impl<H: Handedness> RigDriver<H> for MaintainDistance {
    fn update(&mut self, params: RigUpdateParams<H>) -> Transform<H> {
        let vec_to_focal = (self.focal - params.parent.position).reject_from(self.plane_normal);
        let distance_to_focal = vec_to_focal.length();
        if self.max_distance < distance_to_focal {
            Transform {
                position: params.parent.position
                    + (distance_to_focal - self.max_distance) * vec_to_focal.normalize(),
                rotation: params.parent.rotation,
                phantom: PhantomData,
            }
        } else if distance_to_focal < self.min_distance {
            Transform {
                position: params.parent.position
                    + (distance_to_focal - self.min_distance) * vec_to_focal.normalize(),
                rotation: params.parent.rotation,
                phantom: PhantomData,
            }
        } else {
            *params.parent
        }
    }
}
