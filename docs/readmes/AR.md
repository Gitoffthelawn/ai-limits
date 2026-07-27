# ai-limits

<p align="center">| [DE](DE.md) | [EN](../../README.md) | [ES](ES.md) | [FR](FR.md) | [PT](PT.md) | [RU](RU.md) | [中文](ZH.md) | عربي |</p>

---

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-macos-app.zip"><img src="https://shieldcn.dev/badge/macOS-v0.0.13-grey.svg?logo=apple" alt="تنزيل لنظام macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.0.13-blue.svg?logo=windows" alt="تنزيل لنظام Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.0.13-yellow.svg?logo=linux" alt="تنزيل لنظام Linux"></a>
</p>

---

<div dir="rtl">

تطبيق محلي لمراقبة حدود واستهلاك اشتراكات الذكاء الاصطناعي في Codex وClaude وCursor.

المزايا:
- يعمل دون اشتراك في واجهة برمجة التطبيقات (API)،
- بلا تسجيل دخول،
- جميع المزوّدين في مكان واحد،
- مجاني بالكامل،
- يحافظ على الخصوصية. بلا خدمات طرف ثالث أو وكلاء أو تسجيلات،
- تطبيق سطح مكتب خفيف لنظام Mac وWindows وLinux،
- إشعارات بالحدود،
- مفتوح المصدر.

</div>

![ai-limits macOS](macos.png)

<p align="center">
  <img src="windows.png" alt="ai-limits على Windows" width="24%">
  <img src="linux.png" alt="ai-limits على Linux" width="24%">
  <img src="macos-light-settings.png" alt="إعدادات ai-limits" width="24%">
  <img src="macos-help.png" alt="مساعدة ai-limits" width="24%">
</p>

<div dir="rtl">

## الميزات

- يعرض الحدود وتاريخ ووقت إعادة التعيين والرموز المتاحة وإعادة التعيين اليدوية المتاحة،
- يعمل مع Codex وClaude وCursor،
- يستخرج البيانات من الملفات المحلية وواجهات سطر أوامر المزوّدين وواجهات برمجة التطبيقات،
- منطق احتياطي: إذا تعذّر الوصول إلى مصدر، يتحقق من مصدر آخر،
- تطبيق سطح مكتب خفيف (macOS وWindows وLinux)،
- واجهة سطر أوامر بعدة صيغ إخراج: `./bin/ai-limits`،
- إشعارات النظام عند بلوغ عتبات الحدود،
- تحديث يدوي وتلقائي مرن للبيانات.

## البدائل

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| تطبيق سطح مكتب وواجهة سطر أوامر | ✅ | ✅ | ❌ |
| Codex وClaude وCursor | ✅ | ✅ | ✅ |
| macOS وWindows وLinux | ✅ | ❌ | ❌ |
| بلا خدمة وسيطة | ✅ | ✅ | ✅ |

المقارنة الكاملة لـ 18 بديلاً و16 معياراً متوفرة في [كتالوج البدائل](../product/analogues.tsv).

## دعم المنصات والقيود

- macOS: التطبيق موقّع ومُوثَّق؛ والإشعارات تعمل،
- Windows وLinux: الإصدارات متاحة؛ ويتطوّر الدعم استناداً إلى ملاحظات المستخدمين،
- إشعارات تطبيق سطح المكتب متاحة حالياً على macOS فقط،
- قد لا تعمل بعض المصادر المحلية لـ Codex وClaude على Windows وLinux بعد (المصادر عبر واجهة سطر الأوامر تعمل في كل مكان).

## الترخيص

[رخصة MIT](../../LICENSE)

</div>
