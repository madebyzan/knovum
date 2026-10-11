import type { MenuItemOptions, MenuOptions, SubmenuOptions } from "@tauri-apps/api/menu";
import { beforeEach, expect, it, vi } from "vitest";

interface ItemResource {
  rid: number;
  close(): Promise<void>;
}
const menuMocks = vi.hoisted(() => ({
  create: vi.fn<(options: MenuOptions) => Promise<ReturnType<typeof menu>>>(),
  createItem: vi.fn<(options: MenuItemOptions) => Promise<ItemResource>>(),
  createSubmenu: vi.fn<(options: SubmenuOptions) => Promise<ItemResource>>(),
  log: vi.fn<(message: string) => Promise<void>>(),
}));
vi.mock("@tauri-apps/api/menu", () => ({
  Menu: { new: menuMocks.create },
  MenuItem: { new: menuMocks.createItem },
  CheckMenuItem: { new: menuMocks.createItem },
  IconMenuItem: { new: menuMocks.createItem },
  PredefinedMenuItem: { new: menuMocks.createItem },
  Submenu: { new: menuMocks.createSubmenu },
}));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: menuMocks.log }));

beforeEach(() => {
  vi.resetModules();
  vi.resetAllMocks();
  menuMocks.createItem.mockImplementation(async () => ({
    rid: 1,
    close: vi.fn<() => Promise<void>>().mockResolvedValue(),
  }));
  menuMocks.createSubmenu.mockImplementation(async () => ({
    rid: 2,
    close: vi.fn<() => Promise<void>>().mockResolvedValue(),
  }));
});

function menu() {
  return {
    popup: vi.fn<() => Promise<void>>().mockResolvedValue(),
    close: vi.fn<() => Promise<void>>().mockResolvedValue(),
  };
}

it("retains the native action callbacks after popup resolves on display", async () => {
  const resource = menu();
  menuMocks.create.mockResolvedValue(resource);
  const { menuItem, popupContextMenu } = await import("../popup");
  let renamed = false;
  await popupContextMenu([
    menuItem("Rename", () => {
      renamed = true;
    }),
  ]);
  expect(resource.close).not.toHaveBeenCalled();
  const item = menuMocks.createItem.mock.calls[0]![0];
  item.action!("rename");
  await vi.waitFor(() => expect(renamed).toBe(true));
});

it("releases the previous popup when replacing it", async () => {
  const previous = menu();
  const current = menu();
  menuMocks.create.mockResolvedValueOnce(previous).mockResolvedValueOnce(current);
  const { popupContextMenu } = await import("../popup");
  await popupContextMenu([{ text: "First" }]);
  await popupContextMenu([{ text: "Second" }]);
  expect(previous.close).toHaveBeenCalledOnce();
  expect(current.close).not.toHaveBeenCalled();
});

it("retains explicit resources for submenu actions until their popup is replaced", async () => {
  menuMocks.create.mockImplementation(async () => menu());
  const { menuItem, popupContextMenu } = await import("../popup");
  await popupContextMenu([{ text: "Move", items: [menuItem("Folder", () => {})] }]);
  expect(menuMocks.createItem).toHaveBeenCalledOnce();
  const item = await menuMocks.createItem.mock.results[0]!.value;
  const submenu = await menuMocks.createSubmenu.mock.results[0]!.value;
  expect(item.close).not.toHaveBeenCalled();
  expect(submenu.close).not.toHaveBeenCalled();
  await popupContextMenu([{ text: "Next" }]);
  expect(item.close).toHaveBeenCalledOnce();
  expect(submenu.close).toHaveBeenCalledOnce();
});

it("releases a failed popup and reports its error", async () => {
  const resource = menu();
  resource.popup.mockRejectedValue(new Error("Popup failed"));
  menuMocks.create.mockResolvedValue(resource);
  const { popupContextMenu } = await import("../popup");
  await popupContextMenu([{ text: "Rename" }]);
  expect(resource.close).toHaveBeenCalledOnce();
  expect(menuMocks.log).toHaveBeenCalledWith(expect.stringContaining("menu popup failed"));
});

it("settles sibling creations before releasing a partially constructed menu", async () => {
  const first = { rid: 1, close: vi.fn<() => Promise<void>>().mockResolvedValue() };
  const last = { rid: 3, close: vi.fn<() => Promise<void>>().mockResolvedValue() };
  let finish!: (resource: ItemResource) => void;
  menuMocks.createItem
    .mockResolvedValueOnce(first)
    .mockRejectedValueOnce(new Error("Item creation failed"))
    .mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve;
      }),
    );
  const { popupContextMenu } = await import("../popup");
  const popup = popupContextMenu([{ text: "First" }, { text: "Failed" }, { text: "Last" }]);
  await vi.waitFor(() => expect(menuMocks.createItem).toHaveBeenCalledTimes(3));
  expect(first.close).not.toHaveBeenCalled();
  finish(last);
  await popup;
  expect(first.close).toHaveBeenCalledOnce();
  expect(last.close).toHaveBeenCalledOnce();
  expect(menuMocks.create).not.toHaveBeenCalled();
  expect(menuMocks.log).toHaveBeenCalledWith(expect.stringContaining("Item creation failed"));
});
