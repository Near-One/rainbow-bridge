use arrayref::array_mut_ref;
use near_sdk::{env, near_bindgen, wee_alloc::WeeAlloc};

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

const TREE_STATE_KEY: &[u8] = b"T";
const TREE_ROOT_KEY: &[u8] = b"";

#[near_bindgen]
pub struct BridgeTest();

#[near_bindgen]
impl BridgeTest {
    pub fn send(#[serializer(borsh)] message: Vec<u8>) {
        let mut msg = env::predecessor_account_id().into_bytes();
        let len = msg.len();
        msg.insert(0, len as u8);
        msg.extend_from_slice(&message);
        let msg_hash = env::keccak256(&msg);
        let mut state;
        let n;
        if let Some(cur_state) = env::storage_read(TREE_STATE_KEY) {
            state = cur_state;
            let nref = array_mut_ref!(state, 0, 8);
            n = u64::from_be_bytes(*nref).checked_add(1).unwrap();
            *nref = n.to_be_bytes();
            state.extend_from_slice(&msg_hash);
            if n & n.wrapping_add(1) != 0 {
                let l = state.len();
                let p = l - 32 * (n.trailing_ones() as usize);
                let hash = env::keccak256(&state[p - 64..p]);
                state[p - 64..p - 32].copy_from_slice(&hash);
                state.copy_within(p..l, p - 32);
                state.truncate(state.len() - 32);
            }
        } else {
            state = Vec::with_capacity(40);
            state.extend_from_slice(&1u64.to_be_bytes());
            state.extend_from_slice(&msg_hash);
            n = 1;
        }
        env::storage_write(TREE_STATE_KEY, &state);
        if n > 2 {
            let mut i = 8;
            loop {
                let hash = env::keccak256(&state[i..i + 64]);
                i += 32;
                if i == state.len() - 32 {
                    state[8..40].copy_from_slice(&hash);
                    break;
                }
                state[i..i + 32].copy_from_slice(&hash);
            }
        }
        env::storage_write(TREE_ROOT_KEY, &env::keccak256(&state[..40]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use near_sdk::{testing_env, MockedBlockchain, VMContext};

    #[test]
    fn test_hashes() {
        for (sender, message, hash) in [
            ("ebrjsdxgvoi", &hex!("19370f4ae65c0e77") as &[u8], hex!("6183d25c683c764eb9f6378a03c088a6e32d2cba6e77e29c2aeda84ab2fcb3b7")),
            ("nxafvgnegntc", &hex!("15fbe479d8d37d79b27946abafdaa4158672b9a1fd78038a1c738100b39482fa79de3db7ac9f087cec950985cac5598677cd58402fdac233b84083841e58bef8fc961716c2c75776b964430258bfc952eae0b7803d59eac0927d6d9b855c2818c4a284b5a42162cae96650cdaaf50b71d7dd9a20ef8ee49a5d152d4818de998d2255c88cec8c8b18574acf7a023e7a4aa65c1cb132f1099952830a5339c97cb8f43d0a749549f2c78e2188aad2cc3f205782f03b2ae079ecf137fdb660cedba4c5cd516688e8ebc61762faa4cb039d2054bc84a83f8215e5af059b8b08107569a4d8fb21205c"), hex!("2b4b1e688aafa46b66be0036f0dcc6eee08921746bf7ca1e651802fb4ec5cd82")),
            ("nwepupqo", &hex!("19d94f"), hex!("58c8e7fe4c8f5ec4d67348669cf95c7958d3942e8d24b7cf55b1852750bf660a")),
            ("lthspk", &hex!("ef215fd71bbc"), hex!("afd7fcb45892ebc46616e9f7b32d34dae1e130f08ca8d9f15adc5b50557935e5")),
            ("qhnnacjwncbvdmxjbcsampdwvsanx", &hex!("5b1dbb9b1ea451192c715329e02a016834614b02aa1a"), hex!("e7d9bae1f45a3e4fc5d472b983978b6b256bad9a3a07bb4dee187bdb9a9ce4fe")),
            ("agaeiurobcrgin", &hex!("60f125f283a4f8c534a8fa6d3dace014b92e6aee38addd6464bb7d02b8bd4b526fc991bbf2ad55b026eb3f91146f17961c41a2debcd5bea4"), hex!("63fc06e899381185a985f99a624568f9e81da66f4113a4b13d8b79ac4bfa0f2a")),
            ("itrxrzhwvboqfbpbwobkjlxhpqbchgs", &hex!("f12d45057f37ac04fe"), hex!("6a0eefc45387cabf242f1bb8bc3d2ccbb023fea9c07b14ef2522b37f1dbaf8ae")),
            ("mwgvtvdqzezmlrcl", &hex!("bae0961cd6be47f28f00"), hex!("21bb3023552197abe850fd7234ad68952b327c47d299e6a6264f520e51c2a418")),
            ("gnh", &hex!("2deeebbbb4ab11a0"), hex!("02f9c322d67d6f825a477d916cab19eef711cb1de28258e1c5cc856c321010fd")),
            ("tqeh", &hex!("1ba5daabd1d83924fbb8d0fb782f211ad7fbe784391efce3df14bad596da122978c051cf7c44a4bdbd14146947051ed05fcb028d252b505f92a8e3422ce0e39667674f08550fcd79e9af70eb0a1ad45dcb1a3104"), hex!("eca53bb71637e15f8e1f89a9a682948f270a8266f1439ff74ef0328e1832f061")),
            ("hvmrllmhnysavqm", &hex!(""), hex!("f5aab25d48c7559aa60f8aa17117be1cee0af28ddcd97879d5f0cded644657a6")),
            ("zrpiiiyyrufhcmwnpnjel", &hex!("bc2c7ca320e6ba8c91a269672661e102e14b3a567e0663de923ea55588ca934ff0f2a714e288cbefb49fef393bd57dd2fb64dafa645347b8c35f7ca67a2475c755f1efa9dd4796dc34bcc4cf6f916712c24b1360585b0edaf547168fc2f7"), hex!("f355847fbc95218510bf00db4f156dfa7708d9510a0dec563984fb342cd575f3")),
            ("qr", &hex!("104ed6d9ea56d95c2beb220568515e80dab6634a5e2c14c6"), hex!("f7a58316f0dcad4f25547ef796f6c4e20b2c571976a546ea4ed6ad66e94fe2f4")),
            ("ipwjgntegfgysszm", &hex!("fae8"), hex!("ee82ddc523ed168c216810838d23e27e1213301fd4bd85d9f41affcbae4fbc25")),
            ("pb", &hex!("22fd4796b1a94b7822ad2eee08c004720156bbaf310cbbfced679bc79b216225f4192d651749cb4b72b145c2f9f2533ba1b8f519615057089317f12af559068caefb5fdf8050e7ee3fc97dcb0c8b6a6ac0b1d6ed126f2cf8c6511bdee751b552f858a985cb3fdc1f49df1138d7e272edb5a57d9829b522ae6a6865cb98fa2f8eea816b9ca3ad14c8812647d11549abc926158d7160479826c5c87d9e7723e16ec8c39c64aae6761d5a1c98409c1a475785b194ce5b020a8a20b597493b"), hex!("898c3993009d3c2cafaa37dd2dc880a10acbcc09a353e52a5fee5bf8b1c2c6ff")),
            ("jzgilcgqvxsqyrjybotqypiqqtbi", &hex!("6bf15090c07f"), hex!("1bbf4661e895ba88b90a3dc81c4fe7d4ad2d9593021fa74e0efe49e37ee51cc4")),
            ("ptty", &hex!("645cb63777c0011692980103876ee27cecf429438a3cfcd042ebd1d0df331a91c32c905c9cab197949083fe6e5f21b7911d9fbc1e8c7e7b8"), hex!("55349eb3fc3e6752a96c6c884f1d144d38a5f814cb937ae2cc24b26439e4d66b")),
            ("bd", &hex!("99b4933c52ef201742b4c2"), hex!("4cb128cd96d1234a3a9fea7f510aa8252cb34ecd6b2eadc606b9e982ec2cadb6")),
            ("vma", &hex!(""), hex!("373176be2e358fce322a1d5fe6743ba3c1c93a36c4ddebccf04ebf795ff05904")),
            ("iq", &hex!(""), hex!("e673b7609a5d44099f1608dc581944375aba3876aab489366d1fbfa6b2d0c548")),
        ].iter().copied() {
            testing_env!(VMContext {
                current_account_id: "".to_string(),
                signer_account_id: "".to_string(),
                signer_account_pk: vec![],
                predecessor_account_id: sender.to_string(),
                input: vec![],
                block_index: 0,
                block_timestamp: 0,
                epoch_height: 0,
                account_balance: 0,
                account_locked_balance: 0,
                storage_usage: 1000000,
                attached_deposit: 0,
                prepaid_gas: 1000_000000_000000,
                random_seed: vec![],
                is_view: false,
                output_data_receivers: vec![],
            });
            BridgeTest::send(message.into());
            assert_eq!(env::storage_read(TREE_ROOT_KEY).as_deref(), Some(&hash as &[u8]));
        }
    }
}
