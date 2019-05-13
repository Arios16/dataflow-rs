fn main() {
    let y = test_fn(-2);
    println!("{}", y);
}

// fn test_fn(x: i32) -> i32 {
//     if 0 < x {
//         if x < 5 {
//             return x;
//         }
//     }
//     return 0;
//     // if 0 < x && x < 5 {
//     //     x
//     // } else {
//     //     0
//     // }
// }

fn test_fn(mut x: i32) -> i32 {
    // let y = 1+2;
    // return y;
    assert!(x>0);
    return x;
}

fn black_box(x: i32) {}