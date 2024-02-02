use super::dropdown::*;
use crate::solver::{Month, TargetDate, Weekday};
use yew::prelude::*;

#[derive(PartialEq, Debug, Properties)]
pub struct PickerProps {
    pub on_picked: Callback<Option<TargetDate>>,
}

pub struct Picker {
    leap_year: bool,
    month: Option<Month>,
    weekday: Option<Weekday>,
    day: Option<i8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickerMsg {
    PickMonth(Option<Month>),
    PickWeekday(Option<Weekday>),
    PickDay(Option<i8>),
    NextDay,
}

impl Component for Picker {
    type Message = PickerMsg;
    type Properties = PickerProps;

    fn create(_: &Context<Self>) -> Self {
        Self {
            leap_year: true,
            day: None,
            weekday: None,
            month: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("got {:?}", msg);
        match msg {
            PickerMsg::PickMonth(month) => {
                if self.month != month {
                    self.month = month;
                    self.emit_selection(ctx);
                    true
                } else {
                    false
                }
            }

            PickerMsg::PickWeekday(weekday) => {
                if self.weekday != weekday {
                    self.weekday = weekday;
                    self.emit_selection(ctx);
                    true
                } else {
                    false
                }
            }

            PickerMsg::PickDay(day) => {
                if self.day != day {
                    self.day = day;
                    self.emit_selection(ctx);
                    true
                } else {
                    false
                }
            }

            PickerMsg::NextDay => {
                if let Some(target_date) = self.target_date() {
                    if let Some(next_date) = target_date.next(self.leap_year) {
                        self.month = Some(next_date.month);
                        self.day = Some(next_date.day_of_month);
                        self.weekday = Some(next_date.day_of_week);
                        self.emit_selection(ctx);
                        return true;
                    }
                }

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="target-picker">
                <Dropdown<Weekday>
                    placeholder={"Weekday"}
                    values={Self::weekday_values()}
                    on_change={ctx.link().callback(PickerMsg::PickWeekday)}
                    value={self.weekday}
                    disabled={false}/>
                <Dropdown<Month>
                    placeholder={"Month"}
                    values={Self::month_values()}
                    on_change={ctx.link().callback(PickerMsg::PickMonth)}
                    value={self.month}
                    disabled={false}/>
                <Dropdown<i8>
                    placeholder={"Day"}
                    values={self.day_of_month_values()}
                    on_change={ctx.link().callback(PickerMsg::PickDay)}
                    value={self.day}
                    disabled={self.month.is_none()}/>
                <div class={classes!(
                    "next-button",
                    "button",
                    if self.target_date().is_none() { Some("disabled") } else { None },
                )} onclick={ctx.link().callback(|_| PickerMsg::NextDay)}>{"⪢"}</div>
            </div>
        }
    }
}

impl Picker {
    fn target_date(&self) -> Option<TargetDate> {
        self.month
            .zip(self.weekday)
            .zip(self.day)
            .map(|((month, day_of_week), day_of_month)| TargetDate {
                month,
                day_of_week,
                day_of_month,
            })
    }

    fn month_values() -> Vec<Month> {
        vec![
            Month::January,
            Month::February,
            Month::March,
            Month::April,
            Month::May,
            Month::June,
            Month::July,
            Month::August,
            Month::September,
            Month::November,
            Month::December,
        ]
    }

    fn weekday_values() -> Vec<Weekday> {
        vec![
            Weekday::Sunday,
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
        ]
    }

    fn day_of_month_values(&self) -> Vec<i8> {
        if let Some(month) = &self.month {
            (1..=month.number_days(self.leap_year)).collect()
        } else {
            Vec::default()
        }
    }

    fn emit_selection(&self, ctx: &Context<Self>) {
        ctx.props().on_picked.emit(self.target_date())
    }
}
