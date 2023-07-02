use eframe::epaint::RectShape;
use egui::{
    pos2, vec2, Color32, Event, Id, InputState, Pos2, Rect, Rounding, ScrollArea, Sense, Shape,
    Stroke, Style, Ui, Vec2,
};
use std::ops::{Add, Deref};

/*
Data structured in layers:
Every layer contains one or more boxes, every box can transform into another layer, ie. every layer has 1 parent, except root layer

We need to know
1. WHich layer we are at
1. For each layer, we need to know what is in there, and pan offset and scale

One problem not solved yet, if we use scroll to zoom into next layer, we need to decide `Which` layer to zoom into
It is easy if the zoom focal is within a box, but what to do if focal is not within any box? I can think of 2 choices
a. Dont use scroll to zoom into layer, use double click on a box
b. Pick the closest box by mid point
*/

struct RootLayer {
    boxes: Vec<Abox>,
    ui_config: UIConfig,
}

impl RootLayer {
    fn draw(&mut self, ui: &mut Ui) {
        let (res, painter) = ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

        // this is stupid, events will be return after this scope
        let zoom_delta = ui.input(|i| {
            i.events.clone().into_iter().find_map(|e| match e {
                Event::MouseWheel { delta, .. } => Some(delta.y),
                _ => None,
            })
        });

        if let Some(zoom) = zoom_delta {
            ui.ctx().pointer_latest_pos().map(|zoom_p| {
                let before = self.ui_config.screen_2_world(zoom_p);

                self.ui_config.pan_zoom(res.drag_delta(), zoom);

                let after = self.ui_config.screen_2_world(zoom_p);
                let displacement = after - before;
                self.ui_config.pan_offset -= displacement;
            });
        } else {
            self.ui_config.pan_zoom(res.drag_delta(), 0.0);
        }

        for b in self.boxes.iter_mut() {
            let size = b.size * self.ui_config.zoom_factor;
            let box_a = Rect::from_min_size(self.ui_config.world_2_screen(b.pos), size);
            painter.rect(box_a, Rounding::none(), Color32::RED, Stroke::default());
        }
    }
}

struct ChildLayer {
    boxes: Vec<Abox>,
    ui_config: UIConfig,
    parent: Box<Layer>,
}

enum Layer {
    Root(RootLayer),
    Child(ChildLayer),
}

impl Layer {
    fn draw(&mut self, ui: &mut Ui) {
        match self {
            Layer::Root(root_layer) => {}
            Layer::Child(_) => {}
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Abox {
    pos: Pos2,
    size: Vec2,
    text: String,
}

impl Abox {
    fn into(&self, ui_style: &UIConfig) -> Shape {
        todo!()
    }
}

struct UIConfig {
    zoom_factor: f32,
    pan_offset: Vec2,
}

impl UIConfig {
    fn pan_zoom(&mut self, drag_delta: Vec2, zoom_factor_delta: f32) {
        self.pan_offset = self.pan_offset - drag_delta;

        let o = self.zoom_factor + zoom_factor_delta;
        if o > 0.1 && o < 10.0 {
            self.zoom_factor = o
        }
    }

    fn world_2_screen(&mut self, i: Pos2) -> Pos2 {
        return ((i - self.pan_offset).to_vec2() * self.zoom_factor).to_pos2();
    }

    fn screen_2_world(&mut self, i: Pos2) -> Pos2 {
        return ((i.to_vec2() / self.zoom_factor) + self.pan_offset).to_pos2();
    }
}

pub struct TemplateApp {
    root: RootLayer,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let all_boxes = (0..4)
            .map(|i| Abox {
                pos: pos2(100.0 * i as f32, 100.0 * i as f32),
                size: vec2(20.0, 20.0),
                text: String::from("ooo"),
            })
            .collect();

        Self {
            root: RootLayer {
                boxes: all_boxes,
                ui_config: UIConfig {
                    zoom_factor: 1.0,
                    pan_offset: vec2(0.0, 0.0),
                },
            },
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /*
    We need
    1. A box widget
    1. An arrow widget
    1. All widgets need to be ided
    1. In each frame, we need to:
        a. know current position of every widget
        b. know if anything is being dragged <not sure how to implement>
        c.
    */
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // println!("pre draf {:?}", self.pan_offset);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.root.draw(ui);
            })
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
