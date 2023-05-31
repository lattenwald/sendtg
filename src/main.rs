use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::str;
use mail_parser::{Message, HeaderValue, Addr};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// Telegram address to send messages to, in format
    /// <chat id>@<bot token>. If missing, only messages to addresses
    /// tg:<chat id>@<bot token> will be sent to Telegram.
    telegram: Option<String>,

    #[arg(short, long, default_value = "sendmail")]
    /// MTA to use for emails
    sendmail: String
}

fn main() {
    let args = Args::parse();

    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).expect("Failed reading message from stdin");

    let message = Message::parse(&buf).expect("Failed parsing message");

    let tg_address = args.telegram.or_else(
        || match message.to() {
            HeaderValue::Address(Addr {
                address: Some(address),
                ..
            }) => address.strip_prefix("tg:").map(|addr| addr.to_string()),
            _ => None,
        });

    match tg_address {
        None => to_sendmail(&args.sendmail, &buf),
        Some(tg_address) => to_telegram(&tg_address, message),
    };
}

fn to_sendmail(mta_command: &str, buf: &[u8]) {
    let sendmail = Command::new(mta_command)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Couldn't launch sendmail");
    sendmail
        .stdin.unwrap()
        .write_all(buf)
        .expect("Failed writing message to sendmail");
}

fn to_telegram(tg_address: &str, message: Message) {
    let to = format_address(message.to());
    let from = format_address(message.from());
    let subject = html_escape::encode_text(
        message.subject()
        .unwrap_or("no subject"));
    let body = message
        .text_bodies()
        .filter_map(|part| part.text_contents())
        .collect::<Vec<&str>>()
        .join("\n---\n");
    let body = html_escape::encode_text(&body);
    let msg = format!(r#"<b>{}</b>
From: <code>{}</code>
To: <code>{}</code>

{}"#, subject, from, to, body);

    let (chat_id, bot_token) = tg_address.split_once('@').expect("Invalid tg address");

    let mut json = HashMap::new();
    json.insert("chat_id", chat_id);
    json.insert("text", &msg);
    json.insert("parse_mode", "HTML");

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!("https://api.telegram.org/bot{}/sendMessage", bot_token))
        .json(&json)
        .send()
        .expect("Failed to send message to telegram");
    if !res.status().is_success() {
        let resp_body = res.bytes().unwrap();
        panic!("request failed: {}", str::from_utf8(&resp_body).unwrap());
    }
}

fn format_address(addr: &HeaderValue) -> String {
    match addr {
        HeaderValue::Address(Addr { name, address }) => {
            let addr_part = address.clone().map(|s| s.to_string()).unwrap_or("".to_string());
            match name {
                Some(n) => format!("{} &lt;{}&gt;", html_escape::encode_text(n), addr_part),
                None => addr_part,
            }
        },
        _ => "none".to_string()
    }
}
