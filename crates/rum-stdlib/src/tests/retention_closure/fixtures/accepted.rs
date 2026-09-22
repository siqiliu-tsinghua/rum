/// Fixture protocol only. These operations are NOT production adapters.
/// Each exact operation has an audited contract in inventory/leaves.tsv.
pub mod substrate {
    pub struct Raw {
        bytes: Vec<u8>,
    }

    pub struct Retained {
        bytes: Vec<u8>,
        charged_capacity: usize,
    }

    struct Node {
        value: Retained,
        next: Option<Box<Node>>,
    }

    pub struct Accumulator {
        head: Option<Box<Node>>,
        charged_shells: usize,
    }

    pub struct Outgoing {
        owner: Accumulator,
    }

    pub fn start() -> Accumulator {
        Accumulator {
            head: None,
            charged_shells: 0,
        }
    }

    // The raw value is current callback W, never a retained slot. Callback
    // private allocation is explicitly outside the fixture boundary contract.
    pub fn callback(n: usize) -> Raw {
        Raw { bytes: vec![0; n] }
    }

    pub fn adopt(raw: Raw) -> Retained {
        let charged_capacity = raw.bytes.capacity();
        Retained {
            bytes: raw.bytes,
            charged_capacity,
        }
    }

    // Receipt is established before allocation; moving a Retained does not
    // copy its payload or discharge its receipt. No user Drop/Hash/Eq runs.
    pub fn append(mut acc: Accumulator, value: Retained) -> Accumulator {
        acc.charged_shells = acc
            .charged_shells
            .checked_add(std::mem::size_of::<Node>())
            .expect("fixture receipt overflow before allocation");
        acc.head = Some(Box::new(Node {
            value,
            next: acc.head,
        }));
        acc
    }

    pub fn send(owner: Accumulator) -> Outgoing {
        Outgoing { owner }
    }
}

pub fn next(value: usize) -> usize {
    value + 1usize
}

/// retention-region:demo
pub fn root(rounds: usize) -> substrate::Outgoing {
    let mut acc = substrate::start();
    let mut i = 0usize;
    while i < rounds {
        let value = substrate::adopt(substrate::callback(i));
        acc = substrate::append(acc, value);
        i = next(i);
    }
    substrate::send(acc)
}
