extern crate plotlib;
extern crate serde;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use plotlib::barchart::BarChart;
use plotlib::page::Page;
use plotlib::view::CategoricalView;
use serde::Deserialize;
use std::convert::TryInto;

const ME: &str = "Will Taylor";
const MIN_THRESH: usize = 1000;

#[derive(Deserialize, Debug)]
struct MessageThread {
    participants: Vec<Participant>,
    messages: Vec<Message>,
    title: String,
    is_still_participant: bool,
    thread_type: String,
    thread_path: String,
}

#[derive(Deserialize, Debug)]
struct Participant {
    name: String,
}

#[derive(Deserialize, Debug)]
struct Message {
    sender_name: String,
    timestamp_ms: usize,
    content: Option<String>,
}

#[derive(Debug)]
struct Stats {
    thread_title: String,
    num_participants: u8,
    sent_by_me: usize,
    sent_by_others: usize,
}

fn main() {
    let sub_dirs: Vec<PathBuf> = fs::read_dir("bin/inbox")
        .expect("Couldn't read bin/inbox").into_iter()
        .map(|thread_dir|
            thread_dir.expect("Couldn't get subdirectory").path())
        .filter(|path| path.is_dir())
        .collect();

    let stats: Vec<Vec<Stats>> = sub_dirs.iter()
        .map(|thread_dir| fs::read_dir(&thread_dir)
            .expect(format!("Couldn't read subdirectory {:?}", thread_dir).as_str())
            .into_iter()
            .map(|thread_dir|
                thread_dir.expect("Couldn't read subdirectory").path())
            .filter(|file_or_folder| file_or_folder.is_file())
            .map(|file| std::fs::read_to_string(&file)
                .expect(format!("Couldn't read file {:?}", file).as_str()))
            .map(|content| serde_json::from_str(&content)
                .expect("Couldn't parse content as json"))
            .map(|message_thread| analyse(message_thread))
            .collect())
        .collect();

    let stats: Vec<Stats> = stats.into_iter().flatten().collect();
    let filtered_stats: Vec<Stats> = stats.into_iter()
        .filter(|s| s.sent_by_me + s.sent_by_others > MIN_THRESH)
        .collect();

    let counts: HashMap<String, (usize, usize)> = filtered_stats.into_iter()
        .map(|s| (s.thread_title, (s.sent_by_others, s.sent_by_me)))
        .collect();

    let mut view = CategoricalView::new();
    let bars: Vec<BarChart> = counts.iter()
        .map(|(title, (me, others))| BarChart::new((me + others) as f64).label(title))
        .collect();
    for bar in &bars {
        view = view.add(bar);
    }
    view = view.x_label("Chats")
        .y_label("Total messages");
    Page::single(&view).save("chart.svg").unwrap();
}

fn analyse(message_thread: MessageThread) -> Stats {
    let sent_by_me: usize = message_thread.messages.iter()
        .filter(|m| m.sender_name == ME)
        .count();
    let sent_by_others: usize = message_thread.messages.len() - sent_by_me;

    Stats {
        thread_title: message_thread.title,
        num_participants: message_thread.participants.len() as u8,
        sent_by_me,
        sent_by_others,
    }
}
