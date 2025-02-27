// Non-conventional component-wise operators.

use num::{Signed, Zero};
use std::ops::{Add, Mul};

use simba::scalar::{ClosedDiv, ClosedMul};
use simba::simd::SimdPartialOrd;

use crate::base::allocator::{Allocator, SameShapeAllocator};
use crate::base::constraint::{SameNumberOfColumns, SameNumberOfRows, ShapeConstraint};
use crate::base::dimension::Dim;
use crate::base::storage::{Storage, StorageMut};
use crate::base::{DefaultAllocator, Matrix, MatrixSum, OMatrix, Scalar};
use crate::ClosedAdd;

/// The type of the result of a matrix component-wise operation.
pub type MatrixComponentOp<T, R1, C1, R2, C2> = MatrixSum<T, R1, C1, R2, C2>;

impl<T: Scalar, R: Dim, C: Dim, S: Storage<T, R, C>> Matrix<T, R, C, S> {
    /// Computes the component-wise absolute value.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::Matrix2;
    /// let a = Matrix2::new(0.0, 1.0,
    ///                      -2.0, -3.0);
    /// assert_eq!(a.abs(), Matrix2::new(0.0, 1.0, 2.0, 3.0))
    /// ```
    #[inline]
    #[must_use]
    pub fn abs(&self) -> OMatrix<T, R, C>
    where
        T: Signed,
        DefaultAllocator: Allocator<T, R, C>,
    {
        let mut res = self.clone_owned();

        for e in res.iter_mut() {
            *e = e.abs();
        }

        res
    }

    // TODO: add other operators like component_ln, component_pow, etc. ?
}

macro_rules! component_binop_impl(
    ($($binop: ident, $binop_mut: ident, $binop_assign: ident, $cmpy: ident, $Trait: ident . $op: ident . $op_assign: ident, $desc:expr, $desc_cmpy:expr, $desc_mut:expr);* $(;)*) => {$(
        #[doc = $desc]
        #[inline]
        #[must_use]
        pub fn $binop<R2, C2, SB>(&self, rhs: &Matrix<T, R2, C2, SB>) -> MatrixComponentOp<T, R1, C1, R2, C2>
            where T: $Trait,
                  R2: Dim, C2: Dim,
                  SB: Storage<T, R2, C2>,
                  DefaultAllocator: SameShapeAllocator<T, R1, C1, R2, C2>,
                  ShapeConstraint:  SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> {

            assert_eq!(self.shape(), rhs.shape(), "Componentwise mul/div: mismatched matrix dimensions.");
            let mut res = self.clone_owned_sum();

            for j in 0 .. res.ncols() {
                for i in 0 .. res.nrows() {
                    unsafe {
                        res.get_unchecked_mut((i, j)).$op_assign(rhs.get_unchecked((i, j)).clone());
                    }
                }
            }

            res
        }

        // componentwise binop plus Y.
        #[doc = $desc_cmpy]
        #[inline]
        pub fn $cmpy<R2, C2, SB, R3, C3, SC>(&mut self, alpha: T, a: &Matrix<T, R2, C2, SB>, b: &Matrix<T, R3, C3, SC>, beta: T)
            where T: $Trait + Zero + Mul<T, Output = T> + Add<T, Output = T>,
                  R2: Dim, C2: Dim,
                  R3: Dim, C3: Dim,
                  SA: StorageMut<T, R1, C1>,
                  SB: Storage<T, R2, C2>,
                  SC: Storage<T, R3, C3>,
                  ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> +
                                   SameNumberOfRows<R1, R3> + SameNumberOfColumns<C1, C3> {
            assert_eq!(self.shape(), a.shape(), "Componentwise mul/div: mismatched matrix dimensions.");
            assert_eq!(self.shape(), b.shape(), "Componentwise mul/div: mismatched matrix dimensions.");

            if beta.is_zero() {
                for j in 0 .. self.ncols() {
                    for i in 0 .. self.nrows() {
                        unsafe {
                            let res = alpha.clone() * a.get_unchecked((i, j)).clone().$op(b.get_unchecked((i, j)).clone());
                            *self.get_unchecked_mut((i, j)) = res;
                        }
                    }
                }
            }
            else {
                for j in 0 .. self.ncols() {
                    for i in 0 .. self.nrows() {
                        unsafe {
                            let res = alpha.clone() * a.get_unchecked((i, j)).clone().$op(b.get_unchecked((i, j)).clone());
                            *self.get_unchecked_mut((i, j)) = beta.clone() * self.get_unchecked((i, j)).clone() + res;
                        }
                    }
                }
            }
        }

        #[doc = $desc_mut]
        #[inline]
        pub fn $binop_assign<R2, C2, SB>(&mut self, rhs: &Matrix<T, R2, C2, SB>)
            where T: $Trait,
                  R2: Dim,
                  C2: Dim,
                  SA: StorageMut<T, R1, C1>,
                  SB: Storage<T, R2, C2>,
                  ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> {

            assert_eq!(self.shape(), rhs.shape(), "Componentwise mul/div: mismatched matrix dimensions.");

            for j in 0 .. self.ncols() {
                for i in 0 .. self.nrows() {
                    unsafe {
                        self.get_unchecked_mut((i, j)).$op_assign(rhs.get_unchecked((i, j)).clone());
                    }
                }
            }
        }

        #[doc = $desc_mut]
        #[inline]
        #[deprecated(note = "This is renamed using the `_assign` suffix instead of the `_mut` suffix.")]
        pub fn $binop_mut<R2, C2, SB>(&mut self, rhs: &Matrix<T, R2, C2, SB>)
            where T: $Trait,
                  R2: Dim,
                  C2: Dim,
                  SA: StorageMut<T, R1, C1>,
                  SB: Storage<T, R2, C2>,
                  ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2> {
            self.$binop_assign(rhs)
        }
    )*}
);

/// # Componentwise operations
impl<T: Scalar, R1: Dim, C1: Dim, SA: Storage<T, R1, C1>> Matrix<T, R1, C1, SA> {
    component_binop_impl!(
        component_mul, component_mul_mut, component_mul_assign, cmpy, ClosedMul.mul.mul_assign,
        r"
        Componentwise matrix or vector multiplication.

        # Example

        ```
        # use nalgebra::Matrix2;
        let a = Matrix2::new(0.0, 1.0, 2.0, 3.0);
        let b = Matrix2::new(4.0, 5.0, 6.0, 7.0);
        let expected = Matrix2::new(0.0, 5.0, 12.0, 21.0);

        assert_eq!(a.component_mul(&b), expected);
        ```
        ",
        r"
        Computes componentwise `self[i] = alpha * a[i] * b[i] + beta * self[i]`.

        # Example
        ```
        # use nalgebra::Matrix2;
        let mut m = Matrix2::new(0.0, 1.0, 2.0, 3.0);
        let a = Matrix2::new(0.0, 1.0, 2.0, 3.0);
        let b = Matrix2::new(4.0, 5.0, 6.0, 7.0);
        let expected = (a.component_mul(&b) * 5.0) + m * 10.0;

        m.cmpy(5.0, &a, &b, 10.0);
        assert_eq!(m, expected);
        ```
        ",
        r"
        Inplace componentwise matrix or vector multiplication.

        # Example
        ```
        # use nalgebra::Matrix2;
        let mut a = Matrix2::new(0.0, 1.0, 2.0, 3.0);
        let b = Matrix2::new(4.0, 5.0, 6.0, 7.0);
        let expected = Matrix2::new(0.0, 5.0, 12.0, 21.0);

        a.component_mul_assign(&b);

        assert_eq!(a, expected);
        ```
        ";
        component_div, component_div_mut, component_div_assign, cdpy, ClosedDiv.div.div_assign,
        r"
        Componentwise matrix or vector division.

        # Example

        ```
        # use nalgebra::Matrix2;
        let a = Matrix2::new(0.0, 1.0, 2.0, 3.0);
        let b = Matrix2::new(4.0, 5.0, 6.0, 7.0);
        let expected = Matrix2::new(0.0, 1.0 / 5.0, 2.0 / 6.0, 3.0 / 7.0);

        assert_eq!(a.component_div(&b), expected);
        ```
        ",
        r"
        Computes componentwise `self[i] = alpha * a[i] / b[i] + beta * self[i]`.

        # Example
        ```
        # use nalgebra::Matrix2;
        let mut m = Matrix2::new(0.0, 1.0, 2.0, 3.0);
        let a = Matrix2::new(4.0, 5.0, 6.0, 7.0);
        let b = Matrix2::new(4.0, 5.0, 6.0, 7.0);
        let expected = (a.component_div(&b) * 5.0) + m * 10.0;

        m.cdpy(5.0, &a, &b, 10.0);
        assert_eq!(m, expected);
        ```
        ",
        r"
        Inplace componentwise matrix or vector division.

        # Example
        ```
        # use nalgebra::Matrix2;
        let mut a = Matrix2::new(0.0, 1.0, 2.0, 3.0);
        let b = Matrix2::new(4.0, 5.0, 6.0, 7.0);
        let expected = Matrix2::new(0.0, 1.0 / 5.0, 2.0 / 6.0, 3.0 / 7.0);

        a.component_div_assign(&b);

        assert_eq!(a, expected);
        ```
        ";
        // TODO: add other operators like bitshift, etc. ?
    );

    /// Computes the infimum (aka. componentwise min) of two matrices/vectors.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::Matrix2;
    /// let u = Matrix2::new(4.0, 2.0, 1.0, -2.0);
    /// let v = Matrix2::new(2.0, 4.0, -2.0, 1.0);
    /// let expected = Matrix2::new(2.0, 2.0, -2.0, -2.0);
    /// assert_eq!(u.inf(&v), expected)
    /// ```
    #[inline]
    #[must_use]
    pub fn inf(&self, other: &Self) -> OMatrix<T, R1, C1>
    where
        T: SimdPartialOrd,
        DefaultAllocator: Allocator<T, R1, C1>,
    {
        self.zip_map(other, |a, b| a.simd_min(b))
    }

    /// Computes the supremum (aka. componentwise max) of two matrices/vectors.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::Matrix2;
    /// let u = Matrix2::new(4.0, 2.0, 1.0, -2.0);
    /// let v = Matrix2::new(2.0, 4.0, -2.0, 1.0);
    /// let expected = Matrix2::new(4.0, 4.0, 1.0, 1.0);
    /// assert_eq!(u.sup(&v), expected)
    /// ```
    #[inline]
    #[must_use]
    pub fn sup(&self, other: &Self) -> OMatrix<T, R1, C1>
    where
        T: SimdPartialOrd,
        DefaultAllocator: Allocator<T, R1, C1>,
    {
        self.zip_map(other, |a, b| a.simd_max(b))
    }

    /// Computes the (infimum, supremum) of two matrices/vectors.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::Matrix2;
    /// let u = Matrix2::new(4.0, 2.0, 1.0, -2.0);
    /// let v = Matrix2::new(2.0, 4.0, -2.0, 1.0);
    /// let expected = (Matrix2::new(2.0, 2.0, -2.0, -2.0), Matrix2::new(4.0, 4.0, 1.0, 1.0));
    /// assert_eq!(u.inf_sup(&v), expected)
    /// ```
    #[inline]
    #[must_use]
    pub fn inf_sup(&self, other: &Self) -> (OMatrix<T, R1, C1>, OMatrix<T, R1, C1>)
    where
        T: SimdPartialOrd,
        DefaultAllocator: Allocator<T, R1, C1>,
    {
        // TODO: can this be optimized?
        (self.inf(other), self.sup(other))
    }

    /// Adds a scalar to `self`.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::Matrix2;
    /// let u = Matrix2::new(1.0, 2.0, 3.0, 4.0);
    /// let s = 10.0;
    /// let expected = Matrix2::new(11.0, 12.0, 13.0, 14.0);
    /// assert_eq!(u.add_scalar(s), expected)
    /// ```
    #[inline]
    #[must_use = "Did you mean to use add_scalar_mut()?"]
    pub fn add_scalar(&self, rhs: T) -> OMatrix<T, R1, C1>
    where
        T: ClosedAdd,
        DefaultAllocator: Allocator<T, R1, C1>,
    {
        let mut res = self.clone_owned();
        res.add_scalar_mut(rhs);
        res
    }

    /// Adds a scalar to `self` in-place.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::Matrix2;
    /// let mut u = Matrix2::new(1.0, 2.0, 3.0, 4.0);
    /// let s = 10.0;
    /// u.add_scalar_mut(s);
    /// let expected = Matrix2::new(11.0, 12.0, 13.0, 14.0);
    /// assert_eq!(u, expected)
    /// ```
    #[inline]
    pub fn add_scalar_mut(&mut self, rhs: T)
    where
        T: ClosedAdd,
        SA: StorageMut<T, R1, C1>,
    {
        for e in self.iter_mut() {
            *e += rhs.clone()
        }
    }
}

// Calculus
impl<T: Into<f64> + Copy, R1: Dim, C1: Dim, SA: Storage<T, R1, C1>> Matrix<T, R1, C1, SA>
where
    DefaultAllocator: Allocator<f64, R1, C1>,
{
    /// Computes the gradient of self and returns a pair specifying the y (row) component and x
    /// (col) component of this gradient respectively.
    ///
    /// The gradient is calculated using the second order finite differences
    /// (<https://en.wikipedia.org/wiki/Finite_difference_coefficient>) with the central difference
    /// being used for central elements and either the forward or backward difference being used
    /// around the edges.
    ///
    /// A value of `None` is returned if the matrix is too small to compute these differences
    /// (currently size must be >=3 in all dimensions). If it is known at compile time that this
    /// will never be the case, consider using [`gradient_unchecked`](Self::gradient_unchecked).
    pub fn gradient(&self) -> Option<(OMatrix<f64, R1, C1>, OMatrix<f64, R1, C1>)> {
        let height = self.nrows();
        let width = self.ncols();

        if height < 3 || width < 3 {
            None
        } else {
            // Values around the edge of the matrix will use either forward or backward 2nd order
            // approximation for gradient. Other values in the middle will use the central 2nd order
            // approximation. See citation in doc comment for details.
            let to_f64 = |x| <T as Into<f64>>::into(x);
            // Gives gradient at x1
            let forward_approx =
                |x1, x2, x3| (-3.0 * to_f64(x1) + 4.0 * to_f64(x2) - to_f64(x3)) / 2.0;
            // Gives gradient at x2
            let central_approx = |x1, x3| (to_f64(x3) - to_f64(x1)) / 2.0;
            // Gives gradient at x3
            let backward_approx =
                |x1, x2, x3| (to_f64(x1) - 4.0 * to_f64(x2) + 3.0 * to_f64(x3)) / 2.0;

            let mut grad_y = OMatrix::zeros_generic(R1::from_usize(height), C1::from_usize(width));
            let mut grad_x = OMatrix::zeros_generic(R1::from_usize(height), C1::from_usize(width));
            for x in 0..width {
                // Top and bottom row of grad_y
                grad_y[(0, x)] = forward_approx(self[(0, x)], self[(1, x)], self[(2, x)]);
                grad_y[(height - 1, x)] = backward_approx(
                    self[(height - 3, x)],
                    self[(height - 2, x)],
                    self[(height - 1, x)],
                );
            }
            for y in 0..height {
                // Left and right column of grad_x
                grad_x[(y, 0)] = forward_approx(self[(y, 0)], self[(y, 1)], self[(y, 2)]);
                grad_x[(y, width - 1)] = backward_approx(
                    self[(y, width - 3)],
                    self[(y, width - 2)],
                    self[(y, width - 1)],
                );
            }
            for x in 1..(width - 1) {
                // Remaining elements in top and bottom row of grad_x
                grad_x[(0, x)] = central_approx(self[(0, x - 1)], self[(0, x + 1)]);
                grad_x[(height - 1, x)] =
                    central_approx(self[(height - 1, x - 1)], self[(height - 1, x + 1)]);
            }
            for y in 1..(height - 1) {
                // Remaining elements in left and right column of grad_y
                grad_y[(y, 0)] = central_approx(self[(y - 1, 0)], self[(y + 1, 0)]);
                grad_y[(y, width - 1)] =
                    central_approx(self[(y - 1, width - 1)], self[(y + 1, width - 1)]);
            }
            for x in 1..(width - 1) {
                for y in 1..(width - 1) {
                    // All elements not on an edge
                    grad_x[(y, x)] = central_approx(self[(y, x - 1)], self[(y, x + 1)]);
                    grad_y[(y, x)] = central_approx(self[(y - 1, x)], self[(y + 1, x)]);
                }
            }
            Some((grad_y, grad_x))
        }
    }

    /// The same as [`gradient`](Self::gradient) but panics if the matrix is too small instead of
    /// returning an option.
    pub fn gradient_unchecked(&self) -> (OMatrix<f64, R1, C1>, OMatrix<f64, R1, C1>) {
        self.gradient().expect(
            "Error calculating gradient of matrix: one of the dimensions has size of less than 3.",
        )
    }
}
