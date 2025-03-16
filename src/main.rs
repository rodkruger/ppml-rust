use std::time::Instant;
use tfhe::prelude::{FheDecrypt, FheEncrypt};
use tfhe::{ConfigBuilder, FheInt32, generate_keys, set_server_key};

const SCALING_FACTOR: i32 = 10000;

/// Homomorphic computation of tanh(x) using Taylor series approximation
fn homomorphic_taylor_tanh(x: &FheInt32) -> FheInt32 {
    let x2 = x * x / SCALING_FACTOR; // x² (downscale to prevent overflow)
    let x3 = &x2 * x / SCALING_FACTOR; // x³
    let x5 = &x3 * &x2 / SCALING_FACTOR; // x⁵

    let term1 = &x3 / 3i32; // x³ / 3
    let term2 = &x5 / 7i32; // x⁵ / 7.5 (approximated as 7)

    x - term1 + term2
}

fn main() {
    // 1. Configure and generate keys
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    let start = Instant::now(); // Start timer

    let mut x = -1.0;
    while x <= 1.0 {
        let plaintext = (x * SCALING_FACTOR as f64) as i32;
        let encrypted_x = FheInt32::encrypt(plaintext, &client_key);
        let encrypted_tanh = homomorphic_taylor_tanh(&encrypted_x);
        let decrypted_result: i32 = FheInt32::decrypt(&encrypted_tanh, &client_key);
        let result = decrypted_result as f64 / SCALING_FACTOR as f64;
        println!("{}", result);

        x += 0.01;
    }

    println!("Time elapsed: {:?}", start.elapsed());
    // 2800 seconds
}
