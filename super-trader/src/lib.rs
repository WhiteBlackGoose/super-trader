use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crossbeam_queue::ArrayQueue;
use egui::{Align, Button, Color32, DragValue, Frame, Layout, RichText, TextStyle};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use gloo_timers::future::{IntervalStream, TimeoutFuture};
use rand_distr::Distribution;
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};
use wasm_bindgen_futures::spawn_local;

const STEP_MS: u64 = 200;

#[derive(Default)]
struct HelloApp {
    prices: Rc<RefCell<VecDeque<f64>>>,
    cash: f64,
    shares_count: u64,
    ref_portfolio_worth: f64,
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
            ui.columns(2, |cols| {
                Frame::group(cols[0].style()).show(&mut cols[0], |ui| {
                    ui.heading("Actions");
                    if ui
                        .add(
                            Button::new(
                                RichText::new(format!("BUY x1 for {:.1}€", last_price_buy))
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
                        .add(
                            Button::new(
                                RichText::new(format!("SELL x1 for {:.1}€", last_price_sell))
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
                });
                Frame::group(cols[1].style()).show(&mut cols[1], |ui| {
                    let portfolio_worth = self.cash + self.shares_count as f64 * last_price_sell;
                    egui::Grid::new("stats_grid")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Cash");
                            ui.monospace(format!("{:.1}€", self.cash));
                            ui.end_row();

                            ui.label("Shares");
                            ui.monospace(format!("{}", self.shares_count));
                            ui.end_row();

                            ui.label("Worth");
                            ui.monospace(RichText::new(format!("{:.1}€", portfolio_worth)).color(
                                {
                                    if self.shares_count == 0 {
                                        Color32::DARK_GRAY
                                    } else if portfolio_worth > self.ref_portfolio_worth {
                                        Color32::DARK_GREEN
                                    } else {
                                        Color32::DARK_RED
                                    }
                                },
                            ));
                            ui.end_row();
                        });
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
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

                let ctx = cc.egui_ctx.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let normal = rand_distr::Normal::new(0.0, 3.0).unwrap();
                    let mut last_value = 300.0;
                    loop {
                        data_producer.borrow_mut().push_back(last_value);
                        if data_producer.borrow_mut().len() > 100 {
                            data_producer.borrow_mut().pop_front();
                        }
                        last_value += normal.sample(&mut rand::rng());
                        TimeoutFuture::new(STEP_MS as u32).await;
                        ctx.request_repaint();
                    }
                });
                Ok(Box::new(HelloApp {
                    prices: data,
                    cash: 1000.0,
                    shares_count: 0,
                    ..Default::default()
                }))
            }),
        )
        .await
        .unwrap();
    Ok(())
}
