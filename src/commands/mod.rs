use crate::extensions::*;

//use super::Config;
use crate::util;
use itertools::Itertools;
use poise::serenity_prelude::Member;
use poise::Command;

pub use crate::prelude::*;
pub use poise::serenity_prelude::{Guild, Mentionable, UserId};
pub mod errors;
pub mod poise_commands;
pub use errors::*;

pub mod info;
pub use info::*;
pub mod pfp;
pub use pfp::*;
pub mod note;
pub use note::*;
pub mod mute;
pub use mute::*;
pub mod poll;
pub use poll::*;
pub mod help;
pub use help::*;
pub mod tag;
pub use tag::*;
pub mod role;
pub use role::*;
pub mod version;
pub use version::*;

pub fn all_commands() -> Vec<poise::Command<UserData, Error>> {
    vec![
        poise_commands::register(),
        poise_commands::delete(),
        pfp(),
        info(),
        help(),
        mute(),
        role(),
        version(),
        Command {
            subcommands: vec![note_add(), note_undo(), note_list()],
            ..note()
        },
        Command {
            subcommands: vec![poll_vote(), poll_multi()],
            ..poll()
        },
        Command {
            subcommands: vec![tag_get(), tag_list(), tag_set(), tag_delete()],
            ..tag()
        },
    ]
}

pub static SELECTION_EMOJI: [&str; 19] = [
    "1️⃣",
    "2️⃣",
    "3️⃣",
    "4️⃣",
    "5️⃣",
    "6️⃣",
    "7️⃣",
    "8️⃣",
    "9️⃣",
    "🔟",
    "\u{1f1e6}",
    "\u{1f1e7}",
    "\u{1f1e8}",
    "\u{1f1e9}",
    "\u{1f1f0}",
    "\u{1f1f1}",
    "\u{1f1f2}",
    "\u{1f1f3}",
    "\u{1f1f4}",
];

pub async fn member_or_self(ctx: Ctx<'_>, member: Option<Member>) -> Res<Member> {
    if let Some(member) = member {
        Ok(member)
    } else {
        Ok(ctx
            .author_member()
            .await
            .user_error("failed to fetch message author")?)
    }
}

//#[group]
//#[only_in(guilds)]
//#[commands(
//restart, warn, note, notes, latency, say, purge, unban, spurge, blocklist, set_tag, delete_tag,
//kick, emojistats
//)]
//#[checks(moderator, channel_allows_commands)]
//struct Moderator;

//#[group]
//#[only_in(guilds)]
//#[commands(ban, delban, mute)]
//#[checks(helper_or_mod, channel_allows_commands)]
//struct HelperOrMod;

//#[group]
//#[commands(
//// info
//modping, pfp, move_users, repo, set_fetch, fetch, desc, git, dotfiles, poll, role, top,
//tag, invite, version, highlights, ask, uptime
//)]
//#[checks(channel_allows_commands)]
//struct General;
/*

#[tracing::instrument(skip_all)]
pub async fn disambiguate_user_mention(
    ctx: &client::Context,
    guild: &Guild,
    msg: &Message,
    name: &str,
) -> Result<Option<UserId>> {
    lazy_static::lazy_static! {
        static ref DISCRIMINATOR: Regex = Regex::new(r"#\d{4}$").unwrap();
    }
    if let Some(user_id) = name
        .parse::<UserId>()
        .ok()
        .filter(|id| id.0 > 10_000_000_000_000_000)
    {
        Ok(Some(user_id))
    } else if let Some(member) =
        async { guild.member(&ctx, name.parse::<u64>().ok()?).await.ok() }.await
    {
        Ok(Some(member.user.id))
    } else if DISCRIMINATOR.is_match(name) {
        let (name, discriminator) = name.split_once('#').unwrap();
        let discriminator: u16 = discriminator.parse().unwrap();
        if let Some(members) = async { guild.search_members(&ctx, name, None).await.ok() }.await {
            Ok(members
                .iter()
                .find(|m| {
                    m.user.name.to_lowercase() == name.to_lowercase()
                        && m.user.discriminator == discriminator
                })
                .map(|m| m.user.id))
        } else {
            Ok(None)
        }
    } else {
        let member_options = guild
            .members_containing(name, false, true)
            .await
            .into_iter()
            .map(|(mem, _)| mem.clone())
            .collect_vec();

        if member_options.len() == 1 {
            Ok(Some(member_options.first().unwrap().user.id))
        } else {
            Ok(await_reaction_selection(
                &ctx,
                &msg,
                msg.author.id,
                member_options.clone(),
                "Ambiguous user mention",
                |m| format!("{} ({})", m.mention(), m.user.tag()),
            )
            .await
            .context("Failed to request user selection")?
            .map(|member| member.user.id))
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn await_reaction_selection<'a, T: 'static + Clone + Send + Sync>(
    ctx: &client::Context,
    replying_to: &Message,
    by: UserId,
    options: Vec<T>,
    title: &str,
    show: impl Fn(&T) -> String,
) -> Result<Option<T>> {
    if options.is_empty() {
        return Ok(None);
    }

    tracing::debug!("starting a reaction selection menu");

    let options = SELECTION_EMOJI
        .iter()
        .map(|a| a.to_string())
        .zip(options.into_iter())
        .collect_vec();

    let description = options
        .iter()
        .map(|(emoji, value)| format!("{} - {}", emoji, show(&value)))
        .join("\n");

    let selection_message = replying_to
        .reply_embed(&ctx, |e| {
            e.title(title).description(description);
        })
        .await
        .context("Failed to send selection message")?;

    react_async(
        &ctx,
        &selection_message,
        options
            .iter()
            .map(|(emoji, _)| ReactionType::Unicode(emoji.to_string()))
            .collect_vec(),
    );

    let selection = {
        let options = options.clone();
        selection_message
            .await_reaction(&ctx)
            .author_id(by)
            .timeout(std::time::Duration::from_secs(30))
            .filter(move |x| match &x.emoji {
                ReactionType::Unicode(x) => SELECTION_EMOJI[..options.len()].contains(&x.as_str()),
                _ => false,
            })
            .await
    };

    tracing::debug!("Got reaction selection or timeout");

    let _ = selection_message.delete(&ctx).await;

    let selection = match selection {
        Some(selection) => selection,
        None => return Ok(None),
    };

    match selection.as_ref() {
        ReactionAction::Added(react) => match &react.emoji {
            ReactionType::Unicode(chosen_emoji) => Ok(options
                .iter()
                .find(|(emoji, _)| emoji == chosen_emoji)
                .map(|(_, x)| x.clone())),
            _ => unreachable!("previously verified in filter"),
        },
        _ => unreachable!("previously verified in filter"),
    }
}

pub fn react_async(ctx: &client::Context, msg: &Message, reactions: Vec<ReactionType>) {
    let msg = msg.clone();
    let ctx = ctx.clone();
    tokio::spawn(
        async move {
            for emoji in reactions {
                let _ = msg.react(&ctx, emoji).await;
            }
        }
        .instrument(tracing::info_span!("react-async")),
    );
}
*/
