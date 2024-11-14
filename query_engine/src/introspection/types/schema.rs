#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum IntrospectionMode {
    /// Introspection only.
    IntrospectionOnly,
    /// Enables introspection.
    #[default]
    Enabled,
    /// Disables introspection.
    Disabled,
}
