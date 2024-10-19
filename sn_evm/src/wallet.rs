use evmlib::{wallet::{get_random_private_key_for_wallet, Wallet}, Network};
use evmlib::utils::get_evm_network_from_env;
use tokio::{runtime::Runtime, task};
use std::fs::File;
use std::fs;
use std::io::Write;
use std::path::Path;
use rpassword::read_password;
// use tokio;

const WALLET_DIR_NAME: &str = "wallet";
pub const ENCRYPTED_MAIN_SECRET_KEY_FILENAME: &str = "main_secret_key.encrypted";

pub fn get_random_private_key() -> String {
    get_random_private_key_for_wallet()
}

pub fn get_gas_token_details(private_key: &String) {
    let network = get_evm_network_from_env()
    .expect("Failed to get EVM network from environment variables");
    let wallet = Wallet::new_from_private_key(network, &private_key)
                            .expect("Could not init deployer wallet");

    // let gas_tokens = wallet.balance_of_gas_tokens();
    // let rt = tokio::runtime::Runtime::new().unwrap();

    task::block_in_place(|| {
        let rt = Runtime::new().expect("failed to create Tokio runtime");

        rt.block_on(async {
            println!("Running inside the block in place!");
            match wallet.balance_of_gas_tokens().await {
                Ok(balance) => println!("balance of gas tokens: {:?}", balance),
                Err(e) => eprintln!("Error: {:?}", e),
            }
            match wallet.balance_of_tokens().await {
                Ok(balance) => println!("balance of tokens: {:?}", balance),
                Err(e) => eprintln!("Error: {:?}", e),
            }
            
        })
    })
}

pub fn create_a_evm_wallet(private_key: &String) -> String {
    let network = get_evm_network_from_env()
                        .expect("Failed to get EVM network from environment variables");
    let wallet = Wallet::new_from_private_key(network, &private_key)
                                                .expect("Could not init deployer wallet");
    hex::encode(wallet.address())
}

pub fn create_file_with_keys(private_key: String, public_key: String) -> String {
    let dir_path: &str = "safe/client/wallets";
    fs::create_dir_all(dir_path).expect("could not create the directory");
    let full_path_wallet = format!("{}/{}", dir_path, public_key);    
    let mut file = File::create(full_path_wallet.clone()).expect("could not create file");
    file.write_all(private_key.as_bytes()).expect("Not able to write into file");
    let file_path = Path::new(&full_path_wallet).canonicalize().expect("Not able to find the absolute path for the file");
    file_path.to_string_lossy().to_string()
}

pub fn wallet_encryption_status(root_dir: &Path) -> bool {
    let wallelt_file_path = root_dir.join(ENCRYPTED_MAIN_SECRET_KEY_FILENAME);
    wallelt_file_path.is_file()
}

pub fn wallet_encryption_storage(dir_path: &str, content: &str) -> String {
    // ensure the directory exists;
    fs::create_dir_all(dir_path).expect("could not create the directory");
    let file_path = format!("{}/{}", dir_path, ENCRYPTED_MAIN_SECRET_KEY_FILENAME);

    let mut file = File::create((&file_path)).expect("Not able to create the file");
    file.write_all(content.as_bytes()).expect("Not able to write into the file");
    let file_path = Path::new(&file_path).canonicalize().expect("Not able to find the absolute path for the file");
     file_path.to_string_lossy().to_string()
}

pub fn prompt_the_user_for_password() -> Option<String> {
    println!("Please enter the password: ");
    let pwd = match read_password() {
        Ok(pwd) => pwd,
        Err(e) => {
            eprintln!("Failed to read password: {}",e);
            return None;
        }
    };
    Some(pwd)
}