//! Nested struct validation with path-aware error reporting.
//!
//! Run with: `cargo run --example nested`

use rusdantic::prelude::*;

#[derive(Rusdantic, Debug, Clone)]
struct Address {
    #[rusdantic(length(min = 1))]
    street: String,

    #[rusdantic(length(min = 1))]
    city: String,

    #[rusdantic(pattern(regex = r"^\d{5}(-\d{4})?$"))]
    zip_code: String,
}

#[derive(Rusdantic, Debug)]
#[rusdantic(rename_all = "camelCase")]
struct UserProfile {
    #[rusdantic(length(min = 2, max = 50))]
    full_name: String,

    #[rusdantic(email)]
    email_address: String,

    #[rusdantic(nested)]
    home_address: Address,

    #[rusdantic(nested)]
    shipping_addresses: Vec<Address>,

    #[rusdantic(range(min = 0))]
    loyalty_points: Option<i32>,
}

fn main() {
    // Deeply nested invalid data
    let json = r#"{
        "fullName": "A",
        "emailAddress": "bad-email",
        "homeAddress": {
            "street": "",
            "city": "Portland",
            "zip_code": "abc"
        },
        "shippingAddresses": [
            {"street": "123 Oak St", "city": "Seattle", "zip_code": "98101"},
            {"street": "", "city": "", "zip_code": "nope"}
        ],
        "loyaltyPoints": -5
    }"#;

    match rusdantic::from_json::<UserProfile>(json) {
        Ok(profile) => println!("Profile: {:?}", profile),
        Err(e) => println!("Validation errors:\n{}", e),
    }
}
