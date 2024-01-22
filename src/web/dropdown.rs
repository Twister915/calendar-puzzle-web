use yew::prelude::*;
use std::fmt;
use yew::html::Scope;

#[derive(Clone, PartialEq, Properties)]
pub struct DropdownProps<P: PartialEq + Clone + 'static> {
    pub values: Vec<P>,
    pub placeholder: String,
    pub on_change: Callback<Option<P>>,
    pub disabled: bool,
}

pub struct Dropdown<P> {
    values: Vec<DropdownValue<P>>,
    placeholder: String,
    on_change: Callback<Option<P>>,

    picked: Option<usize>,
    user_input: Option<String>,
    input_focused: bool,
    list_focused: bool,
    disabled: bool,
}

struct DropdownValue<P> {
    value: P,
    display: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DropdownMsg {
    InputFocus(bool),
    ListFocus(bool),
    InputChange(Option<String>),
    MakeSelection(usize),
    Reset
}

impl<P> Component for Dropdown<P>
where P: PartialEq + Eq + Clone + fmt::Display + 'static
{
    type Message = DropdownMsg;
    type Properties = DropdownProps<P>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let values = Self::wrap_values(&props.values);
        Self {
            values,
            placeholder: props.placeholder.clone(),
            on_change: props.on_change.clone(),
            disabled: props.disabled,

            picked: None,
            input_focused: false,
            list_focused: false,
            user_input: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("got {:?}", msg);
        match msg {
            DropdownMsg::InputChange(new_input) => {
                self.user_input = new_input.filter(|v| !v.is_empty());
                self.update_pick(self.idx_from_input());
            },
            DropdownMsg::InputFocus(focus) => {
                self.input_focused = focus;
            }
            DropdownMsg::ListFocus(focus) => {
                self.list_focused = focus;
            }
            DropdownMsg::MakeSelection(idx) => {
                self.user_input = Some(self.values[idx].display.clone());
                self.list_focused = false;
                self.update_pick(Some(idx));
            }
            DropdownMsg::Reset => {
                self.user_input = None;
                self.update_pick(None);
            }
        }

        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let mut any_changes = false;
        let new_props = ctx.props();
        if self.placeholder != new_props.placeholder {
            self.placeholder = new_props.placeholder.clone();
            any_changes = true;
        }

        let values_same = self.values.len() == new_props.values.len() && self.values.iter().zip(new_props.values.iter()).all(|(v1, v2)| v1.value == *v2);
        if !values_same {
            let current_picked = self.picked().map(|v| &v.value);
            let new_values = Self::wrap_values(&new_props.values);
            let new_picked = current_picked.and_then(|picked_item| new_values.iter().enumerate().filter_map(|(idx, new_v)| if new_v.value == *picked_item {
                Some(idx)
            } else {
                None
            }).next());

            self.values = new_values;
            self.picked = new_picked;
            any_changes = true;
        }

        self.on_change = new_props.on_change.clone();

        if self.disabled != new_props.disabled {
            self.disabled = new_props.disabled;
            if self.disabled {
                self.picked = None;
                self.user_input = None;
                self.list_focused = false;
                self.input_focused = false;
            }

            any_changes = true;
        }

        any_changes
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <div class={classes!(
                "dropdown",
                if self.picked.is_some() { "has-pick" } else { "no-pick" },
                if self.disabled { "disabled" } else { "enabled" },
                if self.show_list() { "active" } else { "inactive" },
                if self.is_input_err() { Some("error") } else { None },
            )}>
                {self.view_input(link)}
                {self.view_list(link)}
            </div>
        }
    }
}

impl<P> Dropdown<P> where P: PartialEq + Eq + Clone + fmt::Display + 'static {
    fn has_selection(&self) -> bool {
        self.picked().is_some()
    }

    fn view_input(&self, link: &Scope<Self>) -> Html {
        let onkeyup = link.callback(|e: web_sys::KeyboardEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            DropdownMsg::InputChange(Some(value))
        });

        html! {
            <div class="input">
                <input
                    placeholder={self.placeholder.clone()}
                    onfocus={link.callback(|_| DropdownMsg::InputFocus(true))}
                    onblur={link.callback(|_| DropdownMsg::InputFocus(false))}
                    {onkeyup}
                    value={self.user_input.clone().unwrap_or(String::default())}
                    disabled={self.disabled}
                />
                {
                    if self.user_input.is_some() {
                        html!{ <div class="clear" onclick={link.callback(|_| DropdownMsg::Reset)}> {"X"} </div> }
                    } else {
                        html! { <></> }
                    }
                }
            </div>
        }
    }

    fn view_list(&self, link: &Scope<Self>) -> Html {
        html! {
            <div class={classes!("autocomplete", if self.show_list() { "show" } else {"hide"})}
                onmouseenter={link.callback(|_| DropdownMsg::ListFocus(true))}
                onmouseout={link.callback(|_| DropdownMsg::ListFocus(false))}>
                {self.matching_options().map(|(idx, v)| html! {
                    <div onclick={link.callback(move |_| DropdownMsg::MakeSelection(idx))}
                        onmouseenter={link.callback(|_| DropdownMsg::ListFocus(true))}
                        class={classes!("entry", if self.is_picked(v) { "picked" } else { "unpicked" })}>{v.display.clone()}</div>
                }).collect::<Html>()}
            </div>
        }
    }

    fn matching_options(&self) -> impl Iterator<Item=(usize, &DropdownValue<P>)> {
        self.values.iter()
            .enumerate()
            .filter(|(_, v)| self.user_input.as_ref().map(|input| v.display.starts_with(input)).unwrap_or(true))
    }

    fn picked(&self) -> Option<&DropdownValue<P>> {
        self.picked.and_then(|idx| self.values.get(idx))
    }

    fn wrap_values(v: &Vec<P>) -> Vec<DropdownValue<P>> {
        return v.iter()
            .map(|v| DropdownValue {value: v.clone(), display: format!("{}", v)})
            .collect()
    }

    fn is_picked(&self, option: &DropdownValue<P>) -> bool {
        self.picked().map(|p| &p.value) == Some(&option.value)
    }

    fn idx_from_input(&self) -> Option<usize> {
        self.user_input
            .as_ref()
            .and_then(|input|
                self.values.iter()
                    .enumerate()
                    .filter_map(|(idx, value)| if value.display == *input { Some(idx) } else { None })
                    .next())
    }

    fn update_pick(&mut self, new_pick: Option<usize>) -> bool {
        if self.picked != new_pick {
            self.picked = new_pick;
            self.on_change.emit(self.picked().map(|p| p.value.clone()));
            true
        } else {
            false
        }
    }

    fn show_list(&self) -> bool {
        self.list_focused || self.input_focused
    }

    fn is_input_err(&self) -> bool {
        if let Some(input) = &self.user_input {
            !self.values.iter().any(|v| v.display.starts_with(input))
        } else {
            false
        }
    }
}