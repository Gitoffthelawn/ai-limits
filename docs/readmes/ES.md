# ai-limits

| [DE](DE.md) | [EN](../../README.md) | ES | [FR](FR.md) | [PT](PT.md) | [RU](RU.md) | [中文](ZH.md) | [عربي](AR.md) |

---

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-macos-app.zip"><img src="https://shieldcn.dev/badge/macOS-v0.0.13-grey.svg?logo=apple" alt="Descargar para macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.0.13-blue.svg?logo=windows" alt="Descargar para Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.0.13-yellow.svg?logo=linux" alt="Descargar para Linux"></a>
</p>

---

Aplicación local para controlar los límites y el uso de las suscripciones de IA en Codex, Claude y Cursor.

Ventajas:
- Funciona sin suscripción a la API,
- Sin inicios de sesión,
- Todos los proveedores en un solo lugar,
- Completamente gratuita,
- Privada. Sin servicios de terceros, proxies ni registros,
- Aplicación de escritorio ligera para Mac, Windows y Linux,
- Notificaciones de límites,
- Código abierto.

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="ai-limits en Windows" width="24%">
  <img src="screenshots/linux.png" alt="ai-limits en Linux" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="Ajustes de ai-limits" width="24%">
  <img src="screenshots/macos-help.png" alt="Ayuda de ai-limits" width="24%">
</p>

## Funciones

- Muestra los límites, la fecha y hora de reinicio, los tokens disponibles y los reinicios manuales disponibles,
- Funciona con Codex, Claude y Cursor,
- Obtiene los datos de archivos locales, CLI de los proveedores y APIs,
- Lógica de respaldo: si una fuente no está disponible, comprueba otra,
- Aplicación de escritorio ligera (macOS, Windows, Linux),
- Interfaz CLI con varios formatos de salida: `./bin/ai-limits`,
- Notificaciones del sistema al alcanzar umbrales de límite,
- Actualización de datos manual y automática flexible.

## Alternativas

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| App de escritorio y CLI | ✅ | ✅ | ❌ |
| Codex, Claude y Cursor | ✅ | ✅ | ✅ |
| macOS, Windows y Linux | ✅ | ❌ | ❌ |
| Sin servicio intermediario | ✅ | ✅ | ✅ |

La comparación completa de 18 alternativas y 16 criterios está en el [catálogo de alternativas](../product/analogues.tsv).

## Compatibilidad de plataformas y limitaciones

- macOS: la aplicación está firmada y notarizada; las notificaciones funcionan,
- Windows y Linux: hay builds disponibles; el soporte evoluciona según los comentarios de los usuarios,
- Las notificaciones de la aplicación de escritorio solo están disponibles por ahora en macOS,
- Algunas fuentes locales de Codex y Claude pueden no funcionar aún en Windows y Linux (las fuentes a través de CLI funcionan en todas partes).

## Licencia

[Licencia MIT](../../LICENSE)
