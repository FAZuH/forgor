use crate::ui::traits::Updateable;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Page {
    Timer,
    Settings,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RouterMsg {
    Quit,
    Stay,
    GoTo(Page),
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RouterCmd {
    Quit,
}

impl From<Page> for RouterMsg {
    fn from(value: Page) -> Self {
        Self::GoTo(value)
    }
}

#[derive(Debug)]
pub struct Router {
    active_page: Option<Page>,
}

impl Router {
    pub fn new(page: Page) -> Self {
        Self {
            active_page: Some(page),
        }
    }

    pub fn quit(&mut self) {
        self.active_page = None
    }

    pub fn active_page(&self) -> Option<Page> {
        self.active_page
    }

    pub fn is_quit(&self) -> bool {
        self.active_page.is_none()
    }
}

impl Updateable<RouterMsg, RouterCmd> for Router {
    fn update(&mut self, msg: RouterMsg) -> Vec<RouterCmd> {
        let mut ret = vec![];
        match msg {
            RouterMsg::Quit => {
                self.active_page = None;
                ret.push(RouterCmd::Quit)
            }
            RouterMsg::Stay => {}
            RouterMsg::GoTo(page) => self.active_page = Some(page),
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit() {
        let mut router = Router::new(Page::Timer);
        router.quit();

        assert!(router.is_quit())
    }

    #[test]
    fn navigate() {
        let mut router = Router::new(Page::Timer);
        assert_eq!(router.active_page(), Some(Page::Timer));

        router.update(RouterMsg::GoTo(Page::Settings));
        assert_eq!(router.active_page(), Some(Page::Settings));
    }
}
