//! Path geometry.

use std::hash::{Hash, Hasher};

/// A point.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point with the given x and y coordinates.
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub(crate) fn to_tsp(self) -> tiny_skia_path::Point {
        tiny_skia_path::Point::from_xy(self.x, self.y)
    }
}

/// A size.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Size(tiny_skia_path::Size, f64, f64);

impl Size {
    /// Creates a new `Size` from width and height.
    ///
    /// Returns `None` if either the width or the height is not > 0.
    pub fn from_wh(width: f32, height: f32) -> Option<Self> {
        Self::from_wh64(f64::from(width), f64::from(height))
    }

    /// Create a size retaining exact PDF dimensions.
    pub fn from_wh64(width: f64, height: f64) -> Option<Self> {
        if !width.is_finite() || !height.is_finite() {
            return None;
        }
        Some(Self(
            tiny_skia_path::Size::from_wh(width as f32, height as f32)?,
            width,
            height,
        ))
    }
    /// Original height, without reduction to renderer precision.
    pub fn height64(&self) -> f64 {
        self.2
    }
    /// Original width, without reduction to renderer precision.
    pub fn width64(&self) -> f64 {
        self.1
    }
    /// Returns the width of the size.
    pub fn width(&self) -> f32 {
        self.0.width()
    }

    /// Returns the height of the size.
    pub fn height(&self) -> f32 {
        self.0.height()
    }
}

/// A rectangle defined by left, top, right and bottom edges.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rect(tiny_skia_path::Rect);

impl Eq for Rect {}

impl Hash for Rect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.left().to_bits().hash(state);
        self.0.top().to_bits().hash(state);
        self.0.right().to_bits().hash(state);
        self.0.bottom().to_bits().hash(state);
    }
}

impl Rect {
    /// Creates new `Rect`.
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Option<Self> {
        Some(Self(tiny_skia_path::Rect::from_ltrb(
            left, top, right, bottom,
        )?))
    }

    /// Apply a transform to the rect.
    pub fn transform(self, transform: Transform) -> Option<Self> {
        Some(Self(self.0.transform(transform.to_tsp())?))
    }

    /// Creates new `Rect`.
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Option<Self> {
        Some(Self(tiny_skia_path::Rect::from_xywh(x, y, w, h)?))
    }

    /// Returns the left edge.
    pub fn left(&self) -> f32 {
        self.0.left()
    }

    /// Returns the top edge.
    pub fn top(&self) -> f32 {
        self.0.top()
    }

    /// Returns the right edge.
    pub fn right(&self) -> f32 {
        self.0.right()
    }

    /// Returns the bottom edge.
    pub fn bottom(&self) -> f32 {
        self.0.bottom()
    }

    /// Returns rect's width.
    pub fn width(&self) -> f32 {
        self.0.width()
    }

    /// Returns rect's height.
    pub fn height(&self) -> f32 {
        self.0.height()
    }

    pub(crate) fn to_tsp(self) -> tiny_skia_path::Rect {
        self.0
    }

    pub(crate) fn from_tsp(rect: tiny_skia_path::Rect) -> Self {
        Self(rect)
    }

    pub(crate) fn expand(&mut self, other: &Rect) {
        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        *self = Rect::from_ltrb(left, top, right, bottom).unwrap();
    }

    pub(crate) fn to_pdf_rect(self) -> pdf_writer::Rect {
        pdf_writer::Rect::new(
            self.left(),
            self.top(),
            self.left() + self.width(),
            self.top() + self.height(),
        )
    }
}

/// A quadrilateral.
///
/// The points should be given in the following order:
/// 1. bottom-left
/// 2. bottom-right
/// 3. top-right
/// 4. top-left
///
/// Where the coordinate system is Y-down:
/// ```md
///      |
///      |
/// -----+---->  X
///      |
///      |
///      v
///
///      Y
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct Quadrilateral(pub [Point; 4]);

impl From<Rect> for Quadrilateral {
    fn from(r: Rect) -> Self {
        Self([
            Point::from_xy(r.left(), r.bottom()),
            Point::from_xy(r.right(), r.bottom()),
            Point::from_xy(r.right(), r.top()),
            Point::from_xy(r.left(), r.top()),
        ])
    }
}

/// An affine transformation matrix.
///
/// Unlike other types, doesn't guarantee to be valid. This is Skia quirk.
/// Meaning Transform(0, 0, 0, 0, 0, 0) is ok, while it's technically not.
/// Non-finite values are also not an error.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Transform([f64; 6]);

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// Identity transform.
    pub fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }
    /// Create from single-precision coefficients (retained for API compatibility).
    pub fn from_row(sx: f32, ky: f32, kx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        Self::from_row64([sx, ky, kx, sy, tx, ty].map(f64::from))
    }
    /// Create without reducing PDF coordinate precision.
    pub fn from_row64(coefficients: [f64; 6]) -> Self {
        Self(coefficients)
    }
    /// Double-precision coefficients in PDF matrix order.
    pub fn coefficients64(self) -> [f64; 6] {
        self.0
    }
    /// Translation.
    pub fn from_translate(tx: f32, ty: f32) -> Self {
        Self::from_row(1.0, 0.0, 0.0, 1.0, tx, ty)
    }
    /// Scaling.
    pub fn from_scale(sx: f32, sy: f32) -> Self {
        Self::from_row(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }
    /// Skewing.
    pub fn from_skew(kx: f32, ky: f32) -> Self {
        Self::from_tsp(tiny_skia_path::Transform::from_skew(kx, ky))
    }
    /// Rotation in degrees.
    pub fn from_rotate(angle: f32) -> Self {
        Self::from_tsp(tiny_skia_path::Transform::from_rotate(angle))
    }
    /// Rotation at a position.
    pub fn from_rotate_at(angle: f32, tx: f32, ty: f32) -> Self {
        Self::from_tsp(tiny_skia_path::Transform::from_rotate_at(angle, tx, ty))
    }
    /// The sx component, for renderer APIs.
    pub fn sx(&self) -> f32 {
        self.0[0] as f32
    }
    /// The ky component, for renderer APIs.
    pub fn ky(&self) -> f32 {
        self.0[1] as f32
    }
    /// The kx component, for renderer APIs.
    pub fn kx(&self) -> f32 {
        self.0[2] as f32
    }
    /// The sy component, for renderer APIs.
    pub fn sy(&self) -> f32 {
        self.0[3] as f32
    }
    /// The tx component, for renderer APIs.
    pub fn tx(&self) -> f32 {
        self.0[4] as f32
    }
    /// The ty component, for renderer APIs.
    pub fn ty(&self) -> f32 {
        self.0[5] as f32
    }
    /// Inverse, when finite and nonsingular.
    pub fn invert(&self) -> Option<Self> {
        let [a, b, c, d, e, f] = self.0;
        let det = a * d - b * c;
        if det == 0.0 || !det.is_finite() {
            return None;
        }
        let inv = [
            d / det,
            -b / det,
            -c / det,
            a / det,
            (c * f - d * e) / det,
            (b * e - a * f) / det,
        ];
        inv.iter().all(|v| v.is_finite()).then_some(Self(inv))
    }
    pub(crate) fn pre_concat(&self, other: Self) -> Self {
        let [a, b, c, d, e, f] = self.0;
        let [g, h, i, j, k, l] = other.0;
        Self([
            a * g + c * h,
            b * g + d * h,
            a * i + c * j,
            b * i + d * j,
            a * k + c * l + e,
            b * k + d * l + f,
        ])
    }
    pub(crate) fn post_concat(&self, other: Self) -> Self {
        other.pre_concat(*self)
    }
    pub(crate) fn to_tsp(self) -> tiny_skia_path::Transform {
        tiny_skia_path::Transform::from_row(
            self.sx(),
            self.ky(),
            self.kx(),
            self.sy(),
            self.tx(),
            self.ty(),
        )
    }
    pub(crate) fn from_tsp(ts: tiny_skia_path::Transform) -> Self {
        Self::from_row(ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty)
    }
    pub(crate) fn to_pdf_transform(self) -> [f32; 6] {
        self.0.map(|v| v as f32)
    }
}
impl Eq for Transform {}
impl Hash for Transform {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for v in self.0 {
            (if v == 0.0 { 0.0 } else { v }).to_bits().hash(state);
        }
    }
}

/// A path.
pub struct Path(
    pub(crate) tiny_skia_path::Path,
    pub(crate) Option<Vec<PathCommand64>>,
);

impl Path {
    /// Apply a transformation to the path.
    pub fn transform(self, transform: Transform) -> Option<Self> {
        let precise = self.1.map(|commands| {
            commands
                .into_iter()
                .map(|c| c.transformed(transform))
                .collect()
        });
        Some(Self(self.0.transform(transform.to_tsp())?, precise))
    }
}

/// A path builder.
#[derive(Default)]
pub struct PathBuilder(tiny_skia_path::PathBuilder);

impl PathBuilder {
    /// Create a new path.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds beginning of a contour.
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y)
    }

    /// Adds a line from the last point.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y)
    }

    /// Adds a quad curve from the last point to `x`, `y`.
    pub fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y)
    }
    /// Adds a cubic curve from the last point to `x`, `y`.
    pub fn cubic_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y)
    }

    /// Close the current contour.
    pub fn close(&mut self) {
        self.0.close()
    }

    /// Push a rectangle to the path.
    pub fn push_rect(&mut self, rect: Rect) {
        self.0.push_rect(rect.to_tsp())
    }

    /// Finish the current path.
    pub fn finish(self) -> Option<Path> {
        Some(Path(self.0.finish()?, None))
    }
}

/// Double-precision path commands. PDF coordinates need not be reduced to renderer precision.
#[derive(Clone, Copy, Debug)]
pub enum PathCommand64 {
    /// Begin a contour.
    Move([f64; 2]),
    /// Add a line.
    Line([f64; 2]),
    /// Add a cubic Bezier curve.
    Cubic([f64; 2], [f64; 2], [f64; 2]),
    /// Close a contour.
    Close,
}

impl PathCommand64 {
    pub(crate) fn transformed(self, t: Transform) -> Self {
        let [a, b, c, d, e, f] = t.coefficients64();
        let p = |p: [f64; 2]| [a * p[0] + c * p[1] + e, b * p[0] + d * p[1] + f];
        match self {
            Self::Move(a) => Self::Move(p(a)),
            Self::Line(a) => Self::Line(p(a)),
            Self::Cubic(a, b, c) => Self::Cubic(p(a), p(b), p(c)),
            Self::Close => Self::Close,
        }
    }
}

impl Path {
    /// Build a path retaining double precision in the PDF. The f32 shadow is used only for
    /// conservative bounds/stroke calculations. Reject non-finite or extreme coordinates and
    /// commands before the first move. Quadratic curves should be converted to cubics by callers.
    pub fn from_commands64(commands: Vec<PathCommand64>) -> Option<Self> {
        let mut builder = tiny_skia_path::PathBuilder::new();
        let mut started = false;
        let valid = |p: [f64; 2]| p.iter().all(|v| v.is_finite() && v.abs() <= 1e9);
        for command in &commands {
            match *command {
                PathCommand64::Move(p) if valid(p) => {
                    builder.move_to(p[0] as f32, p[1] as f32);
                    started = true;
                }
                PathCommand64::Line(p) if started && valid(p) => {
                    builder.line_to(p[0] as f32, p[1] as f32)
                }
                PathCommand64::Cubic(a, b, c) if started && valid(a) && valid(b) && valid(c) => {
                    builder.cubic_to(
                        a[0] as f32,
                        a[1] as f32,
                        b[0] as f32,
                        b[1] as f32,
                        c[0] as f32,
                        c[1] as f32,
                    )
                }
                PathCommand64::Close if started => builder.close(),
                _ => return None,
            }
        }
        Some(Self(builder.finish()?, Some(commands)))
    }
}
