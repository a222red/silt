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
