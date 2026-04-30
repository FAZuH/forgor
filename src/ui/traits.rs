use crate::ui::UiError;

pub trait Runner {
    fn run(&mut self) -> Result<(), UiError>;
}

pub trait Updateable {
    type Msg;
    type Cmd;
    fn update(&mut self, msg: Self::Msg) -> Self::Cmd;
}

pub trait View<C> {
    type Result;
    fn render(&self, canvas: C) -> Self::Result;

    fn render_mut(&mut self, canvas: C) -> Self::Result {
        self.render(canvas)
    }
}

pub trait StatefulView<C> {
    type State;
    type Result;
    fn render_stateful(&self, canvas: C, state: &mut Self::State) -> Self::Result;

    fn render_stateful_mut(&mut self, canvas: C, state: &mut Self::State) -> Self::Result {
        self.render_stateful(canvas, state)
    }
}

impl<T, C> StatefulView<C> for T
where
    T: View<C>,
{
    type State = ();
    type Result = <T as View<C>>::Result;

    fn render_stateful(&self, canvas: C, _state: &mut Self::State) -> Self::Result {
        self.render(canvas)
    }
}
