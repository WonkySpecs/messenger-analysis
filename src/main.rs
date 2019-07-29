extern crate serde;
extern crate plotlib;

use std::fs;
use std::collections::HashMap;
use serde::Deserialize;
use plotlib::barchart::BarChart;
use plotlib::view::CategoricalView;
use plotlib::page::Page;

const ME: &str = "Will Taylor";
const MIN_THRESH: u32 = 1000;

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

struct Stats {
    thread_title: String,
    num_participants: u8,
    sent_by_me: u32,
    sent_by_others: u32,
}

fn main() {
    let mut counts: HashMap<String, (u32, u32)> = HashMap::new();
    if let Ok(thread_dirs) = fs::read_dir("bin/inbox") {
        for thread in thread_dirs {
            let thread = thread.expect("Couldn't get thread");
            if let Ok(thread_dir) = fs::read_dir(thread.path()) {
                for file_or_folder in thread_dir {
                    let path = file_or_folder.expect("Couldn't get file").path();
                    if path.is_file() {
                        let content: String = std::fs::read_to_string(&path)
                            .expect("Couldn't read file contents");
                        let json: MessageThread = serde_json::from_str(&content)
                            .expect("Failed to parse file contents as json");
                        let stats = analyse(json);
                        if stats.sent_by_me + stats.sent_by_others > MIN_THRESH {
                            counts.insert(stats.thread_title,
                                          (stats.sent_by_me, stats.sent_by_others));
                        }
                    }
                }
            }
        }
    }
    let mut view = CategoricalView::new();
    let bars: Vec<BarChart> = counts.iter()
        .map(|(title, (me, other))| BarChart::new((me + other).into()).label(title))
        .collect();
    for bar in &bars {
        view = view.add(bar);
    }
    view = view.x_label("Chats")
        .y_label("Total messages");
    Page::single(&view).save("chart.svg").unwrap();
}

fn analyse(message_thread: MessageThread) -> Stats {
    let mut me: u32 = 0;
    let mut others: u32 = 0;
    for message in message_thread.messages {
        if message.sender_name == ME {
            me += 1;
        } else {
            others += 1;
        }
    }

    Stats {
        thread_title: message_thread.title,
        num_participants: message_thread.participants.len() as u8,
        sent_by_me: me,
        sent_by_others: others,
    }
}
