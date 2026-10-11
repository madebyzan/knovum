import type { Resource } from "@tauri-apps/api/core";
import {
  CheckMenuItem,
  IconMenuItem,
  Menu,
  MenuItem,
  PredefinedMenuItem,
  Submenu,
  type MenuOptions,
} from "@tauri-apps/api/menu";

type MenuEntries = NonNullable<MenuOptions["items"]>;

/** Own each item resource so its Tauri action channel survives menu construction. */
export async function createMenuResources(entries: MenuEntries) {
  const resources: Resource[] = [];
  try {
    const items = await createItems(entries, resources);
    const menu = await Menu.new({ items });
    resources.push(menu);
    return { menu, close: () => Promise.all(resources.map((resource) => resource.close())) };
  } catch (error) {
    await Promise.allSettled(resources.map((resource) => resource.close()));
    throw error;
  }
}

async function createItems(entries: MenuEntries, resources: Resource[]): Promise<MenuEntries> {
  // Settle every creation before cleanup can run, including failed sibling branches.
  const creations = await Promise.allSettled(entries.map((entry) => createItem(entry, resources)));
  return creations.map((creation) => {
    if (creation.status === "rejected") throw creation.reason;
    return creation.value;
  });
}

async function createItem(entry: MenuEntries[number], resources: Resource[]) {
  if ("rid" in entry) return entry;
  let item: MenuItem | Submenu | CheckMenuItem | IconMenuItem | PredefinedMenuItem;
  if ("items" in entry) {
    item = await Submenu.new({ ...entry, items: await createItems(entry.items ?? [], resources) });
  } else if ("item" in entry) item = await PredefinedMenuItem.new(entry);
  else if ("checked" in entry) item = await CheckMenuItem.new(entry);
  else if ("icon" in entry) item = await IconMenuItem.new(entry);
  else item = await MenuItem.new(entry);
  resources.push(item);
  return item;
}
