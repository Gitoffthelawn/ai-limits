// Equalizes Limits/Plan section slot heights across provider cards so cards in
// a row align even when one card has fewer lines. See
// docs/desktop/ui/provider-blocks.md#section-slot-alignment.

const SECTION_SLOT_KINDS = ["limits", "plan"];

let providerList = null;
let sectionSlotAlignmentFrame = 0;

export function initSectionSlotAlignment(list) {
  providerList = list;
}

function applySectionSlotAlignment() {
  sectionSlotAlignmentFrame = 0;

  if (!providerList) {
    return;
  }

  const sectionsByKind = new Map(SECTION_SLOT_KINDS.map((kind) => [kind, []]));
  const allSections = [];

  // Measuring with the min-height transition still active can read a
  // mid-flight value left over from the previous sync instead of the
  // section's natural content height, producing a stray second jump.
  // Disabling the transition for the reset-and-measure pass keeps the
  // measurement accurate; it's re-enabled next frame so the resulting
  // min-height change still animates normally.
  for (const section of providerList.querySelectorAll(".provider-section[data-section-slot]")) {
    section.style.transition = "none";
    section.style.minHeight = "";
    allSections.push(section);
    const sections = sectionsByKind.get(section.dataset.sectionSlot);
    if (sections) {
      sections.push(section);
    }
  }

  for (const sections of sectionsByKind.values()) {
    if (!sections.length) {
      continue;
    }

    const maxHeight = sections.reduce(
      (height, section) => Math.max(height, section.getBoundingClientRect().height),
      0,
    );

    for (const section of sections) {
      section.style.minHeight = `${maxHeight}px`;
    }
  }

  window.requestAnimationFrame(() => {
    for (const section of allSections) {
      section.style.transition = "";
    }
  });
}

export function scheduleSectionSlotAlignment() {
  if (!providerList || sectionSlotAlignmentFrame) {
    return;
  }

  sectionSlotAlignmentFrame = window.requestAnimationFrame(applySectionSlotAlignment);
}
