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
greeting = { $user_tag } Добро пожаловать!

## Settings related messages.

settings-changed = Настройки успешно изменены.
settings-input = Отправьте мне настройки в том же формате, что и выше:
    (<a href="https://github.com/crabvk/perception-guardian#bot-settings">Описание настроек</a>)
settings-greeting-changed = OK, теперь я буду использовать новое приветствие:

    { $greeting }
settings-input-greeting = { $greeting }

    Введите новое приветствие:
    (<a href="https://core.telegram.org/api/entities#allowed-entities">Список доступных HTML тегов</a>)
settings-text-required = Требуется ввести текст.
settings-cancel = Редактирование настройки отменено.
settings-message-outdated = Сообщение устарело.
