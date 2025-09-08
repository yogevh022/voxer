#[macro_export]
macro_rules! avg {
    // average value by unique identifier
    ($val:expr => $identifier:expr) => {{
        use std::cell::RefCell;
        use std::collections::HashMap;

        thread_local! {
            static AVG: RefCell<HashMap<&'static str, (f64, usize)>> = RefCell::new(HashMap::new());
        }

        let i: &'static str = $identifier;
        let v = $val as f64;
        AVG.with(|avg| {
            let mut avg_map = avg.borrow_mut();
            let mut entry = avg_map.remove(i).unwrap_or_default();
            entry.0 += v;
            entry.1 += 1;
            avg_map.insert(i, entry);
            entry.0 / entry.1 as f64
        })
    }};
}

#[macro_export]
macro_rules! call_every {
    ($name:ident, $interval:expr, $func:expr) => {
        thread_local! {
            static $name: std::cell::RefCell<usize> = std::cell::RefCell::new(0);
        }

        $name.with(|counter| {
            let mut c = counter.borrow_mut();
            *c += 1;
            if *c % $interval == 0 {
                $func();
            }
        });
    };
}

#[macro_export]
macro_rules! const_labels {
    ($label:literal, $count:expr) => {
        {
            let mut result: [&'static str; $count] = [""; $count];
            let mut i = 0usize;
            while i < $count {
                result[i] = match i {   // :)
                    0 => concat!($label, "_0"),
                    1 => concat!($label, "_1"),
                    2 => concat!($label, "_2"),
                    3 => concat!($label, "_3"),
                    4 => concat!($label, "_4"),
                    5 => concat!($label, "_5"),
                    6 => concat!($label, "_6"),
                    7 => concat!($label, "_7"),
                    8 => concat!($label, "_8"),
                    9 => concat!($label, "_9"),
                    10 => concat!($label, "_10"),
                    11 => concat!($label, "_11"),
                    12 => concat!($label, "_12"),
                    13 => concat!($label, "_13"),
                    14 => concat!($label, "_14"),
                    15 => concat!($label, "_15"),
                    16 => concat!($label, "_16"),
                    _ => concat!($label, "_N"),
                };
                i += 1;
            }
            result
        }
    };
}

#[macro_export]
macro_rules! impl_try_from_uint {
    ($uint:ty => $enum_type:ty) => {
        impl TryFrom<$uint> for $enum_type {
            type Error = ();

            fn try_from(value: $uint) -> Result<Self, Self::Error> {
                if value < Self::__Count as $uint {
                    Ok(unsafe { std::mem::transmute(value) })
                } else {
                    Err(())
                }
            }
        }
    };
}