pub trait PluginComponent {
    fn is_initialized(&self) -> bool;
    fn update(&mut self);
}
