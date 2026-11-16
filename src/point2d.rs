// SPDX-License-Identifier: GPL-3.0-only
// (C) 2025 Vassilij Nadarajah
// packages@vassilijnadarajah.de

//! The point type and the trait that lets the metrics take other point representations.

use std::ops::{Add, Mul, Sub};

/// A point in the plane, or the vector from the origin to it.
///
/// Coordinates are planar and carry no unit of their own: `x` and `y` must be in the same unit and
/// in a projected coordinate system, not in degrees of longitude and latitude.
///
/// ```
/// use bos_geo::Point2D;
///
/// let a = Point2D::new(1.0, 2.0);
/// let b = Point2D::new(4.0, 6.0);
/// assert_eq!((b - a).length(), 5.0);
/// assert_eq!(a + (b - a) * 0.5, Point2D::new(2.5, 4.0));
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point2D {
    /// Coordinate along the first axis (easting).
    pub x: f64,
    /// Coordinate along the second axis (northing).
    pub y: f64,
}

impl Point2D {
    /// Creates a point from anything that converts to `f64` without loss, such as `f32` or `i32`.
    ///
    /// ```
    /// use bos_geo::Point2D;
    ///
    /// assert_eq!(Point2D::new(3, 4), Point2D { x: 3.0, y: 4.0 });
    /// ```
    pub fn new<T: Into<f64>>(x: T, y: T) -> Self {
        Point2D {
            x: x.into(),
            y: y.into(),
        }
    }

    /// Euclidean length of the vector: the distance of the point from the origin.
    ///
    /// ```
    /// use bos_geo::Point2D;
    ///
    /// assert_eq!(Point2D::new(3.0, 4.0).length(), 5.0);
    /// ```
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// The vector scaled to length 1. Both components are NaN for the zero vector, which has no
    /// direction.
    ///
    /// ```
    /// use bos_geo::Point2D;
    ///
    /// assert_eq!(Point2D::new(3.0, 4.0).normalized(), Point2D::new(0.6, 0.8));
    /// ```
    pub fn normalized(self) -> Self {
        let length = self.length();
        Point2D {
            x: self.x / length,
            y: self.y / length,
        }
    }

    /// Dot product of two vectors.
    ///
    /// ```
    /// use bos_geo::Point2D;
    ///
    /// assert_eq!(Point2D::new(1.0, 2.0).dot(Point2D::new(3.0, 4.0)), 11.0);
    /// ```
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Cross product of two vectors in the plane: the signed area of the parallelogram they span,
    /// positive if `other` is counter-clockwise from `self`.
    ///
    /// ```
    /// use bos_geo::Point2D;
    ///
    /// let east = Point2D::new(1.0, 0.0);
    /// let north = Point2D::new(0.0, 1.0);
    /// assert_eq!(east.cross(north), 1.0);
    /// assert_eq!(north.cross(east), -1.0);
    /// ```
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }
}

impl Add for Point2D {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// Scales a vector.
impl Mul<f64> for Point2D {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

/// A type that can be read as a point in the plane.
///
/// The metrics take slices of any such type, so a line can be passed the way it is stored.
///
/// ```
/// use bos_geo::{AsPoint2D, Point2D};
///
/// let point = Point2D::new(1.0, 2.0);
/// assert_eq!([1.0, 2.0].to_point2d(), point);
/// assert_eq!((1.0, 2.0).to_point2d(), point);
/// assert_eq!([1_i32, 2].to_point2d(), point);
/// assert_eq!(vec![1.0_f32, 2.0].to_point2d(), point);
/// ```
pub trait AsPoint2D {
    /// The point this value stands for.
    fn to_point2d(&self) -> Point2D;
}

impl AsPoint2D for Point2D {
    fn to_point2d(&self) -> Point2D {
        *self
    }
}

/// A vector of exactly two coordinates, `[x, y]`.
///
/// # Panics
///
/// `to_point2d` panics if the vector does not have exactly two elements.
impl<T: Into<f64> + Copy> AsPoint2D for Vec<T> {
    fn to_point2d(&self) -> Point2D {
        assert!(self.len() == 2, "Vec must have exactly two elements.");
        Point2D::new(self[0], self[1])
    }
}

/// An array `[x, y]`.
impl<T: Into<f64> + Copy> AsPoint2D for [T; 2] {
    fn to_point2d(&self) -> Point2D {
        Point2D::new(self[0], self[1])
    }
}

/// A tuple `(x, y)`.
impl<T: Into<f64> + Copy> AsPoint2D for (T, T) {
    fn to_point2d(&self) -> Point2D {
        Point2D::new(self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_of_a_vector() {
        assert_eq!(Point2D::new(3, 4).length(), 5.0);
        assert_eq!(Point2D::new(0, 0).length(), 0.0);
    }

    #[test]
    fn normalized_vector_has_length_one() {
        assert_eq!(Point2D::new(3, 4).normalized(), Point2D::new(0.6, 0.8));
    }

    #[test]
    fn normalized_zero_vector_is_nan() {
        let normalized = Point2D::new(0, 0).normalized();
        assert!(normalized.x.is_nan() && normalized.y.is_nan());
    }

    #[test]
    fn dot_and_cross_product() {
        let a = Point2D::new(1.0, 2.0);
        let b = Point2D::new(3.0, 4.0);
        assert_eq!(a.dot(b), 11.0);
        assert_eq!(a.cross(b), -2.0);
        assert_eq!(b.cross(a), 2.0);
    }
}
