
fn test_fn(mut x: i32, arr: &[i32]) -> i32 {
    return arr[x as usize];
}

fn test_fn2(mut x: i32, arr: &[i32]) -> i32 {
    assert!(x<0);
    return arr[x as usize];
}

fn test_fn3(mut x: i32, arr: &[i32]) -> i32 {
    assert!(x>=0);
    return arr[x as usize];
}

fn test_fn4(mut x: i32, mut y: i32){
    let arr = [1,2,3,4];
    let idx = x*y;
    println!("{}", arr[idx as usize]);
}


fn test_fn5(mut x: i32, mut y: i32){
    let arr = [1,2,3,4];
    if x <= 0 {
        if y <= 0 {
            let idx = x*y;
            println!("{}", arr[idx as usize]);
        }
    }
}

fn test_fn6(mut x: i32) -> i32 {
    let arr: Vec<i32> = (0..20).collect();
    while x < 0 {
        x += 4;
    }
    return arr[x as usize];
}

fn test_fn7(mut x: i32) -> i32 {
    let arr: Vec<i32> = (0..20).collect();
    let mut z = 2;
    let mut y = 2;
    while x < 0 {
        x += 1;
        y = z;
        z -= 1;
    }
    return arr[y as usize];
}
