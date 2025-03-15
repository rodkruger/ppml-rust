use tfhe::prelude::{FheDecrypt, FheEncrypt};
use tfhe::{ConfigBuilder, FheUint32, generate_keys, set_server_key};

/// Homomorphic computation of tanh(x) using Taylor series approximation
fn homomorphic_taylor_tanh(x: &FheUint32, scaling_factor: u32) -> FheUint32 {
    let x2 = x * x / scaling_factor;     // x² (downscale to prevent overflow)
    let x3 = &x2 * x / scaling_factor;   // x³
    let x5 = &x3 * &x2 / scaling_factor; // x⁵

    let term1 = &x3 / 3u32;  // x³ / 3
    let term2 = &x5 / 7u32;  // x⁵ / 7.5 (approximated as 7)

    x - term1 + term2
}

fn main() {
    // 1️⃣ Configure and generate keys
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    // 2️⃣ Encrypt an input value (Fixed-point scaling)
    let scaling_factor = 10000;  // **Reduced scaling for better precision**
    let plaintext: u32 = (0.8 * scaling_factor as f64) as u32;
    let encrypted_x = FheUint32::encrypt(plaintext, &client_key);

    // 3️⃣ Compute tanh homomorphically with correct scaling
    let encrypted_tanh = homomorphic_taylor_tanh(&encrypted_x, scaling_factor);

    // 4️⃣ Decrypt and rescale result
    let decrypted_result: u32 = FheUint32::decrypt(&encrypted_tanh, &client_key);
    let result = decrypted_result as f64 / scaling_factor as f64;
    println!("Encrypted tanh approximation: {}", result);
}
