use hex::FromHex;
use rlp::RlpStream;
use serde::{Deserialize, Deserializer};
use crate::{EthProver};

//#[macro_use]
//extern crate lazy_static;
use lazy_static::lazy_static;

#[derive(Debug)]
struct Hex(pub Vec<u8>);

impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let mut s = <String as Deserialize>::deserialize(deserializer)?;
        if s.starts_with("0x") {
            s = s[2..].to_string();
        }
        if s.len() % 2 == 1 {
            s.insert_str(0, "0");
        }
        Ok(Hex(Vec::from_hex(&s).map_err(|err| {
            serde::de::Error::custom(err.to_string())
        })?))
    }
}

// TESTS

use near_bindgen::MockedBlockchain;
use near_bindgen::{testing_env, VMContext};

lazy_static! {
    static ref WEB3RS: web3::Web3<web3::transports::Http> = {
        let (eloop, transport) = web3::transports::Http::new(
            "https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2",
        )
        .unwrap();
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
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        is_view,
        output_data_receivers: vec![],
    }
}

#[test]
fn simple_tx_res() {
    testing_env!(get_context(vec![], false));

    let contract = EthProver::init("ethbridge".to_string());

    // Following data could be extracted by:
    // NODE_URL="https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2" TX_HASH="0xb540248a9cca048c5861dec953d7a776bc1944319b9bd27a462469c8a437f4ff" EVENT_INDEX=0 node extract.js

    let log_index = 0;
    let receipt_index = 0;
    let header_data = Vec::from_hex("f9021aa0f779e50b45bc27e4ed236840e5dbcf7afab50beaf553be56bf76da977e10cc73a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479452bc44d5378309ee2abf1539bf71de1b7d7be3b5a014c996b6934d7991643669e145b8355c63aa02cbde63d390fcf4e6181d5eea45a079b7e79dc739c31662fe6f25f65bf5a5d14299c7a7aa42c3f75b9fb05474f54ca0e28dc05418692cb7baab7e7f85c1dedb8791c275b797ea3b1ffcaec5ef2aa271b9010000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000010000000000000000000000000000000000000000000000000000000408000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000010000000000000000000000000000000000000000000000000000000400000000000100000000000000000000000000080000000000000000000000000000000000000000000100002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000903234373439353837313930323034343383890fe68395ba8e82d0d9845dd84a079150505945206e616e6f706f6f6c2e6f7267a0a35425f443452cf94ba4b698b00fd7b3ff4fc671dea3d5cc2dcbedbc3766f45e88af7fec6031063a17").unwrap();
    let receipt_data = Vec::from_hex("f901a60182d0d9b9010000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000010000000000000000000000000000000000000000000000000000000408000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000010000000000000000000000000000000000000000000000000000000400000000000100000000000000000000000000080000000000000000000000000000000000000000000100002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000f89df89b94dac17f958d2ee523a2206206994597c13d831ec7f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000006cc5f688a315f3dc28a7781717a9a798a59fda7ba00000000000000000000000007e7a32d9dc98c485c489be8e732f97b4ffe3a4cda000000000000000000000000000000000000000000000000000000001a13b8600").unwrap();
    let log_entry = Vec::from_hex("f89b94dac17f958d2ee523a2206206994597c13d831ec7f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000006cc5f688a315f3dc28a7781717a9a798a59fda7ba00000000000000000000000007e7a32d9dc98c485c489be8e732f97b4ffe3a4cda000000000000000000000000000000000000000000000000000000001a13b8600").unwrap();
    let proof = vec![
        vec![
            Vec::from_hex("2080").unwrap(),
            Vec::from_hex("f901a60182d0d9b9010000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000010000000000000000000000000000000000000000000000000000000408000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000010000000000000000000000000000000000000000000000000000000400000000000100000000000000000000000000080000000000000000000000000000000000000000000100002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000f89df89b94dac17f958d2ee523a2206206994597c13d831ec7f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000006cc5f688a315f3dc28a7781717a9a798a59fda7ba00000000000000000000000007e7a32d9dc98c485c489be8e732f97b4ffe3a4cda000000000000000000000000000000000000000000000000000000001a13b8600").unwrap(),
        ],
    ].iter().map(|node| {
        let mut stream = RlpStream::new();
        stream.begin_list(node.len());
        for item in node {
            stream.append(item);
        }
        stream.out()
    }).collect();

    assert!(contract.verify_log_entry(
        log_index,
        log_entry,
        receipt_index,
        receipt_data,
        header_data,
        proof,
        true
    ));
}

#[test]
fn complex_tx_res() {
    testing_env!(get_context(vec![], false));

    let contract = EthProver::init("ethbridge".to_string());

    // Following data could be extracted by:
    // NODE_URL="https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2" TX_HASH="0xa7e1633e8099ea2b72496207b76a0e04a761c52f48c82bfcf6b327495258e4e0" EVENT_INDEX=0 node extract.js

    let log_index = 0;
    let receipt_index = 190;
    let header_data = Vec::from_hex("f9021ea059822ac0f17af7578ac8cf2c655a4ccfb1f5622ecad88992a2522e91a3cc401da01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794829bd824b016326a401d083b33d092293333a830a01f54c61cebc3d08954929f359668498e191c1a34adb47f5db2881d414b69631ca0adf860aabb9786198a7d91dec6514ecebfd5aba4565fb7a81f8e68cf819b3000a0dddbc1d4cc2f1f522a3ba1f627d7645945d4922d6eb82c318c7be64735ec0e0db901000416012000c9a88c580040302001880844228042b418e12800bd011510a9580700f15880012010400858c08042dab1408b200403c1484425ac90c800d02c0800050050c04b4cc4b01083211905c820409a000e0c1346200508024c8020c892907120102c0a2840332290050028808c2104611a0c90944a020c002853968810640c0190030c342500584004239a2704228212001dc80004016803a15610920180070c89000a802115094a4082450080522001318488a8098a08c28000188025580490a4aa00b2802012001050868800140d0fd20800d8200682226060119420049032000832001a4011780096081c7654002800100d9484102144ec889094a44a903233363631373837353134373338393883984f928398684283986153845eac64b1947070796520e4b883e5bda9e7a59ee4bb99e9b1bca0814ab4871d625ff894a6736dcf195f4ce1c0f5dd8a3d42a1e9587c2a9e1d522988dd7482ba86e3b14a").unwrap();
    let receipt_data = Vec::from_hex("f901a701839590b6b9010000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000001000000000000000000000000000000010000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000004000000000000000010000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000600000000000f89df89b945885d2a27bd4c6d111b83bc3fc359ed951e8e6f8f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000009b24ee0bfbf708b541fb65b6087d6e991a0d11a8a00000000000000000000000005a012de1a4c89b496e0a276158552abe6f843a6fa0000000000000000000000000000000000000000000000001135631d5283a4000").unwrap();
    let log_entry = Vec::from_hex("f89b945885d2a27bd4c6d111b83bc3fc359ed951e8e6f8f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000009b24ee0bfbf708b541fb65b6087d6e991a0d11a8a00000000000000000000000005a012de1a4c89b496e0a276158552abe6f843a6fa0000000000000000000000000000000000000000000000001135631d5283a4000").unwrap();
    let proof = vec![
        vec![
            Vec::from_hex("0ceed761f9ef77af663b4d3ba5254eda2ba35ad0b32a06d57d350be62c8f2b98").unwrap(),
            Vec::from_hex("297cb12d9b6b25bff0bd3199bfd2fdb4b3fca45416ecc355958b68dc1b7bc7a3").unwrap(),
            Vec::from_hex("870bde7cc109d51463b1b8e77ebb36cb4021ddc7df0fd31c3140cad3260236e8").unwrap(),
            Vec::from_hex("8ddcf32ce5a1ac912bddc28f2f25df0921aaff8f100f2979028b27baa5b191f1").unwrap(),
            Vec::from_hex("4de7d3b04b549af847ffcc757f8aa121b02bd00c1233e90547fac99da7049e19").unwrap(),
            Vec::from_hex("206564eec53442ea3334438820c75c1f8bdb617430956f07a08e66f5990458ae").unwrap(),
            Vec::from_hex("b11876d993891898b6397d0fa744783ba5bf35e79a329c7b9710e9026c2957c8").unwrap(),
            Vec::from_hex("719cc527cb7fcec3efaf017c5c4801609fa41572befae0468c24ac48fd389437").unwrap(),
            Vec::from_hex("e59bf6fa8f9359d07651b581b3875fc8aea5b85ba0eab9e23503dfa273b9b0f0").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
        ],
        vec![
            Vec::from_hex("a123f503897e190fa46ac478716778ebff6c1661b676c910cb4d4c6764457e06").unwrap(),
            Vec::from_hex("4ca92d89e85b0bc5a8670844967a027f012db12fb027808532c0ad61a751747e").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
        ],
        vec![
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("578357a5e1c44fdccf2cc34d27142b04ec433fe3c8bc57a35c4971f6a82a4da4").unwrap(),
            Vec::from_hex("4d6d3155937f76f01a359ebff4336a504ede8e283e555e1194468883854b7b11").unwrap(),
            Vec::from_hex("ce5478625cca3b625badaa26d231cbd072735b72307b7bce43a7917bd8ba04be").unwrap(),
            Vec::from_hex("66d11ae6215298daa654e04de8649aeb1e116f7138bf3d921c5cbe0a52589116").unwrap(),
            Vec::from_hex("e80733626a805d2974d9b97db52e18d2ce28f532f96552098bf0e656c6bb39a9").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
            Vec::from_hex("").unwrap(),
        ],
        vec![
            Vec::from_hex("46935fcc8ffcc89c8eae03fef89ac9fbde75be14b64c810fb1447d328907456d").unwrap(),
            Vec::from_hex("15ec39ed1284cdf4840d6b7511ff34aed164fd9e4b920b1fdb6fde54958d3a17").unwrap(),
            Vec::from_hex("e02aae4672f8b80f468759b092e4eb809edc8cf4b513df11b21d4233556c0b4d").unwrap(),
            Vec::from_hex("0efe37f3d3919b0bed73152bee545e033eb1fad5ddd8e383747decfd5ae93231").unwrap(),
            Vec::from_hex("16616fa5c192ddf9d0c2e1b1d70dc02a83f490b7d729072ac129d9f58f1e166d").unwrap(),
            Vec::from_hex("edb363ac38c62779bf449b0a51bb0b79a9541e8e5b12a8660185fa729795bedc").unwrap(),
            Vec::from_hex("c59dfcdfefed6a01b761ffd820cec8b8f52d9cb287759441cbd73e2a78ba2416").unwrap(),
            Vec::from_hex("893e6dacc036de93725e0e5112d849a735837a94536635318227b448535c6eb2").unwrap(),
            Vec::from_hex("dd8b4c24d7246d9c3a86fef130a5a3bc53f10d1ab706789d6914b8580c4c0202").unwrap(),
            Vec::from_hex("17c0abe1c74e0d01db47e4e2f2d75555d0ea3a2fc590670186ac23dd31944ec0").unwrap(),
            Vec::from_hex("4535f5fc448300be236c1e7f28576296b449592f5adbac59bf35a40e2f7701be").unwrap(),
            Vec::from_hex("bbf8eca1063220d003071437da87689887491c999913f5aba6ed21915c78407c").unwrap(),
            Vec::from_hex("1875e64c3cb31254ff0cf8041d4053c33e3f6aad8faad2c9323729324eaf1a9e").unwrap(),
            Vec::from_hex("7546cbb2b0137e669738c5ab69fb2071f755b253468f90da48855d41411d2e77").unwrap(),
            Vec::from_hex("d9677df9ddf784f9d20f24737b40cb372478114b225df8af225c0f40aaa843f5").unwrap(),
            Vec::from_hex("5b34e3dcfa33c72c4645793531246a91b74af04d564eebd63d00d0e274880283").unwrap(),
            Vec::from_hex("").unwrap(),
        ],
        vec![
            Vec::from_hex("20").unwrap(),
            Vec::from_hex("f901a701839590b6b9010000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000001000000000000000000000000000000010000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000004000000000000000010000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000600000000000f89df89b945885d2a27bd4c6d111b83bc3fc359ed951e8e6f8f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000009b24ee0bfbf708b541fb65b6087d6e991a0d11a8a00000000000000000000000005a012de1a4c89b496e0a276158552abe6f843a6fa0000000000000000000000000000000000000000000000001135631d5283a4000").unwrap(),
        ],
    ].iter().map(|node| {
        let mut stream = RlpStream::new();
        stream.begin_list(node.len());
        for item in node {
            stream.append(item);
        }
        stream.out()
    }).collect();

    assert!(contract.verify_log_entry(
        log_index,
        log_entry,
        receipt_index,
        receipt_data,
        header_data,
        proof,
        true
    ));
}
