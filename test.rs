fn  test_fn(_1: i32) -> i32 {
    let mut _0: i32; let mut _2: bool; let mut _3: bool;
    let mut _4: i32; let mut _5: bool; let mut _6: i32;
    bb0: {
        _4 = _1;
        _3 = Le(const 0i32, move _4);
        switchInt(move _3) -> [false: bb1, otherwise: bb2];
    }
    bb1: {
        _2 = const false;
        goto -> bb3;
    }
    bb2: {
        _6 = _1;
        _5 = Lt(move _6, const 10i32);
        _2 = move _5;
        goto -> bb3;
    }
    bb3: {
        switchInt(move _2) -> [false: bb5, otherwise: bb4];
    }
    bb4: {
        _0 = _1;
        goto -> bb6;
    }
    bb5: {
        _0 = const 0i32;
        goto -> bb6;
    }
    bb6: {
        return;                         
    }
}
