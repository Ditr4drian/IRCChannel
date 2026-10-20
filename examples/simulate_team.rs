//! Fake teammates on #ditra so a solo operator can demo the TUI.
//!
//!   cargo run --example simulate_team
//!
//! Ctrl+C stops all bots.

use std::time::Duration;

use ircchannel::config::Config;
use ircchannel::irc::{IrcClient, IrcCommand, IrcEvent};
use tokio::time::{interval, sleep};

struct Bot {
    nick: &'static str,
    lines: &'static [&'static str],
}

const BOTS: &[Bot] = &[
    Bot {
        nick: "ana",
        lines: &[
            "buenas, ya estoy en la vpn",
            "voy a revisar awx un rato",
            "¿alguien tocó el proxy de pruebas?",
        ],
    },
    Bot {
        nick: "beto",
        lines: &[
            "yo ando en el cluster-test",
            "deploy de gbicreator v0.5 ok por aquí",
            "si ven 8081 raro avisen",
        ],
    },
    Bot {
        nick: "carla",
        lines: &[
            "grafana está pidiendo café otra vez",
            "metí un panel nuevo en el dashboard",
            "¿quedamos en #ditra para el postmortem?",
        ],
    },
    Bot {
        nick: "diego",
        lines: &[
            "ssh al servidor de pruebas ok",
            "no reinicien ergo pls, estoy midiendo",
            "listo, ya pueden tocar",
        ],
    },
    Bot {
        nick: "eva",
        lines: &[
            "buenas team",
            "PR de inventarios listo para review",
            "me desconecto 10 min, cualquier cosa ping",
        ],
    },
];

#[tokio::main]
async fn main() {
    println!("simulate_team → connecting {} bots to #ditra", BOTS.len());
    println!("open another terminal: cargo run --release");
    println!("Ctrl+C to stop\n");

    let mut handles = Vec::new();
    for (i, bot) in BOTS.iter().enumerate() {
        let bot = Bot {
            nick: bot.nick,
            lines: bot.lines,
        };
        handles.push(tokio::spawn(async move {
            let nick = bot.nick;
            if let Err(e) = run_bot(bot, i as u64).await {
                eprintln!("[{nick}] stopped: {e}");
            }
        }));
        sleep(Duration::from_millis(400)).await;
    }

    // run until ctrl+c
    tokio::signal::ctrl_c().await.ok();
    println!("\nstopping…");
    for h in handles {
        h.abort();
    }
    sleep(Duration::from_millis(300)).await;
}

async fn run_bot(bot: Bot, offset: u64) -> anyhow::Result<()> {
    let mut cfg = Config::default();
    cfg.identity.nick = bot.nick.into();
    cfg.identity.username = bot.nick.into();
    cfg.identity.realname = format!("sim:{nick}", nick = bot.nick);
    cfg.session.channels = vec!["#ditra".into()];

    let mut handle = IrcClient::connect(&cfg).await?;
    let mut registered = false;
    let mut line_idx = 0usize;
    let mut tick = interval(Duration::from_secs(4 + offset)); // stagger chatter
    let mut first = true;

    loop {
        tokio::select! {
            ev = handle.events.recv() => {
                match ev {
                    Some(IrcEvent::Registered { .. }) => {
                        registered = true;
                        println!("[{}] online", bot.nick);
                    }
                    Some(IrcEvent::Join { user, channel }) if user.nick == bot.nick => {
                        println!("[{}] joined {channel}", bot.nick);
                    }
                    Some(IrcEvent::Disconnected { reason }) => {
                        anyhow::bail!("{reason}");
                    }
                    Some(_) | None => {}
                }
            }
            _ = tick.tick() => {
                if !registered {
                    continue;
                }
                if first {
                    first = false;
                    // greet once
                    let _ = handle.commands.send(IrcCommand::Privmsg {
                        target: "#ditra".into(),
                        text: format!("hola, soy {} (sim)", bot.nick),
                    });
                    continue;
                }
                let text = bot.lines[line_idx % bot.lines.len()];
                line_idx += 1;
                let _ = handle.commands.send(IrcCommand::Privmsg {
                    target: "#ditra".into(),
                    text: text.into(),
                });
                println!("[{}] {}", bot.nick, text);

                // occasional /me-style action via CTCP
                if line_idx % 5 == 0 {
                    let _ = handle.commands.send(IrcCommand::Privmsg {
                        target: "#ditra".into(),
                        text: format!("\x01ACTION revisa logs\x01"),
                    });
                }
            }
        }
    }
}
