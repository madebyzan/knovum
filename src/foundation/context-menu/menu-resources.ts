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

type MenuItemDefinitions = NonNullable<MenuOptions["items"]>;

/** Own each item resource so its Tauri action channel survives menu construction. */
export async function createMenuResources(definitions: MenuItemDefinitions) {
  const resources: Resource[] = [];
  try {
    const items = await createItems(definitions, resources);
    const menu = await Menu.new({ items });
    resources.push(menu);
    return { menu, close: () => Promise.all(resources.map((resource) => resource.close())) };
  } catch (error) {
    await Promise.allSettled(resources.map((resource) => resource.close()));
    throw error;
  }
}

async function createItems(
  definitions: MenuItemDefinitions,
  resources: Resource[],
): Promise<MenuItemDefinitions> {
  // Settle every creation before cleanup can run, including failed sibling branches.
  const creations = await Promise.allSettled(
    definitions.map((definition) => createItem(definition, resources)),
  );
  return creations.map((creation) => {
    if (creation.status === "rejected") throw creation.reason;
    return creation.value;
  });
}

async function createItem(definition: MenuItemDefinitions[number], resources: Resource[]) {
  if ("rid" in definition) return definition;
  let item: MenuItem | Submenu | CheckMenuItem | IconMenuItem | PredefinedMenuItem;
  if ("items" in definition) {
    item = await Submenu.new({ ...definition, items: await createItems(definition.items ?? [], resources) });
  } else if ("item" in definition) item = await PredefinedMenuItem.new(definition);
  else if ("checked" in definition) item = await CheckMenuItem.new(definition);
  else if ("icon" in definition) item = await IconMenuItem.new(definition);
  else item = await MenuItem.new(definition);
  resources.push(item);
  return item;
}
