### OTUS. Rust Developer. Basic
# Курсовая работа 
## "Веб-сервис для запуска и контроля выполнения расчетов"

#### Автор: Дмитрий Коновалов
<br>

## Описание
Проект состоит из двух Rust-сервисов и Redis:
- `calc_runner` — HTTP API на Axum для запуска расчетов. 
Запускает расчет, присваивает ему номер, хранит состояние расчета и результат в Redis. 
Предоставляет API для запуска и контроля. Эндопинты:
    -  `/api/calc/base_calc`
    -  `/api/calc/mass_calc`
    -  `/api/calc/{id}`
    -  `/api/calc/result/{id}`
    - `/api/stats/user/{id}`
    - `/api/stats/active_calcs`
    - `/health`

- `tgbot` — Telegram-бот на Teloxide, передаёт команды пользователей в calc_runner, показывает статус и результаты, есть админ-команда активных расчётов.
Список команд идентичен API `calc_runner`

- `redis` — хранилище состояний расчетов.

**Расчеты**: для демонстрации сделаны абстрактные расчеты с разным набором параметров - базовый и массовый.

<br>

## Установка и запуск
1. Потребуются Docker и Docker Compose.
2. Клонируйте репозиторий:
```
git clone git@github.com:DmitriyEKonovalov/otus_rust_project.git
cd otus_rust_project/services
```
3. Создайте `.env` рядом с `docker-compose.yml` (если файла нет) и задайте переменные окружения:
```
REDIS_PASSWORD=
CALC_RUNNER_PORT=3000
CALC_RUNNER_BASE_URL=http://calc_runner:3000/api
TELEGRAM_BOT_TOKEN=<токен бота>  # или BOT_ID
BOT_ID=
ADMIN_USER_IDS=123456789,987654321
MAX_ACTIVE_CALCS=10
TELOXIDE_PROXY=
HTTPS_PROXY=
HTTP_PROXY=
ALL_PROXY=
```
Необязательные переменные можно оставить пустыми; `REDIS_HOST` и `REDIS_PORT` берутся из docker-compose.

4. Соберите и запустите сервисы:
```
docker compose up --build
```
5. Проверка:
   - API: `curl http://localhost:3000/health`
   - В Telegram отправьте боту `/start` и `/help` для запуска расчётов.

<br>

## Локальный запуск
Локальный запуск без Docker: `cargo run -p calc_runner` и `cargo run -p tgbot` с теми же переменными окружения.
