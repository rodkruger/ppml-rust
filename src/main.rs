/*
use std::time::Instant;
use tfhe::prelude::*;
use tfhe::{ConfigBuilder, FheInt32, generate_keys, set_server_key};

const SCALING_FACTOR: i32 = 100;

fn homomorphic_taylor_tanh(x: &FheInt32) -> FheInt32 {
    let x2 = x * x;
    let x3 = &x2 * x;
    x3
}

fn main() {
    // Configura e gera chaves
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    let start = Instant::now();

    let mut x = -1.0;
    while x <= 1.0 {
        let plaintext = (x * SCALING_FACTOR as f64) as i32;
        let encrypted_x = FheInt32::encrypt(plaintext, &client_key);
        let encrypted_tanh = homomorphic_taylor_tanh(&encrypted_x);
        let decrypted_result: i32 = FheInt32::decrypt(&encrypted_tanh, &client_key);
        let result = decrypted_result as f64 / (SCALING_FACTOR.pow(3) as f64);
        println!("x = {:.2}, tanh ≈ {:.5}", x, result);

        x += 0.01;
    }

    println!("Time elapsed: {:?}", start.elapsed());
    // 2800 segundos
}
*/
use clap::{Arg, Command};
use std::ops::Add;
use std::time::Instant;
use tfhe::prelude::*;
use tfhe::shortint::parameters::PARAM_GPU_MULTI_BIT_MESSAGE_2_CARRY_2_GROUP_3_KS_PBS;
use tfhe::{CompressedServerKey, ConfigBuilder, FheInt32, generate_keys, set_server_key};

const SCALING_FACTOR: i32 = 100; // 2 decimal places
const LUT_INPUTS: [i32; 201] = [
    -100, -99, -98, -97, -96, -95, -94, -93, -92, -91, -90, -89, -88, -87, -86, -85, -84, -83, -82,
    -81, -80, -79, -78, -77, -76, -75, -74, -73, -72, -71, -70, -69, -68, -67, -66, -65, -64, -63,
    -62, -61, -60, -59, -58, -57, -56, -55, -54, -53, -52, -51, -50, -49, -48, -47, -46, -45, -44,
    -43, -42, -41, -40, -39, -38, -37, -36, -35, -34, -33, -32, -31, -30, -29, -28, -27, -26, -25,
    -24, -23, -22, -21, -20, -19, -18, -17, -16, -15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5,
    -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
    22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
    46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69,
    70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
    94, 95, 96, 97, 98, 99, 100,
];

const LUT_OUTPUTS: [i32; 201] = [
    -76, -75, -75, -74, -74, -73, -73, -73, -72, -72, -71, -71, -70, -70, -69, -69, -68, -68, -67,
    -66, -66, -65, -65, -64, -64, -63, -62, -62, -61, -61, -60, -59, -59, -58, -57, -57, -56, -55,
    -55, -54, -53, -52, -52, -51, -50, -50, -49, -48, -47, -46, -46, -45, -44, -43, -43, -42, -41,
    -40, -39, -38, -37, -37, -36, -35, -34, -33, -32, -31, -30, -30, -29, -28, -27, -26, -25, -24,
    -23, -22, -21, -20, -19, -18, -17, -16, -15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5, -4,
    -3, -2, -1, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
    22, 23, 24, 25, 26, 27, 28, 29, 30, 30, 31, 32, 33, 34, 35, 36, 37, 37, 38, 39, 40, 41, 42, 43,
    43, 44, 45, 46, 46, 47, 48, 49, 50, 50, 51, 52, 52, 53, 54, 55, 55, 56, 57, 57, 58, 59, 59, 60,
    61, 61, 62, 62, 63, 64, 64, 65, 65, 66, 66, 67, 68, 68, 69, 69, 70, 70, 71, 71, 72, 72, 73, 73,
    73, 74, 74, 75, 75, 76,
];

fn homomorphic_lut_tanh(
    x: &FheInt32,
    lut_inputs: &[FheInt32],
    lut_outputs: &[FheInt32],
) -> FheInt32 {
    let mut result = FheInt32::encrypt_trivial(0);

    for (i, input_value) in lut_inputs.iter().enumerate() {
        let is_equal = x.eq(input_value);
        let selected_output = is_equal.if_then_else(&lut_outputs[i], &FheInt32::encrypt_trivial(0));
        result = result.add(&selected_output);
    }

    result
}

fn main() {
    // Parse command line args
    let matches = Command::new("tfhe_app")
        .about("TFHE CPU/GPU toggle example")
        .arg(
            Arg::new("gpu")
                .long("gpu")
                .help("Use GPU acceleration")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let use_gpu = matches.get_flag("gpu");
    let client_key;

    if use_gpu {
        println!("Running with GPU acceleration ...");
        let config = ConfigBuilder::with_custom_parameters(
            PARAM_GPU_MULTI_BIT_MESSAGE_2_CARRY_2_GROUP_3_KS_PBS,
        )
            .build();
        let (_client_key, _) = generate_keys(config);
        client_key = _client_key;
        let compressed_server_key = CompressedServerKey::new(&client_key);
        let gpu_key = compressed_server_key.decompress_to_gpu();
        set_server_key(gpu_key);
    } else {
        println!("Running in CPU mode ...");
        let config = ConfigBuilder::default().build();
        let (_client_key, server_key) = generate_keys(config);
        client_key = _client_key;
        set_server_key(server_key);
    }

    let start = Instant::now();

    let lut_inputs_encrypted: Vec<FheInt32> = LUT_INPUTS
        .iter()
        .map(|&x| FheInt32::encrypt_trivial(x))
        .collect();

    let lut_outputs_encrypted: Vec<FheInt32> = LUT_OUTPUTS
        .iter()
        .map(|&y| FheInt32::encrypt_trivial(y))
        .collect();

    // 3. Test over interval [-1.0, 1.0]
    let mut x = -1.0;
    while x <= 1.0 {
        let plaintext = (x * SCALING_FACTOR as f64).round() as i32;
        let encrypted_x = FheInt32::encrypt(plaintext, &client_key);

        let encrypted_tanh =
            homomorphic_lut_tanh(&encrypted_x, &lut_inputs_encrypted, &lut_outputs_encrypted);

        let decrypted_result: i32 = FheInt32::decrypt(&encrypted_tanh, &client_key);
        let result = decrypted_result as f64 / SCALING_FACTOR as f64;

        println!("{}", result);

        x += 0.01;
    }

    println!("Time elapsed: {:?}", start.elapsed());
    // Time elapsed: 16858.127381438s
}

/*
use std::time::Instant;
use tfhe::prelude::*;
use tfhe::{ConfigBuilder, FheInt32, generate_keys, set_server_key, CompressedServerKey};
use tfhe::shortint::parameters::PARAM_GPU_MULTI_BIT_MESSAGE_2_CARRY_2_GROUP_3_KS_PBS;

// 1. Configure and generate keys with GPU parameters
let config = ConfigBuilder::with_custom_parameters(PARAM_GPU_MULTI_BIT_MESSAGE_2_CARRY_2_GROUP_3_KS_PBS, None).build();
let (client_key, server_key) = generate_keys(config);
let compressed_server_key = CompressedServerKey::new(&client_key);
let gpu_key = compressed_server_key.decompress_to_gpu();
set_server_key(gpu_key);
 */
