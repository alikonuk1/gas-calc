use rand::Rng;
use serde_derive::Deserialize;
use std::io;

#[derive(Deserialize)]
struct CoinGeckoResponse {
    ethereum: Option<CurrencyInfo>,
    mantle: Option<CurrencyInfo>,
}

#[derive(Deserialize)]
struct CurrencyInfo {
    usd: f64,
}

struct L2FeeEstimation {
    l2_gas_price: f64,
    l2_gas_used: u64,
    l1_gas_price: f64,
    l1_gas_used: u64,
}

impl L2FeeEstimation {
    fn new(l2_gas_price: f64, l2_gas_used: u64, l1_gas_price: f64, l1_gas_used: u64) -> Self {
        L2FeeEstimation {
            l2_gas_price,
            l2_gas_used,
            l1_gas_price,
            l1_gas_used,
        }
    }

    fn calculate_total_fee_in_mnt(&self) -> f64 {
        (self.l2_gas_price * self.l2_gas_used as f64 + self.l1_gas_price * self.l1_gas_used as f64)
            / 1_000_000_000.0
    }
}

fn random_f64_in_range(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}


fn get_input(prompt: &str) -> Result<String, io::Error> {
    let mut input = String::new();
    println!("{}", prompt);
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chain_selection: u8 = get_input("Select chain for gas calculation:\n1: Ethereum\n2: Mantle")
        .expect("Failed to read line")
        .parse()
        .expect("Please select a valid option");

    let conversion_response: CoinGeckoResponse = reqwest::get(
        "https://api.coingecko.com/api/v3/simple/price?ids=ethereum,mantle&vs_currencies=usd",
    )
    .await?
    .json()
    .await?;

    let eth_to_usd = conversion_response.ethereum.ok_or("Ethereum price not found")?.usd;
    let mnt_to_usd = conversion_response.mantle.ok_or("Mantle price not found")?.usd;

    let days: u64 = get_input("Enter the number of days for the simulation: ")
        .expect("Failed to read line")
        .parse()
        .expect("Please enter a valid number");

    let min_gas_price: f64 = get_input("Enter the minimum gas price (in Gwei): ")
        .expect("Failed to read line")
        .parse()
        .expect("Please enter a valid number");

    let max_gas_price: f64 = get_input("Enter the maximum gas price (in Gwei): ")
        .expect("Failed to read line")
        .parse()
        .expect("Please enter a valid number");

    let eth_gas_used: u64 = get_input("Enter the Ethereum gas used: ")
        .expect("Failed to read line")
        .parse()
        .expect("Please enter a valid number for Ethereum gas used");

    let mut total_eth_fee_consumption = 0.0;
    let mut total_usd_fee_consumption = 0.0;

    if chain_selection == 1 {
        for day in 0..days {
            let gas_price = random_f64_in_range(min_gas_price, max_gas_price);

            let eth_fee = gas_price * eth_gas_used as f64 / 1_000_000_000.0;
            let eth_fee_usd = eth_fee * eth_to_usd;
            
            total_eth_fee_consumption += eth_fee;
            total_usd_fee_consumption += eth_fee_usd;

            println!("Day {}: Total Transaction Fee: {:.18} ETH", day, eth_fee);
            println!("(~${:.2} USD)", eth_fee_usd);
        }
        println!("\nTotal ETH fees over {} days: {:.18} ETH", days, total_eth_fee_consumption);
        println!("Total USD equivalent: ${:.2}", total_usd_fee_consumption);
    } else if chain_selection == 2 {
        let l2_min_gas_price: f64 = get_input("Enter the minimum L2 gas price (in Gwei): ")
        .expect("Failed to read line")
        .parse()
        .expect("Please enter a valid number");

        let l2_max_gas_price: f64 = get_input("Enter the maximum L2 gas price (in Gwei): ")
        .expect("Failed to read line")
        .parse()
        .expect("Please enter a valid number");

        let l2_gas_used: u64 = get_input("Enter the L2 gas used: ")
        .expect("Failed to read line")
        .parse()
        .expect("Please enter a valid number for L2 gas used");

        for day in 0..days {
            let gas_price = random_f64_in_range(min_gas_price, max_gas_price);
            let l2_gas_price = random_f64_in_range(l2_min_gas_price, l2_max_gas_price);

            let fee_estimate =
                L2FeeEstimation::new(l2_gas_price, l2_gas_used, gas_price, eth_gas_used);
            let total_fee = fee_estimate.calculate_total_fee_in_mnt();
            let total_fee_usd = total_fee * mnt_to_usd;
            
            total_eth_fee_consumption += total_fee;
            total_usd_fee_consumption += total_fee_usd;

            println!("Day {}: Total Transaction Fee: {:.18} MNT", day, total_fee);
            println!("(~${:.2} USD)", total_fee_usd);
        }

        println!("\nTotal MNT fees over {} days: {:.18} MNT", days, total_eth_fee_consumption);
        println!("Total USD equivalent: ${:.2}", total_usd_fee_consumption);
    } else {
        println!("Invalid selection. Please run the program again.");
        return Ok(());
    }

    Ok(())
}
