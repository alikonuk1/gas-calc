use std::io;
use reqwest::Error;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CoinGeckoResponse {
    ethereum: Option<CurrencyInfo>,
    mantle: Option<CurrencyInfo>,
}

#[derive(Deserialize)]
struct CurrencyInfo {
    usd: f64,
}

struct MantleFeeEstimation {
    l2_gas_price: f64,
    l2_gas_used: u64,
    l1_gas_price: f64,
    overhead: f64,
    eth_to_mnt_ratio: f64,
}

impl MantleFeeEstimation {
    fn new(l2_gas_price: f64, l2_gas_used: u64, l1_gas_price: f64, overhead: f64, eth_to_mnt_ratio: f64) -> Self {
        MantleFeeEstimation {
            l2_gas_price,
            l2_gas_used,
            l1_gas_price,
            overhead,
            eth_to_mnt_ratio,
        }
    }

    fn calculate_l2_execution_fee(&self) -> f64 {
        self.l2_gas_price * self.l2_gas_used as f64 / 1_000_000_000.0
    }

    fn calculate_l1_rollup_fee(&self) -> f64 {
        self.l1_gas_price * self.overhead * self.eth_to_mnt_ratio / 1_000_000_000.0
    }

    fn calculate_total_fee_in_mnt(&self) -> f64 {
        self.calculate_l2_execution_fee() + self.calculate_l1_rollup_fee()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut input = String::new();

    println!("Select chain for gas calculation:");
    println!("1: Ethereum");
    println!("2: Mantle");
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let chain_selection: u8 = input.trim().parse().expect("Please select a valid option");
    input.clear();

    let conversion_response: CoinGeckoResponse = reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=ethereum,mantle&vs_currencies=usd")
        .await?
        .json()
        .await?;
    
    let eth_to_usd = conversion_response.ethereum.unwrap().usd;
    let mnt_to_usd = conversion_response.mantle.unwrap().usd;
    let eth_to_mnt_ratio = eth_to_usd / mnt_to_usd;

    if chain_selection == 1 {
        println!("Enter Ethereum gas price (in Gwei): ");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let eth_gas_price: f64 = input.trim().parse().expect("Please enter a valid number");
        input.clear();

        println!("Enter Ethereum gas used: ");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let eth_gas_used: u64 = input.trim().parse().expect("Please enter a valid number");

        let eth_fee = eth_gas_price * eth_gas_used as f64 / 1_000_000_000.0;
        let eth_fee_usd = eth_fee * eth_to_usd;
        println!("Total Transaction Fee: {:.18} ETH", eth_fee);
        println!("(~${:.2} USD)", eth_fee_usd);
    } else if chain_selection == 2 {
        println!("Enter L2 gas price (in Gwei equivalent of MNT): ");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let l2_gas_price: f64 = input.trim().parse().expect("Please enter a valid number");
        input.clear();

        println!("Enter L2 gas used: ");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let l2_gas_used: u64 = input.trim().parse().expect("Please enter a valid number");
        input.clear();

        println!("Enter L1 gas price (in Gwei equivalent of MNT): ");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let l1_gas_price: f64 = input.trim().parse().expect("Please enter a valid number");
        input.clear();

        println!("Enter overhead (a fixed overhead value): ");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let overhead: f64 = input.trim().parse().expect("Please enter a valid number");
        input.clear();

        let fee_estimate = MantleFeeEstimation::new(l2_gas_price, l2_gas_used, l1_gas_price, overhead, eth_to_mnt_ratio);
        let total_fee = fee_estimate.calculate_total_fee_in_mnt();
        let total_fee_usd = total_fee * mnt_to_usd;
        println!("Total Transaction Fee: {:.18} MNT (~${:.2} USD)", total_fee, total_fee_usd);
    } else {
        println!("Invalid selection. Please run the program again.");
    }

    Ok(())
}
