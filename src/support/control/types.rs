/// Represents the on/off state of a controller or device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwitchState {
    #[default]
    Off,
    On,
}
