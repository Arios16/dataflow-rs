fn main() {
    println!("Hello, World!");
}

fn test_fn(x: i32) -> i32 {
    if 0 <= x && x < 10 {
        return x;
    } else {
        return 0;
    }
}
