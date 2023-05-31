// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub(crate) mod send_client;
pub(crate) mod verifying_client;

use super::Client;

use sn_dbc::{Dbc, PublicAddress, Token};
use sn_domain::wallet::{Error, LocalWallet, Result, SendWallet};
use sn_protocol::NetworkAddress;

use bls::SecretKey;
use merkletree::{
    merkle::{next_pow2, MerkleTree},
    proof::Proof,
    store::VecStore,
};
use std::iter::Iterator;

/// A wallet client can be used to send and
/// receive tokens to/from other wallets.
pub struct WalletClient<W: SendWallet> {
    client: Client,
    wallet: W,
}

impl<W: SendWallet> WalletClient<W> {
    /// Create a new wallet client.
    pub fn new(client: Client, wallet: W) -> Self {
        Self { client, wallet }
    }

    /// Send tokens to nodes closest to the data we want to make storage payment for.
    // TODO: provide fix against https://en.wikipedia.org/wiki/Preimage_attack
    pub async fn pay_for_storage(
        &mut self,
        content_addrs: impl Iterator<Item = &NetworkAddress>,
    ) -> Result<(Dbc, Vec<Proof<[u8; 32]>>)> {
        // FIXME: calculate the amount to pay to each node, perhaps just 1 nano to begin with.
        let amount = Token::from_nano(1);

        // Let's build the Merkle-tree from list of addresses to obtain the reason-hash
        let mut addrs: Vec<_> = content_addrs
            .map(|addr| {
                let mut arr = [0; 32];
                arr.copy_from_slice(&addr.as_bytes());
                arr
            })
            .collect();

        // Merkletree requires the number of leaves to be a power of 2, and at least 2 leaves.
        let num_of_leaves = usize::max(2, next_pow2(addrs.len()));
        println!(">> ADD LEAVES?? {} - {}", addrs.len(), num_of_leaves);
        for _ in addrs.len()..num_of_leaves {
            // fill it up with blank value leafs
            addrs.push([0; 32]);
        }

        let tree =
            MerkleTree::<[u8; 32], Sha256Hasher, VecStore<_>>::new(addrs.clone().into_iter())
                .map_err(|err| Error::StoragePaymentReason(err.to_string()))?;

        println!(">> TREE ({num_of_leaves} leaves): {:?}", tree);
        let mut proofs = vec![];
        for (index, orig) in addrs.into_iter().enumerate() {
            let leaf = tree.read_at(index).unwrap();
            println!(">> LEAF {index}: {leaf:?}");

            let proof = tree
                .gen_proof(index)
                .map_err(|err| Error::StoragePaymentReason(err.to_string()))?;
            println!(">> PROOF for {index}: {proof:?}");
            proofs.push(proof.clone());

            // <SECTION TO BE REMOVED>
            println!(">>=== ORIG {index}: {orig:?}");

            let mut hasher = Sha256Hasher::default();
            let leaf_to_validate = hasher.leaf(orig);
            println!(">>=== LEAF from Chunk {index}: {leaf_to_validate:?}");

            use typenum::{UInt, UTerm, B0, B1};
            let proof: Proof<[u8; 32], UInt<UInt<UTerm, B1>, B0>> =
                Proof::new::<UTerm, UTerm>(None, proof.lemma().to_vec(), proof.path().to_vec())
                    .unwrap();

            println!(">>=== PROOF RECEIVED for Chunk: {proof:?}");

            let validated = if leaf_to_validate == proof.item() {
                let proof_validated = proof.validate::<Sha256Hasher>().unwrap();
                println!(">> LEAF matched!. PROOF validated? {proof_validated}");

                let root = proof.root();
                let root_matched = root == tree.root();
                println!(">> ROOT matched {index} ?: {root:?} ==> {root_matched}");

                proof_validated && root_matched
            } else {
                println!(">> LEAF doesn't match");
                false
            };

            println!(">> VALIDATED WITH ORIG {index}: {validated}");

            println!();
            // </ SECTION TO BE REMOVED>
        }

        // The reason hash is set to be the root of the merkle-tree of chunks to pay for
        let reason_hash = tree.root().into();
        println!(">>>> Reason hash: {reason_hash:?}");

        // FIXME: calculate closest nodes to pay for storage
        let to = PublicAddress::new(SecretKey::random().public_key());

        let dbcs = self
            .wallet
            .send(vec![(amount, to)], &self.client, Some(reason_hash))
            .await?;

        match &dbcs[..] {
            [info, ..] => Ok((info.dbc.clone(), proofs)),
            [] => Err(Error::CouldNotSendTokens(
                "No DBCs were returned from the wallet.".into(),
            )),
        }
    }

    /// Send tokens to another wallet.
    pub async fn send(&mut self, amount: Token, to: PublicAddress) -> Result<Dbc> {
        let dbcs = self
            .wallet
            .send(vec![(amount, to)], &self.client, None)
            .await?;
        match &dbcs[..] {
            [info, ..] => Ok(info.dbc.clone()),
            [] => Err(Error::CouldNotSendTokens(
                "No DBCs were returned from the wallet.".into(),
            )),
        }
    }

    /// Return the wallet.
    pub fn into_wallet(self) -> W {
        self.wallet
    }
}

/// Use the client to send a DBC from a local wallet to an address.
pub async fn send(from: LocalWallet, amount: Token, to: PublicAddress, client: &Client) -> Dbc {
    if amount.as_nano() == 0 {
        panic!("Amount must be more than zero.");
    }

    let mut wallet_client = WalletClient::new(client.clone(), from);
    let new_dbc = wallet_client
        .send(amount, to)
        .await
        .expect("Tokens shall be successfully sent.");

    let mut wallet = wallet_client.into_wallet();
    wallet
        .store()
        .await
        .expect("Wallet shall be successfully stored.");
    wallet
        .store_created_dbc(new_dbc.clone())
        .await
        .expect("Created dbc shall be successfully stored.");

    new_dbc
}

use merkletree::hash::Algorithm;
use tiny_keccak::{Hasher, Sha3};

struct Sha256Hasher {
    engine: Sha3,
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self {
            engine: Sha3::v256(),
        }
    }
}

impl std::hash::Hasher for Sha256Hasher {
    fn finish(&self) -> u64 {
        // merkletree::Algorithm trait is not calling this as per its doc:
        // https://docs.rs/merkletree/latest/merkletree/hash/trait.Algorithm.html
        error!(
            "Hasher's contract (finish function is supposedly not used) is deliberately broken by design"
        );
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.engine.update(bytes)
    }
}

impl Algorithm<[u8; 32]> for Sha256Hasher {
    fn hash(&mut self) -> [u8; 32] {
        let sha3 = self.engine.clone();
        let mut hash = [0u8; 32];
        sha3.finalize(&mut hash);
        hash
    }
}
