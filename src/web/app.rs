use yew::prelude::*;
use crate::solver::TargetDate;
use super::picker::*;
use super::solver::*;

pub struct App {
    target: Option<TargetDate>
}

#[derive(Debug)]
pub enum AppMsg {
    TargetPicked(Option<TargetDate>)
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            target: None
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("got {:?}", msg);
        match msg {
            AppMsg::TargetPicked(target) => {
                self.target = target;
            }
        }

        true
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="app">
                <Picker on_picked={ctx.link().callback(AppMsg::TargetPicked)} />
                <SolverCmp target={self.target} />
            </div>
        }
    }
}

