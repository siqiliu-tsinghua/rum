/// A declared bound on native stack consumption during destruction.
///
/// Implementing this safe, opt-in marker promises that destroying the value and
/// any state it can reach uses bounded native stack space, independent of the
/// depth of reachable ownership chains. The promise covers the value's own
/// `Drop`, compiler-generated field destruction, captured values, and destruction
/// by the last strong holder of shared ownership.
///
/// This declaration imposes no time bound, makes no allocator guarantee, and
/// does not promise that foreign code cannot panic. It adds no threading bounds.
///
/// The compiler enforces only that the implementor has taken a stance, never
/// that the stance is true. No `unsafe` code may ever rely on this declaration.
/// `rum-core` and `rum-runtime` contain no `unsafe`; this marker
/// must not become a reason to introduce any.
///
/// Opt in with an explicit empty implementation and a rationale describing why
/// destruction is bounded. There is no blanket or automatic implementation;
/// frame tracing, frame abandonment, and ownership inventory membership do not
/// imply this declaration.
pub trait BoundedDestruction {}
