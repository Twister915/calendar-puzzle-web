use yew::prelude::*;
use crate::solver::{Month, TargetDate, Weekday};
use super::dropdown::*;

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
}

impl Component for Picker {
    type Message = PickerMsg;
    type Properties = PickerProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            leap_year: true,
            day: None,
            weekday: None,
            month: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("got {:?}", msg);
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
        }
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="target-picker">
                <Dropdown<Weekday>
                    placeholder={"Weekday"}
                    values={Self::weekday_values()}
                    on_change={ctx.link().callback(|new_weekday| PickerMsg::PickWeekday(new_weekday))}
                    disabled={false}/>
                <Dropdown<Month>
                    placeholder={"Month"}
                    values={Self::month_values()}
                    on_change={ctx.link().callback(|new_month| PickerMsg::PickMonth(new_month))}
                    disabled={false}/>
                <Dropdown<i8>
                    placeholder={"Day"}
                    values={self.day_of_month_values()}
                    on_change={ctx.link().callback(|new_day| PickerMsg::PickDay(new_day))}
                    disabled={self.month.is_none()}/>
            </div>
        }
    }
}

impl Picker {
    fn target_date(&self) -> Option<TargetDate> {
        self.month.zip(self.weekday).zip(self.day).map(|((month, day_of_week), day_of_month)| TargetDate{month, day_of_week, day_of_month})
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