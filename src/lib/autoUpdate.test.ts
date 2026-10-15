import { describe, expect, it } from "vitest";

// Le module importe les plugins Tauri : on teste la règle pure via un import dynamique mocké.
import { vi } from "vitest";
vi.mock("@tauri-apps/plugin-updater", () => ({ check: vi.fn() }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: vi.fn() }));

describe("shouldCheckAtStartup", () => {
  it("vérifie seulement hors dev et si le réglage est actif", async () => {
    const { shouldCheckAtStartup } = await import("./autoUpdate");
    expect(shouldCheckAtStartup(false, true)).toBe(true);
    expect(shouldCheckAtStartup(true, true)).toBe(false);
    expect(shouldCheckAtStartup(false, false)).toBe(false);
  });
});
