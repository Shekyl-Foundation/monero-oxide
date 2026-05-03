use rand_core::OsRng;

use multiexp::BatchVerifier;
use ciphersuite::group::Group;
use dalek_ff_group::{Scalar, EdwardsPoint};

use shekyl_generators::T;

use crate::{Output, sal::*};

#[cfg(feature = "multisig")]
mod legacy_multisig;
#[cfg(feature = "multisig")]
mod multisig;

#[test]
fn test_sal() {
  let x = Scalar::random(&mut OsRng);
  let y = Scalar::random(&mut OsRng);

  let O = (EdwardsPoint::generator() * x) + (EdwardsPoint(*T) * y);
  let I = EdwardsPoint::random(&mut OsRng);
  let C = EdwardsPoint::random(&mut OsRng);

  let L = I * x;

  let rerandomized_output = RerandomizedOutput::new(&mut OsRng, Output::new(O, I, C).unwrap());
  let input = rerandomized_output.input();
  let opening = OpenedInputTuple::open(&rerandomized_output, &x, &y).unwrap();
  let (L_, proof) = SpendAuthAndLinkability::prove(&mut OsRng, [0; 32], &opening);
  assert_eq!(L_, L);
  let mut verifier = BatchVerifier::new(1);
  proof.verify(&mut OsRng, &mut verifier, [0; 32], &input, L);
  assert!(verifier.verify_vartime());
}

/// Verify that `with_commitment_blind` honors the caller-supplied `r_c` and
/// produces a valid SAL proof. The wallet-binding contract is:
/// `C_tilde = C + r_c * G`, so `c_blind() == -r_c`.
#[test]
fn test_sal_with_caller_supplied_commitment_blind() {
  let x = Scalar::random(&mut OsRng);
  let y = Scalar::random(&mut OsRng);

  let O = (EdwardsPoint::generator() * x) + (EdwardsPoint(*T) * y);
  let I = EdwardsPoint::random(&mut OsRng);
  let C = EdwardsPoint::random(&mut OsRng);

  let L = I * x;

  let r_c = Scalar::random(&mut OsRng);
  let rerandomized_output = RerandomizedOutput::with_commitment_blind(
    &mut OsRng,
    Output::new(O, I, C).unwrap(),
    r_c,
  );

  // Caller-binding contract: c_blind() returns the additive inverse of the
  // supplied r_c, and C_tilde lies on C + r_c * G.
  assert_eq!(rerandomized_output.c_blind(), -r_c);
  assert_eq!(rerandomized_output.input().C_tilde(), C + (EdwardsPoint::generator() * r_c));

  let input = rerandomized_output.input();
  let opening = OpenedInputTuple::open(&rerandomized_output, &x, &y).unwrap();
  let (L_, proof) = SpendAuthAndLinkability::prove(&mut OsRng, [0; 32], &opening);
  assert_eq!(L_, L);
  let mut verifier = BatchVerifier::new(1);
  proof.verify(&mut OsRng, &mut verifier, [0; 32], &input, L);
  assert!(verifier.verify_vartime());
}
