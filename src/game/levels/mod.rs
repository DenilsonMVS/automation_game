use enum_dispatch::enum_dispatch;

use crate::{MainContext, game::{levels::{first::FirstLevel, main_menu::MainMenu}, scene::Scene}, graphics::renderer::Renderer};


pub mod main_menu;
pub mod first;

#[derive(Debug)]
pub enum SceneOptionId {
    MainMenu,
    First,
}

impl SceneOptionId {
    pub fn create_scene<'a>(&self, renderer: &'a Renderer) -> SceneOption<'a> {
        match self {
            Self::MainMenu => MainMenu::new(renderer).into(),
            Self::First => FirstLevel::new(renderer).into(),
        }
    }
}

#[enum_dispatch(Scene)]
pub enum SceneOption<'a> {
    MainMenu(MainMenu<'a>),
    First(FirstLevel<'a>),
}
