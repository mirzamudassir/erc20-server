#[inline]
pub fn default_env(key: &str, value: &str) {
	if std::env::var(key).is_err() {
		std::env::set_var(key, value);
	}
}
