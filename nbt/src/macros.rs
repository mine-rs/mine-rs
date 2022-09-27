#[macro_export]
macro_rules! nbt {
    ({ $($key:tt : $value:tt),* $(,)? }) => {
        $crate::Compound::new({
            #[allow(unused_mut)]
            let mut map = ::std::collections::BTreeMap::<::std::borrow::Cow<str>, $crate::Value>::new();
            $(map.insert($key.into(), nbt!(@value $value));)*
            map
        })
    };
    (@value $ident:ident) => { $ident };
    (@value $lit:literal) => { $lit.into() };
    (@value $other:tt) => { nbt!($other).into() };
    ([L; $($lit:literal),* $(,)?]) => { nbt!([Long; $($lit),*]) };
    ([Long; $($lit:literal),* $(,)?]) => {
        $crate::Value::LongArray(vec![$($lit),*])
    };
    ([I; $($lit:literal),* $(,)?]) => { nbt!([Int; $($lit),*]) };
    ([Int; $($lit:literal),* $(,)?]) => {
        $crate::Value::IntArray(vec![$($lit),*])
    };
}
