#[macro_use] extern crate serenity;
extern crate clap;
extern crate regex;
#[macro_use] extern crate lazy_static;
extern crate typemap;

use serenity::client::{Client, Context};
use serenity::model::{Message, Member};
use clap::{Arg, App};
use regex::Regex;
use typemap::Key;

type List = Vec<u64>;

struct Channels;
struct Roles;
struct DeletionMessage;

impl Key for Channels {
    type Value = List;
}

impl Key for Roles {
    type Value = List;
}

impl Key for DeletionMessage {
    type Value = String;
}

fn main() {
    let matches = App::new("Shard Calculator Bot")
        .version("0.1.0")
        .author("ducc https://github.com/ducc")
        .about("A bot that lets you calculate shard ids for discord guilds!")
        .arg(Arg::with_name("token")
            .short("t")
            .long("token")
            .required(true)
            .takes_value(true)
            .help("Discord oauth application token"))
        .arg(Arg::with_name("channels")
            .short("c")
            .long("channels")
            .required(true)
            .takes_value(true)
            .help("Channels to delete commands in"))
        .arg(Arg::with_name("roles")
            .short("r")
            .long("roles")
            .required(true)
            .takes_value(true)
            .help("Roles that are allowed to run commands"))
        .arg(Arg::with_name("message")
            .short("m")
            .long("message")
            .required(true)
            .takes_value(true)
            .help("Message to DM after deleting user commands"))
        .get_matches();

    let token = matches.value_of("token").unwrap();
    let mut client = Client::login(&token);
    {
        let c = matches.value_of("channels").unwrap().split(",");
        let mut channels = vec![];
        for channel in c {
            channels.push(channel.to_string().parse::<u64>().unwrap());
        }

        let channels = matches
            .value_of("channels")
            .unwrap()
            .split(",")
            .into_iter()
            .map(|c| c.to_string().parse::<u64>().unwrap())
            .collect();

        let roles = matches
            .value_of("roles")
            .unwrap()
            .split(",")
            .into_iter()
            .map(|c| c.to_string().parse::<u64>().unwrap())
            .collect();

        let m = matches.value_of("message").unwrap().to_string();
        let mut data = client.data.lock().unwrap();

        data.insert::<Channels>(channels);
        data.insert::<Roles>(roles);
        data.insert::<DeletionMessage>(m);
    }

    client.on_ready(|_, ready| {
        println!("ready! {}#{} {}", ready.user.name, ready.user.discriminator, ready.user.id);
    });
    client.on_message(message_handler);
    client.with_framework(|f| f
        .configure(|c| c
            .prefix("!")
            .ignore_bots(true)
            .allow_whitespace(true)
            .on_mention(true)
            .allow_dm(true)
            .ignore_webhooks(true))
        .on("shard", shard_command));

    if let Err(e) = client.start() {
        println!("client error: {:?}", e);
    }
}

fn message_handler(ctx: Context, msg: Message) {
    lazy_static! {
        static ref PATTERN: Regex = Regex::new("^(?:(?:!!!+)|(?:;+)|(?:\\.\\.\\.+)).+").unwrap();
    }

    if msg.is_private() || msg.author.bot {
        return;
    }

    {
        let data = ctx.data.lock().unwrap();
        if !data.get::<Channels>().unwrap().contains(&msg.channel_id.0) {
            return;
        }

        let guild = match msg.guild() {
            Some(guild) => guild,
            None => {
                println!("Could not get message guild!");
                return;
            }
        };
        let guild = guild.read().unwrap();
        let member: Member = match guild.member(msg.author.id) {
            Ok(member) => member,
            Err(_) => {
                //
                return;
            }
        };
        let roles = data.get::<Roles>().unwrap();
        for role in member.roles {
            if roles.contains(&role.0) {
                return;
            }
        }
    }

    if PATTERN.is_match(msg.content.as_ref()) {
        if let Err(_) = msg.delete() {
            // error = message was already deleted
            return;
        }
        let data = ctx.data.lock().unwrap();
        let deletion_message = data.get::<DeletionMessage>().unwrap();
        let _ = msg.author.dm(deletion_message.as_ref());
    }
}

command!(shard_command(_ctx, msg, _args, total: i64, guild: i64) {
    let res = (guild >> 22) % total;
    if let Err(e) = msg.channel_id.say(format!("Shard: {}", res).as_ref()) {
        println!("error sending shard num: {:?}", e);
    }
});