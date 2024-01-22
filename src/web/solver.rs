use web_sys::wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::JsCast;
use yew::html::Scope;
use yew::prelude::*;
use crate::solver::{Month, Weekday, Solution, solve, SolverMsg, TaggedMask, TargetDate, PUZZLE_WIDTH, PUZZLE_HEIGHT, CellTag, BOARD_LABELS, BoardLabel};

#[derive(PartialEq, Debug, Properties)]
pub struct SolverProps {
    pub target: Option<TargetDate>,
}

#[derive(Debug, PartialEq)]
pub enum SolverCmpMsg {
    StartSolving,
    Reset,
    ChangeSpeed(i32),
    TickSolver,
    FocusPiece(Option<usize>),
}

pub struct SolverCmp {
    target: Option<TargetDate>,
    solver: Option<SolverState>,
    focus_piece: Option<usize>,
    speed: i32,
}

struct SolvingState {
    frames: Box<dyn Iterator<Item=SolverMsg>>,
    last_frame: TaggedMask,
    steps: usize,
    _ticker: Ticker,
}

enum SolverState {
    Solving(SolvingState),
    Solved(Solution),
    Impossible(usize)
}

impl Component for SolverCmp {
    type Message = SolverCmpMsg;
    type Properties = SolverProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            target: ctx.props().target,
            solver: None,
            speed: 35,
            focus_piece: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("got {:?}", msg);
        match msg {
            SolverCmpMsg::StartSolving => {
                if let Some(target_date) = self.target {
                    self.init_solver(target_date, ctx.link());
                    true
                } else {
                    false
                }
            }

            SolverCmpMsg::Reset => {
                self.take_solver()
            }

            SolverCmpMsg::ChangeSpeed(speed) => {
                todo!()
            }

            SolverCmpMsg::TickSolver => {
                if let Some(solver) = &mut self.solver {
                    match solver {
                        SolverState::Solving(state) => {
                            for _ in 0..self.speed {
                                match state.frames.next() {
                                    Some(SolverMsg::Unsolved(_, last_frame)) => {
                                        state.last_frame = last_frame;
                                        state.steps += 1;
                                    },
                                    Some(SolverMsg::Solved(solution)) => {
                                        *solver = SolverState::Solved(solution);
                                        return true;
                                    },
                                    Some(SolverMsg::Impossible) => {
                                        *solver = SolverState::Impossible(state.steps + 1);
                                        return true;
                                    },
                                    None => panic!("impossible state???")
                                }
                            }

                            true
                        },
                        SolverState::Impossible(_) | SolverState::Solved(_) => {
                            false
                        }
                    }
                } else {
                    false
                }
            }

            SolverCmpMsg::FocusPiece(focus) => {
                self.focus_piece = focus;
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let new_target = ctx.props().target;
        if self.target != ctx.props().target {
            self.target = new_target;
            if let Some(target) = self.target {
                self.init_solver(target, ctx.link());
            } else {
                self.take_solver();
            }

            true
        } else {
            false
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="solver">
                <div class="status">
                    {"Status: "}
                    {
                        match self.solver.as_ref() {
                            Some(SolverState::Solving(state)) => format!("solving... on step {}", state.steps),
                            Some(SolverState::Solved(solution)) => format!("solved in {} steps", solution.steps),
                            Some(SolverState::Impossible(steps)) => format!("impossible, determined in {} steps", steps),
                            None => "select target date".to_string(),
                        }
                    }
                </div>
                <div class="tip">
                    {"tip: hover over any square to see what piece is covering it"}
                </div>
                {self.view_board(ctx)}
            </div>
        }
    }
}

impl SolverCmp {

    fn init_solver(&mut self, target_date: TargetDate, link: &Scope<Self>) {
        let mut frames = Box::new(solve(target_date));
        self.solver = Some(match frames.next() {
            Some(SolverMsg::Unsolved(_, last_frame)) => SolverState::Solving(SolvingState {
                frames,
                last_frame,
                _ticker: Ticker::create(50, link.callback(|_| SolverCmpMsg::TickSolver)),
                steps: 0,
            }),
            Some(SolverMsg::Impossible) => SolverState::Impossible(0),
            v => panic!("unsupported initial state {:?}", v)
        });
    }

    fn take_solver(&mut self) -> bool {
        self.solver.take().is_some()
    }

    fn view_board(&self, ctx: &Context<Self>) -> Html {
        let tagged_mask = self.tagged_mask();
        html! {
            <div class="board">
                {(0..PUZZLE_HEIGHT).map(move |y| html! {
                    <div class="row">
                        {(0..PUZZLE_WIDTH).map(move |x| html! {
                            <div
                                class={classes!(
                                    "cell",
                                    match BOARD_LABELS[y][x] {
                                        BoardLabel::MonthLabel(_) => "lbl-month",
                                        BoardLabel::DayLabel(_) => "lbl-day",
                                        BoardLabel::DayOfWeekLabel(_) => "lbl-weekday",
                                        BoardLabel::Unlabeled => "lbl-blank",
                                    },
                                    tagged_mask.zip(self.focus_piece).and_then(|(tm, focus_piece_idx)| if tm.get(x, y) == CellTag::Covered(focus_piece_idx as u8) { Some("focus-light" )} else { Some("focus-dim") })
                                )}
                                onmouseenter={ctx.link().callback(move |_| SolverCmpMsg::FocusPiece(tagged_mask.and_then(|tm| if let CellTag::Covered(piece_idx) = tm.get(x, y) { Some(piece_idx as usize) } else { None })))}
                                onmouseout={ctx.link().callback(|_| SolverCmpMsg::FocusPiece(None))}
                            >
                                {
                                    if let Some(tagged_mask) = tagged_mask {
                                        match tagged_mask.get(x, y) {
                                            CellTag::Covered(piece_idx) => html! {<div class={classes!("contents", "covering", format!("piece-{}", piece_idx))}> {format!("{}", piece_idx)} </div>},
                                            CellTag::Winner => html! { <div class="contents winning-space"></div> },
                                            CellTag::Uncovered => html! {<> </>}
                                        }
                                    } else {
                                        html! { <> </> }
                                    }
                                }
                                <div class="lbl">
                                    {
                                        match BOARD_LABELS[y][x] {
                                            BoardLabel::MonthLabel(month) => match month {
                                                Month::January => "JAN",
                                                Month::February => "FEB",
                                                Month::March => "MAR",
                                                Month::April => "APR",
                                                Month::May => "MAY",
                                                Month::June => "JUN",
                                                Month::July => "JUL",
                                                Month::August => "AUG",
                                                Month::September => "SEP",
                                                Month::October => "OCT",
                                                Month::November => "NOV",
                                                Month::December => "DEC",
                                            }.to_string(),
                                            BoardLabel::DayLabel(day) => format!("{}", day),
                                            BoardLabel::DayOfWeekLabel(weekday) => match weekday {
                                                Weekday::Sunday => "SUN",
                                                Weekday::Monday => "MON",
                                                Weekday::Tuesday => "TUES",
                                                Weekday::Wednesday => "WED",
                                                Weekday::Thursday => "THUR",
                                                Weekday::Friday => "FRI",
                                                Weekday::Saturday => "SAT",
                                            }.to_string(),
                                            BoardLabel::Unlabeled => String::default(),
                                        }
                                    }
                                </div>
                            </div>
                        }).collect::<Html>()}
                    </div>
                }).collect::<Html>()}
            </div>
        }
    }

    fn tagged_mask(&self) -> Option<TaggedMask> {
        if let Some(state) = &self.solver {
            Some(match state {
                SolverState::Solved(solution) => solution.mask,
                SolverState::Solving(state) => state.last_frame,
                _ => return None,
            })
        } else {
            None
        }
    }
}

struct Ticker {
    #[allow(unused)]
    _callback: Closure<dyn FnMut()>,
    id: i32,
}

impl Ticker {
    pub fn create(interval: i32, target: Callback<()>) -> Self {
        let callback = Closure::wrap(Box::new(move || {
            target.emit(())
        }) as Box<dyn FnMut()>);
        let cb_ref = callback.as_ref().unchecked_ref();
        let id = web_sys::window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(cb_ref, interval).unwrap();
        Self{ _callback: callback, id }
    }
}

impl Drop for Ticker {
    fn drop(&mut self) {
        web_sys::window().unwrap().clear_interval_with_handle(self.id);
    }
}