## Time units formatting for the `DURATION` function.

duration-hours = { $value ->
    [one] { $value } hour
   *[many] { $value } hours
}

duration-minutes = { $value ->
    [one] { $value } minute
   *[many] { $value } minutes
}

duration-seconds = { $value ->
    [one] { $value } second
   *[many] { $value } seconds
}

## Bot reply messages.

captcha-caption = { $user_tag } Choose what is shown in the picture. You have { DURATION($duration) }.
captcha-time-over = { $user_tag } Time is over.
    You can try to join the group again after { DURATION($duration) }.
captcha-incorrect-answer = { $user_tag } Incorrect answer.
    You can try to join the group again after { DURATION($duration) }.

query-wrong-user = Not your keyboard.
query-correct = Correct!
query-wrong = Wrong!

make-me-admin = Great! Now make me an <b>admin</b>, so I can restrict newcomers until they pass the captcha 😉
welcome = { $user_tag } Welcome!

## Settings related messages.

settings-select-kind = Choose setting to change:
settings-select-language = Currently, I'm using English.
    Choose new language:
settings-select-language-default = Currently, language is not set and I'm using English as default.
    Choose new language:
settings-select-ban-channels-all = Currently, I'm banning all channels.
    Choose whether to ban channels:
settings-select-ban-channels-linked = Currently, I'm banning all channels except the linked one ({ $linked_chat_id }).
    Choose whether to ban channels:
settings-select-ban-channels-none = Currently, I'm not banning channels.
    Choose whether to ban channels:
settings-language-set = Language changed to { $lang ->
        [ru] Russian
       *[en] English
    }.
settings-ban-channels-set = Now I'll ban all channels.
settings-ban-channels-linked-set = Now I'll ban all channels except for the linked one ({ $linked_chat_id }).
settings-ban-channels-none-set = Now I won't ban any channels.
settings-welcome-message-set = Now I'll use new welcome message:
    { $welcome_message }
settings-input-welcome-message = Enter new welcome message:
    (<a href="https://core.telegram.org/api/entities#allowed-entities">List of allowed HTML tags</a>)
settings-text-required = Send me some text.
settings-cancel = Setting editing is canceled.
settings-message-outdated = The message is outdated.
