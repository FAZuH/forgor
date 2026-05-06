use crate::ui::traits::Updateable;

/// Top-level views available in the application.
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

/// Manages the active top-level view of the application.
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

    #[test]
    fn from_page_to_msg() {
        assert_eq!(RouterMsg::from(Page::Timer), RouterMsg::GoTo(Page::Timer));
        assert_eq!(
            RouterMsg::from(Page::Settings),
            RouterMsg::GoTo(Page::Settings)
        );
    }

    #[test]
    fn update_stay_noop() {
        let mut router = Router::new(Page::Timer);
        let cmds = router.update(RouterMsg::Stay);

        assert_eq!(router.active_page(), Page::Timer);
        assert!(cmds.is_empty());
    }

    #[test]
    fn update_go_to_returns_empty() {
        let mut router = Router::new(Page::Timer);
        let cmds = router.update(RouterMsg::GoTo(Page::Settings));

        assert!(cmds.is_empty());
    }
}
