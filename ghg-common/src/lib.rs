use std::ops::Deref;

pub fn assign_shared<T>(f: &std::rc::Rc<std::cell::RefCell<T>>, value: T) {
    *f.borrow_mut() = value;
}

pub fn read_shared<T: Copy>(f: &std::rc::Rc<std::cell::RefCell<T>>) -> T {
    *f.borrow().deref()
}

#[macro_export]
macro_rules! clone {
    ($i:ident) => (let $i = $i.clone();)
}

#[macro_export]
macro_rules! clone_all {
    ($($i:ident),+) => {
        $(clone!($i);)+
    }
}

#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + ghg_common::count!($($xs)*));
}
