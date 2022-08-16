use std::ops::Mul;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::paint3d::{Point2, Point3, Segment2, Segment3};

/// 三阶矩阵
struct Matrix3(Point3, Point3, Point3);

macro_rules! matrix3 {
    ($a11:expr,$a12:expr,$a13:expr,$a21:expr,$a22:expr,$a23:expr,$a31:expr,$a32:expr,$a33:expr,) => {
        Matrix3(($a11, $a12, $a13), ($a21, $a22, $a23), ($a31, $a32, $a33))
    };
}

impl Matrix3 {
    /// 三阶行列式
    fn determinant3(&self) -> f64 {
        self.0 .0 * self.1 .1 * self.2 .2
            + self.0 .1 * self.1 .2 * self.2 .0
            + self.0 .2 * self.1 .0 * self.2 .1
            - self.0 .2 * self.1 .1 * self.2 .0
            - self.0 .1 * self.1 .0 * self.2 .2
            - self.0 .0 * self.2 .1 * self.1 .2
    }
}

impl Mul for Matrix3 {
    type Output = Matrix3;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(
            (
                self.0 .0 * rhs.0 .0 + self.0 .1 * rhs.1 .0 + self.0 .2 * rhs.2 .0,
                self.0 .0 * rhs.0 .1 + self.0 .1 * rhs.1 .1 + self.0 .2 * rhs.2 .1,
                self.0 .0 * rhs.0 .2 + self.0 .1 * rhs.1 .2 + self.0 .2 * rhs.2 .2,
            ),
            (
                self.1 .0 * rhs.0 .0 + self.1 .1 * rhs.1 .0 + self.1 .2 * rhs.2 .0,
                self.1 .0 * rhs.0 .1 + self.1 .1 * rhs.1 .1 + self.1 .2 * rhs.2 .1,
                self.1 .0 * rhs.0 .2 + self.1 .1 * rhs.1 .2 + self.1 .2 * rhs.2 .2,
            ),
            (
                self.2 .0 * rhs.0 .0 + self.2 .1 * rhs.1 .0 + self.2 .2 * rhs.2 .0,
                self.2 .0 * rhs.0 .1 + self.2 .1 * rhs.1 .1 + self.2 .2 * rhs.2 .1,
                self.2 .0 * rhs.0 .2 + self.2 .1 * rhs.1 .2 + self.2 .2 * rhs.2 .2,
            ),
        )
    }
}

impl Mul<Point3> for Matrix3 {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        (
            self.0 .0 * rhs.0 + self.0 .1 * rhs.1 + self.0 .2 * rhs.2,
            self.1 .0 * rhs.0 + self.1 .1 * rhs.1 + self.1 .2 * rhs.2,
            self.2 .0 * rhs.0 + self.2 .1 * rhs.1 + self.2 .2 * rhs.2,
        )
    }
}

/// 求解非齐次线性方程组
///
/// `AX=B`
fn linear_equation(a: Matrix3, b: Point3) -> Option<Point3> {
    let det_a = a.determinant3();
    if det_a != 0. {
        let (a1, a2, a3) = (
            matrix3! {
                b.0, a.0.1, a.0.2,
                b.1, a.1.1, a.1.2,
                b.2, a.2.1, a.2.2,
            },
            matrix3! {
                a.0.0, b.0, a.0.2,
                a.1.0, b.1, a.1.2,
                a.2.0, b.2, a.2.2,
            },
            matrix3! {
                a.0.0, a.0.1, b.0,
                a.1.0, a.1.1, b.0,
                a.2.0, a.2.1, b.0,
            },
        );
        Some((
            a1.determinant3() / det_a,
            a2.determinant3() / det_a,
            a3.determinant3() / det_a,
        ))
    } else {
        None
    }
}

/// 绕轴逆时针旋转
fn rotal3(x: f64, y: f64, z: f64, rx: f64, ry: f64, rz: f64) -> Point3 {
    (matrix3! {
        1.,  0.,        0.,
        0.,  rx.cos(),  -rx.sin(),
        0.,  rx.sin(),  rx.cos(),
    }) * (matrix3! {
        ry.cos(),   0.,  ry.sin(),
        0.,         1.,  0.,
        -ry.sin(),  0.,  ry.cos(),
    }) * (matrix3! {
        rz.cos(),  -rz.sin(),  0.,
        rz.sin(),  rz.cos(),   0.,
        0.,        0.,         1.,
    }) * (x, y, z)
}

/// 成环
fn ring<Point: Copy>(points: &[Point]) -> Vec<(Point, Point)> {
    let mut ring = Vec::new();
    let len = points.len();
    for n in 0..len {
        ring.push((points[n], points[(n + 1) % len]));
    }
    ring
}

/// 点是否在区域内
///
/// 水平向下的射线穿过边的次数为奇数
///
fn pnpoly(points: &[Point2], x: f64, y: f64) -> bool {
    let (mut maxx, mut maxy, mut minx, mut miny) = (0., 0., 0., 0.);
    for (ax, ay) in points {
        if *ax > maxx {
            maxx = *ax
        }
        if *ax < minx {
            minx = *ax
        }
        if *ay > maxy {
            maxy = *ay
        }
        if *ay < miny {
            miny = *ay
        }
    }
    if x < minx || x > maxx || y < miny || y > maxy {
        return false;
    }
    let mut result = false;
    for ((ax, ay), (bx, by)) in ring(points) {
        if ((x > ax) != (x > bx)) && y > ((ay - by) * (x - ax) / (ax - bx)) + ay {
            result = !result;
        }
    }
    result
}

/// 点透视
fn perspective_project(
    object: Point3,
    screen_center: Point3,
    k: f64,
    w: f64,
    h: f64,
) -> Option<Point2> {
    let (sc02, sc12, sc22) = (
        screen_center.0.powi(2),
        screen_center.1.powi(2),
        screen_center.2.powi(2),
    );
    // 点与观察者在屏幕同侧
    if (object.0 * screen_center.0 + object.1 * screen_center.1 + object.2 * screen_center.2
        > sc02 + sc12 + sc22)
        == (k > 1.)
    {
        return None;
    }
    // 视线与屏幕的交点
    let cross3 = linear_equation(
        matrix3! {
            screen_center.0,                 screen_center.1,                 screen_center.2,
            object.1 - k * screen_center.1,  k * screen_center.0 - object.0,  0.,
            0.,                              object.2 - k * screen_center.2,  k * screen_center.1 - object.1,
        },
        (
            sc02 + sc12 + sc22,
            k * screen_center.0 * object.1 - k * screen_center.1 * object.0,
            k * screen_center.1 * object.2 - k * screen_center.2 * object.1,
        ),
    )?;
    let (l0, l1) = ((sc02 + sc12).sqrt(), (sc02 + sc12 + sc22).sqrt());
    let (wc1, hc0, l02, wc0) = (
        w * screen_center.1,
        h * screen_center.0,
        2. * l0,
        w * screen_center.0,
    );
    let (s2, s1, s3, s4) = (
        (wc1 + hc0) / l02,
        (wc1 - hc0) / l02,
        (wc0 + hc0) / l02,
        (wc0 - hc0) / l02,
    );
    let screen_points = [
        (screen_center.0 - s1, screen_center.1 + s4),
        (screen_center.0 + s2, screen_center.1 - s3),
        (screen_center.0 + s1, screen_center.1 - s4),
        (screen_center.0 - s2, screen_center.1 + s3),
    ];
    let cross2 = (matrix3! {
        screen_center.1 / l0,   screen_center.0 * screen_center.2 / (l0 * l1),  0.,
        -screen_center.0 / l0,  screen_center.1 * screen_center.2 / (l0 * l1),  0.,
        0.,                      l0 / l1,                                        0.,
    }) * (
        cross3.0 - (screen_center.0 - s1),
        cross3.1 - (screen_center.1 + s4),
        cross3.2 - (screen_center.2 + h * l0 / (2. * l1)),
    );
    // if !pnpoly(&screen_points, cross3.0, cross3.1) {
    //     return None;
    // }
    Some((cross2.0, cross2.1))
}

pub fn transform(
    segments: &[Segment3],
    // `dx, dy, dz`
    trans: (f64, f64, f64),
    // `rx, ry, rz`
    rotal: (f64, f64, f64),
    // `screen_center, k, w, h`
    screen: (Point3, f64, f64, f64),
) -> Vec<Segment2> {
    vec![((0., 0.), (150., 150.))]
}
