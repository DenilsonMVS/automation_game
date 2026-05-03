use crate::ui::simple_area_manager::AreaId;


#[derive(Debug, PartialEq, Eq)]
pub enum ActionType {
    NoneHover,
    AreaHover(AreaId),
    NonePress,
    AreaPress(AreaId),
    NoneRelease,
    AreaRelease(AreaId),
    Grabbing(AreaId),
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
