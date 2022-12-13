use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn plus(&self, coord: &Vector2) -> Self {
        Self {
            x: self.x + coord.x,
            y: self.y + coord.y,
        }
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        let mag = if mag > 0.0 { mag } else { 1.0 };
        Self {
            x: self.x / mag,
            y: self.y / mag,
        }
    }
}

pub fn circle_rect_collision(
    circle: &Vector2,
    radius: f64,
    rect: &Vector2,
    w: f64,
    h: f64,
) -> Result<(), Vector2> {
    // get distance from closest edges
    let dx = match () {
        _ if circle.x < rect.x => rect.x,
        _ if circle.x > rect.x + w => rect.x + w,
        _ => circle.x,
    } - circle.x;

    let dy = match () {
        _ if circle.y < rect.y => rect.y,
        _ if circle.y > rect.y + h => rect.y + h,
        _ => circle.y,
    } - circle.y;

    // if the distance is less than the radius, collision!
    if dx * dx + dy * dy < radius * radius {
        Err(Vector2 { x: dx, y: dy })
    } else {
        Ok(())
    }
}

pub fn circle_circle_collision(
    p1: &Vector2,
    r1: f64,
    p2: &Vector2,
    r2: f64,
) -> Result<(), Vector2> {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    if dx * dx + dy * dy < r1 * r2 {
        Err(Vector2 { x: dx, y: dy })
    } else {
        Ok(())
    }
}
