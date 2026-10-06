use std::io;

fn main() {
    // Read the first number
    let mut input1 = String::new();
    io::stdin()
        .read_line(&mut input1)
        .expect("Failed to read line");
    let num1: f64 = input1
        .trim()
        .parse()
        .expect("Please type a valid number");

    // Read the second number
    let mut input2 = String::new();
    io::stdin()
        .read_line(&mut input2)
        .expect("Failed to read line");
    let num2: f64 = input2
        .trim()
        .parse()
        .expect("Please type a valid number");

    // Print the sum and product, each on a new line
    println!("{}", num1 + num2);
    println!("{}", num1 * num2);
}