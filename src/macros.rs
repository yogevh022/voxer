
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