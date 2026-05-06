use crate::ui::traits::Updateable;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Page {
    Timer,
    Settings,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RouterMsg {
    Stay,
    GoTo(Page),
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RouterCmd {
    None,
}

impl From<Page> for RouterMsg {
    fn from(value: Page) -> Self {
        Self::GoTo(value)
    }
}

#[derive(Debug)]
pub struct Router {
    active_page: Page,
}

impl Router {
    pub fn new(page: Page) -> Self {
        Self { active_page: page }
    }

    pub fn active_page(&self) -> Page {
        self.active_page
    }
}

impl Updateable<RouterMsg, RouterCmd> for Router {
    fn update(&mut self, msg: RouterMsg) -> Vec<RouterCmd> {
        let ret = vec![];
        match msg {
            RouterMsg::Stay => {}
            RouterMsg::GoTo(page) => self.active_page = page,
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate() {
        let mut router = Router::new(Page::Timer);
        assert_eq!(router.active_page(), Page::Timer);

        router.update(RouterMsg::GoTo(Page::Settings));
        assert_eq!(router.active_page(), Page::Settings);
    }
}
