use core::pin::Pin;

use alloc::boxed::Box;
use futures::Future;
use reactor::middleware::Middleware;
use reactor::ReactorEvent;

/// Translates three 2D joysticks into a 6DOF space mouse report
/// Arranged in a triangle, with the first joystick at the top, and the other two at the bottom
/// all facing outwards
///          | (j1)
///
/// (j2) \       / (j3)
/// The "sub" axis is referenced as the axis relative to each joystick
/// Y is the axis perpendicular to the plane of the joysticks
/// X is the axis parallel to the plane of the joysticks
/// So for example rotating the device around the Z axis (yaw) would be the sum of all X axis inputs
pub fn calculate_triangular_6dof(j1x: i16, j1y: i16, j2x: i16, j2y: i16, j3x: i16, j3y: i16) -> ReactorEvent {
	// TODO: Implement this
	ReactorEvent::Joystick6DoF {
		x: (j1x + j2x + j3x) / 3,
		y: (j1y + j2y + j3y) / 3,
		z: (j1x - j2x) / 2,
		rx: (j1y - j2y) / 2,
		ry: (j1x + j2x - 2 * j3x) / 3,
		rz: (j1y + j2y - 2 * j3y) / 3,
	}
}

#[derive(Debug, Default)]
pub struct Joystick6DOFMid();

impl Middleware for Joystick6DOFMid {
	fn process(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = Option<ReactorEvent>> + '_>> {
		Box::pin(async move {
			match value {
				ReactorEvent::Analog6Axis(j1x, j1y, j2x, j2y, j3x, j3y) => Some(calculate_triangular_6dof(j1x, j1y, j2x, j2y, j3x, j3y)),
				_ => None,
			}
		})
	}
}
