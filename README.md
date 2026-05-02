# Off-brand Sentinel

This is a Discord bot written in **Rust** aiming to recreate the features of [Sentinel](https://github.com/seiama/sentinel),
a Discord bot on the [PaperMC Discord server](https://discord.gg/papermc).

> [!IMPORTANT]
> This is **not** the official bot running on the PaperMC Discord server, but a fan-remake. The reason for this
> is that the original Sentinel bot has a rather complicated setup process and no documentation at all. The goal
> of this project for me was to create a Sentinel instance which *just runs*, without compromising on features.

## Getting started

First, you need to download the executable appropriate for your OS. The [releases](https://github.com/Strokkur424/off-brand-sentinel/releases/latest)
tab contains download files for **Linux (x86)**, **Windows (x86)**, and **macOS**. For any other platforms (i.e. Raspberry Pi/Linux ARM), you will
need to build the source code yourself. Instructions for this can be found under [Building](#building).

Move the executable into its own folder. To run the bot, use the following command (you may need to replace the slash characters `/` with
backslashes `\` on Windows):

```bash
./off-brand-sentinel ./data/
```

This will run the bot whilst creating any files in the `data` directory. If you leave out the argument, the current working directory
will be used instead.

On first run, the bot will startup, generate the files, and shut down again. This is because you **haven't given it a token** yet.
You can create a new Discord bot [on the Discord developer portal](https://discord.com/developers/applications). You will need two
create **two applications/bots if you want to use the Factoids feature**, however Factoids is optional.

Create your bot(s). Ensure the only installation method is set to **Guild Install**. The Sentinel bot requires the following permissions:

- `Kick Members`, `Ban Members`, and `Moderate Members`.

When adding it to your guild, ensure that the bot's role is above any other member's roles.

The Factoids bot does not require any permissions.

Once you have obtained your bot token(s), open `config.toml` and enter into the appropriate fields.
Lastly, you may want to configure the channels the bot will use for sending modmail/report messages and create appeals. These are technically
optional, but highly recommended. The modmail/report message channels can be in the same channel, but it is recommended for the appeals
channel to have its own channel.

## Features

### Commands

**User commands**:

- `/modmail`
- `/about`

**Punishment commands**:

- `/timeout`, `/warn`, `/ban`, `/kick`.

**General management commands**:

- `/note` -- set a user note without the user being notified
- `/punishment (search user|show)` -- display information about past punishment
- `/punishment reason` -- update the reason of a past punishment
- `/punishment stale` -- mark a punishment as stale; this also acts as an unban or removes a timeout

**Factoid commands**
*Not implemented yet.*

### Context menu entries

These are accessible when you right-click a user's message `Apps > Sentinel`.

**User entries**:

- `Report` -- reports a message

**Moderator entries**:

- `Ban` -- bans the user who sent the message
- `Kick` -- kicks the user who sent the message
- `Quick Ban` -- bans the user who sent the message without asking for a reason first

## Building

In order to compile this project, you need to install Rust. For this, follow the instructions on [this page](https://rust-lang.org/tools/install/).

Once Rust is installed, compiling the project is as simple as running the following command:
```bash
cargo build --build --locked --manifest-path=sentinel-app/Cargo.toml
```

You can find the build executable under `target/release/sentinel-app(.exe)`.

