# ai-limits

<p align="center">| <a href="DE.md">DE</a> | <a href="../../README.md">EN</a> | <a href="ES.md">ES</a> | FR | <a href="PT.md">PT</a> | <a href="RU.md">RU</a> | <a href="ZH.md">中文</a> | <a href="AR.md">عربي</a> |</p>

<p align="center">
   Application locale pour suivre les limites et l’usage des abonnements IA dans Codex, Claude et Cursor.
</p>

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-macos-app.zip"><img src="https://shieldcn.dev/badge/macOS-v0.0.13-grey.svg?logo=apple" alt="Télécharger pour macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.0.13-blue.svg?logo=ri:FaWindows" alt="Télécharger pour Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.0.13-yellow.svg?logo=linux" alt="Télécharger pour Linux"></a>
</p>

---

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="ai-limits sous Windows" width="24%">
  <img src="screenshots/linux.png" alt="ai-limits sous Linux" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="Réglages d’ai-limits" width="24%">
  <img src="screenshots/macos-help.png" alt="Aide d’ai-limits" width="24%">
</p>

## Avantages

- Fonctionne sans abonnement API,
- Aucune connexion requise,
- Tous les fournisseurs au même endroit,
- Entièrement gratuite,
- Privée. Aucun service tiers, proxy ni inscription,
- Application de bureau légère pour Mac, Windows et Linux,
- Notifications de limites,
- Open source.

## Fonctionnalités

- Affiche les limites, la date et l’heure de réinitialisation, les jetons disponibles et les réinitialisations manuelles disponibles,
- Fonctionne avec Codex, Claude et Cursor,
- Récupère les données depuis des fichiers locaux, les CLI des fournisseurs et les API,
- Logique de bascule : si une source est indisponible, une autre est vérifiée,
- Application de bureau légère (macOS, Windows, Linux),
- Interface CLI avec plusieurs formats de sortie : `./bin/ai-limits`,
- Notifications système lorsque les seuils de limite sont atteints,
- Actualisation des données manuelle et automatique flexible.

## Alternatives

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| Application de bureau et CLI | ✅ | ✅ | ❌ |
| Codex, Claude et Cursor | ✅ | ✅ | ✅ |
| macOS, Windows et Linux | ✅ | ❌ | ❌ |
| Sans service intermédiaire | ✅ | ✅ | ✅ |

La comparaison complète portant sur 18 alternatives et 16 critères se trouve dans le [catalogue des alternatives](../product/analogues.tsv).

## Prise en charge des plateformes et limitations

- macOS : l’application est signée et notarisée ; les notifications fonctionnent,
- Windows et Linux : des builds sont disponibles ; le support évolue selon les retours des utilisateurs,
- Les notifications de l’application de bureau ne sont pour l’instant disponibles que sur macOS,
- Certaines sources locales de Codex et Claude peuvent ne pas encore fonctionner sous Windows et Linux (les sources via CLI fonctionnent partout).

## Licence

[Licence MIT](../../LICENSE)
