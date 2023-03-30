use crate::*;

#[test] fn create_effect() {
    let ctx = Box::leak(Box::new(Runtime::new()));

    let effect_ran = ctx.create_signal(false);

    ctx.create_effect(Box::new(move || effect_ran.set(true)));

    assert!(effect_ran.get());
}

#[test] fn rerun_on_sub_update() {
    let ctx = Box::leak(Box::new(Runtime::new()));

    let sub = ctx.create_signal("Value isn't used");
    let times_run = ctx.create_signal(0);

    ctx.create_effect(Box::new(move || {
        sub.get();
        
        times_run.update(|c| c + 1);
    }));

    sub.set("Value changed!");

    assert_eq!(times_run.get(), &2);
}

#[test] fn unsubscribe() {
    use std::{rc::Rc, cell::RefCell};
    
    let ctx = Box::leak(Box::new(Runtime::new()));

    let sub = ctx.create_signal("Value isn't used");
    let times_run = Rc::new(RefCell::new(0));
    let times_run_cloned = times_run.clone();

    ctx.create_effect(Box::new(move || {
        if *times_run_cloned.as_ref().borrow() < 2 { sub.get(); }

        *times_run_cloned.borrow_mut() += 1;
    }));

    sub.set("Value changed!"); // Effect should rerun here
    sub.set("Value changed again!"); // Effect should also rerun here
    sub.set("Value changed for a third time!"); // Effect shouldn't rerun here

    assert_eq!(*times_run.borrow(), 3);
}
