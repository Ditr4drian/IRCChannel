// quick smoke via library
use ircchannel::config::Config;
use ircchannel::irc::{IrcClient, IrcEvent};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut cfg = Config::default();
    cfg.identity.nick = format!("tui{}", std::process::id() % 1000);
    cfg.session.channels = vec!["#ditra".into()];
    let mut h = IrcClient::connect(&cfg).await.expect("connect");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
    let mut registered = false;
    let mut joined = false;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), h.events.recv()).await {
            Ok(Some(IrcEvent::Registered { nick })) => {
                println!("registered as {nick}");
                registered = true;
            }
            Ok(Some(IrcEvent::Join { user, channel })) => {
                println!("join {} -> {channel}", user.nick);
                if user.nick.eq_ignore_ascii_case(&cfg.identity.nick) {
                    joined = true;
                }
            }
            Ok(Some(IrcEvent::Privmsg { from, target, text })) => {
                println!("<{target}> {}: {text}", from.nick);
            }
            Ok(Some(IrcEvent::Disconnected { reason })) => {
                println!("disconnected: {reason}");
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => {}
        }
        if registered && joined {
            let _ = h.commands.send(ircchannel::irc::IrcCommand::Privmsg {
                target: "#ditra".into(),
                text: format!("hello from ircchannel smoke ({})", cfg.identity.nick),
            });
            tokio::time::sleep(Duration::from_millis(400)).await;
            let _ = h
                .commands
                .send(ircchannel::irc::IrcCommand::Quit("smoke done".into()));
            println!("smoke OK");
            return;
        }
    }
    eprintln!("smoke incomplete reg={registered} join={joined}");
    std::process::exit(1);
}
