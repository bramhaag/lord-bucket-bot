#[macro_use]
extern crate serenity;
extern crate clap;

use serenity::Client;
use clap::{Arg, App};

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
        .get_matches();

    let token = matches.value_of("token").unwrap();
    let mut client = Client::login(&token);

    client.on_ready(|_ctx, ready|
        println!("ready! {}#{} {}", ready.user.name, ready.user.discriminator, ready.user.id));

    client.with_framework(|f| f
        .configure(|c| c
            .prefix("!")
            .ignore_bots(true)
            .allow_whitespace(true)
            .on_mention(true)
            .allow_dm(true)
            .ignore_webhooks(true))
        .command("shard", |c| c
            .min_args(2)
            .usage("usage: !shard <total> <guild>")
            .exec(shard_command)));

    if let Err(e) = client.start() {
        println!("client error: {:?}", e);
    }
}

command!(shard_command(_ctx, msg, _args, total: i64, guild: i64) {
    let res = (guild >> 22) % total;
    if let Err(e) = msg.channel_id.say(format!("Shard: {}", res).as_ref()) {
        println!("error sending shard num: {:?}", e);
    }
});