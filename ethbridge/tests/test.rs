extern crate web3;

use eth_bridge::{EthBridge,types::H128,header::NodeWithMerkleProof};
use web3::futures::Future;
use web3::types::{H256, Block};
use rlp::{RlpStream};
use futures::future::{join_all};
use std::panic;
use ethereum_types;
use serde::{Deserialize,Deserializer};
use hex::{FromHex, ToHex};

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate hex_literal;

fn catch_unwind_silent<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F) -> std::thread::Result<R> {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(f);
    panic::set_hook(prev_hook);
    result
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DagMerkleRoot {
    #[serde(deserialize_with = "from_hex_list")]
    dag_merkle_roots: Vec<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BlockWithProofs {
    proof_length: u64,

    #[serde(deserialize_with = "from_hex")]
    header_rlp: Vec<u8>,
    #[serde(deserialize_with = "from_hex")]
    merkle_root: Vec<u8>,
    #[serde(deserialize_with = "from_hex_list")]
    elements: Vec<Vec<u8>>,
    #[serde(deserialize_with = "from_hex_list")]
    merkle_proofs: Vec<Vec<u8>>,   
}

pub fn from_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| Vec::from_hex(&string).map_err(|err| Error::custom(err.to_string())))
}

pub fn from_hex_list<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
    where D: Deserializer<'de>
{
    use serde::de::Error;
    Vec::deserialize(deserializer)
        .and_then(|vec| vec.map(|v| Vec::from_hex(&v).map_err(|err| Error::custom(err.to_string()))))
}

// Wish to avoid this code and use web3+rlp libraries directly
fn rlp_append<TX>(header: &Block<TX>, stream: &mut RlpStream) {
    stream.begin_list(15);
    stream.append(&header.parent_hash);
    stream.append(&header.uncles_hash);
    stream.append(&header.author);
    stream.append(&header.state_root);
    stream.append(&header.transactions_root);
    stream.append(&header.receipts_root);
    stream.append(&header.logs_bloom);
    stream.append(&header.difficulty);
    stream.append(&header.number.unwrap());
    stream.append(&header.gas_limit);
    stream.append(&header.gas_used);
    stream.append(&header.timestamp);
    stream.append(&header.extra_data.0);
    stream.append(&header.mix_hash.unwrap());
    stream.append(&header.nonce.unwrap());
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_bindgen::MockedBlockchain;
    use near_bindgen::{testing_env, VMContext};

    lazy_static! {
        static ref WEB3RS: web3::Web3<web3::transports::Http> = {
            let (eloop, transport) = web3::transports::Http::new("https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2").unwrap();
            eloop.into_remote();
            web3::Web3::new(transport)
        };
    }

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice.near".to_string(),
            signer_account_id: "bob.near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol.near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(9),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
        }
    }

    fn get_blocks(web3rust: &web3::Web3<web3::transports::Http>, start: usize, stop: usize)
        -> (Vec<H256>, Vec<Vec<u8>>)
    {

        let futures = (start..stop).map(
            |i| web3rust.eth().block((i as u64).into())
        ).collect::<Vec<_>>();

        let block_headers = join_all(futures).wait().unwrap();

        let mut hashes: Vec<H256> = vec![];
        let mut blocks: Vec<Vec<u8>> = vec![];
        for block_header in block_headers {
            let mut stream = RlpStream::new();
            rlp_append(&block_header.clone().unwrap(), &mut stream);
            hashes.push(block_header.clone().unwrap().hash.unwrap());
            blocks.push(stream.out());
        }

        (hashes, blocks)
    }

    #[test]
    fn add_dags_merkle_roots() {
        testing_env!(get_context(vec![], false));

        let dags_merkle_roots: DagMerkleRoot = serde_json::from_reader(
            std::fs::File::open(std::path::Path::new("./dags_merkle_roots.json"))?
        )?;

        let dags_merkle_roots2: Vec<H128> = vec![
            H128(ethereum_types::H128(hex!("55b891e842e58f58956a847cbbf67821"))),
            H128(ethereum_types::H128(hex!("fba03a3d1902b9256ebe9177d03242fe"))),
            H128(ethereum_types::H128(hex!("2b186dc65b93be71780e5194fd44fc70"))),
            H128(ethereum_types::H128(hex!("94c0532d49523cd9309057a847ef0dbd"))),
            H128(ethereum_types::H128(hex!("f61d6da773315bdd4c79418186ebaa4a"))),
            H128(ethereum_types::H128(hex!("28e89dd2e1e5e09ee3e4cf412af58a0e"))),
            H128(ethereum_types::H128(hex!("54a0171c74e7336634f5b6b61f2b302c"))),
            H128(ethereum_types::H128(hex!("3be685b693d9ddfc342406fcc8d98512"))),
            H128(ethereum_types::H128(hex!("1887acc39d0818a7c6d47e33904a150a"))),
            H128(ethereum_types::H128(hex!("e1434e68f6a9f30252e2f31be8db9658"))),
            H128(ethereum_types::H128(hex!("a5e981ffaa1f770de8a1d21550f49755"))),
            H128(ethereum_types::H128(hex!("f4a55238db60864330a300e1d05dba16"))),
            H128(ethereum_types::H128(hex!("f4b2032ab23f95f9c9516db6d43372ce"))),
            H128(ethereum_types::H128(hex!("5fa11b8f22bd56e5bbb4cb0f843b6730"))),
            H128(ethereum_types::H128(hex!("ad4e75d7abf04b5798d8d0c832bf6833"))),
            H128(ethereum_types::H128(hex!("7df3208dec48fb446e0f89da95843d8a"))),
            H128(ethereum_types::H128(hex!("250e4cae8e10486589190b68608af301"))),
            H128(ethereum_types::H128(hex!("a55b182e12b1433a4935514bb729d2b2"))),
            H128(ethereum_types::H128(hex!("99456d6b4f8886afbbafa6a758830a92"))),
            H128(ethereum_types::H128(hex!("cfd122fe8a0b3c8984e1a603e97bae53"))),
            H128(ethereum_types::H128(hex!("0d05ebdd6eae46efa4b0c7694e6db158"))),
            H128(ethereum_types::H128(hex!("7e59bb58278cbd8f9470fe8636c4edee"))),
            H128(ethereum_types::H128(hex!("c48e2800c2442220eb1d0a9d9d08b437"))),
            H128(ethereum_types::H128(hex!("185f8beff965e31b7859b9b63fc79f97"))),
            H128(ethereum_types::H128(hex!("6e6c22abdb238266d3fa0f2902f85d7c"))),
            H128(ethereum_types::H128(hex!("7345950e2b649e10596ae6be11782110"))),
            H128(ethereum_types::H128(hex!("0cc51bae63bfb29add017e4a0f89f97a"))),
            H128(ethereum_types::H128(hex!("0a5a13ee1aea57228395fc64b8a1852e"))),
            H128(ethereum_types::H128(hex!("ecb847d99f761b457747886f4e0c81d7"))),
            H128(ethereum_types::H128(hex!("9eaf4241ffab9b2d693b96420dbd0356"))),
            H128(ethereum_types::H128(hex!("93f46416f3ef2d5ea57fe1a25c89cfea"))),
            H128(ethereum_types::H128(hex!("ec1ba1810cafc7c0fe76e7bf50809bb2"))),
            H128(ethereum_types::H128(hex!("5ce691721774a58d63e53da2b80f0dbc"))),
            H128(ethereum_types::H128(hex!("f570455f0bfca4359608d92ba076c0cc"))),
            H128(ethereum_types::H128(hex!("1cdc79438ea2129bc739fc9497f53c14"))),
            H128(ethereum_types::H128(hex!("52bfc78f0fc5839e04f1c729c73a1469"))),
            H128(ethereum_types::H128(hex!("d711787384841b856ff7f4d53e5e42df"))),
            H128(ethereum_types::H128(hex!("63dd408ecfdd6e71d45cddfd45aff23b"))),
            H128(ethereum_types::H128(hex!("b0b09781e2c5249c9c248e0062a681ea"))),
            H128(ethereum_types::H128(hex!("0d9d5d09f198c9637b510bbac6f33f34"))),
            H128(ethereum_types::H128(hex!("b572f9b06f63d012d848174bd1191588"))),
            H128(ethereum_types::H128(hex!("d7ab790f4a80e62b38d3a8ae4d170832"))),
            H128(ethereum_types::H128(hex!("9184028922c8de7accdd9d72106aed6b"))),
            H128(ethereum_types::H128(hex!("9d52e83fb1ccb288a8bbd7094ea25221"))),
            H128(ethereum_types::H128(hex!("cb56adf452205662e1f83e51c0c496b5"))),
            H128(ethereum_types::H128(hex!("761eb4593abc7603cf0b5ea95d3661bd"))),
            H128(ethereum_types::H128(hex!("35ca47a1892c4524442a83fdc5231d3d"))),
            H128(ethereum_types::H128(hex!("289f4c7339489b0d07c8716fbf169c74"))),
            H128(ethereum_types::H128(hex!("75ec671be4712c1ce838fff26ef1122d"))),
            H128(ethereum_types::H128(hex!("ab650e5529ec2ce4147efe135a061eb1"))),
            H128(ethereum_types::H128(hex!("e0e637747620e8c1c0ef440b99eb9ce7"))),
            H128(ethereum_types::H128(hex!("94c0e63214f027f2ddd3ea463e44beb8"))),
            H128(ethereum_types::H128(hex!("8548626524a60410aee37ee400d237fc"))),
            H128(ethereum_types::H128(hex!("d80eb32a857a1f84b23801f6e4242459"))),
            H128(ethereum_types::H128(hex!("4853cb0907651c681f1dfbab0646a828"))),
            H128(ethereum_types::H128(hex!("ecd1edccd4844736d8a8e01d4ab21e59"))),
            H128(ethereum_types::H128(hex!("fb58a3ad252f9d576dcd1cfb23d32b89"))),
            H128(ethereum_types::H128(hex!("583b5070f416adbbf796976b2ca27066"))),
            H128(ethereum_types::H128(hex!("259d6fdcd7c3e46dd1a57ae64abda536"))),
            H128(ethereum_types::H128(hex!("d0c6caf2ce368aa85881e8c3bca18192"))),
            H128(ethereum_types::H128(hex!("7d54a3c9d517fba4ffb88cace0276c43"))),
            H128(ethereum_types::H128(hex!("630201121608bdec230db5d012bacfb4"))),
            H128(ethereum_types::H128(hex!("0da36e18ac524cab0cbd44ed0e70bf0e"))),
            H128(ethereum_types::H128(hex!("864cf4a44dfa1f5419a85613e03340b3"))),
            H128(ethereum_types::H128(hex!("d0369950eb82302e887caaca083d31b7"))),
            H128(ethereum_types::H128(hex!("2993e04f04c9b8476e92871886d88d7a"))),
            H128(ethereum_types::H128(hex!("dd49abb10a5bfaff4503b3a31874ac65"))),
            H128(ethereum_types::H128(hex!("96f5bb80bb703cd6b940b0fab926195a"))),
            H128(ethereum_types::H128(hex!("10e2c9baae90477c9be2f10365c29130"))),
            H128(ethereum_types::H128(hex!("696469c514035c0cdf657865a76c8b05"))),
            H128(ethereum_types::H128(hex!("e988c9b6348ae392d81e9d224c608247"))),
            H128(ethereum_types::H128(hex!("81a816b9971534a48e6ec21994b78c81"))),
            H128(ethereum_types::H128(hex!("5498cb9019ba94f896e2c04140cd036a"))),
            H128(ethereum_types::H128(hex!("17fa73eaa092e4bce97e3ba4b770a0b8"))),
            H128(ethereum_types::H128(hex!("e8c7b08816fc5215dfbe44cd46b47dec"))),
            H128(ethereum_types::H128(hex!("c30789092db881251b0c5f7373e0c6f0"))),
            H128(ethereum_types::H128(hex!("f397a1ac039c5e8bc374d1fd03568042"))),
            H128(ethereum_types::H128(hex!("33ec1f25215eae69085a3fbf7a6b27fa"))),
            H128(ethereum_types::H128(hex!("f6fdd17ce7427518d0631e269924f45b"))),
            H128(ethereum_types::H128(hex!("036c902bf005559ba3082e5f2201e614"))),
            H128(ethereum_types::H128(hex!("1fc45e655afc624fb90a7e0795b20b86"))),
            H128(ethereum_types::H128(hex!("bc94ffd5e4f606a12f0c0425d7bf1013"))),
            H128(ethereum_types::H128(hex!("21abfc7ec366c0b93e047d0d9d9df4bf"))),
            H128(ethereum_types::H128(hex!("b8a9f1c0b2d0601e00bb6fa35f3970e2"))),
            H128(ethereum_types::H128(hex!("d67fcb43ff2287a0cf8cf1f0a78ebc85"))),
            H128(ethereum_types::H128(hex!("ade2d8bdd4c48bd437b41d2a36424ef1"))),
            H128(ethereum_types::H128(hex!("d5550bdc493b35a3480c7a5f5d93e939"))),
            H128(ethereum_types::H128(hex!("b069c39e1059a068f9aa767b5a2c39d1"))),
            H128(ethereum_types::H128(hex!("e151a181c34b360acc4ae8f41f0eb923"))),
            H128(ethereum_types::H128(hex!("fa407454a0690b03f714c08ec72b3247"))),
            H128(ethereum_types::H128(hex!("10ffffcebaf525fbadcbe4aa46104680"))),
            H128(ethereum_types::H128(hex!("25569aef3173e2e81bd94a5e7904fc1b"))),
            H128(ethereum_types::H128(hex!("28681502310381ebc0ae31947c3cb188"))),
            H128(ethereum_types::H128(hex!("5db958abc1654596872a50938a0c9b24"))),
            H128(ethereum_types::H128(hex!("7c744e082a52a74767b70a72ec4489a9"))),
            H128(ethereum_types::H128(hex!("5b18ccdaa7efd9b3aff6bad60d547c81"))),
            H128(ethereum_types::H128(hex!("86322eab36c65090a3b7fdb5d7bc091c"))),
            H128(ethereum_types::H128(hex!("8423baac6908031fd9d08157f686b2dc"))),
            H128(ethereum_types::H128(hex!("08a1ade53581b4c029e1c002e51ceaf3"))),
            H128(ethereum_types::H128(hex!("f1ed7d196dff54c3421321acf939e08e"))),
            H128(ethereum_types::H128(hex!("2752d9c907207388e62373ed510c4e88"))),
            H128(ethereum_types::H128(hex!("c3c06fa841383ac60ccb91e4e05580d5"))),
            H128(ethereum_types::H128(hex!("a4c95f5a9ed58116110e43e663425608"))),
            H128(ethereum_types::H128(hex!("2c5bd140dff9063bba7ec0a206a3a4a0"))),
            H128(ethereum_types::H128(hex!("a5848a52ea19a2e85afeb598ce50eb47"))),
            H128(ethereum_types::H128(hex!("ff6279dc1306e5169f95f0b060e34b39"))),
            H128(ethereum_types::H128(hex!("da33c34ef46e9dd360b8dbe6531901b4"))),
            H128(ethereum_types::H128(hex!("83b7e0dbe63ffc49ffc59bae4b7b683e"))),
            H128(ethereum_types::H128(hex!("5c051f94fa62a73c11cfee276461fdb0"))),
            H128(ethereum_types::H128(hex!("798e3ba76c500e8177f392003ed1872b"))),
            H128(ethereum_types::H128(hex!("583d7265ee7126131854bbcb0de1f310"))),
            H128(ethereum_types::H128(hex!("90e4980b35640a8b3bb682ef2606e476"))),
            H128(ethereum_types::H128(hex!("6d431024b5bffd1270c0d041a05b815f"))),
            H128(ethereum_types::H128(hex!("496322b442254a79d1dd0dfdd6f51def"))),
            H128(ethereum_types::H128(hex!("92182683f38300b23bc0412e4138ac05"))),
            H128(ethereum_types::H128(hex!("212df134572585d10dd251f536025085"))),
            H128(ethereum_types::H128(hex!("63e2dbdb3937238a5d08cdf2b578b4e1"))),
            H128(ethereum_types::H128(hex!("96b819206e1d15573307e27b6ad290db"))),
            H128(ethereum_types::H128(hex!("0c54a577923b77c5a4ee726412c43be2"))),
            H128(ethereum_types::H128(hex!("155b53faed668b73ad702c93296a3e01"))),
            H128(ethereum_types::H128(hex!("896d7317a2f611e7363d93db93bcb72a"))),
            H128(ethereum_types::H128(hex!("a39c09d3a4ba25f3ce6691b85b390f3d"))),
            H128(ethereum_types::H128(hex!("7148171957df73a82553216488e35859"))),
            H128(ethereum_types::H128(hex!("ca049d60e60b7b69047e42f0b436ff67"))),
            H128(ethereum_types::H128(hex!("6f402a4a8208e9e49d4bf06f6ce7e11e"))),
            H128(ethereum_types::H128(hex!("95773e0c271ded0e10d2b47221c91e0e"))),
            H128(ethereum_types::H128(hex!("80fd5388433e89d3e74da2637216e3d8"))),
            H128(ethereum_types::H128(hex!("e35fe60581edd06fe880059a63952380"))),
            H128(ethereum_types::H128(hex!("24a5b87aba928ac920362a8bb3a853c1"))),
            H128(ethereum_types::H128(hex!("5a82f1cd0c0c58f0fbebb02c062dd029"))),
            H128(ethereum_types::H128(hex!("d8a989f4d05f65c07cd4f78d4c83d6de"))),
            H128(ethereum_types::H128(hex!("7e100ed69fa83cb97318cf268e063802"))),
            H128(ethereum_types::H128(hex!("5f7d7cb3363d1c4b41736787c8fa3a36"))),
            H128(ethereum_types::H128(hex!("03292bdeef76208a33368b1dd89c5f4f"))),
            H128(ethereum_types::H128(hex!("6b619e4bfd91e47efc4c6a18d6d2ddd4"))),
            H128(ethereum_types::H128(hex!("49e98cfac5039df5711f7bc82ca704fc"))),
            H128(ethereum_types::H128(hex!("bd17f87c484f37449d0cb26bee85352d"))),
            H128(ethereum_types::H128(hex!("b29204f91eeec3a61cf80f78d341e981"))),
            H128(ethereum_types::H128(hex!("0e2806dac2236f555aa1b60d44e6bb94"))),
            H128(ethereum_types::H128(hex!("84762739d031e5c2809951560a9aeaa2"))),
            H128(ethereum_types::H128(hex!("df1404d9feadf66ce9b6106bd730323f"))),
            H128(ethereum_types::H128(hex!("bf36c772e3f353b177dd77ff0af7f658"))),
            H128(ethereum_types::H128(hex!("c01a75724444ea62092d205d4f1faff8"))),
            H128(ethereum_types::H128(hex!("0eb6c4edf01055c26f19606f80660a82"))),
            H128(ethereum_types::H128(hex!("c5475e77e5b769f6e97f0aee53bb2927"))),
            H128(ethereum_types::H128(hex!("3a2a5f7f0ca0c8270800aa61bf75a256"))),
            H128(ethereum_types::H128(hex!("e2fbc1e07d14ac6e3a96cc9055750013"))),
            H128(ethereum_types::H128(hex!("226e5bbb1137417f87d4d0a638739739"))),
            H128(ethereum_types::H128(hex!("745c89d0db4461d9cf03e483f9ed2d66"))),
            H128(ethereum_types::H128(hex!("70ab39feaf98c852e8fac994ca8cc297"))),
            H128(ethereum_types::H128(hex!("cd9d7ebd5e7484375ec35bda9ebfad9b"))),
            H128(ethereum_types::H128(hex!("080de890fd9263b983b58e52f6dee214"))),
            H128(ethereum_types::H128(hex!("f67c8e857d379a60f7bf47b13ec08dc8"))),
            H128(ethereum_types::H128(hex!("b0b8ce46fdfa7f8b0091182cd9e52c19"))),
            H128(ethereum_types::H128(hex!("3fe2d70b44670254ddeaed4e46ba2d6a"))),
            H128(ethereum_types::H128(hex!("1e0f257e0107db4a3be7208c3490f3e8"))),
            H128(ethereum_types::H128(hex!("d0eb4a9ff0dc08a9149b275e3a64e93d"))),
            H128(ethereum_types::H128(hex!("eeab095cfa3a4dc8de4daf9c3e5affbe"))),
            H128(ethereum_types::H128(hex!("bee906bac51d709fa6c8d852834506fb"))),
            H128(ethereum_types::H128(hex!("85cd74d6633623e3e09d3b2ea0e8eebd"))),
            H128(ethereum_types::H128(hex!("f296dfe85523c5ab10cda4edaa513a52"))),
            H128(ethereum_types::H128(hex!("7d8ced87ed7fd15b2e4bbc0264e76f99"))),
            H128(ethereum_types::H128(hex!("ae69988dd1df0ff853e6ee66a5fe3210"))),
            H128(ethereum_types::H128(hex!("4469c4d95255369c6461be2862b915b4"))),
            H128(ethereum_types::H128(hex!("5709b43c1560bff7d265cfd850627680"))),
            H128(ethereum_types::H128(hex!("deb4f8617f931348359a3811076a30eb"))),
            H128(ethereum_types::H128(hex!("f881b9bdedd6f655e33220d24e1cc2eb"))),
            H128(ethereum_types::H128(hex!("ad903ea64fc18d570cd9a50e86bf033c"))),
            H128(ethereum_types::H128(hex!("4b3ac2630be5f8aab921697d1d1404bd"))),
            H128(ethereum_types::H128(hex!("07d5dd8bb48e7a72880b329cff744c4a"))),
            H128(ethereum_types::H128(hex!("84567d5b5e74e94c2373574d42ade1be"))),
            H128(ethereum_types::H128(hex!("63cf6b1ebbb29334730d8b9321cd264d"))),
            H128(ethereum_types::H128(hex!("83094b1464a6bbf92363619af081e20e"))),
            H128(ethereum_types::H128(hex!("7a93ae31b228b723301bf96ab9b0a09f"))),
            H128(ethereum_types::H128(hex!("16873ac9aead7c99286cce23dd91b4ee"))),
            H128(ethereum_types::H128(hex!("bf293be8af1eb38d7080957c7e1f8aeb"))),
            H128(ethereum_types::H128(hex!("967668d49545810fcf18632a5a3431e9"))),
            H128(ethereum_types::H128(hex!("475d5bbd6272a2695f66d2056da42bd9"))),
            H128(ethereum_types::H128(hex!("afc7e6ef08b5b8dc7a2bb1027160cd9c"))),
            H128(ethereum_types::H128(hex!("aa694f10ce796540ed77418cd9b35c86"))),
            H128(ethereum_types::H128(hex!("8be1f7a470d0c1edbbec6728fb0ff366"))),
            H128(ethereum_types::H128(hex!("7444078510fe6d9b3cf94188059a1366"))),
            H128(ethereum_types::H128(hex!("3739215eb46221b4040eea02c7757573"))),
            H128(ethereum_types::H128(hex!("a71b11286fff39e65eb3c8b3ac9a7219"))),
            H128(ethereum_types::H128(hex!("4b48bc59af9ddec38279e60178263779"))),
            H128(ethereum_types::H128(hex!("6076a0b6743690958cf040bfaefac391"))),
            H128(ethereum_types::H128(hex!("bead81dbb9227ba51a02f827f8dee2c5"))),
            H128(ethereum_types::H128(hex!("89508f9f01576f81853e8b92ba917838"))),
            H128(ethereum_types::H128(hex!("d075a5b5dcf20971f2e70e816bbcbb7e"))),
            H128(ethereum_types::H128(hex!("009554c550589a814909c9805279c743"))),
            H128(ethereum_types::H128(hex!("b470cf622846d536ad7b288b9074d667"))),
            H128(ethereum_types::H128(hex!("b87704373978613853240a3ec9368e8b"))),
            H128(ethereum_types::H128(hex!("7127b8d0e757abd6830b787afd829201"))),
            H128(ethereum_types::H128(hex!("f0cab8ea67e0a38ad606ab83ba6bc67e"))),
            H128(ethereum_types::H128(hex!("a408633718e44f4817c329af0395aabb"))),
            H128(ethereum_types::H128(hex!("4607a3ecef00a24da74521f22a6f8bee"))),
            H128(ethereum_types::H128(hex!("917cb60d42ccc40442e48be457f51dea"))),
            H128(ethereum_types::H128(hex!("90222d408a76f7f55fbb18282bef90da"))),
            H128(ethereum_types::H128(hex!("481d56afbd0ba6978e0ab2ada7b3506c"))),
            H128(ethereum_types::H128(hex!("604d874175bd36f8a02ce56b31ca827c"))),
            H128(ethereum_types::H128(hex!("6dc7717dfba128a330ea277dca94141d"))),
            H128(ethereum_types::H128(hex!("86226285351eba0c6e818826b1c562fb"))),
            H128(ethereum_types::H128(hex!("ae7280a5b84931846adff138820f221c"))),
            H128(ethereum_types::H128(hex!("be628492637e26e6489375f3a2938180"))),
            H128(ethereum_types::H128(hex!("7559678bfebb6f78e5c8026b17eadca3"))),
            H128(ethereum_types::H128(hex!("f38e7a19c004dd22688cf0079680bb1c"))),
            H128(ethereum_types::H128(hex!("c3b0e6a2b106f925aa2f92aac6213f8c"))),
            H128(ethereum_types::H128(hex!("eec733087a807a87a0c346de11513e12"))),
            H128(ethereum_types::H128(hex!("4c6d1ee77b414dc3bc448ecc0769a376"))),
            H128(ethereum_types::H128(hex!("303db177352ecf1920f09ba9fc8c6514"))),
            H128(ethereum_types::H128(hex!("8e38c47ebaf4ce8dc05178f3c5a9e86b"))),
            H128(ethereum_types::H128(hex!("104570237e9cbf0f4836ec8c4ff42f65"))),
            H128(ethereum_types::H128(hex!("4776ebe704f27086bcb98059906e8e3a"))),
            H128(ethereum_types::H128(hex!("c5aa722b23a6deef1d15a95f32dc4797"))),
            H128(ethereum_types::H128(hex!("c6188b4ee8720e1efa99aebeb02c7a67"))),
            H128(ethereum_types::H128(hex!("32701ac4e10f922048e0a7368e1f0452"))),
            H128(ethereum_types::H128(hex!("e5988223410c1d4f4260994faaf952b3"))),
            H128(ethereum_types::H128(hex!("2a92d9428c88e74bf47e545ea2025857"))),
            H128(ethereum_types::H128(hex!("04ca250a42e1f227955846abb768a035"))),
            H128(ethereum_types::H128(hex!("05b4a77d503468b71c0e730753fc1a56"))),
            H128(ethereum_types::H128(hex!("d7caf66b03181401cda1369c123d19f6"))),
            H128(ethereum_types::H128(hex!("6d3e29cb829b58d3fe90129c20dc9abb"))),
            H128(ethereum_types::H128(hex!("41b4f0817f11f8016023d74dea3eec97"))),
            H128(ethereum_types::H128(hex!("aeaa60d08ac92150b54908f7f8a92857"))),
            H128(ethereum_types::H128(hex!("c9453b8e185fb93ea0e1282e8803eff0"))),
            H128(ethereum_types::H128(hex!("e87f027df74563c88e700dfe057432ee"))),
            H128(ethereum_types::H128(hex!("af377ff39afc683033823eeb3ed0f10b"))),
            H128(ethereum_types::H128(hex!("f56a0b076a6bfc3eea7b1804b946d947"))),
            H128(ethereum_types::H128(hex!("69ba2470b6623fa3b9d68124e329513e"))),
            H128(ethereum_types::H128(hex!("575aee5f222f5ae9cca0973be3ad572f"))),
            H128(ethereum_types::H128(hex!("da97a6cd52c728a6f3bca987ebfa8cad"))),
            H128(ethereum_types::H128(hex!("4b5536ec8aad2250a2e38f6bfcdf58f4"))),
            H128(ethereum_types::H128(hex!("8fd3b4c5ad2c5743a6aae9f8219a60c6"))),
            H128(ethereum_types::H128(hex!("145b1a9812d684da23e74fead96c8552"))),
            H128(ethereum_types::H128(hex!("7617defe6ad9c021bc9bd7c809675624"))),
            H128(ethereum_types::H128(hex!("d9a2e97eaf84cce6294581acce315ed7"))),
            H128(ethereum_types::H128(hex!("3199b22620f39d534cd96fa8a032998b"))),
            H128(ethereum_types::H128(hex!("b1ca9b7eb944ea1f16364a1222b9afcd"))),
            H128(ethereum_types::H128(hex!("ecd0e506f3792f650fe5a00694afc356"))),
            H128(ethereum_types::H128(hex!("3b96f1eb7ad3124a51372cbe56f5c5e4"))),
            H128(ethereum_types::H128(hex!("962a5ed01d20d1202172cae5c4b1c7ed"))),
            H128(ethereum_types::H128(hex!("b5e9dc0e5c554931dba835dc88102421"))),
            H128(ethereum_types::H128(hex!("4596b31e8bf6c1f24b122de58efc7e1b"))),
            H128(ethereum_types::H128(hex!("224536fd41573a41daf7e131be8bdb09"))),
            H128(ethereum_types::H128(hex!("ef9661b2ac61737aa4bbba6fcad9f860"))),
            H128(ethereum_types::H128(hex!("26c9661a65164390de94c2d38c1f568a"))),
            H128(ethereum_types::H128(hex!("cc0b4699871953942cea3d167e8c9956"))),
            H128(ethereum_types::H128(hex!("575617f32549dc68ceb014b2f69d3b80"))),
            H128(ethereum_types::H128(hex!("932544c41c0e2d7af28189e513fb7ec5"))),
            H128(ethereum_types::H128(hex!("4b8e46de3ce76638280b9a699dfdb620"))),
            H128(ethereum_types::H128(hex!("53406aff68e56538b48fb98364e1a5a5"))),
            H128(ethereum_types::H128(hex!("928ae8d7116355d36b946a8182fc9923"))),
            H128(ethereum_types::H128(hex!("e30282bce7cdf44def0f840b6321e335"))),
            H128(ethereum_types::H128(hex!("beed3d40f310c0c6d0e18443f3304a60"))),
            H128(ethereum_types::H128(hex!("e2725bfdbac45fa18dabf0eb892f03d9"))),
            H128(ethereum_types::H128(hex!("07b43c42513772bc09aac4e471d67b16"))),
            H128(ethereum_types::H128(hex!("8609ba6e215f939caae8770e47d25f8a"))),
            H128(ethereum_types::H128(hex!("4287aec47a1da79aa2351f31cbd4ed0c"))),
            H128(ethereum_types::H128(hex!("b033cc4424fc38cbf7992491211c84c5"))),
            H128(ethereum_types::H128(hex!("cce1d898301da9cddb02d7f36181f8c2"))),
            H128(ethereum_types::H128(hex!("79e12de9d9e677ac2322705cc8a922b1"))),
            H128(ethereum_types::H128(hex!("c448a85e856037d8e88f672979a551eb"))),
            H128(ethereum_types::H128(hex!("467403ae25f597deb3c1094a2d33d413"))),
            H128(ethereum_types::H128(hex!("d7e03948dfccb6abb773409bd4a3c930"))),
            H128(ethereum_types::H128(hex!("674a8c75924d08965e7039c2e41f7940"))),
            H128(ethereum_types::H128(hex!("9220bbcb1742381fd5936662dee7210f"))),
            H128(ethereum_types::H128(hex!("505e4a4e5a49243957ee68bcf2ddb9e4"))),
            H128(ethereum_types::H128(hex!("85952e0b3c1032f7cad908bbd3a2b8a3"))),
            H128(ethereum_types::H128(hex!("f6e25da02626214f2dca471706a057d0"))),
            H128(ethereum_types::H128(hex!("dc7efbb16d990fb6db9e68efbc7fe740"))),
            H128(ethereum_types::H128(hex!("a3231a207b1daf19693a1a5ad18c6ac4"))),
            H128(ethereum_types::H128(hex!("90c5a0bbbc65a3fe44f2be3f860c5f0e"))),
            H128(ethereum_types::H128(hex!("3d8f53b6024c3b33b9097cc678de9a28"))),
            H128(ethereum_types::H128(hex!("1ad8cb3b8d1d4e04bb25330acd10b3e7"))),
            H128(ethereum_types::H128(hex!("c4830b15a969f30d1592527eda63bf82"))),
            H128(ethereum_types::H128(hex!("9d51b6f0c5be845ef775b6b900f0c993"))),
            H128(ethereum_types::H128(hex!("abdb6ff729edfa1fdf81725236fe166c"))),
            H128(ethereum_types::H128(hex!("f92a2b3fb5ebe93ee6fdac51e55f58d0"))),
            H128(ethereum_types::H128(hex!("bad463d68b2067ee099b35bc976d4262"))),
            H128(ethereum_types::H128(hex!("8a326abf1bf139fd19a9931aad716e2b"))),
            H128(ethereum_types::H128(hex!("21a32ae99babd87319e21b115291fa93"))),
            H128(ethereum_types::H128(hex!("aed51baf66ff4910f3b84c6dddd277d0"))),
            H128(ethereum_types::H128(hex!("65c3bbb3015925ae57d939a67bb3e1a2"))),
            H128(ethereum_types::H128(hex!("97bc9538e14c7d221d3fba271fe1a9a3"))),
            H128(ethereum_types::H128(hex!("6394e2557149a2acf674610e834f02a7"))),
            H128(ethereum_types::H128(hex!("280dcfe6935188046eefb81a77e043db"))),
            H128(ethereum_types::H128(hex!("313d0d27a7b82f6e85b32037b3458025"))),
            H128(ethereum_types::H128(hex!("af7416b95834809dc8619c24d9f70575"))),
            H128(ethereum_types::H128(hex!("9e14b1882ac75f1b7ac8735e89bd1dcf"))),
            H128(ethereum_types::H128(hex!("f770f4047a86f36727fcde69c0cb8b68"))),
            H128(ethereum_types::H128(hex!("004610125634efd77979c429a95f16e9"))),
            H128(ethereum_types::H128(hex!("9fb78c563cc2617353fb943c5c6029d9"))),
            H128(ethereum_types::H128(hex!("addc6c96bafb15254e0e2c2a21f6eca0"))),
            H128(ethereum_types::H128(hex!("b2e1d71c4419cf35d2ccb202727e9006"))),
            H128(ethereum_types::H128(hex!("22c2cf6192e5f767d518ba32d2628f27"))),
            H128(ethereum_types::H128(hex!("d4a9a8dedeaa916c20451f72d868e54c"))),
            H128(ethereum_types::H128(hex!("e15c7e3a6935f188aab577be046518f8"))),
            H128(ethereum_types::H128(hex!("d00f06b2b19fb192d885586001624318"))),
            H128(ethereum_types::H128(hex!("3c1133d7e7085944fa800c1365d4b4f3"))),
            H128(ethereum_types::H128(hex!("3963a16de74721a202e7f10d66278fe4"))),
            H128(ethereum_types::H128(hex!("2f886a0a39058911d72b46e15bc34672"))),
            H128(ethereum_types::H128(hex!("bf8c454a96a689eb71c30d9639aaecee"))),
            H128(ethereum_types::H128(hex!("761b3e46118bc24bc62987107f3d12c6"))),
            H128(ethereum_types::H128(hex!("891583dc69ff4a5e64070d942aaa435f"))),
            H128(ethereum_types::H128(hex!("d8b34532a52763f1afd495aa3e36b2ef"))),
            H128(ethereum_types::H128(hex!("2f9e4d03913cd937e09c451b3ed20dcb"))),
            H128(ethereum_types::H128(hex!("93d22323cd8c06ec945733ee811d8ac8"))),
            H128(ethereum_types::H128(hex!("2a9d9c385dc260a178c9dd5902499f7e"))),
            H128(ethereum_types::H128(hex!("45e79066792ee537ae6106b3c897d44c"))),
            H128(ethereum_types::H128(hex!("4e00df4f849deba8f05284dba1a8daf6"))),
            H128(ethereum_types::H128(hex!("9ed2f8a53f69dee1e9b2d4a332ac80d5"))),
            H128(ethereum_types::H128(hex!("b0cb763b4c0e4bddbdeab130195681bb"))),
            H128(ethereum_types::H128(hex!("c25c64f479521ed7a68cb75637498e67"))),
            H128(ethereum_types::H128(hex!("a66e88f5a0279ebbfc9063d5d7fc9681"))),
            H128(ethereum_types::H128(hex!("97f23e83e5a2c1e6209a1e0baa4c9048"))),
            H128(ethereum_types::H128(hex!("08efb5ef7d86b52c486f88ea92865e2e"))),
            H128(ethereum_types::H128(hex!("750b98718c4d7f9b63a0fe4135a00143"))),
            H128(ethereum_types::H128(hex!("bd71d4d32938661a8e4e8e198f6e3c71"))),
            H128(ethereum_types::H128(hex!("dac6dce2e49f253706ee5ea4549abb67"))),
            H128(ethereum_types::H128(hex!("1dfa7fc8cff2108f4de96a6f6404321b"))),
            H128(ethereum_types::H128(hex!("58fa94796612dacc2f2a60fbac5f85d6"))),
            H128(ethereum_types::H128(hex!("af4a599a7afc59244662fb56a32f38cb"))),
            H128(ethereum_types::H128(hex!("7b2920aac8c076c5fccfdf3325fc8455"))),
            H128(ethereum_types::H128(hex!("b3328f0b1057958da28bab59330133a7"))),
            H128(ethereum_types::H128(hex!("ad4e0add9ad103421f47d88eeb5c711f"))),
            H128(ethereum_types::H128(hex!("4825b9d42589e834f61e6ef705641713"))),
            H128(ethereum_types::H128(hex!("3da44d4f1d8bb790537ec42ba2af168c"))),
            H128(ethereum_types::H128(hex!("87db7dab6b1aa2857fcf861273b9a58d"))),
            H128(ethereum_types::H128(hex!("c32c902e1389ebda24a09ae882575370"))),
            H128(ethereum_types::H128(hex!("cf17c3f198e852d5123942c402918656"))),
            H128(ethereum_types::H128(hex!("9f1cf97072ee00922c301340a19c91b7"))),
            H128(ethereum_types::H128(hex!("b3e163f4cbeac4437a962c84a85a1e5b"))),
            H128(ethereum_types::H128(hex!("a70314ea9655ebf03ee78a4a320d1ecc"))),
            H128(ethereum_types::H128(hex!("2ab485395195fd37e0fd5b2336f0a00a"))),
            H128(ethereum_types::H128(hex!("9f77060b503e1fbccf8b682215821b07"))),
            H128(ethereum_types::H128(hex!("a4fd17b615f2794b3fbb98ac81e0c5e7"))),
            H128(ethereum_types::H128(hex!("3e7faa44b3e919bf089ce8962a41596b"))),
            H128(ethereum_types::H128(hex!("f1cb06f527cfdb2bfb3e3341c878101d"))),
            H128(ethereum_types::H128(hex!("fe8cedf87702d7b090a0f07571607d86"))),
            H128(ethereum_types::H128(hex!("f569a8f30771d73544ad99fb1610b174"))),
            H128(ethereum_types::H128(hex!("1e332a7f9b33fc91369ba33503353023"))),
            H128(ethereum_types::H128(hex!("e04c52de8e81749474a0a3ef746c4c9d"))),
            H128(ethereum_types::H128(hex!("e961634b1721573ccbaf4c195ece7bd4"))),
            H128(ethereum_types::H128(hex!("c50b42bd793d49f0505df93353c4acde"))),
            H128(ethereum_types::H128(hex!("f8a9ea7fd860ad32e03ed50aebeb92f2"))),
            H128(ethereum_types::H128(hex!("f6a622025cb1659a5bce3c4cc7ed0680"))),
            H128(ethereum_types::H128(hex!("b6a78250c0253c2a8a985beb3ed16309"))),
            H128(ethereum_types::H128(hex!("d2ba47f421049058107969e08458e7bc"))),
            H128(ethereum_types::H128(hex!("66809b4880f156c8f539441829d11b90"))),
            H128(ethereum_types::H128(hex!("980b88f3b17ad1bf46ddc89356df550c"))),
            H128(ethereum_types::H128(hex!("083177d975088d3b3acb85c5e767948f"))),
            H128(ethereum_types::H128(hex!("07a3e31da3988ccc22a48cb61890ed83"))),
            H128(ethereum_types::H128(hex!("12c4f7a7402ada8fac7c2ddc784ca2cb"))),
            H128(ethereum_types::H128(hex!("a7bd8cdd867b4b3812f3066b3db3c006"))),
            H128(ethereum_types::H128(hex!("aa098d01c41cc948c138f864a8a62481"))),
            H128(ethereum_types::H128(hex!("18457233e28062083f7d23b2e481189d"))),
            H128(ethereum_types::H128(hex!("1702cda0b76772ba09cea0edc5e5746e"))),
            H128(ethereum_types::H128(hex!("db200270afe9e05cba79d94ff6d2da8c"))),
            H128(ethereum_types::H128(hex!("b93ce415bb6beb51157141149e34bd0e"))),
            H128(ethereum_types::H128(hex!("6266741ef0b85a2fd5ac4a1fb816835b"))),
            H128(ethereum_types::H128(hex!("8dba28245cf055574881b05fef9953a6"))),
            H128(ethereum_types::H128(hex!("e4af90f7979c2c631633131d642dd8bd"))),
            H128(ethereum_types::H128(hex!("97f98f4275be120a445cd0275e2cd73a"))),
            H128(ethereum_types::H128(hex!("150a9c0526b11752453a23d8b18a8f3b"))),
            H128(ethereum_types::H128(hex!("010bbf6895ade2375c8478a0c3151ce5"))),
            H128(ethereum_types::H128(hex!("355796530fdacf6d87bcc370f17fc71e"))),
            H128(ethereum_types::H128(hex!("9a404317c26f415ed025f32dfabe8598"))),
            H128(ethereum_types::H128(hex!("15d2eb783afced72c733f6ce90bf7349"))),
            H128(ethereum_types::H128(hex!("fb9f445a7acf24b91e6cbe8f9489a7c2"))),
            H128(ethereum_types::H128(hex!("6f03e5d4ef52a7c05a5a5fd28b159b5b"))),
            H128(ethereum_types::H128(hex!("2466fb6d4eb8aa1c700e728fded218df"))),
            H128(ethereum_types::H128(hex!("676cfafe2fbcffd070ddb236d2bb0021"))),
            H128(ethereum_types::H128(hex!("91e33a111622283750412eea13c83f35"))),
            H128(ethereum_types::H128(hex!("88b1f25057c3bac8ee1eeca2ff2209a3"))),
            H128(ethereum_types::H128(hex!("c10d6e9c953ebdc8ece36c5cd6223387"))),
            H128(ethereum_types::H128(hex!("1fb01164b818aa63387a0ec14be5e3e7"))),
            H128(ethereum_types::H128(hex!("aca8367a8bfd04541cc836e293255b77"))),
            H128(ethereum_types::H128(hex!("8b74b13c0d49da16c37a8de608c18e7e"))),
            H128(ethereum_types::H128(hex!("79e4197b401889e0756cedda74f46812"))),
            H128(ethereum_types::H128(hex!("fdfc1643dbd6ad08bd6a4eba37a0e3c3"))),
            H128(ethereum_types::H128(hex!("3c4b6a74dd034b4e72bc84652a09a3ff"))),
            H128(ethereum_types::H128(hex!("2f31fab52ef05919d280c2abcf422fab"))),
            H128(ethereum_types::H128(hex!("4a2f98048e8605e4d439ff8554ab6e63"))),
            H128(ethereum_types::H128(hex!("3b7e760d63c75a4c368dd53425084427"))),
            H128(ethereum_types::H128(hex!("dbd55facc2eed4edae760a2ba92b4f39"))),
            H128(ethereum_types::H128(hex!("f0e079daee7e4fae706c60345eaed7c6"))),
            H128(ethereum_types::H128(hex!("00cd47758ac9dfe055865748f9f15b3a"))),
            H128(ethereum_types::H128(hex!("5bb13e4d95fd1a1d551a0a8bbb724fbb"))),
            H128(ethereum_types::H128(hex!("2ab38207d22885d80753f77eb8f11188"))),
            H128(ethereum_types::H128(hex!("77af57a3e73852729f602770889d41f6"))),
            H128(ethereum_types::H128(hex!("4e76a9575455c544259f6fa4dc28ec73"))),
            H128(ethereum_types::H128(hex!("dced4535167e2f1ff0075e1fbca1f32b"))),
            H128(ethereum_types::H128(hex!("94ac1540daf6ee75412dfedf521ac26b"))),
            H128(ethereum_types::H128(hex!("a5f4edb22058795428c0e3f0984c4e42"))),
            H128(ethereum_types::H128(hex!("c21b458e1b2973ad8d1a42865476e420"))),
            H128(ethereum_types::H128(hex!("b8edf8bfd4fa0dcd9d68ca62de7f8163"))),
            H128(ethereum_types::H128(hex!("8ca26ec2b20aaa2a003f4732b133d55e"))),
            H128(ethereum_types::H128(hex!("518a1c489f3d366a6175a6c27caedb6d"))),
            H128(ethereum_types::H128(hex!("11aa2bd74ac3e01a7ee5d413d3607681"))),
            H128(ethereum_types::H128(hex!("3f2e4621aff23efdbe33e8c828c4d45a"))),
            H128(ethereum_types::H128(hex!("ff373231626f5dbcb6f1e0216aaec0ba"))),
            H128(ethereum_types::H128(hex!("d400fa2884977ede9fb3667d23d984db"))),
            H128(ethereum_types::H128(hex!("3ec403699e9f2e18d23f8e777cafa676"))),
            H128(ethereum_types::H128(hex!("b6d0a777a0ce6e68f7bdeb79f37ae378"))),
            H128(ethereum_types::H128(hex!("9e20d21eaa17f971543fe70ac15df078"))),
            H128(ethereum_types::H128(hex!("8566b7226d025a7fcd4d61a7cb76edef"))),
            H128(ethereum_types::H128(hex!("86d6b187f841cb3fcc92a27060e8f9ea"))),
            H128(ethereum_types::H128(hex!("bf8d2e422a91c28cc445cb08a87bc401"))),
            H128(ethereum_types::H128(hex!("0d64d2fbbad9c1b0530052ea47d4539b"))),
            H128(ethereum_types::H128(hex!("0d5279a8100c0a044cf13882942f3c5e"))),
            H128(ethereum_types::H128(hex!("4d914c7455896b6aeafbfaf5bbb7c69f"))),
            H128(ethereum_types::H128(hex!("34859258bb4bac5923fb3c73c8e95fbb"))),
            H128(ethereum_types::H128(hex!("cb2409f400cef34f88c7f29b731d7d59"))),
            H128(ethereum_types::H128(hex!("7e9bc66a6bc2a5c5692306db1b6c474f"))),
            H128(ethereum_types::H128(hex!("9b125466b31b3e4275b9c3f477ad9bb4"))),
            H128(ethereum_types::H128(hex!("0682605f164269183277a0bfa1a7aa86"))),
            H128(ethereum_types::H128(hex!("fdf029dca743acf24f4cbebbc846e990"))),
            H128(ethereum_types::H128(hex!("4e5fa1902e699e059e6ac657585525f3"))),
            H128(ethereum_types::H128(hex!("c70fced2684c45f39a227f1b0e6a2639"))),
            H128(ethereum_types::H128(hex!("2e8ca3caf417021d3209da66f0d125b8"))),
            H128(ethereum_types::H128(hex!("18a51613c5ae13e32c5bcf4965b78583"))),
            H128(ethereum_types::H128(hex!("402ece445768d17f2790dc0cdbf7f9fa"))),
            H128(ethereum_types::H128(hex!("7ef9cfe23a9f2d9b5fa10cc91e601f1c"))),
            H128(ethereum_types::H128(hex!("ba2dd4d240fbd92f1d46cd90a63cb301"))),
            H128(ethereum_types::H128(hex!("29cff10b881311ce6292a765b9086853"))),
            H128(ethereum_types::H128(hex!("9a016ffcb23883d0328d0183e035ed18"))),
            H128(ethereum_types::H128(hex!("ffcbd9d2ef2912e64d811c45700fcaec"))),
            H128(ethereum_types::H128(hex!("0ce6dccd903d0e4809ad3e300da7a455"))),
            H128(ethereum_types::H128(hex!("846bc24d47b884f73346f81788688374"))),
            H128(ethereum_types::H128(hex!("bb5094794e59876aa8301adb7126a2c3"))),
            H128(ethereum_types::H128(hex!("b8f55aa699f64d44a2c944246686298a"))),
            H128(ethereum_types::H128(hex!("b824a33ef385a5293d2570326a7227c3"))),
            H128(ethereum_types::H128(hex!("a9d724a77e1f87aaec95df4050649b1b"))),
            H128(ethereum_types::H128(hex!("c0a76950b77ced186bdd52e1dc3ba8b7"))),
            H128(ethereum_types::H128(hex!("38c28c14bfa88693bf306588e08ae09f"))),
            H128(ethereum_types::H128(hex!("04f9280b95c9812a51299359c770b913"))),
            H128(ethereum_types::H128(hex!("94f78ea30bbbc5024bc05c06fc80a3a3"))),
            H128(ethereum_types::H128(hex!("57e2ff4bcecff754849ea5c15684df75"))),
            H128(ethereum_types::H128(hex!("9064b534593c012115617b850814974e"))),
            H128(ethereum_types::H128(hex!("96b84b73c9bd4fe4686b9c5e47057279"))),
            H128(ethereum_types::H128(hex!("79422e6008157852225f4960b952ce94"))),
            H128(ethereum_types::H128(hex!("16e6b48564c7f62bb0d6a5d8562f71e1"))),
            H128(ethereum_types::H128(hex!("9ba574947f637db0922b5a3411b39073"))),
            H128(ethereum_types::H128(hex!("9c0ea3342b493673eb6ba7c0f3e33001"))),
            H128(ethereum_types::H128(hex!("13219cddbc960a443b68f73348bdabb7"))),
            H128(ethereum_types::H128(hex!("9612e953f80bcec6fd9c11e80642e07c"))),
            H128(ethereum_types::H128(hex!("7e1a63fe5d68d66440f69212f0d06a7d"))),
            H128(ethereum_types::H128(hex!("29388bc27f370d522dc179cfddcefcf0"))),
            H128(ethereum_types::H128(hex!("22d098169c99d823adbc0279467cdd55"))),
            H128(ethereum_types::H128(hex!("8cb0763f7a0ec5aa0761bfbb049fe2c1"))),
            H128(ethereum_types::H128(hex!("ccd344c4e3b6b028253a724d7014b831"))),
            H128(ethereum_types::H128(hex!("8a409beab35999899ae65a0b4519e081"))),
            H128(ethereum_types::H128(hex!("472da5358a5c51cbe55d4beadfa8d2ef"))),
            H128(ethereum_types::H128(hex!("af3eb432bc504607d20dd9b93fdf0382"))),
            H128(ethereum_types::H128(hex!("15652ee8ba674184c53238346e7c2818"))),
            H128(ethereum_types::H128(hex!("1038ca6c9eca2e2240ae40dcee168d7f"))),
            H128(ethereum_types::H128(hex!("8f99e10b2925562e1061f7ad3a7f591e"))),
            H128(ethereum_types::H128(hex!("ec7875f6e017ec743ce364f257c79a70"))),
            H128(ethereum_types::H128(hex!("df953a9008d4f96c63da1993439a81c6"))),
            H128(ethereum_types::H128(hex!("c44353ffa531580a5bbbbb8faa0855cd"))),
            H128(ethereum_types::H128(hex!("4d2020a9a71a5351f415552099e2760a"))),
            H128(ethereum_types::H128(hex!("39c14bb1c059f585fd4ba723dc1c66ca"))),
            H128(ethereum_types::H128(hex!("4fbbf8b5795d2a8b7a83b3768869b5e1"))),
            H128(ethereum_types::H128(hex!("8210215bfb4ace609e0fb0973ac511de"))),
            H128(ethereum_types::H128(hex!("16fba09f0e8ecd584d0524a4c5723a8b"))),
            H128(ethereum_types::H128(hex!("8619f37f7511cec0777ab7efe07ae451"))),
            H128(ethereum_types::H128(hex!("a4ff22443285056f6e6be1bba3a886e8"))),
            H128(ethereum_types::H128(hex!("a13fb0480ab2fb0cdb3a1373694323f9"))),
            H128(ethereum_types::H128(hex!("c9b9c6a82b04c72d8970ec3e015c2607"))),
            H128(ethereum_types::H128(hex!("064449044bd883c413684d6d29eef904"))),
            H128(ethereum_types::H128(hex!("b1896f87a9ae66ec233de69dd5b5fc50"))),
            H128(ethereum_types::H128(hex!("4d39896b61686334603e8accbb0288ff"))),
            H128(ethereum_types::H128(hex!("e0687ca2fc99a162a61e83da0f54dfea"))),
            H128(ethereum_types::H128(hex!("e12c1c535fc6f6498de80b0da9094c80"))),
            H128(ethereum_types::H128(hex!("4731d5e4b9421510c3ddf73d87a9c4a3"))),
            H128(ethereum_types::H128(hex!("247a38f6f0fcc658fc35057787bd9054"))),
            H128(ethereum_types::H128(hex!("8974ef985b8dc87bbf14f16657f3bd30"))),
            H128(ethereum_types::H128(hex!("89073a016fd5c618095291915c5912e1"))),
            H128(ethereum_types::H128(hex!("86f16e07d569b7570af0031fb6c36af4"))),
            H128(ethereum_types::H128(hex!("12d5be9063d155f9df791be6c35f1865"))),
            H128(ethereum_types::H128(hex!("8962f4ccba3d7cb61f0a501d474f9906"))),
            H128(ethereum_types::H128(hex!("efee4030b687d7ae3eb2d27673f65343"))),
            H128(ethereum_types::H128(hex!("2464c007ea23451f778ed2bf0c017b7b"))),
            H128(ethereum_types::H128(hex!("6dbe36b12f350e01689f6295d686b8b9"))),
            H128(ethereum_types::H128(hex!("025cafd23d97228050066bf4128a4fa3"))),
            H128(ethereum_types::H128(hex!("de0bb757deba77755678381d08a12bf1"))),
            H128(ethereum_types::H128(hex!("1c542ed68bb6f818c4bee47703298d08"))),
            H128(ethereum_types::H128(hex!("8178b368306f2ae1abd7e68583d67dad"))),
            H128(ethereum_types::H128(hex!("cb4d956080d6f8bb617a2d51e5ac1296"))),
            H128(ethereum_types::H128(hex!("06dc4c3c0f4d7c5f4784f3e865433730"))),
            H128(ethereum_types::H128(hex!("7134fde96ed353c8d6ac24ec7efccaf9"))),
            H128(ethereum_types::H128(hex!("1466a522411631d6c9c5c706558985c1"))),
            H128(ethereum_types::H128(hex!("95e26e89a1c7b9df6a7c2c56223292b5"))),
            H128(ethereum_types::H128(hex!("6555486abac007b7fadaf6e04f896239"))),
            H128(ethereum_types::H128(hex!("472a1d76a9734964c42b92b36993ed49"))),
            H128(ethereum_types::H128(hex!("cea176b441e44750555739f29e045e74"))),
            H128(ethereum_types::H128(hex!("feb06e244cdbf8e8cb1d5fab49c6e583"))),
            H128(ethereum_types::H128(hex!("499c5a7fc475c7ab9e888a9f8aeb5765"))),
            H128(ethereum_types::H128(hex!("d3b05f45181c0b1b4a0354ffe3d95c9a"))),
            H128(ethereum_types::H128(hex!("b768bcc1492f03e1819620dfa2e130d5"))),
            H128(ethereum_types::H128(hex!("a38c13579500a7c7a90c80153ab22f4f"))),
            H128(ethereum_types::H128(hex!("adb73d387c59a6ba8eff18a6b0320ca4"))),
            H128(ethereum_types::H128(hex!("09c8e0bdd1ba73602063d89d11ff5210"))),
            H128(ethereum_types::H128(hex!("ecab8f890f1f3d6ff564eebab034ebb3"))),
            H128(ethereum_types::H128(hex!("9823e2e8911c5a0fb2ff6feb52be2c0f"))),
            H128(ethereum_types::H128(hex!("4fa03dbaabcf99e71903f3177444bfaa"))),
            H128(ethereum_types::H128(hex!("a4c9667d00998d262c47f6d2c79f43c2"))),
            H128(ethereum_types::H128(hex!("b3e65ee26384d25bc7750b3f37e72883"))),
            H128(ethereum_types::H128(hex!("7dea46d1a183f3070760eee0bf5fce3c"))),
            H128(ethereum_types::H128(hex!("ecb2963c22a757569fe659635f4b0243"))),
            H128(ethereum_types::H128(hex!("9ed968203144981e6e697db052910c27"))),
            H128(ethereum_types::H128(hex!("4aa6ca6ebef942d8766065b2e590fd32"))),
        ];

        let mut contract = EthBridge::default();
        contract.init(0, dags_merkle_roots.clone());
        assert_eq!(dags_merkle_roots[0], contract.dag_merkle_root(0));
        assert_eq!(dags_merkle_roots[10], contract.dag_merkle_root(10));
        assert_eq!(dags_merkle_roots[511], contract.dag_merkle_root(511));

        let result = catch_unwind_silent(|| contract.dag_merkle_root(512));
        assert!(result.is_err());
    }

    // #[test]
    // fn add_400000_block_only() {
    //     testing_env!(get_context(vec![], false));

    //     // Check on 400000 block from this answer: https://ethereum.stackexchange.com/a/67333/3032
    //     let (hashes, blocks) = get_blocks(&WEB3RS, 400_000, 400_001);

    //     // $ ../ethrelay/ethashproof/cmd/relayer/relayer 400000
    //     // digest: 0x3fbea7af642a4e20cd93a945a1f5e23bd72fc5261153e09102cf718980aeff38
    //     // ethash result: 0x00000000000ca599ebe9913fa00da78a4d1dd2fa154c4fd2aad10ccbca52a2a1
    //     // Proof length: 24
    //     // [400000.json]

    //     // let dag_nodes = 

    //     let mut contract = EthBridge::default();
    //     let result = catch_unwind_silent(panic::AssertUnwindSafe(
    //         || contract.add_block_headers(
    //             blocks.clone(),
    //             blocks.map(|b| b.nonce).to_vec(),
    //             dag_nodes
    //         )
    //     ));
    //     assert!(result.is_err());
    //     contract.add_block_headers(400_000, blocks);
    //     assert_eq!(hashes[0], (contract.block_hash_unsafe(400_000).unwrap().0).0.into());
    // }

    // #[test]
    // fn add_20_blocks_from_8000000() {
    //     testing_env!(get_context(vec![], false));

    //     let start: usize = 8_000_000;
    //     let stop: usize = 8_000_020;

    //     let (hashes, blocks) = get_blocks(&WEB3RS, start, stop);
        
    //     let mut contract = EthBridge::default();
    //     contract.add_block_headers(start as u64, blocks);

    //     for i in start..stop {
    //         assert_eq!(hashes[i - start], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    // }

    // #[test]
    // fn add_3_sequential_ranges_of_blocks() {
    //     testing_env!(get_context(vec![], false));

    //     let (hashes1, blocks1) = get_blocks(&WEB3RS, 8_000_000, 8_000_010);
    //     let (hashes2, blocks2) = get_blocks(&WEB3RS, 8_000_010, 8_000_020);
    //     let (hashes3, blocks3) = get_blocks(&WEB3RS, 8_000_020, 8_000_030);
        
    //     let mut contract = EthBridge::default();
    //     contract.add_block_headers(8_000_000 as u64, blocks1);
    //     contract.add_block_headers(8_000_010 as u64, blocks2);
    //     contract.add_block_headers(8_000_020 as u64, blocks3);

    //     for i in 8_000_000..8_000_010 {
    //         assert_eq!(hashes1[i - 8_000_000], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    //     for i in 8_000_010..8_000_020 {
    //         assert_eq!(hashes2[i - 8_000_010], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    //     for i in 8_000_020..8_000_030 {
    //         assert_eq!(hashes3[i - 8_000_020], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    // }

    // #[test]
    // fn add_3_intersecting_ranges_of_blocks() {
    //     testing_env!(get_context(vec![], false));

    //     let (hashes1, blocks1) = get_blocks(&WEB3RS, 8_000_000, 8_000_010);
    //     let (hashes2, blocks2) = get_blocks(&WEB3RS, 8_000_005, 8_000_020);
    //     let (hashes3, blocks3) = get_blocks(&WEB3RS, 8_000_015, 8_000_030);
        
    //     let mut contract = EthBridge::default();
    //     contract.add_block_headers(8_000_000 as u64, blocks1);
    //     contract.add_block_headers(8_000_005 as u64, blocks2);
    //     contract.add_block_headers(8_000_015 as u64, blocks3);

    //     for i in 8_000_000..8_000_010 {
    //         assert_eq!(hashes1[i - 8_000_000], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    //     for i in 8_000_005..8_000_020 {
    //         assert_eq!(hashes2[i - 8_000_005], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    //     for i in 8_000_015..8_000_030 {
    //         assert_eq!(hashes3[i - 8_000_015], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    // }
}
