# ai-limits

| DE | [EN](../../README.md) | [ES](ES.md) | [FR](FR.md) | [PT](PT.md) | [RU](RU.md) | [中文](ZH.md) | [عربي](AR.md) |

---

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-macos-app.zip"><img src="https://shieldcn.dev/badge/macOS-v0.0.13-grey.svg?logo=apple" alt="Für macOS herunterladen"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.0.13-blue.svg?logo=windows" alt="Für Windows herunterladen"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.0.13-yellow.svg?logo=linux" alt="Für Linux herunterladen"></a>
</p>

---

Lokale App zur Kontrolle von Limits und Nutzung von KI-Abonnements in Codex, Claude und Cursor.

Vorteile:
- Funktioniert ohne API-Abonnement,
- Keine Anmeldung,
- Alle Anbieter an einem Ort,
- Komplett kostenlos,
- Privat. Keine Drittanbieterdienste, Proxys oder Registrierungen,
- Leichtgewichtige Desktop-App für Mac, Windows und Linux,
- Benachrichtigungen zu Limits,
- Open Source.

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="ai-limits unter Windows" width="24%">
  <img src="screenshots/linux.png" alt="ai-limits unter Linux" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="Einstellungen von ai-limits" width="24%">
  <img src="screenshots/macos-help.png" alt="Hilfe zu ai-limits" width="24%">
</p>

## Funktionen

- Zeigt Limits, Reset-Datum und -Uhrzeit, verfügbare Tokens und verfügbare manuelle Resets,
- Arbeitet mit Codex, Claude und Cursor,
- Bezieht Daten aus lokalen Dateien, Anbieter-CLIs und APIs,
- Fallback-Logik: ist eine Quelle nicht verfügbar, wird eine andere geprüft,
- Leichtgewichtige Desktop-App (macOS, Windows, Linux),
- CLI-Schnittstelle mit mehreren Ausgabeformaten: `./bin/ai-limits`,
- Systembenachrichtigungen beim Erreichen von Limit-Schwellenwerten,
- Manuelle und flexible automatische Datenaktualisierung.

## Alternativen

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| Desktop-App und CLI | ✅ | ✅ | ❌ |
| Codex, Claude und Cursor | ✅ | ✅ | ✅ |
| macOS, Windows und Linux | ✅ | ❌ | ❌ |
| Ohne Zwischendienst | ✅ | ✅ | ✅ |

Der vollständige Vergleich über 18 Alternativen und 16 Kriterien steht im [Alternativenkatalog](../product/analogues.tsv).

## Plattformunterstützung und Einschränkungen

- macOS: die App ist signiert und notarisiert; Benachrichtigungen funktionieren,
- Windows und Linux: Builds sind verfügbar; der Support entwickelt sich anhand von Nutzerfeedback weiter,
- Desktop-Benachrichtigungen sind derzeit nur unter macOS verfügbar,
- Einige lokale Quellen von Codex und Claude funktionieren unter Windows und Linux möglicherweise noch nicht (Quellen über die CLI funktionieren überall).

## Lizenz

[MIT-Lizenz](../../LICENSE)
