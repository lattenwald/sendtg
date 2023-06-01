# Emails to telegram!

Wanna send your cron emails to telegram? I gotchu.

# Usage

Either use it as MTA in your cron daemon (you can in cronie), or put some shell script to `/usr/sbin/sendmail` (or whatever
your cron daemon uses) which runs `sendtg --telegram <chat_id>@<bot_token>`.

    Usage: sendtg [OPTIONS]

    Options:
      -t, --telegram <TELEGRAM>  Telegram address to send messages to, in format <chat id>@<bot token>. If missing, only messages to addresses tg:<chat id>@<bot token> will be sent to Telegram
      -s, --sendmail <SENDMAIL>  MTA to use for emails [default: sendmail]
      -h, --help                 Print help
      -V, --version              Print version

See example `sendmail` script. Find where your system expects to find it: `cat /usr/include/paths.h | grep SENDMAIL`.

# TODO

  - attachments
