use num_bigint::BigUint;
use std::{
    cmp::Ordering,
    io::{self, Write},
    time::Instant,
};

fn main() {
    loop {
        // Prompt the user for a Fibonacci number index
        print!("Enter Fibonacci number index (or 'q' to quit): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();
        if input.eq_ignore_ascii_case("q") {
            break;
        }

        let input_value = match input.parse::<u64>() {
            Ok(num) => num,
            Err(_) => {
                println!("Please enter a valid number");
                continue;
            }
        };

        // Calculate the Fibonacci number and save the duration of the calculation
        let start_time = Instant::now();
        let calc_result = calculate_fibonacci(input_value);
        let duration = format_duration(start_time.elapsed().as_secs_f64());

        match calc_result {
            Ok(fibonacci_result) => {
                println!(
                    "\nCalculated the {}th Fibonacci number",
                    thousands_separator(input_value)
                );
                println!("Fibonacci calculation duration: {}", duration);

                // Start time of the conversion duration
                let conversion_start_time = Instant::now();

                // Use scientific notation when the result is larger than 10^35
                let use_scientific_notation = fibonacci_result > BigUint::from(10u32).pow(35);

                // Convert the result based on the use_scientific_notation boolean
                let result = if use_scientific_notation {
                    scientific_notation(&fibonacci_result)
                } else {
                    fibonacci_result.to_string()
                };
                // Save the duration of the conversion
                let conversion_duration =
                    format_duration(conversion_start_time.elapsed().as_secs_f64());

                if use_scientific_notation {
                    println!(
                        "Result to Scientific notation duration: {}",
                        conversion_duration
                    );
                } else {
                    println!("Result to String duration: {}", conversion_duration);
                }

                println!("Result:\n{}", result);
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
        println!("\n");
    }
}

/// Calculates the nth Fibonacci number using a parallel computation approach.
///
/// This function takes a `u64` value `n` as input and returns the nth Fibonacci number
/// as a `BigUint` result. It uses a recursive helper function `fib_pair` to perform
/// the Fibonacci calculation in a parallel manner for large numbers.
///
/// # Arguments
/// * `n` - The index of the Fibonacci number to calculate.
///
/// # Returns
/// A `Result<BigUint, String>` where the `BigUint` represents the nth Fibonacci number,
/// or a `String` error message if the calculation fails.
fn calculate_fibonacci(n: u64) -> Result<BigUint, String> {
    if n == 0 {
        return Ok(BigUint::ZERO);
    }

    fn fib_pair(n: u64) -> (BigUint, BigUint) {
        if n == 0 {
            return (BigUint::ZERO, BigUint::from(1u32));
        }

        let (a, b) = fib_pair(n >> 1);
        let two = BigUint::from(2u32);

        // Execute the Fibonacci pair calculation in parallel
        let (c, d) = rayon::join(|| &a * (&b * &two - &a), || &a * &a + &b * &b);

        // Determine the result based on if n is even or odd
        let result = if n & 1 == 0 {
            (c, d)
        } else {
            let sum = &c + &d;
            (d, sum)
        };

        result
    }

    let (result, _) = fib_pair(n);
    Ok(result)
}

/// Converts a `BigUint` number to a string representation in scientific notation.
///
/// This function takes a `BigUint` number as input and returns a string representation
/// of the number in scientific notation format. The function ensures that the output
/// string has a fixed number of significant digits (5 by default) and adjusts the
/// exponent accordingly.
///
/// # Arguments
/// * `number` - The `BigUint` number to be converted to scientific notation.
///
/// # Returns
/// A `String` representing the input `BigUint` number in scientific notation format.
fn scientific_notation(number: &BigUint) -> String {
    let first_digits_count = 5 as u8;

    // Handle zero case
    if number == &BigUint::new(vec![]) {
        return "0.0e0".to_string();
    }
    // Determine the total number of digits using an approximation
    let bits = number.bits() as u64;
    let mut total_digits = ((bits as f64) * 0.30102999566398114) as u32; // Calculate the total number of digits using the approximation of log10(2) ≈ 0.30102999566398114

    // Correct the total digits approximation to get to the correct number of digits
    let base = BigUint::from(10u32);
    let mut power = base.pow(total_digits / 2).pow(2)
        * if total_digits % 2 == 1 {
            BigUint::from(10u32)
        } else {
            BigUint::from(1u32)
        };

    match number.cmp(&power) {
        Ordering::Less => {
            total_digits -= 1;
            power /= 10u32;
        }
        Ordering::Greater if &power * 10u32 <= *number => {
            total_digits += 1;
            power *= 10u32;
        }
        _ => {}
    }

    // Extract the first digits from the number based on the first_digits_count
    let shift = total_digits - (first_digits_count - 1) as u32;
    let divisor = base.pow(shift);
    let mut first_digits = number / divisor;

    // Correct the first digits if they exceed the upper or lower bounds
    let upper_bound = BigUint::from(10u32).pow(first_digits_count as u32);
    let lower_bound = BigUint::from(10u32).pow((first_digits_count - 1) as u32);

    while first_digits >= upper_bound {
        first_digits /= 10u32;
        total_digits += 1;
    }
    while first_digits < lower_bound {
        first_digits *= 10u32;
        total_digits -= 1;
    }
    // Format the result to scientific notation
    let divider = 10u32.pow(first_digits_count as u32 - 1);
    let integer_part = &first_digits / divider;
    let decimals_part = &first_digits % divider;
    let result = format!(
        "{}.{:04}e+{}",
        integer_part,
        decimals_part,
        thousands_separator(total_digits as u64)
    );

    result
}

/// Formats a duration value as a human-readable string.
///
/// This function takes a duration value in seconds and formats it as a string
/// with the appropriate time unit (microseconds, milliseconds, or seconds).
/// The function will choose the most appropriate unit based on the magnitude
/// of the duration value.
///
/// # Arguments
/// * `duration` - The duration value in seconds to be formatted.
///
/// # Returns
/// A `String` representing the input duration value in a human-readable format.
fn format_duration(duration: f64) -> String {
    if duration < 1e-3 {
        format!("{}μs", (duration * 1e6).round() as u16)
    } else if duration < 1.0 {
        format!("{}ms", (duration * 1e3).round() as u16)
    } else {
        format!("{:.3}s", duration)
    }
}

/// Formats a number with a thousands separator.
///
/// This function takes a `u32` number and returns a `String` representation of the number with a thousands separator (`,`) inserted every three digits.
///
/// # Arguments
/// * `number` - The number to be formatted with a thousands separator.
///
/// # Returns
/// A `String` representing the input number with a thousands separator.
fn thousands_separator(number: u64) -> String {
    number
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}
