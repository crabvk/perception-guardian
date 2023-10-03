# Perception Guardian

Telegram bot with image/emoji CAPTCHA challenge/response.

<img src="screenshot.png" alt="screenshot" width="468" height="393"/>

## Features

* Show CAPTCHA for new members.
* Ban channels except for the linked one (if set) [[optional]](#bot-settings).
* Change bot language for a group (only English and Russian available at the time).
* Set custom greeting.

## Configuration

This bot loads environment variables from a *.env* file.  
Copy [.example.env](.example.env) to *.env*, read comments and edit file accordingly.

## Bot settings

In a group use `/settings` commands to show and change bot settings.

List of available settings:

| Setting                 | Description                                                               | Type             | Possible values |
| ----------------------- | ------------------------------------------------------------------------- | ---------------- | --------------- |
| `language`              | Language the bot speaks                                                   | Enum             | en, ru          |
| `ban_channels`          | Ban channels of anonymous users[^1]                                       | Boolean          | true, false     |
| `captcha_expire`        | Captcha will disappear after this timeout (in seconds)                    | Unsigned Integer |                 |
| `message_expire`        | Expiration timeout (in seconds) for greeting and other temporary messages | Unsigned Integer |                 |
| `ignore_expire`         | Temporary don't show CAPTCHA again for users who didn't pass it           | Unsigned Integer |                 |
| `delete_entry_messages` | Whether to delete "User joined/left the group" messages                   | Boolean          | true, false     |

[^1]: If a group has linked channel it'll be added as an exception.

Use `/greeting` command to change greeting for newcomers.  
Note that greeting text must include "{user_tag}" substring.

## Webhook setup with Nginx

```nginx
http {
    upstream guardian {
        server WEBHOOK_ADDR fail_timeout=0;
    }

    server {
        # ...

        location /webhook {
            set $token SECRET_TOKEN;

            if ($http_x_telegram_bot_api_secret_token = $token) {
                proxy_pass http://guardian$request_uri;
            }
        }

        location / {
            return 403;
        }
    }
}
```

where `WEBHOOK_ADDR` is the same `address:port` as `WEBHOOK_ADDR` value in your .env file,
and `SECRET_TOKEN` is your bot's token with ":" replaced to "_".

## TODO

* Translate /help output and error messages sent to user.
* Limit number of new chat members per minute, don't show captcha if limit has reached.
* Periodicly delete expired `ignore` set key/scores in Redis.
* `/stats` command to show bot statistics: number of users passed/not passed captcha for a group, etc.
* Add more emojis.

## Resources

* [Emoji Meanings Encyclopedia](https://emojis.wiki/)
