use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use chrono::{DateTime, Local};
use egui::{Button, Color32, Hyperlink, Label, RichText};
use egui_plot::{Line, Plot};
use gloo_timers::future::TimeoutFuture;
use rand_distr::Distribution;
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};

const STEP_MS: u64 = 200;
const INIT_CASH: f64 = 1000.0;

#[derive(Default)]
struct HelloApp {
    prices: Rc<RefCell<VecDeque<f64>>>,
    cash: f64,
    shares_count: u64,
    ref_portfolio_worth: f64,
    game_over: Rc<RefCell<Option<DateTime<Local>>>>,
    game_begin: DateTime<Local>,
}

impl eframe::App for HelloApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Some(last_price) = self.prices.borrow().back().cloned() else {
            return;
        };
        let last_price_buy = last_price * 1.01;
        let last_price_sell = last_price * 0.99;

        let can_buy = self.cash >= last_price_buy;
        let can_sell = self.shares_count >= 1;
        egui::TopBottomPanel::bottom("ppp").show(ctx, |ui| {
            ui.set_width(ui.available_width());
            let w = ui.available_width();
            let portfolio_worth = self.cash + self.shares_count as f64 * last_price_sell;
            egui::Grid::new("stats_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    if let Some(_game_over_time) = *self.game_over.borrow() {
                        ui.vertical_centered(|ui| {
                            ui.add_sized(
                                [w / 2.0, 60.0],
                                Label::new(
                                    RichText::new("Stock collapsed")
                                        .size(32.0)
                                        .color(Color32::RED),
                                ),
                            );
                            ui.label(RichText::new(
                                "Game over, your remaining stocks are sold for 0",
                            ));
                            self.shares_count = 0;
                        });
                    } else {
                        if ui
                            .add_sized(
                                [w / 2.0, 60.0],
                                Button::new(
                                    RichText::new(format!("Buy {:.0}€", last_price_buy))
                                        .color(Color32::WHITE) // white text
                                        .size(32.0)
                                        .strong(),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend)
                                .fill(if can_buy {
                                    Color32::DARK_GREEN
                                } else {
                                    Color32::GRAY
                                }),
                            )
                            .clicked()
                            && can_buy
                        {
                            if self.shares_count == 0 {
                                self.ref_portfolio_worth = self.cash;
                            }
                            self.cash -= last_price_buy;
                            self.shares_count += 1;
                        }
                        if ui
                            .add_sized(
                                [w / 2.0, 60.0],
                                Button::new(
                                    RichText::new(format!("Sell {:.0}€", last_price_sell))
                                        .color(Color32::WHITE) // white text
                                        .size(32.0)
                                        .strong(),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend)
                                .fill(if can_sell {
                                    Color32::DARK_RED
                                } else {
                                    Color32::GRAY
                                }),
                            )
                            .clicked()
                            && can_sell
                        {
                            self.cash += last_price_sell;
                            self.shares_count -= 1;
                        }
                    }

                    ui.end_row();

                    let font_size = 24.0;
                    ui.label(RichText::new("Cash").size(font_size));
                    ui.monospace(RichText::new(format!("{:.1}€", self.cash)).size(font_size));
                    ui.end_row();

                    ui.label(RichText::new("Shares").size(font_size));
                    ui.monospace(RichText::new(format!("{}", self.shares_count)).size(font_size));
                    ui.end_row();

                    ui.label(RichText::new("Worth").size(font_size));
                    ui.monospace(
                        RichText::new(format!("{:.1}€", portfolio_worth))
                            .color({
                                if self.shares_count == 0 {
                                    Color32::DARK_GRAY
                                } else if portfolio_worth > self.ref_portfolio_worth {
                                    Color32::DARK_GREEN
                                } else {
                                    Color32::DARK_RED
                                }
                            })
                            .size(font_size),
                    );
                    ui.end_row();

                    ui.label(RichText::new("Total profit").size(font_size));
                    ui.monospace(
                        RichText::new(format!("{:.1}€", portfolio_worth - INIT_CASH))
                            .color({
                                if portfolio_worth > INIT_CASH {
                                    Color32::DARK_GREEN
                                } else {
                                    Color32::DARK_RED
                                }
                            })
                            .size(font_size),
                    );
                    ui.end_row();

                    ui.label(RichText::new("ROI (per minute)").size(font_size));
                    let minutes = Local::now()
                        .signed_duration_since(self.game_begin)
                        .as_seconds_f64()
                        / 60.0;
                    let growth = portfolio_worth / INIT_CASH;
                    let roi = growth.powf(1.0 / minutes);
                    ui.monospace(
                        RichText::new(format!("{:.2}%", (roi - 1.0) * 100.0)).size(font_size),
                    );
                    ui.end_row();
                });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(Hyperlink::from_label_and_url(
                    "🔗 Repo @ Github ⭐",
                    "https://github.com/WhiteBlackGoose/super-trader",
                ));
            });
            let pts: Vec<[f64; 2]> = self
                .prices
                .borrow()
                .iter()
                .enumerate()
                .map(|(i, y)| [i as f64, *y])
                .collect();

            let x_len = self.prices.borrow().len();
            Plot::new("my_plot")
                .allow_zoom(false)
                .allow_drag(false)
                .auto_bounds([true, true])
                .allow_scroll(false)
                .allow_boxed_zoom(false)
                .x_axis_formatter({
                    move |name, _r| {
                        let i = x_len as f64 - name.value;
                        format!("-{:.0}s", i * STEP_MS as f64 / 1000.0)
                    }
                })
                .y_axis_formatter(|name, _| format!("{:.0}€", name.value))
                .show(ui, |plot_ui| {
                    // main series with filled area
                    plot_ui.line(Line::new("price", pts));
                });
        });
    }
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    let canvas = eframe::web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("the_canvas")
        .unwrap()
        .dyn_into::<eframe::web_sys::HtmlCanvasElement>()
        .unwrap();

    let runner = eframe::WebRunner::new();
    runner
        .start(
            canvas,
            web_options,
            Box::new(|cc| {
                let data = Rc::new(RefCell::new(VecDeque::new()));
                let data_producer = data.clone();
                let game_over = Rc::new(RefCell::new(None));
                let game_over_in = game_over.clone();

                let ctx = cc.egui_ctx.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let normal = rand_distr::Normal::new(0.0, 1.0).unwrap();
                    let mut last_value = 100.0;
                    loop {
                        data_producer.borrow_mut().push_back(last_value);
                        if data_producer.borrow_mut().len() > 100 {
                            data_producer.borrow_mut().pop_front();
                        }
                        last_value += normal.sample(&mut rand::rng());
                        if last_value <= 0.0 {
                            *game_over_in.borrow_mut() = Some(Local::now());
                            break;
                        }
                        TimeoutFuture::new(STEP_MS as u32).await;
                        ctx.request_repaint();
                    }
                });
                Ok(Box::new(HelloApp {
                    prices: data,
                    cash: INIT_CASH,
                    shares_count: 0,
                    game_over,
                    game_begin: Local::now(),
                    ..Default::default()
                }))
            }),
        )
        .await
        .unwrap();
    Ok(())
}
