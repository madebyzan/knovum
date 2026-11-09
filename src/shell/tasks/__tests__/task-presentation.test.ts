import { describe, expect, it } from "vitest";

import type { AppTask } from "../../../foundation/tasks/task-model";
import {
  canCancelTask,
  isActiveTask,
  isTerminalTask,
  taskStatusClass,
  taskStatusIcon,
  taskStatusIconClass,
  taskSubtitle,
} from "../task-presentation";

describe("task-presentation", () => {
  it("determines active, cancellable, and terminal status flags", () => {
    expect(isActiveTask("queued")).toBe(true);
    expect(isActiveTask("running")).toBe(true);
    expect(isActiveTask("failed")).toBe(false);

    expect(canCancelTask("queued")).toBe(true);
    expect(canCancelTask("running")).toBe(true);
    expect(canCancelTask("attention")).toBe(true);
    expect(canCancelTask("failed")).toBe(false);

    expect(isTerminalTask("succeeded")).toBe(true);
    expect(isTerminalTask("cancelled")).toBe(true);
    expect(isTerminalTask("running")).toBe(false);
  });

  it("assigns appropriate status classes and icons", () => {
    expect(taskStatusClass("failed")).toBe("text-fg-danger");
    expect(taskStatusClass("succeeded")).toBe("text-fg-success");
    expect(taskStatusClass("running")).toBe("text-fg-accent");
    expect(taskStatusClass("queued")).toBe("text-fg-tertiary");

    expect(taskStatusIconClass("succeeded")).toBe("text-fg-success");
    expect(taskStatusIconClass("failed")).toBe("text-fg-danger");
    expect(taskStatusIconClass("attention")).toBe("text-fg-secondary");
    expect(taskStatusIconClass("cancelled")).toBe("text-fg-secondary");
    expect(taskStatusIconClass("queued")).toBe("text-fg-tertiary");

    expect(taskStatusIcon("queued")).toBeDefined();
    expect(taskStatusIcon("attention")).toBeDefined();
    expect(taskStatusIcon("succeeded")).toBeDefined();
    expect(taskStatusIcon("cancelled")).toBeDefined();
    expect(taskStatusIcon("failed")).toBeDefined();
  });

  it("formats task subtitle from scope and subject", () => {
    const fullTask: AppTask = {
      id: "1",
      title: "Dubbing",
      status: "running",
      startedAt: 1,
      scope: { kind: "space", id: "s1", label: "My Space" },
      subject: { kind: "node", id: "n1", label: "interview.mp4", typeLabel: "Video" },
    };
    expect(taskSubtitle(fullTask)).toBe("My Space · Video · interview.mp4");

    const subjectOnlyTask: AppTask = {
      id: "2",
      title: "Audio",
      status: "queued",
      startedAt: 1,
      subject: { kind: "node", id: "n2", label: "podcast.mp3" },
    };
    expect(taskSubtitle(subjectOnlyTask)).toBe("podcast.mp3");

    const scopeOnlyTask: AppTask = {
      id: "3",
      title: "Analysis",
      status: "succeeded",
      startedAt: 1,
      scope: { kind: "space", id: "s1", label: "Workspace" },
    };
    expect(taskSubtitle(scopeOnlyTask)).toBe("Workspace");

    const emptyTask: AppTask = {
      id: "4",
      title: "Global Task",
      status: "running",
      startedAt: 1,
    };
    expect(taskSubtitle(emptyTask)).toBeUndefined();
  });
});
