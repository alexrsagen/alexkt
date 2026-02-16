use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

use crate::serde_bigint::{deserialize_bigint, serialize_bigint};

/// Represents a point (possibly) on an elliptic curve.
///
/// This is either the point at infinity, or a point with affine coordinates `x` and `y`.
/// It is not guaranteed to be on the curve.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Point {
    Infinity,
    Point {
        /// x-coordinate of point
        #[serde(
            serialize_with = "serialize_bigint",
            deserialize_with = "deserialize_bigint"
        )]
        x: BigInt,
        /// y-coordinate of point
        #[serde(
            serialize_with = "serialize_bigint",
            deserialize_with = "deserialize_bigint"
        )]
        y: BigInt,
    },
}

#[derive(Clone, Debug)]
pub struct ProjectivePoint {
    x: BigInt,
    y: BigInt,
    z: BigInt,
}

impl ProjectivePoint {
    pub fn infinity() -> Self {
        ProjectivePoint {
            x: Zero::zero(),
            y: One::one(),
            z: Zero::zero(),
        }
    }
}

impl From<&Point> for ProjectivePoint {
    fn from(point: &Point) -> Self {
        match point {
            Point::Infinity => Self::infinity(),
            Point::Point { x, y } => ProjectivePoint {
                x: x.clone(),
                y: y.clone(),
                z: One::one(),
            },
        }
    }
}

pub trait WithCurve {
    fn curve(&self) -> &EllipticCurve;
}

pub trait WithPublicKey {
    fn public_key(&self) -> &PublicKey;
}

pub trait WithPrivateKey {
    fn private_key(&self) -> &PrivateKey;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurveKeyPair {
    pub curve: EllipticCurve,
    pub public: PublicKey,
    pub private: PrivateKey,
}

impl CurveKeyPair {
    pub fn new(
        p: BigInt,
        a: BigInt,
        b: BigInt,
        gx: BigInt,
        gy: BigInt,
        kx: BigInt,
        ky: BigInt,
        n: BigInt,
        key: BigInt,
    ) -> Self {
        Self {
            curve: EllipticCurve::new(p, a, b),
            public: PublicKey::new(gx, gy, kx, ky),
            private: PrivateKey::new(n, key),
        }
    }
}

impl WithCurve for CurveKeyPair {
    fn curve(&self) -> &EllipticCurve {
        &self.curve
    }
}

impl WithPublicKey for CurveKeyPair {
    fn public_key(&self) -> &PublicKey {
        &self.public
    }
}

impl WithPrivateKey for CurveKeyPair {
    fn private_key(&self) -> &PrivateKey {
        &self.private
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicKey {
    /// Base point (G)
    pub g: Point,
    /// Public key point (K)
    pub k: Point,
}

impl PublicKey {
    pub fn new(gx: BigInt, gy: BigInt, kx: BigInt, ky: BigInt) -> Self {
        Self {
            g: Point::Point { x: gx, y: gy },
            k: Point::Point { x: kx, y: ky },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrivateKey {
    /// Order of base point (G)
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub n: BigInt,
    /// Private key
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub key: BigInt,
}

impl PrivateKey {
    pub fn new(n: BigInt, key: BigInt) -> Self {
        Self { n, key }
    }

    pub fn private(&self) -> BigInt {
        &self.n - &self.key
    }
}

/// Weierstrass equation of this curve = y^2 = x^3 + a * x + b (mod p)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EllipticCurve {
    /// Finite field order
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub p: BigInt,
    /// Curve parameter a (see equation)
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub a: BigInt,
    /// Curve parameter b (see equation)
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub b: BigInt,
}

impl EllipticCurve {
    pub fn new(p: BigInt, a: BigInt, b: BigInt) -> Self {
        Self { p, a, b }
    }

    pub fn mod_inverse(&self, a: &BigInt) -> BigInt {
        let mut s = (BigInt::zero(), BigInt::one());
        let mut r = (self.p.clone(), a.clone());

        while !r.0.is_zero() {
            let q = &r.1 / &r.0;
            core::mem::swap(&mut r.0, &mut r.1);
            r.0 -= &q * &r.1;
            core::mem::swap(&mut s.0, &mut s.1);
            s.0 -= &q * &s.1;
        }

        if r.1 >= BigInt::zero() {
            s.1 % &self.p
        } else {
            -s.1 % &self.p
        }
    }

    pub fn double_point(&self, point: &ProjectivePoint) -> ProjectivePoint {
        if point.y.is_zero() {
            return ProjectivePoint::infinity();
        }

        let three = BigInt::from(3);
        let two = BigInt::from(2);

        let t = (&point.x * &point.x * &three + &self.a * &point.z * &point.z).mod_floor(&self.p);
        let u = (&point.y * &point.z * &two).mod_floor(&self.p);
        let v = (&u * &point.x * &point.y * &two).mod_floor(&self.p);
        let w = (&t * &t - &v * &two).mod_floor(&self.p);
        let x = (&u * &w).mod_floor(&self.p);
        let y = (&t * (&v - &w) - &u * &u * &point.y * &point.y * &two).mod_floor(&self.p);
        let z = (&u * &u * &u).mod_floor(&self.p);

        ProjectivePoint { x, y, z }
    }

    /// Adds two points on the curve together.
    ///
    /// If the points are the same, it doubles the point.
    ///
    /// If one of the points is the point at infinity, it returns the other point.
    ///
    /// If both points are the point at infinity, it returns the point at infinity.
    pub fn add_points(&self, point1: &Point, point2: &Point) -> Point {
        let point1: ProjectivePoint = point1.into();
        let point2: ProjectivePoint = point2.into();
        self.projective_to_affine(self.add_points_proj(&point1, &point2))
    }

    pub fn add_points_proj(
        &self,
        point1: &ProjectivePoint,
        point2: &ProjectivePoint,
    ) -> ProjectivePoint {
        if point1.z.is_zero() {
            return point2.clone();
        } else if point2.z.is_zero() {
            return point1.clone();
        }

        let t0 = (&point1.y * &point2.z).mod_floor(&self.p);
        let t1 = (&point2.y * &point1.z).mod_floor(&self.p);
        let u0 = (&point1.x * &point2.z).mod_floor(&self.p);
        let u1 = (&point2.x * &point1.z).mod_floor(&self.p);
        if u0 == u1 {
            if t0 == t1 {
                return self.double_point(point1);
            } else {
                return ProjectivePoint::infinity();
            }
        }

        let t = (&t0 - &t1).mod_floor(&self.p);
        let u = (&u0 - &u1).mod_floor(&self.p);
        let u2 = (&u * &u).mod_floor(&self.p);
        let v = (&point1.z * &point2.z).mod_floor(&self.p);
        let w = (&t * &t * &v - &u2 * (&u0 + &u1)).mod_floor(&self.p);
        let u3 = (&u * &u2).mod_floor(&self.p);
        let x = (&u * &w).mod_floor(&self.p);
        let y = (&t * (&u0 * &u2 - &w) - &t0 * &u3).mod_floor(&self.p);
        let z = (&u3 * &v).mod_floor(&self.p);

        ProjectivePoint { x, y, z }
    }

    pub fn projective_to_affine(&self, point: ProjectivePoint) -> Point {
        if point.z.is_zero() {
            return Point::Infinity;
        }

        let z_inv = self.mod_inverse(&point.z);
        let x = (&point.x * &z_inv).mod_floor(&self.p);
        let y = (&point.y * &z_inv).mod_floor(&self.p);

        Point::Point { x, y }
    }

    /// Multiplies a point by a scalar.
    ///
    /// Uses the double-and-add algorithm.
    pub fn multiply_point(&self, n: &BigInt, point: &Point) -> Point {
        let mut result = ProjectivePoint::infinity();
        let mut temp: ProjectivePoint = point.into();

        let mut n = n.clone();
        while n > BigInt::zero() {
            if (&n % BigInt::from(2)) == BigInt::one() {
                result = self.add_points_proj(&result, &temp);
            }
            temp = self.double_point(&temp);
            n >>= 1;
        }

        self.projective_to_affine(result)
    }

    /// TODO: Check if this method returns the same result as add_points_proj
    pub fn add_points_affine(&self, one: &Point, two: &Point) -> Option<Point> {
        if let (Point::Point { x: x1, y: y1 }, Point::Point { x: x2, y: y2 }) = (one, two) {
            if let Some(rtn) = (x1 - x2).modinv(&self.p) {
                if !rtn.is_zero() {
                    let s = (y1 - y2) * &rtn;
                    let x = (s.pow(2) - x1 - x2).modpow(&BigInt::one(), &self.p);
                    let y = (&s * (x1 - &x) - y1).modpow(&BigInt::one(), &self.p);
                    return Some(Point::Point { x, y });
                }
            }

            let one_x_pow_2_mul_3: BigInt = x1.pow(2) * 3;
            let two_y_mul_2: BigInt = y2 * 2;
            let two_x_mul_2: BigInt = x2 * 2;

            let s = (one_x_pow_2_mul_3 + &self.a) * two_y_mul_2.modinv(&self.p)?;
            let x = (s.pow(2) - two_x_mul_2).modpow(&BigInt::one(), &self.p);
            let y = (s * (x1 - &x) - y1).modpow(&BigInt::one(), &self.p);
            Some(Point::Point { x, y })
        } else if let Point::Infinity = one {
            Some(Point::Infinity)
        } else if let Point::Infinity = two {
            Some(Point::Infinity)
        } else {
            None
        }
    }

    pub fn has_point(&self, point: &Point) -> bool {
        match point {
            Point::Infinity => false,
            Point::Point { x, y } => {
                (y.pow(2).modpow(&BigInt::one(), &self.p)) == self.get_y_pow2_for_x(&x)
            }
        }
    }

    pub fn get_y_pow2_for_x(&self, x: &BigInt) -> BigInt {
        (&x.pow(3) + &self.a * x + &self.b).modpow(&BigInt::one(), &self.p)
    }

    pub fn get_y_for_y_pow2(&self, y_pow2: &BigInt) -> Option<BigInt> {
        mod_sqrt(y_pow2, &self.p)
    }

    pub fn get_point_for_x(&self, x: BigInt) -> Option<Point> {
        let y_pow2 = self.get_y_pow2_for_x(&x);
        self.get_y_for_y_pow2(&y_pow2)
            .map(|y| Point::Point { x, y })
    }
}

/// Calculates the legendre symbol of `p`: `1`, `0`, or `-1 mod p`
pub fn ls(a: &BigInt, p: &BigInt) -> BigInt {
    let exp = (p - BigInt::one()) / BigInt::from(2);
    a.modpow(&exp, p)
}

/// Calculates the modular square root of `n` such that `result^2 = n (mod p)`
/// using the Tonelli-Shanks algorithm. Returns `None` if `p` is not prime.
///
/// # Arguments
///
/// * `n` - The number to find the square root of
/// * `p` - The prime modulus (_must_ be prime)
pub fn mod_sqrt(n: &BigInt, p: &BigInt) -> Option<BigInt> {
    if !ls(n, p).is_one() {
        return None;
    }

    // Factor out powers of 2 from p - 1
    let mut q = p - 1;
    let mut s = BigInt::zero();
    while (&q & &BigInt::one()).is_zero() {
        s += 1;
        q >>= 1;
    }

    if s.is_one() {
        let result = n.modpow(&((p + 1) / 4), p);
        return Some(p - result);
    }

    // Find a non-square z such as ( z | p ) = -1
    let mut z = BigInt::from(2);
    while ls(&z, p) != p - 1 {
        z += 1;
    }

    let mut c = z.modpow(&q, p);
    let mut t = n.modpow(&q, p);
    let mut m = s;
    let mut result = n.modpow(&((q + 1) >> 1), p);

    while !t.is_one() {
        let mut i = BigInt::zero();
        let mut z = t.clone();
        while !z.is_one() && i < &m - 1 {
            z = &z * &z % p;
            i += 1;
        }

        let mut e = &m - &i - 1;
        let mut b = c.clone();
        while &e > &BigInt::zero() {
            b = &b * &b % p;
            e -= 1;
        }

        c = &b * &b % p;
        t = &t * &c % p;
        m = i;
        result = result * &b % p;
    }

    Some(p - result)
}
