use crate::ui::simple_area_manager::AreaId;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ActionType {
    NoneHover,
    AreaHover(AreaId, u16),
    NonePress,
    AreaPress(AreaId, u16),
    NoneRelease,
    AreaRelease(AreaId, u16),
    Grabbing(AreaId, u16),
    Typing(AreaId, u16),
}

#[derive(Debug)]
pub struct Events {
    pub mouse_actions: ActionType,
    pub scroll: AreaId,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            mouse_actions: ActionType::NoneHover,
            scroll: AreaId::default(),
        }
    }
}
