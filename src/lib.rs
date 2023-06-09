#[cfg(test)] mod tests;

use std::{
    cell::{RefCell, Cell},
    any::Any,
    marker::PhantomData,
    collections::HashSet
};

use slotmap::{SlotMap, new_key_type};

new_key_type! {
    struct SignalId;
    struct EffectId;
}

struct SignalValue {
    subscribers: HashSet<EffectId>,
    value: Box<dyn Any>
}

struct Effect {
    /// Set of all signals that this effect is subscribed to,
    /// exists so that the signal can efficiently unsubscribe
    /// from all signals before it is rerun
    subscriptions: HashSet<SignalId>,
    value: Box<dyn Fn()>
}

pub struct Runtime {
    signal_values: RefCell<SlotMap<SignalId, RefCell<SignalValue>>>,
    effects: RefCell<SlotMap<EffectId, RefCell<Effect>>>,
    running_effect: Cell<Option<EffectId>>
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            signal_values: RefCell::new(SlotMap::with_key()),
            effects: RefCell::new(SlotMap::with_key()),
            running_effect: Cell::new(None)
        }
    }

    pub fn create_signal<T: 'static>(
        &'static self, value: T
    ) -> Signal<T> {
        let id = self.signal_values.borrow_mut()
            .insert(RefCell::new(SignalValue {
                subscribers: HashSet::new(),
                value: Box::new(value)
            }));

        return Signal {
            ctx: self,
            id,
            phantom: PhantomData
        };
    }

    pub fn create_effect(&'static self, effect: Box<dyn Fn()>) {
        let id = self.effects.borrow_mut().insert(RefCell::new(Effect {
            subscriptions: HashSet::new(),
            value: effect
        }));
        self.run_effect(id);
    }

    fn run_effect(&'static self, id: EffectId) {
        self.running_effect.set(Some(id));

        // Unsubscribe from all current subscriptions
        for sub in &self.effects.borrow()[id].borrow().subscriptions {
            unsafe {
                (*self.signal_values.borrow()[*sub].as_ptr())
                    .subscribers.remove(&id);
            }
        }
        unsafe {
            (*self.effects.borrow()[id].as_ptr())
                .subscriptions.clear();
        }

        (self.effects.borrow()[id].borrow().value)();

        self.running_effect.set(None);
    }
}

#[derive(Clone, Copy)]
pub struct Signal<T: 'static> {
    ctx: &'static Runtime,
    id: SignalId,
    phantom: PhantomData<T>
}

impl<T: 'static> Signal<T> {
    pub fn get(&self) -> &T {
        if let Some(effect) = self.ctx.running_effect.get() {
            unsafe {
                (*self.ctx.signal_values.borrow()[self.id].as_ptr())
                    .subscribers.insert(effect);
                (*self.ctx.effects.borrow()[effect].as_ptr())
                    .subscriptions.insert(self.id);
            }
        }

        return unsafe {
            (*self.ctx.signal_values.borrow()[self.id].as_ptr())
                .value.as_ref()
                .downcast_ref()
                .unwrap()
        };
    }

    fn rerun_subs(&self) {
        for sub in &self.ctx.signal_values.borrow()[
            self.id
        ].borrow().subscribers {
            self.ctx.run_effect(*sub);
        }
    }

    pub fn set(&self, value: T) {
        unsafe {
            *((*self.ctx.signal_values.borrow()[self.id]
                .as_ptr()).value.downcast_ref::<T>().unwrap()
                as *const T as *mut T) = value;
        }

        self.rerun_subs();
    }

    //TODO: Change update semantics to pass-by-value
    pub fn update<F: FnOnce(T) -> T>(&self, f: F) {
        self.set(f(unsafe {
            std::ptr::read((*self.ctx.signal_values.borrow()[self.id]
                .as_ptr()).value.downcast_ref::<T>().unwrap()
                as *const T
            )
        }));

        self.rerun_subs();
    }

    pub fn mutate_with<F: FnOnce(&mut T)>(&self, f: F) {
        f(self.ctx.signal_values.borrow()[self.id]
            .borrow_mut().value.downcast_mut::<T>().unwrap()
        );

        self.rerun_subs();
    }
}
