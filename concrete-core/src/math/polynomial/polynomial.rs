use std::fmt::Debug;
use std::iter::Iterator;

use crate::math::tensor::{AsMutSlice, AsMutTensor, AsRefTensor, Tensor};
use crate::numeric::{CastFrom, UnsignedInteger};
use crate::{ck_dim_eq, tensor_traits};

use super::*;

/// A dense polynomial.
///
/// This type represent a dense polynomial in $\mathbb{Z}_{2^q}\[X\] / <X^N + 1>$, composed of $N$
/// integer coefficients encoded on $q$ bits.
///
///  # Example:
///
/// ```
/// use concrete_core::math::polynomial::{Polynomial, PolynomialSize};
/// let poly = Polynomial::allocate(0 as u32, PolynomialSize(100));
/// assert_eq!(poly.polynomial_size(), PolynomialSize(100));
/// ```
#[derive(PartialEq, Debug, Clone)]
pub struct Polynomial<Cont> {
    pub(crate) tensor: Tensor<Cont>,
}

tensor_traits!(Polynomial);

impl<Scalar> Polynomial<Vec<Scalar>>
where
    Scalar: Copy,
{
    /// Allocates a new polynomial.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialSize};
    /// let poly = Polynomial::allocate(0 as u32, PolynomialSize(100));
    /// assert_eq!(poly.polynomial_size(), PolynomialSize(100));
    /// ```
    pub fn allocate(value: Scalar, coef_count: PolynomialSize) -> Polynomial<Vec<Scalar>> {
        Polynomial::from_container(vec![value; coef_count.0])
    }
}

impl<Cont> Polynomial<Cont> {
    /// Creates a polynomial from a container of values.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialSize};
    /// let vec = vec![0 as u32; 100];
    /// let poly = Polynomial::from_container(vec.as_slice());
    /// assert_eq!(poly.polynomial_size(), PolynomialSize(100));
    /// ```
    pub fn from_container(cont: Cont) -> Self {
        Polynomial {
            tensor: Tensor::from_container(cont),
        }
    }

    /// Returns the number of coefficients in the polynomial.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialSize};
    /// let poly = Polynomial::allocate(0 as u32, PolynomialSize(100));
    /// assert_eq!(poly.polynomial_size(), PolynomialSize(100));
    /// ```
    pub fn polynomial_size(&self) -> PolynomialSize
    where
        Self: AsRefTensor,
    {
        PolynomialSize(self.as_tensor().len())
    }

    /// Builds an iterator over `Monomial<&Coef>` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, MonomialDegree, PolynomialSize};
    /// let poly = Polynomial::allocate(0 as u32, PolynomialSize(100));
    /// for monomial in poly.monomial_iter(){
    ///     assert!(monomial.degree().0 <= 99)
    /// }
    /// assert_eq!(poly.monomial_iter().count(), 100);
    /// ```
    pub fn monomial_iter(&self) -> impl Iterator<Item = Monomial<&[<Self as AsRefTensor>::Element]>>
    where
        Self: AsRefTensor,
    {
        self.as_tensor()
            .subtensor_iter(1)
            .enumerate()
            .map(|(i, coef)| Monomial::from_container(coef.into_container(), MonomialDegree(i)))
    }

    /// Builds an iterator over `&Coef` elements, in order of increasing degree.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, MonomialDegree, PolynomialSize};
    /// let poly = Polynomial::allocate(0 as u32, PolynomialSize(100));
    /// for coef in poly.coefficient_iter(){
    ///     assert_eq!(*coef, 0);
    /// }
    /// assert_eq!(poly.coefficient_iter().count(), 100);
    /// ```
    pub fn coefficient_iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &<Self as AsRefTensor>::Element>
    where
        Self: AsRefTensor,
    {
        self.as_tensor().iter()
    }

    /// Returns the monomial of a given degree.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialSize, MonomialDegree};
    /// let poly = Polynomial::from_container(vec![16_u32,8,19,12,3]);
    /// let mono = poly.get_monomial(MonomialDegree(0));
    /// assert_eq!(*mono.get_coefficient(), 16_u32);
    /// let mono = poly.get_monomial(MonomialDegree(2));
    /// assert_eq!(*mono.get_coefficient(), 19_u32);
    /// ```
    pub fn get_monomial(
        &self,
        degree: MonomialDegree,
    ) -> Monomial<&[<Self as AsRefTensor>::Element]>
    where
        Self: AsRefTensor,
    {
        Monomial::from_container(
            self.as_tensor()
                .get_sub(degree.0..=degree.0)
                .into_container(),
            degree,
        )
    }

    /// Builds an iterator over `Monomial<&mut Coef>` elements.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{PolynomialSize, Polynomial};
    /// let mut poly = Polynomial::allocate(0 as u32, PolynomialSize(100));
    /// for mut monomial in poly.monomial_iter_mut(){
    ///     monomial.set_coefficient(monomial.degree().0 as u32);
    /// }
    /// for (i, monomial) in poly.monomial_iter().enumerate(){
    ///     assert_eq!(*monomial.get_coefficient(), i as u32);
    /// }
    /// assert_eq!(poly.monomial_iter_mut().count(), 100);
    /// ```
    pub fn monomial_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = Monomial<&mut [<Self as AsMutTensor>::Element]>>
    where
        Self: AsMutTensor,
    {
        self.as_mut_tensor()
            .subtensor_iter_mut(1)
            .enumerate()
            .map(|(i, coef)| Monomial::from_container(coef.into_container(), MonomialDegree(i)))
    }

    /// Builds an iterator over `&mut Coef` elements, in order of increasing degree.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{PolynomialSize, Polynomial};
    /// let mut poly = Polynomial::allocate(0 as u32, PolynomialSize(100));
    /// for mut coef in poly.coefficient_iter_mut(){
    ///     *coef = 1;
    /// }
    /// for coef in poly.coefficient_iter(){
    ///     assert_eq!(*coef, 1);
    /// }
    /// assert_eq!(poly.coefficient_iter_mut().count(), 100);
    /// ```
    pub fn coefficient_iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &mut <Self as AsMutTensor>::Element>
    where
        Self: AsMutTensor,
    {
        self.as_mut_tensor().iter_mut()
    }

    /// Returns the mutable monomial of a given degree.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialSize, MonomialDegree};
    /// let mut poly = Polynomial::from_container(vec![16_u32,8,19,12,3]);
    /// let mut mono = poly.get_mut_monomial(MonomialDegree(0));
    /// mono.set_coefficient(18);
    /// let mono = poly.get_monomial(MonomialDegree(0));
    /// assert_eq!(*mono.get_coefficient(), 18);
    /// ```
    pub fn get_mut_monomial(
        &mut self,
        degree: MonomialDegree,
    ) -> Monomial<&mut [<Self as AsMutTensor>::Element]>
    where
        Self: AsMutTensor,
    {
        Monomial::from_container(
            self.as_mut_tensor()
                .get_sub_mut(degree.0..=degree.0)
                .into_container(),
            degree,
        )
    }

    /// Fills the current polynomial, with the result of the (slow) product of two polynomials,
    /// reduced modulo $(X^N + 1)$.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialSize, MonomialDegree};
    /// let lhs = Polynomial::from_container(vec![4_u8, 5, 0]);
    /// let rhs = Polynomial::from_container(vec![7_u8, 9, 0]);
    /// let mut res = Polynomial::allocate(0 as u8, PolynomialSize(3));
    /// res.fill_with_wrapping_mul(&lhs, &rhs);
    /// assert_eq!(*res.get_monomial(MonomialDegree(0)).get_coefficient(), 28 as u8);
    /// assert_eq!(*res.get_monomial(MonomialDegree(1)).get_coefficient(), 71 as u8);
    /// assert_eq!(*res.get_monomial(MonomialDegree(2)).get_coefficient(), 45 as u8);
    /// ```
    pub fn fill_with_wrapping_mul<Coef, LhsCont, RhsCont>(
        &mut self,
        lhs: &Polynomial<LhsCont>,
        rhs: &Polynomial<RhsCont>,
    ) where
        Self: AsMutTensor<Element = Coef>,
        Polynomial<LhsCont>: AsRefTensor<Element = Coef>,
        Polynomial<RhsCont>: AsRefTensor<Element = Coef>,
        Coef: UnsignedInteger,
    {
        ck_dim_eq!(self.polynomial_size() => lhs.polynomial_size(), rhs.polynomial_size());
        self.coefficient_iter_mut().for_each(|a| *a = Coef::ZERO);
        let degree = lhs.polynomial_size().0 - 1;
        for lhsi in lhs.monomial_iter() {
            for rhsi in rhs.monomial_iter() {
                let target_degree = lhsi.degree().0 + rhsi.degree().0;
                if target_degree <= degree {
                    let element = self.as_mut_tensor().get_element_mut(target_degree);
                    let new = lhsi.get_coefficient().wrapping_mul(*rhsi.get_coefficient());
                    *element = element.wrapping_add(new);
                } else {
                    let element = self
                        .as_mut_tensor()
                        .get_element_mut(target_degree % (degree + 1));
                    let new = lhsi.get_coefficient().wrapping_mul(*rhsi.get_coefficient());
                    *element = element.wrapping_sub(new);
                }
            }
        }
    }

    /// Fills the current polynomial with the result of the product between an integer polynomial
    /// and binary one, reduced modulo $(X^N + 1)$.
    ///
    /// # Example:
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialSize, MonomialDegree};
    /// let poly = Polynomial::from_container(vec![1_u8, 2, 3]);
    /// let bin_poly = Polynomial::from_container(vec![false, true, true]);
    /// let mut res = Polynomial::allocate(133 as u8, PolynomialSize(3));
    /// res.fill_with_wrapping_binary_mul(&poly, &bin_poly);
    /// dbg!(&res);
    /// assert_eq!(*res.get_monomial(MonomialDegree(0)).get_coefficient(), 251 as u8);
    /// assert_eq!(*res.get_monomial(MonomialDegree(1)).get_coefficient(), 254 as u8);
    /// assert_eq!(*res.get_monomial(MonomialDegree(2)).get_coefficient(), 3 as u8);
    /// ```
    pub fn fill_with_wrapping_binary_mul<Coef, PolyCont, BinCont>(
        &mut self,
        poly: &Polynomial<PolyCont>,
        bin_poly: &Polynomial<BinCont>,
    ) where
        Self: AsMutTensor<Element = Coef>,
        Polynomial<PolyCont>: AsRefTensor<Element = Coef>,
        Polynomial<BinCont>: AsRefTensor<Element = bool>,
        Coef: UnsignedInteger,
    {
        ck_dim_eq!(
            self.polynomial_size() =>
            poly.polynomial_size(),
            bin_poly.polynomial_size()
        );
        self.coefficient_iter_mut().for_each(|a| *a = Coef::ZERO);
        self.update_with_wrapping_add_binary_mul(poly, bin_poly)
    }

    /// Adds the sum of the element-wise product between a list of integer polynomial, and a
    /// list of binary polynomial, to the current polynomial.
    ///
    /// I.e., if the current polynomial is $C(X)$, for a collection of polynomials $(P_i(X)))_i$
    /// and a collection of binary polynomials $(B_i(X))_i$ we perform the operation:
    /// $$
    /// C(X) := C(X) + \sum_i P_i(X) \times B_i(X) mod (X^N + 1)
    /// $$
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{PolynomialList, PolynomialSize, Polynomial, MonomialDegree};
    /// let poly_list = PolynomialList::from_container(
    ///     vec![100 as u8,20,3,4,5,6],
    ///     PolynomialSize(3)
    /// );
    /// let bin_poly_list = PolynomialList::from_container(
    ///     vec![false, true, true, true, false, false],
    ///     PolynomialSize(3)
    /// );
    /// let mut output = Polynomial::allocate(250 as u8, PolynomialSize(3));
    /// output.update_with_wrapping_add_binary_multisum(&poly_list, &bin_poly_list);
    /// assert_eq!(*output.get_monomial(MonomialDegree(0)).get_coefficient(), 231);
    /// assert_eq!(*output.get_monomial(MonomialDegree(1)).get_coefficient(), 96);
    /// assert_eq!(*output.get_monomial(MonomialDegree(2)).get_coefficient(), 120);
    /// ```
    pub fn update_with_wrapping_add_binary_multisum<Coef, InCont, BinCont>(
        &mut self,
        coef_list: &PolynomialList<InCont>,
        bin_list: &PolynomialList<BinCont>,
    ) where
        Self: AsMutTensor<Element = Coef>,
        PolynomialList<InCont>: AsRefTensor<Element = Coef>,
        PolynomialList<BinCont>: AsRefTensor<Element = bool>,
        for<'a> Polynomial<&'a [bool]>: AsRefTensor<Element = bool>,
        for<'a> Polynomial<&'a [Coef]>: AsRefTensor<Element = Coef>,
        Coef: UnsignedInteger,
    {
        for (poly, bin_poly) in coef_list.polynomial_iter().zip(bin_list.polynomial_iter()) {
            self.update_with_wrapping_add_binary_mul(&poly, &bin_poly);
        }
    }

    /// Subtracts the sum of the element-wise product between a list of integer polynomial, and a
    /// list of binary polynomial, to the current polynomial.
    ///
    /// I.e., if the current polynomial is $C(X)$, for a list of polynomials $(P_i(X)))_i$ and a
    /// list of  binary polynomials $(B_i(X))_i$ we perform the operation:
    /// $$
    /// C(X) := C(X) + \sum_i P_i(X) \times B_i(X) mod (X^N + 1)
    /// $$
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{PolynomialList, PolynomialSize, Polynomial, MonomialDegree};
    /// let poly_list = PolynomialList::from_container(
    ///     vec![100 as u8,20,3,4,5,6],
    ///     PolynomialSize(3)
    /// );
    /// let bin_poly_list = PolynomialList::from_container(
    ///     vec![false, true, true, true, false, false],
    ///     PolynomialSize(3)
    /// );
    /// let mut output = Polynomial::allocate(250 as u8, PolynomialSize(3));
    /// output.update_with_wrapping_sub_binary_multisum(&poly_list, &bin_poly_list);
    /// assert_eq!(*output.get_monomial(MonomialDegree(0)).get_coefficient(), 13);
    /// assert_eq!(*output.get_monomial(MonomialDegree(1)).get_coefficient(), 148);
    /// assert_eq!(*output.get_monomial(MonomialDegree(2)).get_coefficient(), 124);
    /// ```
    pub fn update_with_wrapping_sub_binary_multisum<Coef, InCont, BinCont>(
        &mut self,
        coef_list: &PolynomialList<InCont>,
        bin_list: &PolynomialList<BinCont>,
    ) where
        Self: AsMutTensor<Element = Coef>,
        PolynomialList<InCont>: AsRefTensor<Element = Coef>,
        PolynomialList<BinCont>: AsRefTensor<Element = bool>,
        for<'a> Polynomial<&'a [bool]>: AsRefTensor<Element = bool>,
        for<'a> Polynomial<&'a [Coef]>: AsRefTensor<Element = Coef>,
        Coef: UnsignedInteger + CastFrom<bool>,
    {
        for (poly, bin_poly) in coef_list.polynomial_iter().zip(bin_list.polynomial_iter()) {
            self.update_with_wrapping_sub_binary_mul(&poly, &bin_poly);
        }
    }
    /// Adds the result of the product between a integer polynomial and a binary one, reduced
    /// modulo $(X^N+1)$, to the current polynomial.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, MonomialDegree};
    /// let poly = Polynomial::from_container(vec![1_u8,2,3]);
    /// let bin_poly = Polynomial::from_container(vec![false, true, true]);
    /// let mut res = Polynomial::from_container(vec![1_u8, 0, 253]);
    /// res.update_with_wrapping_add_binary_mul(&poly, &bin_poly);
    /// assert_eq!(*res.get_monomial(MonomialDegree(0)).get_coefficient(), 252);
    /// assert_eq!(*res.get_monomial(MonomialDegree(1)).get_coefficient(), 254);
    /// assert_eq!(*res.get_monomial(MonomialDegree(2)).get_coefficient(), 0);
    /// ```
    pub fn update_with_wrapping_add_binary_mul<Coef, PolyCont, BinCont>(
        &mut self,
        polynomial: &Polynomial<PolyCont>,
        bin_polynomial: &Polynomial<BinCont>,
    ) where
        Self: AsMutTensor<Element = Coef>,
        Polynomial<PolyCont>: AsRefTensor<Element = Coef>,
        Polynomial<BinCont>: AsRefTensor<Element = bool>,
        Coef: UnsignedInteger + CastFrom<bool>,
    {
        ck_dim_eq!(
            self.polynomial_size() =>
            polynomial.polynomial_size(),
            bin_polynomial.polynomial_size()
        );
        let degree = polynomial.polynomial_size().0 - 1;
        for lhsi in polynomial.monomial_iter() {
            for rhsi in bin_polynomial.monomial_iter() {
                let target_degree = lhsi.degree().0 + rhsi.degree().0;
                let binary_bit = Coef::cast_from(*rhsi.get_coefficient());
                if target_degree <= degree {
                    let update = self
                        .as_tensor()
                        .get_element(target_degree)
                        .wrapping_add(*lhsi.get_coefficient() * binary_bit);
                    *self.as_mut_tensor().get_element_mut(target_degree) = update;
                } else {
                    let update = self
                        .as_tensor()
                        .get_element(target_degree % (degree + 1))
                        .wrapping_sub(*lhsi.get_coefficient() * binary_bit);
                    *self
                        .as_mut_tensor()
                        .get_element_mut(target_degree % (degree + 1)) = update;
                }
            }
        }
    }

    /// Subtracts the result of the product between an integer polynomial and a binary one, reduced
    /// modulo $(X^N+1)$, to the current polynomial.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, MonomialDegree};
    /// let poly = Polynomial::from_container(vec![1_u8,2,3]);
    /// let bin_poly = Polynomial::from_container(vec![false, true, true]);
    /// let mut res = Polynomial::from_container(vec![255_u8, 255, 1]);
    /// res.update_with_wrapping_sub_binary_mul(&poly, &bin_poly);
    /// assert_eq!(*res.get_monomial(MonomialDegree(0)).get_coefficient(), 4);
    /// assert_eq!(*res.get_monomial(MonomialDegree(1)).get_coefficient(), 1);
    /// assert_eq!(*res.get_monomial(MonomialDegree(2)).get_coefficient(), 254);
    /// ```
    pub fn update_with_wrapping_sub_binary_mul<Coef, PolyCont, BinCont>(
        &mut self,
        polynomial: &Polynomial<PolyCont>,
        bin_polynomial: &Polynomial<BinCont>,
    ) where
        Self: AsMutTensor<Element = Coef>,
        Polynomial<PolyCont>: AsRefTensor<Element = Coef>,
        Polynomial<BinCont>: AsRefTensor<Element = bool>,
        Coef: UnsignedInteger + CastFrom<bool>,
    {
        ck_dim_eq!(
            self.polynomial_size() =>
            polynomial.polynomial_size(),
            bin_polynomial.polynomial_size()
        );
        let degree = polynomial.polynomial_size().0 - 1;
        for lhsi in polynomial.monomial_iter() {
            for rhsi in bin_polynomial.monomial_iter() {
                let target_degree = lhsi.degree().0 + rhsi.degree().0;
                let binary_bit = Coef::cast_from(*rhsi.get_coefficient());
                if target_degree <= degree {
                    let update = self
                        .as_tensor()
                        .get_element(target_degree)
                        .wrapping_sub(*lhsi.get_coefficient() * binary_bit);
                    *self.as_mut_tensor().get_element_mut(target_degree) = update;
                } else {
                    let update = self
                        .as_tensor()
                        .get_element(target_degree % (degree + 1))
                        .wrapping_add(*lhsi.get_coefficient() * binary_bit);
                    *self
                        .as_mut_tensor()
                        .as_mut_slice()
                        .get_mut(target_degree % (degree + 1))
                        .unwrap() = update;
                }
            }
        }
    }

    /// Adds a integer polynomial to another one.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, MonomialDegree};
    /// let mut first = Polynomial::from_container(vec![1u8, 2, 3]);
    /// let second = Polynomial::from_container(vec![255u8, 255, 255]);
    /// first.update_with_wrapping_add(&second);
    /// assert_eq!(*first.get_monomial(MonomialDegree(0)).get_coefficient(), 0);
    /// assert_eq!(*first.get_monomial(MonomialDegree(1)).get_coefficient(), 1);
    /// assert_eq!(*first.get_monomial(MonomialDegree(2)).get_coefficient(), 2);
    /// ```
    pub fn update_with_wrapping_add<Coef, OtherCont>(&mut self, other: &Polynomial<OtherCont>)
    where
        Self: AsMutTensor<Element = Coef>,
        Polynomial<OtherCont>: AsRefTensor<Element = Coef>,
        Coef: UnsignedInteger,
    {
        ck_dim_eq!(
            self.polynomial_size() =>
            other.polynomial_size()
        );
        self.as_mut_tensor()
            .update_with_wrapping_add(other.as_tensor());
    }

    /// Subtracts an integer polynomial to another one.
    ///
    /// # Example
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, MonomialDegree};
    /// let mut first = Polynomial::from_container(vec![1u8, 2, 3]);
    /// let second = Polynomial::from_container(vec![4u8, 5, 6]);
    /// first.update_with_wrapping_sub(&second);
    /// assert_eq!(*first.get_monomial(MonomialDegree(0)).get_coefficient(), 253);
    /// assert_eq!(*first.get_monomial(MonomialDegree(1)).get_coefficient(), 253);
    /// assert_eq!(*first.get_monomial(MonomialDegree(2)).get_coefficient(), 253);
    /// ```
    pub fn update_with_wrapping_sub<Coef, OtherCont>(&mut self, other: &Polynomial<OtherCont>)
    where
        Self: AsMutTensor<Element = Coef>,
        Polynomial<OtherCont>: AsRefTensor<Element = Coef>,
        Coef: UnsignedInteger,
    {
        ck_dim_eq!(
            self.polynomial_size() =>
            other.polynomial_size()
        );
        self.as_mut_tensor()
            .update_with_wrapping_sub(other.as_tensor());
    }

    /// Multiplies (mod $(X^N+1)$), the current polynomial with a monomial of a given degree, and
    /// a coefficient of one.
    ///
    /// # Examples
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, MonomialDegree};
    /// let mut poly = Polynomial::from_container(vec![1u8,2,3]);
    /// poly.update_with_wrapping_monic_monomial_mul(MonomialDegree(2));
    /// assert_eq!(*poly.get_monomial(MonomialDegree(0)).get_coefficient(), 254);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(1)).get_coefficient(), 253);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(2)).get_coefficient(), 1);
    /// ```
    pub fn update_with_wrapping_monic_monomial_mul<Coef>(&mut self, monomial_degree: MonomialDegree)
    where
        Self: AsMutTensor<Element = Coef>,
        Coef: UnsignedInteger,
    {
        let full_cycles_count = monomial_degree.0 / self.as_tensor().len();
        if full_cycles_count % 2 != 0 {
            self.as_mut_tensor()
                .as_mut_slice()
                .iter_mut()
                .for_each(|a| *a = a.wrapping_neg());
        }
        let remaining_degree = monomial_degree.0 % self.as_tensor().len();
        self.as_mut_tensor()
            .as_mut_slice()
            .rotate_right(remaining_degree);
        self.as_mut_tensor()
            .as_mut_slice()
            .iter_mut()
            .take(remaining_degree)
            .for_each(|a| *a = a.wrapping_neg());
    }

    /// Divides (mod $(X^N+1)$), the current polynomial with a monomial of a given degree, and a
    /// coefficient of one.
    ///
    /// # Examples
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, MonomialDegree};
    /// let mut poly = Polynomial::from_container(vec![1u8,2,3]);
    /// poly.update_with_wrapping_unit_monomial_div(MonomialDegree(2));
    /// assert_eq!(*poly.get_monomial(MonomialDegree(0)).get_coefficient(), 3);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(1)).get_coefficient(), 255);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(2)).get_coefficient(), 254);
    /// ```
    pub fn update_with_wrapping_unit_monomial_div<Coef>(&mut self, monomial_degree: MonomialDegree)
    where
        Self: AsMutTensor<Element = Coef>,
        Coef: UnsignedInteger,
    {
        let full_cycles_count = monomial_degree.0 / self.as_tensor().len();
        if full_cycles_count % 2 != 0 {
            self.as_mut_tensor()
                .as_mut_slice()
                .iter_mut()
                .for_each(|a| *a = a.wrapping_neg());
        }
        let remaining_degree = monomial_degree.0 % self.as_tensor().len();
        self.as_mut_tensor()
            .as_mut_slice()
            .rotate_left(remaining_degree);
        self.as_mut_tensor()
            .as_mut_slice()
            .iter_mut()
            .rev()
            .take(remaining_degree)
            .for_each(|a| *a = a.wrapping_neg());
    }

    /// Adds multiple integer polynomials to the current one.
    ///
    /// # Examples
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialList, PolynomialSize};
    /// use concrete_core::math::polynomial::MonomialDegree;
    /// let mut poly = Polynomial::from_container(vec![1u8,2,3]);
    /// let poly_list = PolynomialList::from_container(vec![4u8,5,6,7,8,9], PolynomialSize(3));
    /// poly.update_with_wrapping_add_several(&poly_list);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(0)).get_coefficient(), 12);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(1)).get_coefficient(), 15);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(2)).get_coefficient(), 18);
    /// ```
    pub fn update_with_wrapping_add_several<Coef, InCont>(
        &mut self,
        coef_list: &PolynomialList<InCont>,
    ) where
        Self: AsMutTensor<Element = Coef>,
        PolynomialList<InCont>: AsRefTensor<Element = Coef>,
        for<'a> Polynomial<&'a [Coef]>: AsRefTensor<Element = Coef>,
        Coef: UnsignedInteger,
    {
        for poly in coef_list.polynomial_iter() {
            self.update_with_wrapping_add(&poly);
        }
    }

    /// Subtracts multiple integer polynomials to the current one.
    ///
    /// # Examples
    ///
    /// ```
    /// use concrete_core::math::polynomial::{Polynomial, PolynomialList, PolynomialSize};
    /// use concrete_core::math::polynomial::MonomialDegree;
    /// let mut poly = Polynomial::from_container(vec![1u32,2,3]);
    /// let poly_list = PolynomialList::from_container(vec![4u32,5,6,7,8,9], PolynomialSize(3));
    /// poly.update_with_wrapping_sub_several(&poly_list);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(0)).get_coefficient(), 4294967286);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(1)).get_coefficient(), 4294967285);
    /// assert_eq!(*poly.get_monomial(MonomialDegree(2)).get_coefficient(), 4294967284);
    /// ```
    pub fn update_with_wrapping_sub_several<Coef, InCont>(
        &mut self,
        coef_list: &PolynomialList<InCont>,
    ) where
        Self: AsMutTensor<Element = Coef>,
        PolynomialList<InCont>: AsRefTensor<Element = Coef>,
        for<'a> Polynomial<&'a [Coef]>: AsRefTensor<Element = Coef>,
        Coef: UnsignedInteger,
    {
        for poly in coef_list.polynomial_iter() {
            self.update_with_wrapping_sub(&poly);
        }
    }
}
