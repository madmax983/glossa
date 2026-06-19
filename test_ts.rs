use quote::quote;

fn main() {
    let mut ts = quote! { 1 };
    for _ in 0..50000 {
        ts = quote! { #ts + 1 };
    }
    println!("Built!");
    let _s = ts.to_string();
    println!("Stringified!");
}
