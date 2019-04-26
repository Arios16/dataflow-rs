fn main() {
    let y = test_fn(-2);
    println!("{}", y);
}

fn test_fn(x: i32) -> i32 {
    if x < 0 {
        -x
    } else {
        x
    }
}
