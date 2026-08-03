import { EXTERNAL_LINKS } from "./constants.js";

export async function openExternalUrl(url) {
  if (!url) {
    return;
  }

  const invoke = window.__TAURI__?.core?.invoke;
  if (invoke) {
    await invoke("open_external_url", { url });
    return;
  }

  window.open(url, "_blank", "noopener,noreferrer");
}

export async function openExternalSetup(linkId) {
  return openExternalUrl(EXTERNAL_LINKS[linkId]);
}
