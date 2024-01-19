use yew::prelude::*;

pub struct App {

}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self{}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                {"Day of Month"}
                <input type="range" min="1" max="31" value="1" />
            </div>
        }
    }
}