use std::ops::Add;

use serde::{Deserialize, Serialize};

use crate::crypto::encoding::{Plaintext, PlaintextList};
use crate::crypto::ggsw::GgswCiphertext;
use crate::crypto::glwe::{GlweCiphertext, GlweList};
use crate::crypto::secret::LweSecretKey;
use crate::crypto::{GlweDimension, PlaintextCount, UnsignedTorus};
use crate::math::dispersion::DispersionParameter;
use crate::math::polynomial::{PolynomialList, PolynomialSize};
use crate::math::random;
use crate::math::tensor::{AsMutSlice, AsMutTensor, AsRefSlice, AsRefTensor, Tensor};
use crate::numeric::Numeric;
use crate::{ck_dim_div, ck_dim_eq, tensor_traits};

/// A GLWE secret key
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GlweSecretKey<Container> {
    tensor: Tensor<Container>,
    poly_size: PolynomialSize,
}

tensor_traits!(GlweSecretKey);

impl GlweSecretKey<Vec<bool>> {
    /// Allocates a container for a new key, and fill it with random values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::PolynomialSize;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(10),
    /// );
    /// assert_eq!(secret_key.key_size(), GlweDimension(256));
    /// assert_eq!(secret_key.polynomial_size(), PolynomialSize(10));
    /// ```
    pub fn generate(dimension: GlweDimension, poly_size: PolynomialSize) -> Self {
        GlweSecretKey {
            tensor: random::random_uniform_boolean_tensor(poly_size.0 * dimension.0),
            poly_size,
        }
    }

    /// Consumes the current GLWE secret key and turns it into an LWE secret key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use concrete_core::crypto::secret::GlweSecretKey;
    /// use concrete_core::crypto::{GlweDimension, LweDimension};
    /// use concrete_core::math::polynomial::PolynomialSize;
    /// let glwe_secret_key = GlweSecretKey::generate(
    ///     GlweDimension(2),
    ///     PolynomialSize(10),
    /// );
    /// let lwe_secret_key = glwe_secret_key.into_lwe_secret_key();
    /// assert_eq!(lwe_secret_key.key_size(), LweDimension(20))
    /// ```
    pub fn into_lwe_secret_key(self) -> LweSecretKey<Vec<bool>> {
        LweSecretKey::from_container(self.tensor.into_container())
    }
}

impl<Cont> GlweSecretKey<Cont> {
    /// Creates a key from a container.
    ///
    /// # Notes
    ///
    /// This method does not fill the container with random data. It merely wraps the container in
    /// the appropriate type. For a method that generate a new random key see
    /// [`GlweSecretKey::generate`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::PolynomialSize;
    /// let secret_key = GlweSecretKey::from_container(
    ///     vec![0 as u8; 11 * 256],
    ///     PolynomialSize(11),
    /// );
    /// assert_eq!(secret_key.key_size(), GlweDimension(256));
    /// assert_eq!(secret_key.polynomial_size(), PolynomialSize(11));
    /// ```
    pub fn from_container(cont: Cont, poly_size: PolynomialSize) -> Self
    where
        Cont: AsRefSlice,
    {
        ck_dim_div!(cont.as_slice().len() => poly_size.0);
        GlweSecretKey {
            tensor: Tensor::from_container(cont),
            poly_size,
        }
    }

    /// Returns the size of the secret key.
    ///
    /// This is equivalent to the number of masks in the [`GlweCiphertext`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::PolynomialSize;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(10),
    /// );
    /// assert_eq!(secret_key.key_size(), GlweDimension(256));
    /// ```
    pub fn key_size(&self) -> GlweDimension
    where
        Self: AsRefTensor,
    {
        GlweDimension(self.as_tensor().len() / self.poly_size.0)
    }

    /// Returns the size of the secret key polynomials.
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::PolynomialSize;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(10),
    /// );
    /// assert_eq!(secret_key.polynomial_size(), PolynomialSize(10));
    /// ```
    pub fn polynomial_size(&self) -> PolynomialSize {
        self.poly_size
    }

    /// Returns a borrowed polynomial list from the current key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::{PolynomialCount, PolynomialSize};
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(10),
    /// );
    /// let poly = secret_key.as_polynomial_list();
    /// assert_eq!(poly.polynomial_count(), PolynomialCount(256));
    /// assert_eq!(poly.polynomial_size(), PolynomialSize(10));
    /// ```
    pub fn as_polynomial_list(&self) -> PolynomialList<&[<Self as AsRefTensor>::Element]>
    where
        Self: AsRefTensor,
    {
        PolynomialList::from_container(self.as_tensor().as_slice(), self.poly_size)
    }

    /// Returns a mutably borrowed polynomial list from the current key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::{PolynomialCount, PolynomialSize};
    /// use concrete_core::math::tensor::{AsMutTensor, AsRefTensor};
    /// let mut secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(10),
    /// );
    /// let mut poly = secret_key.as_mut_polynomial_list();
    /// poly.as_mut_tensor().fill_with_element(true);
    /// assert!(secret_key.as_tensor().iter().all(|a| *a));
    /// ```
    pub fn as_mut_polynomial_list(
        &mut self,
    ) -> PolynomialList<&mut [<Self as AsRefTensor>::Element]>
    where
        Self: AsMutTensor,
    {
        let poly_size = self.poly_size;
        PolynomialList::from_container(self.as_mut_tensor().as_mut_slice(), poly_size)
    }

    /// Encrypts a single GLWE ciphertext.
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::{PolynomialSize, PolynomialCount};
    /// use concrete_core::math::tensor::{AsMutTensor, AsRefTensor};
    /// use concrete_core::crypto::encoding::PlaintextList;
    /// use concrete_core::crypto::glwe::GlweCiphertext;
    /// use concrete_core::math::dispersion::LogStandardDev;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(5),
    /// );
    /// let noise = LogStandardDev::from_log_standard_dev(-25.);
    /// let plaintexts = PlaintextList::from_container(
    ///     vec![100000 as u32,200000,300000,400000, 500000]
    /// );
    /// let mut  ciphertext = GlweCiphertext::allocate(0 as u32, PolynomialSize(5), GlweSize(257));
    /// secret_key.encrypt_glwe(&mut ciphertext, &plaintexts, noise);
    /// let mut decrypted = PlaintextList::from_container(vec![0 as u32,0,0,0,0]);
    /// secret_key.decrypt_glwe(&mut decrypted, &ciphertext);
    /// for (dec, plain) in decrypted.plaintext_iter().zip(plaintexts.plaintext_iter()){
    ///     let d0 = dec.0.wrapping_sub(plain.0);
    ///     let d1 = plain.0.wrapping_sub(dec.0);
    ///     let dist = std::cmp::min(d0, d1);
    ///     assert!(dist < 400, "dist: {:?}", dist);
    /// }
    /// ```
    pub fn encrypt_glwe<OutputCont, EncCont, Scalar>(
        &self,
        encrypted: &mut GlweCiphertext<OutputCont>,
        encoded: &PlaintextList<EncCont>,
        noise_parameter: impl DispersionParameter,
    ) where
        Self: AsRefTensor<Element = bool>,
        GlweCiphertext<OutputCont>: AsMutTensor<Element = Scalar>,
        PlaintextList<EncCont>: AsRefTensor<Element = Scalar>,
        Scalar: UnsignedTorus,
    {
        let (mut body, mut masks) = encrypted.get_mut_body_and_mask();
        random::fill_with_random_gaussian(&mut body, 0., noise_parameter.get_standard_dev());
        random::fill_with_random_uniform(&mut masks);
        body.as_mut_polynomial()
            .update_with_wrapping_add_binary_multisum(
                &masks.as_mut_polynomial_list(),
                &self.as_polynomial_list(),
            );
        body.as_mut_polynomial()
            .update_with_wrapping_add(&encoded.as_polynomial());
    }

    /// Encrypts a zero plaintext into a GLWE ciphertext.
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::{PolynomialSize, PolynomialCount};
    /// use concrete_core::math::tensor::{AsMutTensor, AsRefTensor};
    /// use concrete_core::crypto::encoding::PlaintextList;
    /// use concrete_core::crypto::glwe::GlweCiphertext;
    /// use concrete_core::math::dispersion::LogStandardDev;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(5),
    /// );
    /// let noise = LogStandardDev::from_log_standard_dev(-25.);
    /// let mut  ciphertext = GlweCiphertext::allocate(0 as u32, PolynomialSize(5), GlweSize(257));
    /// secret_key.encrypt_zero_glwe(&mut ciphertext, noise);
    /// let mut decrypted = PlaintextList::from_container(vec![0 as u32,0,0,0,0]);
    /// secret_key.decrypt_glwe(&mut decrypted, &ciphertext);
    /// for dec in decrypted.plaintext_iter(){
    ///     let d0 = dec.0.wrapping_sub(0u32);
    ///     let d1 = 0u32.wrapping_sub(dec.0);
    ///     let dist = std::cmp::min(d0, d1);
    ///     assert!(dist < 500, "dist: {:?}", dist);
    /// }
    /// ```
    pub fn encrypt_zero_glwe<Scalar, OutputCont>(
        &self,
        encrypted: &mut GlweCiphertext<OutputCont>,
        noise_parameters: impl DispersionParameter,
    ) where
        Self: AsRefTensor<Element = bool>,
        GlweCiphertext<OutputCont>: AsMutTensor<Element = Scalar>,
        Scalar: UnsignedTorus,
    {
        let (mut body, mut masks) = encrypted.get_mut_body_and_mask();
        random::fill_with_random_gaussian(&mut body, 0., noise_parameters.get_standard_dev());
        random::fill_with_random_uniform(&mut masks);
        body.as_mut_polynomial()
            .update_with_wrapping_add_binary_multisum(
                &masks.as_mut_polynomial_list(),
                &self.as_polynomial_list(),
            );
    }

    /// Encrypts a list of GLWE ciphertexts.
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::{PolynomialSize, PolynomialCount};
    /// use concrete_core::math::tensor::{AsMutTensor, AsRefTensor};
    /// use concrete_core::crypto::encoding::PlaintextList;
    /// use concrete_core::crypto::glwe::{GlweCiphertext, GlweList};
    /// use concrete_core::math::dispersion::LogStandardDev;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(2),
    /// );
    /// let noise = LogStandardDev::from_log_standard_dev(-25.);
    /// let plaintexts = PlaintextList::from_container(vec![1000 as u32,2000,3000,4000]);
    /// let mut  ciphertexts = GlweList::allocate(
    ///     0 as u32,
    ///     PolynomialSize(2),
    ///     GlweDimension(256),
    ///     CiphertextCount(2)
    /// );
    /// secret_key.encrypt_glwe_list(&mut ciphertexts, &plaintexts, noise);
    /// let mut decrypted = PlaintextList::from_container(vec![0 as u32,0,0,0]);
    /// secret_key.decrypt_glwe_list(&mut decrypted, &ciphertexts);
    /// for (dec, plain) in decrypted.plaintext_iter().zip(plaintexts.plaintext_iter()){
    ///     let d0 = dec.0.wrapping_sub(plain.0);
    ///     let d1 = plain.0.wrapping_sub(dec.0);
    ///     let dist = std::cmp::min(d0, d1);
    ///     assert!(dist < 400, "dist: {:?}", dist);
    /// }
    /// ```
    pub fn encrypt_glwe_list<CiphCont, EncCont, Scalar>(
        &self,
        encrypt: &mut GlweList<CiphCont>,
        encoded: &PlaintextList<EncCont>,
        noise_parameters: impl DispersionParameter,
    ) where
        Self: AsRefTensor<Element = bool>,
        GlweList<CiphCont>: AsMutTensor<Element = Scalar>,
        PlaintextList<EncCont>: AsRefTensor<Element = Scalar>,
        Scalar: UnsignedTorus,
        for<'a> PlaintextList<&'a [Scalar]>: AsRefTensor<Element = Scalar>,
    {
        ck_dim_eq!(encrypt.ciphertext_count().0 * encrypt.polynomial_size().0 => encoded.count().0);
        ck_dim_eq!(encrypt.glwe_dimension().0 => self.key_size().0);

        let count = PlaintextCount(encrypt.polynomial_size().0);
        for (mut ciphertext, encoded) in encrypt
            .ciphertext_iter_mut()
            .zip(encoded.sublist_iter(count))
        {
            self.encrypt_glwe(&mut ciphertext, &encoded, noise_parameters.clone());
        }
    }

    /// Encrypts a list of GLWE ciphertexts, with a zero plaintext.
    ///
    /// # Example
    ///
    /// ```rust
    /// use concrete_core::crypto::{*, secret::*};
    /// use concrete_core::math::polynomial::{PolynomialSize, PolynomialCount};
    /// use concrete_core::math::tensor::{AsMutTensor, AsRefTensor};
    /// use concrete_core::crypto::encoding::PlaintextList;
    /// use concrete_core::crypto::glwe::{GlweCiphertext, GlweList};
    /// use concrete_core::math::dispersion::LogStandardDev;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(256),
    ///     PolynomialSize(2),
    /// );
    /// let noise = LogStandardDev::from_log_standard_dev(-25.);
    /// let mut ciphertexts = GlweList::allocate(
    ///     0 as u32,
    ///     PolynomialSize(2),
    ///     GlweDimension(256),
    ///     CiphertextCount(2)
    /// );
    /// secret_key.encrypt_zero_glwe_list(&mut ciphertexts, noise);
    /// let mut decrypted = PlaintextList::from_container(vec![0 as u32,0,0,0]);
    /// secret_key.decrypt_glwe_list(&mut decrypted, &ciphertexts);
    /// for dec in decrypted.plaintext_iter(){
    ///     let d0 = dec.0.wrapping_sub(0u32);
    ///     let d1 = 0u32.wrapping_sub(dec.0);
    ///     let dist = std::cmp::min(d0, d1);
    ///     assert!(dist < 400, "dist: {:?}", dist);
    /// }
    /// ```
    pub fn encrypt_zero_glwe_list<Scalar, OutputCont>(
        &self,
        encrypted: &mut GlweList<OutputCont>,
        noise_parameters: impl DispersionParameter,
    ) where
        Self: AsRefTensor<Element = bool>,
        GlweList<OutputCont>: AsMutTensor<Element = Scalar>,
        Scalar: UnsignedTorus + Add,
    {
        for mut ciphertext in encrypted.ciphertext_iter_mut() {
            self.encrypt_zero_glwe(&mut ciphertext, noise_parameters.clone());
        }
    }

    /// Decrypts a single GLWE ciphertext.
    ///
    /// See ['GlweSecretKey::encrypt_glwe`] for an example.
    pub fn decrypt_glwe<CiphCont, EncCont, Scalar>(
        &self,
        encoded: &mut PlaintextList<EncCont>,
        encrypted: &GlweCiphertext<CiphCont>,
    ) where
        Self: AsRefTensor<Element = bool>,
        PlaintextList<EncCont>: AsMutTensor<Element = Scalar>,
        GlweCiphertext<CiphCont>: AsRefTensor<Element = Scalar>,
        Scalar: UnsignedTorus + Add,
    {
        ck_dim_eq!(encoded.count().0 => encrypted.polynomial_size().0);
        let (body, masks) = encrypted.get_body_and_mask();
        encoded
            .as_mut_tensor()
            .fill_with_one(body.as_tensor(), |a| *a);
        encoded
            .as_mut_polynomial()
            .update_with_wrapping_sub_binary_multisum(
                &masks.as_polynomial_list(),
                &self.as_polynomial_list(),
            );
    }

    /// Decrypts a list of GLWE ciphertexts.
    ///
    /// See ['GlweSecretKey::encrypt_glwe_list`] for an example.
    pub fn decrypt_glwe_list<CiphCont, EncCont, Scalar>(
        &self,
        encoded: &mut PlaintextList<EncCont>,
        encrypted: &GlweList<CiphCont>,
    ) where
        Self: AsRefTensor<Element = bool>,
        PlaintextList<EncCont>: AsMutTensor<Element = Scalar>,
        GlweList<CiphCont>: AsRefTensor<Element = Scalar>,
        Scalar: UnsignedTorus + Add,
        for<'a> PlaintextList<&'a mut [Scalar]>: AsMutTensor<Element = Scalar>,
    {
        ck_dim_eq!(encrypted.ciphertext_count().0 * encrypted.polynomial_size().0 => encoded.count().0);
        ck_dim_eq!(encrypted.glwe_dimension().0 => self.key_size().0);
        for (ciphertext, mut encoded) in encrypted
            .ciphertext_iter()
            .zip(encoded.sublist_iter_mut(PlaintextCount(encrypted.polynomial_size().0)))
        {
            self.decrypt_glwe(&mut encoded, &ciphertext);
        }
    }

    /// This function encrypts a message as a GGSW ciphertext.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use concrete_core::crypto::secret::GlweSecretKey;
    /// use concrete_core::math::polynomial::PolynomialSize;
    /// use concrete_core::crypto::{GlweSize, GlweDimension};
    /// use concrete_core::math::decomposition::{DecompositionLevelCount, DecompositionBaseLog};
    /// use concrete_core::math::dispersion::LogStandardDev;
    /// use concrete_core::crypto::encoding::Plaintext;
    /// use concrete_core::crypto::ggsw::GgswCiphertext;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(2),
    ///     PolynomialSize(10),
    /// );
    /// let mut ciphertext = GgswCiphertext::allocate(
    ///     0 as u32,
    ///     PolynomialSize(10),
    ///     GlweSize(3),
    ///     DecompositionLevelCount(3),
    ///     DecompositionBaseLog(7)
    /// );
    /// let noise = LogStandardDev::from_log_standard_dev(-15.);
    /// secret_key.encrypt_constant_ggsw(&mut ciphertext, &Plaintext(10), noise);
    /// ```
    pub fn encrypt_constant_ggsw<OutputCont, Scalar>(
        &self,
        encrypted: &mut GgswCiphertext<OutputCont>,
        encoded: &Plaintext<Scalar>,
        noise_parameters: impl DispersionParameter,
    ) where
        Self: AsRefTensor<Element = bool>,
        GgswCiphertext<OutputCont>: AsMutTensor<Element = Scalar>,
        OutputCont: AsMutSlice<Element = Scalar>,
        Scalar: UnsignedTorus,
    {
        ck_dim_eq!(self.polynomial_size() => encrypted.polynomial_size());
        ck_dim_eq!(self.key_size() => encrypted.glwe_size().to_glwe_dimension());
        self.encrypt_zero_glwe_list(&mut encrypted.as_mut_glwe_list(), noise_parameters);
        let base_log = encrypted.decomposition_base_log();
        for mut matrix in encrypted.level_matrix_iter_mut() {
            let decomposition = encoded.0
                * (Scalar::ONE
                    << (<Scalar as Numeric>::BITS
                        - (base_log.0 * (matrix.decomposition_level().0 + 1))));
            // We iterate over the rowe of the level matrix
            for (index, row) in matrix.row_iter_mut().enumerate() {
                let rlwe_ct = row.into_rlwe();
                // We retrieve the row as a polynomial list
                let mut polynomial_list = rlwe_ct.into_polynomial_list();
                // We retrieve the polynomial in the diagonal
                let mut level_polynomial = polynomial_list.get_mut_polynomial(index);
                // We get the first coefficient
                let first_coef = level_polynomial.as_mut_tensor().first_mut();
                // We update the first coefficient
                *first_coef = first_coef.wrapping_add(decomposition);
            }
        }
    }

    /// This function encrypts a message as a GGSW ciphertext whose rlwe masks are all zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use concrete_core::crypto::secret::GlweSecretKey;
    /// use concrete_core::math::polynomial::PolynomialSize;
    /// use concrete_core::crypto::{GlweSize, GlweDimension};
    /// use concrete_core::math::decomposition::{DecompositionLevelCount, DecompositionBaseLog};
    /// use concrete_core::math::dispersion::LogStandardDev;
    /// use concrete_core::crypto::encoding::Plaintext;
    /// use concrete_core::crypto::ggsw::GgswCiphertext;
    /// let secret_key = GlweSecretKey::generate(
    ///     GlweDimension(2),
    ///     PolynomialSize(10),
    /// );
    /// let mut ciphertext = GgswCiphertext::allocate(
    ///     0 as u32,
    ///     PolynomialSize(10),
    ///     GlweSize(3),
    ///     DecompositionLevelCount(3),
    ///     DecompositionBaseLog(7)
    /// );
    /// let noise = LogStandardDev::from_log_standard_dev(-15.);
    /// secret_key.trivial_encrypt_constant_ggsw(&mut ciphertext, &Plaintext(10), noise);
    /// ```
    pub fn trivial_encrypt_constant_ggsw<OutputCont, Scalar>(
        &self,
        encrypted: &mut GgswCiphertext<OutputCont>,
        encoded: &Plaintext<Scalar>,
        noise_parameters: impl DispersionParameter,
    ) where
        Self: AsRefTensor<Element = bool>,
        GgswCiphertext<OutputCont>: AsMutTensor<Element = Scalar>,
        OutputCont: AsMutSlice<Element = Scalar>,
        Scalar: UnsignedTorus,
    {
        ck_dim_eq!(self.polynomial_size() => encrypted.polynomial_size());
        ck_dim_eq!(self.key_size() => encrypted.glwe_size().to_glwe_dimension());
        // We fill the ggsw with trivial glwe encryptions of zero:
        for mut glwe in encrypted.as_mut_glwe_list().ciphertext_iter_mut() {
            let (mut body, mut mask) = glwe.get_mut_body_and_mask();
            mask.as_mut_tensor().fill_with_element(Scalar::ZERO);
            random::fill_with_random_gaussian(&mut body, 0., noise_parameters.get_standard_dev());
        }
        let base_log = encrypted.decomposition_base_log();
        for mut matrix in encrypted.level_matrix_iter_mut() {
            let decomposition = encoded.0
                * (Scalar::ONE
                    << (<Scalar as Numeric>::BITS
                        - (base_log.0 * (matrix.decomposition_level().0 + 1))));
            // We iterate over the rowe of the level matrix
            for (index, row) in matrix.row_iter_mut().enumerate() {
                let rlwe_ct = row.into_rlwe();
                // We retrieve the row as a polynomial list
                let mut polynomial_list = rlwe_ct.into_polynomial_list();
                // We retrieve the polynomial in the diagonal
                let mut level_polynomial = polynomial_list.get_mut_polynomial(index);
                // We get the first coefficient
                let first_coef = level_polynomial.as_mut_tensor().first_mut();
                // We update the first coefficient
                *first_coef = first_coef.wrapping_add(decomposition);
            }
        }
    }
}
