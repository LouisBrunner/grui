macro_rules! debug_error {
    ($pred:expr, $format:expr $(, $args:expr )*) => {
        let pred = $pred;
        debug_assert!(pred, $format $(, $args )*);
        if !pred {
          log::error!($format $(, $args )*);
        }
    };
}

pub(crate) use debug_error;
