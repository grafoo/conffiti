use chrono::{DateTime, Duration, Utc};
use eframe::egui;
use git2::{Commit, IndexAddOption, ObjectType, Repository, Signature, Time};
use std::fs;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;

fn main() {
    let _ = eframe::run_native(
        "conffiti",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(App::new(cc))),
    );
}

#[derive(Default)]
struct App {
    days: Vec<(bool, DateTime<Utc>)>,
    begin: DateTime<Utc>,
    begin_rfc2822: String,
    commits: u32,
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

fn commit(time: DateTime<Utc>, commits: u32) -> Result<(), git2::Error> {
    let repo = match Repository::open("/tmp/confitti/conffiti-aux") {
        Ok(repo) => repo,
        Err(e) => panic!("repo init: failure, {}", e),
    };
    let mut index = match repo.index() {
        Ok(index) => index,
        Err(e) => panic!("repo open: failure, {}", e),
    };
    let sig = Signature::new(
        "Manuel Graf",
        "postmaster@yakbarber.org",
        &Time::new(time.timestamp(), 0),
    )
    .expect("make signature: failure");
    let mut file = File::create("/tmp/confitti/conffiti-aux/uuid.txt").unwrap();
    for _ in 1..commits {
        let parent_commit = find_last_commit(&repo)?;
        let uuid = Uuid::new_v4().to_string();
        file.write_all(&uuid.as_bytes()).unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .expect("add failure");
        let oid = index.write_tree()?;
        let tree = repo.find_tree(oid)?;
        let hash = repo.commit(Some("HEAD"), &sig, &sig, "", &tree, &[&parent_commit]);
        println!("{:?}", hash);
    }
    Ok(())
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();
        s.begin = Utc::now() - Duration::days(369);
        s.begin_rfc2822 = s.begin.to_rfc2822();
        s.days = Vec::new();
        for i in 0..371 {
            s.days.push((false, s.begin + Duration::days(i)));
        }
        s.commits = 42;
        s
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Start date:");
                let begin = ui.add(egui::TextEdit::singleline(&mut self.begin_rfc2822));
                if begin.lost_focus() {
                    self.begin = DateTime::parse_from_rfc2822(&self.begin_rfc2822)
                        .unwrap()
                        .into();
                    for i in 0..371 {
                        self.days[i] = (self.days[i].0, self.begin + Duration::days(i as i64));
                    }
                }
                ui.label("number of commits per day:");
                ui.add(egui::DragValue::new(&mut self.commits));
                if ui.add(egui::Button::new("commit")).clicked() {
                    for day in &self.days {
                        if day.0 {
                            match commit(day.1, self.commits) {
                                Ok(o) => o,
                                Err(e) => panic!("commit: failure, {}", e),
                            };
                        }
                    }
                }
                ui.add(egui::Separator::default());
                ui.label("store:");
                if ui.add(egui::Button::new("save")).clicked() {
                    let mut file = File::create("/tmp/confitti/days.txt").unwrap();
                    let days_to_save = self
                        .days
                        .iter()
                        .filter(|d| d.0)
                        .map(|d| format!("{0}", d.1.to_rfc2822()))
                        .collect::<Vec<String>>()
                        .join("\n");
                    file.write_all(&format!("{days_to_save}\n").as_bytes())
                        .unwrap();
                }
                if ui.add(egui::Button::new("load")).clicked() {
                    let days_raw =
                        fs::read_to_string("/tmp/confitti/days.txt").expect("load: failure");
                    let days_raw_v = days_raw.trim().split("\n");
                    for d in days_raw_v {
                        let dt = DateTime::parse_from_rfc2822(&d).expect("parse: failure");
                        for dl in &mut self.days {
                            if dl.1 == dt {
                                dl.0 = true;
                                break;
                            }
                        }
                    }
                }
            });
            let mut i = 0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                for _ in 0..53 {
                    ui.vertical(|ui| {
                        for _ in 0..7 {
                            ui.checkbox(&mut self.days[i].0, "")
                                .on_hover_text(self.days[i].1.to_rfc2822());
                            i += 1;
                        }
                    });
                }
            });
        });
    }
}
