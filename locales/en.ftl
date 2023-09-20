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
settings-select-value = Current <b>{ $name }</b> value: { $value }
    Choose new value:
settings-input-welcome-message = Enter new welcome message:
    (List of allowed HTML entities can be found <a href="https://core.telegram.org/api/entities#allowed-entities">here</a>)
settings-value-set = <b>{ $name }</b> is set: { $value }
settings-text-required = Send me some text.
settings-cancel = Setting editing is canceled.
settings-message-outdated = The message is outdated.
