//! Simple tokenizer test

use html_parser::{Tokenizer, Token};

fn main() {
    // Test 1: Simple tag
    println!("=== Test 1: Simple tag ===");
    let html1 = "<html></html>";
    let mut tok1 = Tokenizer::new(html1);
    match tok1.tokenize() {
        Ok(tokens) => {
            for (i, t) in tokens.iter().enumerate() {
                println!("Token {}: {:?}", i, t);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
    
    // Test 2: With doctype  
    println!("\n=== Test 2: With doctype ===");
    let html2 = "<!doctype html><html></html>";
    let mut tok2 = Tokenizer::new(html2);
    match tok2.tokenize() {
        Ok(tokens) => {
            for (i, t) in tokens.iter().enumerate() {
                println!("Token {}: {:?}", i, t);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
    
    // Test 3: With doctype and newline
    println!("\n=== Test 3: With doctype and newline ===");
    let html3 = "<!doctype html>\n<html></html>";
    let mut tok3 = Tokenizer::new(html3);
    match tok3.tokenize() {
        Ok(tokens) => {
            for (i, t) in tokens.iter().enumerate() {
                println!("Token {}: {:?}", i, t);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
