import type { AppCommand } from "../../../shared/contracts/app-command";
import type { CanvasExchangePort } from "../canvas-exchange/canvas-exchange-port";
import { exportJsonCanvasFile, importJsonCanvasFile } from "../canvas-exchange/transfer";

interface CanvasExchangeCommandInput {
  exchange(): CanvasExchangePort | undefined;
  spaceTitle(): string | undefined;
  importTitle: string;
  exportTitle: string;
}

export function createCanvasExchangeCommands(input: CanvasExchangeCommandInput): AppCommand[] {
  return [
    {
      id: "spaces.import-json-canvas",
      title: input.importTitle,
      category: "canvas",
      run: async () => {
        const exchange = input.exchange();
        if (!exchange) throw new Error("Canvas exchange is not available");
        await importJsonCanvasFile(exchange);
      },
    },
    {
      id: "spaces.export-json-canvas",
      title: input.exportTitle,
      category: "canvas",
      run: async () => {
        const exchange = input.exchange();
        const title = input.spaceTitle();
        if (!exchange || !title) throw new Error("Canvas exchange is not available");
        await exportJsonCanvasFile(exchange, title);
      },
    },
  ];
}
