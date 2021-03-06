
use std::rand;
use std::rand::Rng;

use transaction::{Transaction, TxIn};
use hash::Hashable;
use util;

/**/
fn match_input(in1: &TxIn, in2: &TxIn) -> bool
{
  /* We don't check scriptSig since that will be different
   * for different transactions. */
  in1.prev_hash == in2.prev_hash &&
  in1.prev_index == in2.prev_index &&
  in1.nSequence == in2.nSequence
}

/**
 * Merge unsigned transactions
 * This function takes a bunch of transactions and creates a new, big
 * transaction with all the inputs and outputs from the originals, but
 * no signatures. It also randomizes the ordering.
 */
pub fn merge_unsigned_transactions (txlist: &[Transaction]) -> Option<Transaction>
{
  if txlist.len() == 0 { return None; }

  /* The first transaction will be our ``master'' list of inputs and outputs.
   * Every other transaction needs to match this or else that's a failure.
   */
  let mut master = Transaction {
    nVersion: txlist[0].nVersion,
    nLockTime: txlist[0].nLockTime,
    input: ~[], output: ~[]
  };

  /* Loop through all transactions, merging onto master */
  for tx in txlist.iter() {
    /* Check that version and locktime match, because otherwise it's unclear
     * what to do. (I guess it doesn't matter, in principle some humans will
     * verify this before it gets signed..) */
    if tx.nVersion != master.nVersion {
      println (format! ("err: Tx {:s} did not match {:s} (version {:u} vs {:u})!",
        util::u8_to_hex_string (master.to_hash()),
        util::u8_to_hex_string (tx.to_hash()),
        master.nVersion, tx.nVersion));
      return None;
    }
    if tx.nLockTime != master.nLockTime {
      println (format! ("err: Tx {:s} did not match {:s} (locktime {:u} vs {:u})!",
        util::u8_to_hex_string (master.to_hash()),
        util::u8_to_hex_string (tx.to_hash()),
        master.nLockTime, tx.nLockTime));
      return None;
    }

    /* Pile all the outputs on -- check for duplicate outputs and sum them */
    for tx in tx.output.iter() {
      let mut already_present = false;
      for tx_dup in master.output.mut_iter() {
        if tx_dup.scriptPubKey == tx.scriptPubKey {
          tx_dup.nValue += tx.nValue;
          already_present = true;
        }
      }
      if !already_present {
        master.output.push (tx.clone());
      }
    }

    /* Check for duplicate inputs and bail otherwise. This is pretty-much
     * guaranteed to be a mistake. (Probably there are also duplicate outputs,
     * but those are legal, so I don't want to delete them.) POLS says we
     * crash. */
    for tx in tx.input.iter() {
      for tx_dup in master.input.iter() {
        if match_input (tx, tx_dup) {
          println (format! ("err: Duplicate input {:s}:{:u} detected. Cowardly refusing to merge.",
            util::u8_to_hex_string (tx.prev_hash), tx.prev_index));
          return None;
        }
      }
      let mut new_tx = tx.clone();
      /* Remove any existing signature, except in the case that the sighash type
       * is NONE|ANYONECANPAY, since this is the only signature type that will
       * remain valid after a merger. (Actually, this is not true -- CodeShark
       * has a multisigner which stores some sort of information in here, which
       * is destroyed when I clear it. So TODO support this somehow.) */
      if new_tx.nHashType != 0x82 {
        new_tx.scriptSig = ~[];
      }
      master.input.push (new_tx);
    }
  }

  /* Randomize the inputs and outputs */
  let mut rng = rand::task_rng();
  rng.shuffle_mut (master.input);
  rng.shuffle_mut (master.output);

  Some(master)
}



