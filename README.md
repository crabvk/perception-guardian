# Perception Guardian

Telegram bot with image/emoji CAPTCHA challenge/response.

<img src="screenshot.png" alt="screenshot" width="468" height="393"/>

## Features

* Show CAPTCHA for new members.
* Ban channels [(optional)](#bot-settings).
* Change bot language per group (only English and Russian supported at the time).
* Custom Welcome Message.

## Configuration

Copy [config.example.yaml](config.example.yaml) to *config.yaml*, see comments for config settings descriptions.  
There're no defaults, all config settings are mandatory.

## Bot settings

Use `/settings` command to change bot settings for a group.

## TODO

* Limit number of new chat members per minute, don't show captcha if limit has reached.
* `/vacuum` command to search and delete all "Deleted Account"s in a group.
* Periodicly delete expired `ignore` set key/scores in Redis.
* `/stats` command to show bot statistics: number of users passed/not passed captcha for a group, etc.
* Add more emojis.

## Resources

* [Emoji Meanings Encyclopedia](https://emojis.wiki/)
