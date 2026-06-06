#[macro_export]
macro_rules! mat {
    (
        $(
            [ $($elem:expr),+ $(,)? ]
        ),+ $(,)?
    ) => {
        $crate::math::Matrix {
            inner: [
                $(
                    [ $($elem),+ ],
                )+
            ]
        }
    };
}

#[macro_export]
macro_rules! max {
    ($first:expr, $second:expr $(, $i:expr),*) => {
        {
            let mut curr_max = $first;
            if $second > curr_max {
                curr_max = $second;
            }

            $(
                if $i > curr_max {
                    curr_max = $i;
                }
            )*

            curr_max
        }
    };
}

#[macro_export]
macro_rules! min {
    ($first: expr, $second: expr $(, $i: expr),*) => {
        {
            let mut curr_min = $first;
            if $second < curr_min {
                curr_min = $second;
            }

            $(
                if $i < curr_min {
                    curr_min = $i;
                }
            )*

            curr_min
        }
    };
}
