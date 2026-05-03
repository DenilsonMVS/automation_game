
#[derive(Debug)]
pub enum ActionType {
    NoneHover,
    AreaHover(u32),
    AreaPress(u32),
    NonePress,
    AreaRelease(u32),
    NoneRelease,
}

#[derive(Debug)]
pub struct Events {
    pub mouse_actions: ActionType,
}

impl Default for Events {
    fn default() -> Self {
        Self { mouse_actions: ActionType::NoneHover }
    }
}
