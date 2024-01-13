use chrono::{DateTime, Duration, Utc};
use eframe::egui;

fn main() {
    let _=eframe::run_native(
        "contribuffiti",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(App::new(cc))),
    );
}

#[derive(Default)]
struct App {
    days: Vec<(bool, DateTime<Utc>)>,
    commands: String,
    begin: DateTime<Utc>,
    begin_rfc3339: String,
    commits: String,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();
        s.commands = String::new();
        s.begin = Utc::now() - Duration::days(369);
        s.begin_rfc3339 = s.begin.to_rfc3339();
        s.days = Vec::new();
        for i in 0..371{
            s.days.push((false, s.begin + Duration::days(i)));
        }
        s.commits = "50".to_string();
        s
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.commands = String::new();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let begin = ui.add(egui::TextEdit::singleline(&mut self.begin_rfc3339));
            if begin.lost_focus() {
                self.begin = DateTime::parse_from_rfc3339(&self.begin_rfc3339).unwrap().into();
                for i in 0..371 {
                        self.days[i]=(self.days[i].0, self.begin + Duration::days(i as i64));
                }
            }
            ui.add(egui::TextEdit::singleline(&mut self.commits));
            let mut i = 0;
            ui.horizontal(|ui| {
                for _ in 0..53 {
                    ui.vertical(|ui| {
                        for _ in 0..7 {
                            ui.checkbox(&mut self.days[i].0, "");
                            i += 1;
                        }
                    });
                }
            });
            self.commands += "#!/bin/sh\n";
            for day in &self.days {
                if day.0 {
                    let date = day.1.to_rfc2822();
                    self.commands += format!("for _ in $(seq 0 {0});do uuidgen >uuid;git add uuid;GIT_COMMITTER_DATE='{date}' git commit -m '{date}' --date '{date}';done\n",self.commits.as_str()).as_str();
                }
            }
            ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.commands));
        });
    }
}
