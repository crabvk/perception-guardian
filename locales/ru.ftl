## Time units formatting for the `DURATION` function.

duration-hours = { $value ->
    [one] { $value } час
    [few] { $value} часа
   *[many] { $value } часов
}

duration-minutes = { $value ->
    [one] { $value } минута
    [few] { $value} минуты
   *[many] { $value } минут
}

duration-seconds = { $value ->
    [one] { $value } секунда
    [few] { $value} секунды
   *[many] { $value } секунд
}

## Bot reply messages.

captcha-caption = { $user_tag } Выберите что изображено на картинке. У вас { DURATION($duration) }.
captcha-time-over = { $user_tag } Время вышло.
    Вы можете попробовать зайти в группу снова через { DURATION($duration) }.
captcha-incorrect-answer = { $user_tag } Неправильный ответ.
    Вы можете попробовать зайти в группу снова через { DURATION($duration) }.

query-wrong-user = Не ваша клавиатура.
query-correct = Верно!
query-wrong = Неверно!

make-me-admin = Отлично! Теперь сделайте меня <b>админом</b> чтобы я мог ограничивать новых пользователей пока они не пройдут капчу 😉
welcome = { $user_tag } Добро пожаловать!

## Settings related messages.

settings-select-kind = Выберите настройку для изменения:
settings-select-language = Сейчас я использую русский.
    Выберите новый язык:
settings-select-language-default = Сейчас язык не установлен и я использую английский по-умолчанию.
    Выберите новый язык:
settings-select-ban-channels-all = Сейчас я блокирую все каналы.
    Выберите, блокировать ли каналы:
settings-select-ban-channels-linked = Сейчас я блокирую все каналы за исключением привязанного ({ $linked_chat_id }).
    Выберите, блокировать ли каналы:
settings-select-ban-channels-none = Сейчас я не блокирую каналы.
    Выберите, блокировать ли каналы:
settings-language-set = Язык изменён на { $lang ->
        [ru] русский
       *[en] английский
    }.
settings-ban-channels-set = Теперь я буду блокировать все каналы.
settings-ban-channels-linked-set = Теперь я буду блокировать все каналы за исключением привязанного ({ $linked_chat_id }).
settings-ban-channels-none-set = Теперь я не буду блокировать каналы.
settings-welcome-message-set = Теперь я буду использовать новое приветственное сообщение:
    { $welcome_message }
settings-input-welcome-message = Введите новое приветственное сообщение:
    (<a href="https://core.telegram.org/api/entities#allowed-entities">Список доступных HTML тегов</a>)
settings-text-required = Требуется ввести текст.
settings-cancel = Редактирование настройки отменено.
settings-message-outdated = Сообщение устарело.
