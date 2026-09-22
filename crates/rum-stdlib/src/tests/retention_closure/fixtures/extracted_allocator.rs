use self::allocate as renamed;

pub fn root() {
    renamed();
}

fn allocate() {
    let _values = Vec::<usize>::new();
}
