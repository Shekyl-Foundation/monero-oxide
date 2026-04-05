#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use serde::Deserialize;
use serde_json::json;

use monero_oxide::{
  fcmp::bulletproofs::BatchVerifier,
  transaction::Transaction,
  block::Block,
};

use monero_rpc::{RpcError, Rpc};
use monero_simple_request_rpc::SimpleRequestRpc;

use tokio::task::JoinHandle;

async fn check_block(rpc: impl Rpc, block_i: usize) {
  let hash = loop {
    match rpc.get_block_hash(block_i).await {
      Ok(hash) => break hash,
      Err(RpcError::ConnectionError(e)) => {
        println!("get_block_hash ConnectionError: {e}");
        continue;
      }
      Err(e) => panic!("couldn't get block {block_i}'s hash: {e:?}"),
    }
  };

  #[derive(Deserialize, Debug)]
  struct BlockResponse {
    blob: String,
  }
  let res: BlockResponse = loop {
    match rpc.json_rpc_call("get_block", Some(json!({ "hash": hex::encode(hash) }))).await {
      Ok(res) => break res,
      Err(RpcError::ConnectionError(e)) => {
        println!("get_block ConnectionError: {e}");
        continue;
      }
      Err(e) => panic!("couldn't get block {block_i} via block.hash(): {e:?}"),
    }
  };

  let blob = hex::decode(res.blob).expect("node returned non-hex block");
  let block = Block::read(&mut blob.as_slice())
    .unwrap_or_else(|e| panic!("couldn't deserialize block {block_i}: {e}"));
  assert_eq!(block.hash(), hash, "hash differs");
  assert_eq!(block.serialize(), blob, "serialization differs");

  let txs_len = 1 + block.transactions.len();

  if !block.transactions.is_empty() {
    loop {
      match rpc.get_pruned_transactions(&block.transactions).await {
        Ok(_) => break,
        Err(RpcError::ConnectionError(e)) => {
          println!("get_pruned_transactions ConnectionError: {e}");
          continue;
        }
        Err(e) => panic!("couldn't call get_pruned_transactions: {e:?}"),
      }
    }

    let txs = loop {
      match rpc.get_transactions(&block.transactions).await {
        Ok(txs) => break txs,
        Err(RpcError::ConnectionError(e)) => {
          println!("get_transactions ConnectionError: {e}");
          continue;
        }
        Err(e) => panic!("couldn't call get_transactions: {e:?}"),
      }
    };

    let mut batch = BatchVerifier::new();
    for tx in txs {
      let Transaction::V2 { ref prefix, proofs: Some(ref proofs) } = tx else {
        panic!("non-v2 or proofless non-miner TX in block {block_i}");
      };
      let _sig_hash = tx.signature_hash().expect("no signature hash for TX with proofs");
      assert!(
        proofs.prunable.bulletproof.batch_verify(
          &mut rand_core::OsRng,
          &mut batch,
          &proofs.base.commitments,
        ),
        "BP+ verification failed for TX in block {block_i} ({} inputs, {} outputs)",
        prefix.inputs.len(),
        prefix.outputs.len(),
      );
    }
    assert!(batch.verify());
  }

  println!("Deserialized, hashed, and reserialized {block_i} with {txs_len} TXs");
}

#[tokio::main]
async fn main() {
  let args = std::env::args().collect::<Vec<String>>();

  let mut block_i =
    args.get(1).expect("no start block specified").parse::<usize>().expect("invalid start block");

  let async_parallelism: usize =
    args.get(2).unwrap_or(&"8".to_string()).parse::<usize>().expect("invalid parallelism argument");

  let default_nodes = vec![
    "http://shekyl:oxide@127.0.0.1:18081".to_string(),
  ];
  let mut specified_nodes = vec![];
  {
    let mut i = 0;
    loop {
      let Some(node) = args.get(3 + i) else { break };
      specified_nodes.push(node.clone());
      i += 1;
    }
  }
  let nodes = if specified_nodes.is_empty() { default_nodes } else { specified_nodes };

  let rpc = |url: String| async move {
    SimpleRequestRpc::new(url.clone())
      .await
      .unwrap_or_else(|_| panic!("couldn't create SimpleRequestRpc connected to {url}"))
  };
  let main_rpc = rpc(nodes[0].clone()).await;
  let mut rpcs = vec![];
  for i in 0 .. async_parallelism {
    rpcs.push(rpc(nodes[i % nodes.len()].clone()).await);
  }

  let mut rpc_i = 0;
  let mut handles: Vec<JoinHandle<()>> = vec![];
  let mut height = 0;
  loop {
    let new_height = main_rpc.get_height().await.expect("couldn't call get_height");
    if new_height == height {
      break;
    }
    height = new_height;

    while block_i < height {
      if handles.len() >= async_parallelism {
        handles.swap_remove(0).await.unwrap();

        let mut i = 0;
        while i < handles.len() {
          if handles[i].is_finished() {
            handles.swap_remove(i).await.unwrap();
            continue;
          }
          i += 1;
        }
      }

      handles.push(tokio::spawn(check_block(rpcs[rpc_i].clone(), block_i)));
      rpc_i = (rpc_i + 1) % rpcs.len();
      block_i += 1;
    }
  }
}
