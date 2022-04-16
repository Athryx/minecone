#[macro_export]
macro_rules! array3d_init {
	($value:expr) => {
		array_init::array_init(|_| array_init::array_init(|_| array_init::array_init(|_| $value)))
	};
}
