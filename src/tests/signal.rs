use crate::*;

#[test] fn get() {
    let ctx = Box::leak(Box::new(Runtime::new()));

    let s = ctx.create_signal(42);

    assert_eq!(s.get(), &42);
}

#[test] fn set() {
    let ctx = Box::leak(Box::new(Runtime::new()));

    let s = ctx.create_signal("foo");

    s.set("bar");

    assert_eq!(s.get(), &"bar");
}

#[test] fn update() {
    let ctx = Box::leak(Box::new(Runtime::new()));

    let s = ctx.create_signal(3);

    let cube = |x| x * x * x;

    s.update(cube);

    assert_eq!(s.get(), &27)
}

#[test] fn mutate_with() {
    let ctx = Box::leak(Box::new(Runtime::new()));

    let s = ctx.create_signal(6);

    let b = Box::leak(Box::new(7));

    s.mutate_with(|v| std::mem::swap(v, b));

    assert_eq!(s.get(), &7);
    assert_eq!(*b, 6);
}
