import { createPinia, setActivePinia } from "pinia";
import { afterEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../../../foundation/i18n/instance";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import type { EnrichmentJobProjection } from "../../contracts/generated/EnrichmentJobProjection";
import { enrichmentTask } from "../job-tasks";

const originalLocale = i18n.global.locale.value;
afterEach(() => {
  i18n.global.locale.value = originalLocale;
});

const record = {
  job: {
    id: "job-1",
    spaceId: "space-1",
    nodeId: "node-1",
    jobType: "analyze_visual",
    status: "pending",
    sourceRevision: "revision-1",
    attempts: 0,
    executionTarget: null,
    error: null,
    createdAt: 1,
    updatedAt: 1,
  },
  spaceTitle: "Research Space",
  nodeTitle: "Cover image",
  nodeKind: "image",
} satisfies EnrichmentJobProjection;

describe("enrichment task projection", () => {
  it("preserves Space and node ownership across actionable states", () => {
    for (const [status, expected] of [
      ["pending", "queued"],
      ["running", "running"],
      ["awaiting_configuration", "attention"],
      ["failed", "failed"],
    ] as const) {
      const task = enrichmentTask({ ...record, job: { ...record.job, status } });
      expect(task).toMatchObject({
        id: "job-1",
        status: expected,
        scope: { kind: "space", id: "space-1", label: "Research Space" },
        subject: { kind: "node", id: "node-1", label: "Cover image" },
      });
      expect(task.title).not.toContain("spaces.processing.enrichmentJobType");
    }
  });

  it("keeps the persisted failure reason", () => {
    const task = enrichmentTask({
      ...record,
      job: { ...record.job, status: "failed", error: "Model missing" },
    });
    expect(task).toMatchObject({ status: "failed", statusMessage: "Model missing" });
  });

  it("localizes enrichment task titles and node types across locales", () => {
    const failedPdf = {
      ...record,
      nodeKind: "file",
      job: {
        ...record.job,
        jobType: "parse_document" as const,
        status: "failed" as const,
        error: "PDF exceeds the 2000-visual processing budget",
      },
    };
    i18n.global.locale.value = "zh-CN";
    expect(enrichmentTask(failedPdf)).toMatchObject({
      title: "文档内容提取",
      subject: { typeLabel: "文件" },
    });

    i18n.global.locale.value = "en";
    expect(enrichmentTask(failedPdf)).toMatchObject({
      title: "Document extraction",
      subject: { typeLabel: "File" },
    });
  });

  it("gets actions and durability from registered observation handlers", () => {
    setActivePinia(createPinia());
    const tasks = useTaskStore();
    const cancel = vi.fn<() => Promise<void>>(async () => {});
    const retry = vi.fn<() => Promise<void>>(async () => {});
    const open = vi.fn<() => Promise<void>>(async () => {});
    tasks.reconcileObserved("spaces-enrichment", [{ task: enrichmentTask(record), cancel, open }]);
    expect(tasks.tasks[0]).toMatchObject({
      status: "queued",
      producer: "spaces-enrichment",
      durable: true,
      cancellable: true,
      openable: true,
    });

    tasks.reconcileObserved("spaces-enrichment", [
      {
        task: enrichmentTask({ ...record, job: { ...record.job, status: "failed", error: "Model missing" } }),
        retry,
        open,
      },
    ]);
    expect(tasks.tasks[0]).toMatchObject({ status: "failed", cancellable: false, retryable: true });
  });
});
