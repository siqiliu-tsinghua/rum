pub struct Left {
    pub value: usize,
}
pub struct Right {
    pub value: usize,
}
impl Left {
    pub fn choose(&self) -> usize {
        self.value
    }
    pub fn consume(self) -> usize {
        self.value
    }
    pub fn same(self) -> Self {
        self
    }
}
impl Right {
    pub fn choose(&self) -> usize {
        self.value
    }
}
pub trait Collision {
    fn choose(&self) -> usize;
}
impl Collision for Left {
    fn choose(&self) -> usize {
        self.value
    }
}
pub fn collision<T: Collision>(value: &T) -> usize {
    value.choose()
}
pub fn nominal(value: &Left, other: &Right) -> (usize, usize) {
    (value.choose(), other.choose())
}
pub fn consuming(value: Left) -> usize {
    value.same().consume()
}
pub fn nested(value: &&Left) -> usize {
    value.choose()
}
pub struct Holder {
    pub left: Left,
}
pub fn projected(value: &Holder) -> usize {
    value.left.choose()
}
pub fn annotated(value: &Left) -> usize {
    let exact: &Left = value;
    exact.choose()
}
pub fn patterned(value: Option<&Left>) -> usize {
    match value {
        Some(left) => left.choose(),
        None => 0usize,
    }
}
pub struct Earlier {
    pub value: usize,
}
pub trait First {
    fn select(&self) -> usize;
}
impl First for Earlier {
    fn select(&self) -> usize {
        self.value
    }
}
impl Earlier {
    pub fn select(&mut self) -> usize {
        self.value
    }
}
#[allow(unused_mut)]
pub fn trait_wins(mut value: Earlier) -> usize {
    value.select()
}
pub fn mutable(value: &mut Earlier) -> usize {
    value.select()
}
pub struct Indirect {
    pub left: Left,
}
impl std::ops::Deref for Indirect {
    type Target = Left;
    fn deref(&self) -> &Left {
        &self.left
    }
}
pub fn user_deref(value: &Indirect) -> usize {
    value.choose()
}
pub struct Allocator;
impl Allocator {
    pub fn allocate(&self) {
        allocating_helper();
    }
}
pub fn allocating_helper() {
    let _payload: Vec<u8> = Vec::<u8>::new();
}
pub fn allocation(value: &Allocator) {
    value.allocate();
}
pub struct Generic<T> {
    pub value: T,
}
impl<T> Generic<T> {
    pub fn extract(self) -> T {
        self.value
    }
    pub fn untouched(&self) -> bool {
        true
    }
}
pub fn instances(a: Generic<u8>, b: Generic<usize>) -> (u8, usize) {
    (a.extract(), b.extract())
}
pub fn open<T>(value: Generic<T>) -> T {
    value.extract()
}
pub fn growing(value: Generic<Generic<u8>>) -> Generic<u8> {
    value.extract()
}

impl Left {
    pub fn recurse(&self, n: usize) -> usize {
        if n == 0usize {
            self.value
        } else {
            self.recurse(n - 1usize)
        }
    }
}
pub fn recursive(value: &Left) -> usize {
    value.recurse(2usize)
}

pub struct TraitOnly {
    pub value: usize,
}
impl Collision for TraitOnly {
    fn choose(&self) -> usize {
        self.value
    }
}
pub fn collision_control(value: &TraitOnly) -> usize {
    collision(value)
}
impl Left {
    pub fn echo<T>(&self, value: T) -> T {
        value
    }
}
pub fn method_instances(value: &Left) -> (u8, usize) {
    (value.echo::<u8>(1u8), value.echo::<usize>(1usize))
}

impl Left {
    pub async fn deferred(&self) -> usize {
        self.value
    }
}
pub fn future_receiver(value: &Left) -> impl std::future::Future<Output = usize> + '_ {
    value.deferred()
}
pub fn array_receiver(value: &[Left; 1]) -> usize {
    value.len()
}
