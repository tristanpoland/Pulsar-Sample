use crate::math::Vector3;

use dynasty_rs::prelude::*;
use super::Object;

#[inherit(Object)]
#[derive(Debug)]
pub struct Actor {
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
}

impl Actor {
    pub fn new() -> Self {
        Actor {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = Vector3::new(x, y, z);
    }

    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.rotation = Vector3::new(x, y, z);
    }

    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = Vector3::new(x, y, z);
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.position += Vector3::new(x, y, z);
    }

    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotation += Vector3::new(x, y, z);
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale += Vector3::new(x, y, z);
    }

    pub fn get_position(&self) -> Vector3 {
        self.position
    }

    pub fn get_rotation(&self) -> Vector3 {
        self.rotation
    }

    pub fn get_scale(&self) -> Vector3 {
        self.scale
    }

    pub fn get_transform(&self) -> Transform {
        Transform::new(self.position, self.rotation, self.scale)
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.position = transform.position;
        self.rotation = transform.rotation;
        self.scale = transform.scale;
    }

    pub fn look_at(&mut self, target: Vector3) {
        let direction = target - self.position;
        let rotation = Vector3::new(
            direction.y.atan2(direction.x),
            direction.z.atan2(direction.y),
            direction.x.atan2(direction.z),
        );
        self.rotation = rotation;
    }

    pub fn move_towards(&mut self, target: Vector3, speed: f32) {
        let direction = target - self.position;
        let distance = direction.length();
        if distance <= speed {
            self.position = target;
        } else {
            let velocity = direction.normalize() * speed;
            self.position += velocity;
        }
    }

    pub fn rotate_towards(&mut self, target: Vector3, speed: f32) {
        let direction = target - self.position;
        let rotation = Vector3::new(
            direction.y.atan2(direction.x),
            direction.z.atan2(direction.y),
            direction.x.atan2(direction.z),
        );
        let delta = rotation - self.rotation;
        let distance = delta.length();
        if distance <= speed {
            self.rotation = rotation;
        } else {
            let velocity = delta.normalize() * speed;
            self.rotation += velocity;
        }
    }

    pub fn scale_towards(&mut self, target: Vector3, speed: f32) {
        let direction = target - self.scale;
        let distance = direction.length();
        if distance <= speed {
            self.scale = target;
        } else {
            let velocity = direction.normalize() * speed;
            self.scale += velocity;
        }
    }

    // TODO: Implement this on the rotation field instead
    pub fn look_at_transform(&mut self, target: Transform) {
        self.look_at(target.position);
        self.as_parent().set_rotation(target.rotation);
    }
} 