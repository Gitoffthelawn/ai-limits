# ai-limits

| [DE](DE.md) | [EN](../../README.md) | [ES](ES.md) | [FR](FR.md) | [PT](PT.md) | RU | [中文](ZH.md) | [عربي](AR.md) |

---

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-macos-app.zip"><img src="https://shieldcn.dev/badge/macOS-v0.0.13-grey.svg?logo=apple" alt="Скачать для macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.0.13-blue.svg?logo=windows" alt="Скачать для Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.0.13-yellow.svg?logo=linux" alt="Скачать для Linux"></a>
</p>

---

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

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="ai-limits на Windows" width="24%">
  <img src="screenshots/linux.png" alt="ai-limits на Linux" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="Настройки ai-limits" width="24%">
  <img src="screenshots/macos-help.png" alt="Справка ai-limits" width="24%">
</p>

## Возможности

- Показывает лимиты, дату-время reset, доступные токены, достуаные ручные reset,
- Работает с Codex, Claude и Cursor,
- Получает данные из локальных файлов, CLI провайдеров и API,
- Логика fallback: если один источник недоступен, проверяет другой,
- Легковесное desktop-приложение (macOS, Windows, Linux)
- CLI интерфейс с несколькими вариантами вывода `./bin/ai-limits`,
- Системные уведомления при достижении порогов лимита,
- Ручное и гибкое автоматическое обновление данных.

## Аналоги

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| Desktop-приложение и CLI | ✅ | ✅ | ❌ |
| Codex, Claude и Cursor | ✅ | ✅ | ✅ |
| macOS, Windows и Linux | ✅ | ❌ | ❌ |
| Без промежуточного сервиса | ✅ | ✅ | ✅ |

Полное сравнение по 18 аналогам и 16 критериям — в [каталоге аналогов](../product/analogues.tsv).

## Поддержка платформ и ограничения

- macOS: приложение подписано и нотарифицировано; уведомления работают,
- Windows и Linux: сборки доступны; поддержка развивается на основе обратной связи пользователей,
- Уведомления desktop-приложения пока доступны только на macOS,
- Некоторые локальные источники Codex и Claude пока могут не работать в Windows и Linux (источники через CLI работают везде).

## Лицензия

[Лицензия MIT](../../LICENSE)
