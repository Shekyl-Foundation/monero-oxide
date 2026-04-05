use shekyl_oxide::{
  io::CompressedPoint,
  fcmp::{
    EncryptedAmount, ProofType, ProofBase, PrunableProof, Proofs,
    bulletproofs::Bulletproof,
  },
  transaction::{Input, Output, Timelock, TransactionPrefix, Transaction, NotPruned},
};

fn dummy_compressed_point() -> CompressedPoint {
  CompressedPoint([
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0,
  ])
}

fn make_dummy_bp_plus() -> Bulletproof {
  let lr_len: usize = 6;
  let push_point = |bp: &mut Vec<u8>| {
    bp.push(1);
    bp.extend([0; 31]);
  };
  let push_scalar = |bp: &mut Vec<u8>| bp.extend([0; 32]);
  let mut bp = Vec::with_capacity(((6 + (2 * lr_len)) * 32) + 2);
  for _ in 0 .. 3 {
    push_point(&mut bp);
  }
  for _ in 0 .. 3 {
    push_scalar(&mut bp);
  }
  for _ in 0 .. 2 {
    shekyl_oxide::io::write_varint(&lr_len, &mut bp).unwrap();
    for _ in 0 .. lr_len {
      push_point(&mut bp);
    }
  }
  Bulletproof::read_plus(&mut bp.as_slice()).unwrap()
}

// -- ProofType tests --

#[test]
fn only_fcmp_pp_type_accepted() {
  assert_eq!(u8::from(ProofType::FcmpPlusPlusPqc), 7);
  assert_eq!(ProofType::try_from(7u8).unwrap(), ProofType::FcmpPlusPlusPqc);
}

#[test]
fn proof_type_rejects_legacy_wire_values() {
  for byte in [0u8, 1, 2, 3, 4, 5, 6, 8, 255] {
    assert!(ProofType::try_from(byte).is_err(), "wire value {byte} should be rejected");
  }
}

// -- PrunableProof round-trip --

#[test]
fn fcmp_pp_prunable_round_trip() {
  let bulletproof = make_dummy_bp_plus();
  let prunable = PrunableProof {
    pseudo_outs: vec![dummy_compressed_point(); 2],
    bulletproof,
    reference_block: 42_000,
    fcmp_proof: vec![0xAA; 256],
    pqc_auths: vec![vec![0xBB; 128], vec![0xCC; 128]],
  };

  let mut serialized = vec![];
  prunable.write(&mut serialized).unwrap();
  assert!(!serialized.is_empty());

  let deserialized = PrunableProof::read(2, &mut serialized.as_slice()).unwrap();
  assert_eq!(prunable, deserialized);
}

// -- Full Proofs round-trip --

#[test]
fn fcmp_pp_proofs_round_trip() {
  let bulletproof = make_dummy_bp_plus();
  let proofs = Proofs {
    base: ProofBase {
      fee: 1_000_000,
      encrypted_amounts: vec![EncryptedAmount { amount: [1; 8] }],
      commitments: vec![dummy_compressed_point()],
    },
    prunable: PrunableProof {
      pseudo_outs: vec![dummy_compressed_point()],
      bulletproof,
      reference_block: 100,
      fcmp_proof: vec![0xDE; 512],
      pqc_auths: vec![vec![0xAD; 3309]],
    },
  };

  assert_eq!(proofs.proof_type(), ProofType::FcmpPlusPlusPqc);
  let serialized = proofs.serialize();

  let deserialized = Proofs::read(1, 1, &mut serialized.as_slice()).unwrap().unwrap();
  assert_eq!(proofs, deserialized);
}

// -- Transaction V2 FCMP++ round-trip --

#[test]
fn fcmp_pp_transaction_round_trip() {
  let bulletproof = make_dummy_bp_plus();
  let tx = Transaction::V2 {
    prefix: TransactionPrefix {
      additional_timelock: Timelock::None,
      inputs: vec![Input::ToKey {
        amount: None,
        key_offsets: vec![],
        key_image: dummy_compressed_point(),
      }],
      outputs: vec![Output {
        amount: None,
        key: dummy_compressed_point(),
        view_tag: Some(0x42),
      }],
      extra: vec![],
    },
    proofs: Some(Proofs {
      base: ProofBase {
        fee: 500_000,
        encrypted_amounts: vec![EncryptedAmount { amount: [7; 8] }],
        commitments: vec![dummy_compressed_point()],
      },
      prunable: PrunableProof {
        pseudo_outs: vec![dummy_compressed_point()],
        bulletproof,
        reference_block: 99,
        fcmp_proof: vec![0xFF; 64],
        pqc_auths: vec![vec![0xEE; 64]],
      },
    }),
  };

  let serialized = tx.serialize();
  let deserialized = Transaction::read(&mut serialized.as_slice()).unwrap();
  assert_eq!(tx, deserialized);
  assert_eq!(tx.hash(), deserialized.hash());
}

// -- Coinbase transaction round-trip --

#[test]
fn coinbase_transaction_round_trip() {
  let tx = Transaction::V2 {
    prefix: TransactionPrefix {
      additional_timelock: Timelock::None,
      inputs: vec![Input::Gen(1000)],
      outputs: vec![Output {
        amount: Some(1_000_000_000),
        key: dummy_compressed_point(),
        view_tag: None,
      }],
      extra: vec![1, 2, 3, 4],
    },
    proofs: None,
  };

  let serialized = tx.serialize();
  let deserialized = Transaction::read(&mut serialized.as_slice()).unwrap();
  assert_eq!(tx, deserialized);
  assert!(deserialized.signature_hash().is_none());
}

// -- V1 transactions are rejected --

#[test]
fn v1_transaction_rejected() {
  let mut data: Vec<u8> = vec![];
  shekyl_oxide::io::write_varint(&1u64, &mut data).unwrap();
  shekyl_oxide::io::write_varint(&0u64, &mut data).unwrap(); // timelock
  shekyl_oxide::io::write_varint(&1u64, &mut data).unwrap(); // 1 input
  data.push(255); // Gen
  shekyl_oxide::io::write_varint(&0u64, &mut data).unwrap(); // height 0
  shekyl_oxide::io::write_varint(&1u64, &mut data).unwrap(); // 1 output
  shekyl_oxide::io::write_varint(&100u64, &mut data).unwrap(); // amount
  data.push(2); // output type
  data.extend([0u8; 32]); // key
  shekyl_oxide::io::write_varint(&0u64, &mut data).unwrap(); // extra len

  let result = Transaction::<NotPruned>::read(&mut data.as_slice());
  assert!(result.is_err(), "v1 transaction should be rejected");
}

// -- Edge cases --

#[test]
fn fcmp_pp_empty_proof_round_trip() {
  let bulletproof = make_dummy_bp_plus();
  let prunable = PrunableProof {
    pseudo_outs: vec![dummy_compressed_point()],
    bulletproof,
    reference_block: 0,
    fcmp_proof: vec![],
    pqc_auths: vec![vec![]],
  };
  let serialized = prunable.serialize();
  let deserialized = PrunableProof::read(1, &mut serialized.as_slice()).unwrap();
  assert_eq!(prunable, deserialized);
}

#[test]
fn fcmp_pp_pqc_auth_count_mismatch_rejected() {
  let bulletproof = make_dummy_bp_plus();
  let prunable = PrunableProof {
    pseudo_outs: vec![dummy_compressed_point(); 2],
    bulletproof,
    reference_block: 0,
    fcmp_proof: vec![],
    pqc_auths: vec![vec![0u8; 16]],
  };
  let serialized = prunable.serialize();
  let result = PrunableProof::read(2, &mut serialized.as_slice());
  assert!(result.is_err(), "mismatched pqc_auths count should be rejected");
}

#[test]
fn proof_base_type_zero_is_none() {
  let result = ProofBase::read(1, &mut [0u8].as_slice()).unwrap();
  assert!(result.is_none());
}

#[test]
fn proof_base_legacy_type_bytes_rejected() {
  for byte in [1u8, 2, 3, 4, 5, 6] {
    let result = ProofBase::read(1, &mut [byte].as_slice());
    assert!(result.is_err(), "legacy type byte {byte} should be rejected by ProofBase::read");
  }
}

#[test]
fn transaction_version_always_2() {
  let tx: Transaction<NotPruned> = Transaction::V2 {
    prefix: TransactionPrefix {
      additional_timelock: Timelock::None,
      inputs: vec![Input::Gen(0)],
      outputs: vec![],
      extra: vec![],
    },
    proofs: None,
  };
  assert_eq!(tx.version(), 2);
}
