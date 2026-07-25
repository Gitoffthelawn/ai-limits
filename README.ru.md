# ai-limits

[English](README.md) | Русский

Локальное приложение для контроля лимитов и использования AI-подписок в Codex, Claude и Cursor.

Преимущества:
- Работает без подписки на API,
- Никаких логинов,
- Все провайдеры в одном месте,
- Полностью бесплатно,
- Приватно. Никаких сторонних сервисов, прокси, регистраций,
- Легковестное desktop приложение для Mac, Windows, Linux,
- Уведомления о лимитах,
- Open Source.

## Скачать

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-macos-app.zip"><img src="https://img.shields.io/badge/macOS-v0.0.13-000000?logo=apple&amp;logoColor=white" alt="Скачать для macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-windows-setup.exe"><img src="https://img.shields.io/badge/Windows-v0.0.13-0078D4?logo=windows&amp;logoColor=white" alt="Скачать для Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-linux.AppImage"><img src="https://img.shields.io/badge/Linux-v0.0.13-FCC624?logo=linux&amp;logoColor=black" alt="Скачать для Linux"></a>
</p>

Кнопки загружают последнюю desktop-сборку в один клик.

Также доступен CLI для любителей терминала: `./bin/ai-limits`

## Возможности

- Показывает лимиты, дату-время reset, доступные токены, достуаные ручные reset,
- Работает с Codex, Claude и Cursor,
- Получает данные из локальных файлов, CLI провайдеров и API,
- Логика fallback: если один источник недоступен, проверяет другой,
- Легковесное desktop-приложение (macOS, Windows, Linux)
- CLI интерфейс с несколькими вариантами вывода,
- Системные уведомления при достижении порогов лимита,
- Ручное и гибкое автоматическое обновление данных.

## Аналоги

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| Desktop-приложение и CLI | ✅ | ✅ | ❌ |
| Codex, Claude и Cursor | ✅ | ✅ | ✅ |
| macOS, Windows и Linux | ✅ | ❌ | ❌ |
| Без промежуточного сервиса | ✅ | ✅ | ✅ |

Полное сравнение по 18 аналогам и 16 критериям — в [каталоге аналогов](docs/product/analogues.tsv).

## Поддержка платформ и ограничения

- macOS: приложение подписано и нотарифицировано; уведомления работают,
- Windows и Linux: сборки доступны; поддержка развивается на основе обратной связи пользователей,
- Уведомления desktop-приложения пока доступны только на macOS,
- Некоторые локальные источники Codex и Claude пока могут не работать в Windows и Linux (источники через CLI работают везде).

## Лицензия и коллаборация

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/Лицензия-MIT-green.svg" alt="Лицензия MIT"></a>
  <a href="CONTRIBUTING.md"><img src="https://img.shields.io/badge/Участие-в%20разработке-blue.svg" alt="Участие в разработке"></a>
</p>
