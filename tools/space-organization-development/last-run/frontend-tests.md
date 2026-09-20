# Frontend tests

Source: `338a721ed95500572ba8288605180a0e93e2d122`. Exit code: 1.

```text

 RUN  v4.1.10 /home/runner/work/knovum/knovum

 ✓ src/features/spaces/__tests__/capture-url.test.ts (9 tests) 50ms
 ❯ src/features/spaces/__tests__/konva-scene-projection.test.ts (6 tests | 6 failed) 65ms
     × always paints section backgrounds behind their members 37ms
     × projects collapsed sections without mutating stored geometry or rendering members 1ms
     × projects one identity for each semantic relation 4ms
     × projects parallel relations onto stable distinct curve lanes 4ms
     × projects external relations from collapsed members onto their section boundary 5ms
     × keeps hidden layout data out of the rendered scene 1ms
stderr | src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > does not install a stale load after the active space changes
[Vue warn]: Missing required prop: "interactionEnabled" 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction: onDeleteSelection],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > does not install a stale load after the active space changes
[Vue warn]: Missing required prop: "interactionEnabled" 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction: onDeleteSelection],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > does not install a stale load after the active space changes
[Vue warn]: Unhandled error during execution of render function 
  at <KonvaCanvasSurface ref="surface" key=1 document= {
  version: 1,
  structure: { version: 1, items: [ [Object] ] },
  layout: { version: 1, items: [ [Object] ] },
  viewport: { x: 0, y: 0, zoom: 1 }
}  ... > 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction: onDeleteSelection],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>
[Vue warn]: Unhandled error during execution of component update 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction: onDeleteSelection],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > rejects an in-flight commit after the active space changes
[Vue warn]: Missing required prop: "interactionEnabled" 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction (anonymous)],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > rejects an in-flight commit after the active space changes
[Vue warn]: Unhandled error during execution of render function 
  at <KonvaCanvasSurface ref="surface" key=1 document= {
  version: 1,
  structure: { version: 1, items: [ [Object] ] },
  layout: { version: 1, items: [ [Object] ] },
  viewport: { x: 0, y: 0, zoom: 1 }
}  ... > 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction (anonymous)],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>
[Vue warn]: Unhandled error during execution of component update 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction (anonymous)],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > rejects an in-flight commit after unmount
[Vue warn]: Missing required prop: "interactionEnabled" 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction (anonymous)],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > preserves an in-flight viewport change without replacing committed node frames
[Vue warn]: Missing required prop: "interactionEnabled" 
  at <CanvasSurfaceHost events= {
  onCommitGesture: [AsyncFunction: onCommitGesture],
  onCopyNodes: [AsyncFunction: onCopyNodes],
  onCreateRelation: [AsyncFunction: onCreateRelation],
  onCreateTextNode: [AsyncFunction: onCreateTextNode],
  onCutSelection: [AsyncFunction: onCutSelection],
  onDeleteSelection: [AsyncFunction (anonymous)],
  onDuplicateNodes: [AsyncFunction: onDuplicateNodes],
  onInsertText: [AsyncFunction: onInsertText],
  onInsertUrl: [AsyncFunction: onInsertUrl],
  onImportFiles: [AsyncFunction: onImportFiles],
  onNodeActivate: [AsyncFunction: onNodeActivate],
  onPasteNodes: [AsyncFunction: onPasteNodes],
  onError: [Function: onError],
  onHistoryEntry: [Function: onHistoryEntry],
  onHistoryTransaction: [AsyncFunction: onHistoryTransaction],
  onMutationPendingChange: [Function: onMutationPendingChange],
  onDirectManipulationChange: [Function: onDirectManipulationChange],
  onReady: [Function: onReady],
  onSelectionChange: [Function: onSelectionChange],
  onViewportChange: [Function: onViewportChange]
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

 ✓ src/foundation/commands/__tests__/command-shortcuts.test.ts (11 tests) 69ms
 ❯ src/features/spaces/__tests__/canvas-surface-host.test.ts (4 tests | 4 failed) 153ms
     × does not install a stale load after the active space changes 105ms
     × rejects an in-flight commit after the active space changes 19ms
     × rejects an in-flight commit after unmount 9ms
     × preserves an in-flight viewport change without replacing committed node frames 11ms
 ✓ src/features/spaces/__tests__/structure-commands.test.ts (11 tests) 149ms
 ❯ src/features/spaces/__tests__/canvas-gesture-session.test.ts (6 tests | 5 failed) 130ms
     ✓ owns the direct-manipulation lifecycle independently from persistence busy state 58ms
     × commits keyboard movement without implicitly changing Section membership 21ms
     × serializes rapid keyboard nudges from the latest committed document 14ms
     × calculates queued frame mutations from the latest committed document 12ms
     × keeps geometry interaction busy while an asynchronous frame calculation is pending 13ms
     × keeps a queued nudge bound to the selection that received the command 7ms
stderr | src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > persists reconciliation instead of leaving it as an in-memory repair
[Vue warn]: Missing required prop: "interactionEnabled" 
  at <CanvasSurfaceHost events= {
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > persists reconciliation instead of leaving it as an in-memory repair
[Vue warn]: Unhandled error during execution of render function 
  at <KonvaCanvasSurface ref="surface" key=1 document= {
  version: 1,
  structure: { version: 1, items: [ [Object] ] },
  layout: { version: 1, items: [ [Object] ] },
  viewport: { x: 0, y: 0, zoom: 1 }
}  ... > 
  at <CanvasSurfaceHost events= {
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>
[Vue warn]: Unhandled error during execution of component update 
  at <CanvasSurfaceHost events= {
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > waits for the previous A flush across a keyed A to B to A remount
[Vue warn]: Missing required prop: "interactionEnabled" 
  at <CanvasSurfaceHost events= {
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > waits for the previous A flush across a keyed A to B to A remount
[Vue warn]: Unhandled error during execution of render function 
  at <KonvaCanvasSurface ref="surface" key=1 document= {
  version: 1,
  structure: { version: 1, items: [ [Object] ] },
  layout: { version: 1, items: [ [Object] ] },
  viewport: { x: 0, y: 0, zoom: 1 }
}  ... > 
  at <CanvasSurfaceHost events= {
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>
[Vue warn]: Unhandled error during execution of component update 
  at <CanvasSurfaceHost events= {
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > waits for an in-flight formal commit before remounting the same space
[Vue warn]: Missing required prop: "interactionEnabled" 
  at <CanvasSurfaceHost events= {
  onDeleteSelection: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  },
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > waits for an in-flight formal commit before remounting the same space
[Vue warn]: Unhandled error during execution of render function 
  at <KonvaCanvasSurface ref="surface" key=1 document= {
  version: 1,
  structure: { version: 1, items: [ [Object] ] },
  layout: { version: 1, items: [ [Object] ] },
  viewport: { x: 0, y: 0, zoom: 1 }
}  ... > 
  at <CanvasSurfaceHost events= {
  onDeleteSelection: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  },
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>
[Vue warn]: Unhandled error during execution of component update 
  at <CanvasSurfaceHost events= {
  onDeleteSelection: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  },
  onError: [Function: Mock] {
    _isMockFunction: true,
    getMockImplementation: [Function (anonymous)],
    mock: {
      calls: [],
      contexts: [],
      instances: [],
      invocationCallOrder: [],
      settledResults: [],
      results: [],
      lastCall: [Getter]
    },
    mockImplementation: [Function: mockImplementation],
    mockImplementationOnce: [Function: mockImplementationOnce],
    withImplementation: [Function: withImplementation],
    mockReturnThis: [Function: mockReturnThis],
    mockReturnValue: [Function: mockReturnValue],
    mockReturnValueOnce: [Function: mockReturnValueOnce],
    mockThrow: [Function: mockThrow],
    mockThrowOnce: [Function: mockThrowOnce],
    mockResolvedValue: [Function: mockResolvedValue],
    mockResolvedValueOnce: [Function: mockResolvedValueOnce],
    mockRejectedValue: [Function: mockRejectedValue],
    mockRejectedValueOnce: [Function: mockRejectedValueOnce],
    mockClear: [Function: mockClear],
    mockReset: [Function: mockReset],
    mockRestore: [Function: mockRestore],
    mockName: [Function: mockName],
    getMockName: [Function: getMockName],
    [Symbol(nodejs.dispose)]: [Function (anonymous)]
  }
} importJsonCanvas=fn<Mock> focusNodeIds= []  ... > 
  at <VTUROOT>

 ❯ src/features/spaces/__tests__/canvas-surface-remount.test.ts (3 tests | 3 failed) 232ms
     × persists reconciliation instead of leaving it as an in-memory repair 155ms
     × waits for the previous A flush across a keyed A to B to A remount 35ms
     × waits for an in-flight formal commit before remounting the same space 27ms
 ❯ src/features/spaces/__tests__/konva-editor-port.test.ts (2 tests | 2 failed) 25ms
     × routes clipboard operations through product ports and uses the last canvas pointer 13ms
     × commits node and semantic relation deletion through one host transaction 9ms
 ❯ src/features/spaces/__tests__/node-size.test.ts (22 tests | 3 failed) 32ms
     ✓ gives note a semantic desktop default 4ms
     ✓ gives text a semantic desktop default 0ms
     ✓ gives markdown a semantic desktop default 0ms
     ✓ gives website a semantic desktop default 0ms
     ✓ gives audio a semantic desktop default 1ms
     ✓ gives file a semantic desktop default 0ms
     ✓ gives code a semantic desktop default 0ms
     ✓ gives credential a semantic desktop default 0ms
     × gives section a semantic desktop default 6ms
     ✓ gives object a semantic desktop default 1ms
     ✓ gives portal a semantic desktop default 0ms
     ✓ fits standard and long-form image ratios without cropping or distortion 1ms
     ✓ reserves stable desktop chrome around video presentation geometry 1ms
     ✓ sizes a rendered document card from the page's own ratio 1ms
     ✓ keeps the page area inside the card true to the rendered ratio 0ms
     ✓ uses every available system-preview ratio and leaves unrendered documents at the default 1ms
     × uses the rendered compact geometry for collapsed sections 4ms
     ✓ keeps compact utility nodes fixed 0ms
     ✓ keeps compact utility nodes fixed while visual cards remain resizable 1ms
     ✓ does not impose a presentation cap on resizable nodes 1ms
     × keeps document and section geometry independently resizable 3ms
     ✓ derives initial visual dimensions only from intrinsic content 0ms
 ❯ src/features/spaces/__tests__/konva-node-gesture.test.ts (3 tests | 3 failed) 50ms
     × moves a section and its domain members in one frame commit 20ms
     × rejects the complete drag when a selected operation root is locked 2ms
     × normalizes the native Transformer scale when the gesture ends 17ms
 ✓ src/features/spaces/__tests__/canvas-snapshot-save-queue.test.ts (5 tests) 158ms
 ❯ src/features/spaces/__tests__/canvas-persistence-history.test.ts (3 tests | 1 failed) 106ms
     × writes the clipboard before one atomic cut command 21ms
     ✓ replays delete history with one persistence command per transition 5ms
     ✓ replays import history with one persistence command per transition 75ms
 ❯ src/features/spaces/__tests__/konva-canvas-document.test.ts (6 tests | 6 failed) 67ms
     × creates canonical frames from domain nodes 18ms
     × adopts intrinsic media geometry until the persisted frame becomes authoritative 35ms
     × preserves authored frames while adding and removing domain nodes 2ms
     × rejects content authority on nodes without content-sized geometry 1ms
     × preserves document and layout identities when domain geometry is unchanged 1ms
     × keeps viewport changes made after an import when undoing imported content 2ms
 ✓ src/features/spaces/__tests__/json-canvas-mapping.test.ts (6 tests) 62ms
 ✓ src/features/spaces/__tests__/konva-rich-node-content.test.ts (6 tests) 229ms
 ✓ src/features/spaces/__tests__/canvas-session-commit-guard.test.ts (2 tests) 75ms
 ❯ src/features/spaces/__tests__/canvas-document-workspace.test.ts (4 tests | 1 failed) 264ms
     ✓ keeps a successful database acknowledgement when its renderer is detached 79ms
     ✓ rejects queued old-Space work before a second database call 56ms
     ✓ does not make a successor's camera wait on the old Space transaction 56ms
     × persists the captured camera at the committed revision even after detachment 67ms
 ✓ src/features/spaces/__tests__/archive-actions.test.ts (6 tests) 51ms
 ✓ src/features/spaces/__tests__/canvas-clipboard-paste.test.ts (9 tests) 39ms
 ✓ src/features/spaces/__tests__/canvas-node-interaction.test.ts (7 tests) 21ms
 ❯ src/features/spaces/__tests__/canvas-document.test.ts (9 tests | 5 failed) 47ms
     × accepts engine-neutral layout state without domain content 25ms
     ✓ rejects engine snapshots and copied domain content 3ms
     × rejects duplicate nodes 5ms
     ✓ requires explicit size ownership instead of inferring it from dimensions 1ms
     × uses the same unsigned 32-bit layer range as the Rust contract 1ms
     ✓ rejects duplicated domain membership and lock state 1ms
     × accepts only the current semantic appearance contract 1ms
     ✓ rejects semantic relations embedded in the spatial document 1ms
     × stores only non-default relation presentation overrides 6ms
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts (8 tests | 8 failed) 62ms
     × aligns two Sections and both member sets through the real editor port 31ms
     × does not realign a selected member separately from its Section 11ms
     × moves multiple Sections with keyboard input and does not rewrite memberships 3ms
     × keeps locked descendants attached to an unlocked Section for keyboard input 1ms
     × rejects an entire nudge against a locked operation root 1ms
     × keeps fit separate from moving a Section and does not adopt overlapping content 7ms
     × never sends an optimistic gesture or a rejected gesture to the independent autosave path 3ms
     × records and undoes the entire aligned operation while preserving a newer camera 2ms
 ✓ src/features/spaces/__tests__/tiptap-content.test.ts (7 tests) 274ms
 ❯ src/features/spaces/__tests__/space-session-actions.test.ts (7 tests | 5 failed) 15066ms
     ✓ creates a valid default space and session context 9ms
     ✓ does not reload the already active space 11ms
     × coalesces repeated clicks while the same space is loading 24ms
     × keeps the current canvas snapshot visible until the new Space is ready 5003ms
     × cancels a pending switch when the user returns to the active Space 9ms
     × reuses one session draft without persisting repeated new-Space actions 5005ms
     × persists the draft exactly once before its first content write 5002ms
 ✓ src/features/spaces/__tests__/derivation-action-catalog.test.ts (6 tests) 23ms
stderr | src/features/spaces/__tests__/konva-rich-node-interactions.test.ts > Konva rich node interactions > persists direct note editing and refuses editing for locked notes
[Vue warn]: Missing required prop: "verticalClass" 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[Vue warn]: Missing required prop: "fontSize" 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[Vue warn]: Missing required prop: "textAlign" 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>

stderr | src/features/spaces/__tests__/konva-rich-node-interactions.test.ts > Konva rich node interactions > persists direct note editing and refuses editing for locked notes
[Vue warn]: Missing required prop: "verticalClass" 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[Vue warn]: Missing required prop: "fontSize" 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[Vue warn]: Missing required prop: "textAlign" 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[Vue warn]: Invalid prop: type check failed for prop "defaultFontSize". Expected Number with value NaN, got Undefined  
  at <SpaceCanvasTextSelectionBar key=0 editor= <ref *4> Editor {
  callbacks: {
    create: [ [Function: bound onCreate], [Function: onCreate] ],
    destroy: [ [Function: bound onDestroy], [Function: onDestroy] ],
    update: [ [Function: bound onUpdate], [Function: onUpdate] ],
    beforeCreate: [ [Function: onBeforeCreate] ],
    mount: [ [Function: onMount] ],
    unmount: [ [Function: onUnmount] ],
    contentError: [ [Function: onContentError] ],
    selectionUpdate: [ [Function: onSelectionUpdate] ],
    transaction: [ [Function: onTransaction] ],
    focus: [ [Function: onFocus] ],
    blur: [ [Function: onBlur] ],
    drop: [ [Function (anonymous)] ],
    paste: [ [Function (anonymous)] ],
    delete: [ [Function: onDelete] ],
    beforeTransaction: [ [Function (anonymous)], [Function (anonymous)] ]
  },
  css: <ref *2> HTMLStyleElement {
    [Symbol(listeners)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(listenerOptions)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(isConnected)]: true,
    [Symbol(parentNode)]: HTMLHeadElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: [HTMLHtmlElement],
      [Symbol(rootNode)]: [HTMLDocument],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'HEAD',
      [Symbol(localName)]: 'head',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    [Symbol(rootNode)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(styleNode)]: [Circular *2],
    [Symbol(textAreaNode)]: null,
    [Symbol(formNode)]: null,
    [Symbol(selectNode)]: null,
    [Symbol(mutationListeners)]: [],
    [Symbol(nodeArray)]: [ [Text] ],
    [Symbol(elementArray)]: [],
    [Symbol(childNodes)]: null,
    [Symbol(assignedToSlot)]: null,
    [Symbol(cache)]: {
      querySelector: Map(0) {},
      querySelectorAll: Map(0) {},
      matches: Map(0) {},
      elementsByTagName: Map(0) {},
      elementsByTagNameNS: Map(0) {},
      elementByTagName: Map(0) {},
      elementById: Map(0) {},
      computedStyle: null
    },
    [Symbol(affectsCache)]: [],
    [Symbol(ownerDocument)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(window)]: <ref *3> GlobalWindow [EventTarget] {
      Node: [Function],
      Attr: [class Attr extends Node],
      ShadowRoot: [class ShadowRoot extends DocumentFragment],
      ProcessingInstruction: [class ProcessingInstruction extends CharacterData],
      Element: [Function],
      CharacterData: [class CharacterData extends Node],
      DocumentType: [class DocumentType extends Node],
      AnimationTimeline: [class AnimationTimeline],
      DocumentTimeline: [class DocumentTimeline extends AnimationTimeline],
      HTMLAnchorElement: [class HTMLAnchorElement extends HTMLElement],
      HTMLButtonElement: [class HTMLButtonElement extends HTMLElement],
      HTMLOptGroupElement: [class HTMLOptGroupElement extends HTMLElement],
      HTMLOptionElement: [class HTMLOptionElement extends HTMLElement],
      HTMLElement: [Function],
      HTMLUnknownElement: [class HTMLUnknownElement extends HTMLElement],
      HTMLTemplateElement: [class HTMLTemplateElement extends HTMLElement],
      HTMLInputElement: [class HTMLInputElement extends HTMLElement],
      HTMLSelectElement: [class HTMLSelectElement extends HTMLElement],
      HTMLTextAreaElement: [class HTMLTextAreaElement extends HTMLElement],
      HTMLImageElement: [class HTMLImageElement extends HTMLElement],
      HTMLStyleElement: [class HTMLStyleElement extends HTMLElement],
      HTMLLabelElement: [class HTMLLabelElement extends HTMLElement],
      HTMLSlotElement: [class HTMLSlotElement extends HTMLElement],
      HTMLMetaElement: [class HTMLMetaElement extends HTMLElement],
      HTMLMediaElement: [class HTMLMediaElement extends HTMLElement],
      HTMLAudioElement: [class HTMLAudioElement extends HTMLMediaElement],
      HTMLVideoElement: [class HTMLVideoElement extends HTMLMediaElement],
      HTMLBaseElement: [class HTMLBaseElement extends HTMLElement],
      HTMLDialogElement: [class HTMLDialogElement extends HTMLElement],
      HTMLScriptElement: [class HTMLScriptElement extends HTMLElement],
      HTMLLinkElement: [class HTMLLinkElement extends HTMLElement],
      HTMLIFrameElement: [class HTMLIFrameElement extends HTMLElement],
      HTMLFormElement: [class HTMLFormElement extends HTMLElement],
      HTMLUListElement: [class HTMLUListElement extends HTMLElement],
      HTMLTrackElement: [class HTMLTrackElement extends HTMLElement],
      HTMLTableRowElement: [class HTMLTableRowElement extends HTMLElement],
      HTMLTitleElement: [class HTMLTitleElement extends HTMLElement],
      HTMLTimeElement: [class HTMLTimeElement extends HTMLElement],
      HTMLTableSectionElement: [class HTMLTableSectionElement extends HTMLElement],
      HTMLTableCellElement: [class HTMLTableCellElement extends HTMLElement],
      HTMLTableElement: [class HTMLTableElement extends HTMLElement],
      HTMLSpanElement: [class HTMLSpanElement extends HTMLElement],
      HTMLSourceElement: [class HTMLSourceElement extends HTMLElement],
      HTMLQuoteElement: [class HTMLQuoteElement extends HTMLElement],
      HTMLProgressElement: [class HTMLProgressElement extends HTMLElement],
      HTMLPreElement: [class HTMLPreElement extends HTMLElement],
      HTMLPictureElement: [class HTMLPictureElement extends HTMLElement],
      HTMLParamElement: [class HTMLParamElement extends HTMLElement],
      HTMLParagraphElement: [class HTMLParagraphElement extends HTMLElement],
      HTMLOutputElement: [class HTMLOutputElement extends HTMLElement],
      HTMLOListElement: [class HTMLOListElement extends HTMLElement],
      HTMLObjectElement: [class HTMLObjectElement extends HTMLElement],
      HTMLMeterElement: [class HTMLMeterElement extends HTMLElement],
      HTMLMenuElement: [class HTMLMenuElement extends HTMLElement],
      HTMLMapElement: [class HTMLMapElement extends HTMLElement],
      HTMLLIElement: [class HTMLLIElement extends HTMLElement],
      HTMLLegendElement: [class HTMLLegendElement extends HTMLElement],
      HTMLModElement: [class HTMLModElement extends HTMLElement],
      HTMLHtmlElement: [class HTMLHtmlElement extends HTMLElement],
      HTMLHRElement: [class HTMLHRElement extends HTMLElement],
      HTMLHeadElement: [class HTMLHeadElement extends HTMLElement],
      HTMLHeadingElement: [class HTMLHeadingElement extends HTMLElement],
      HTMLFieldSetElement: [class HTMLFieldSetElement extends HTMLElement],
      HTMLEmbedElement: [class HTMLEmbedElement extends HTMLElement],
      HTMLDListElement: [class HTMLDListElement extends HTMLElement],
      HTMLDivElement: [class HTMLDivElement extends HTMLElement],
      HTMLDetailsElement: [class HTMLDetailsElement extends HTMLElement],
      HTMLDataListElement: [class HTMLDataListElement extends HTMLElement],
      HTMLDataElement: [class HTMLDataElement extends HTMLElement],
      HTMLTableColElement: [class HTMLTableColElement extends HTMLElement],
      HTMLTableCaptionElement: [class HTMLTableCaptionElement extends HTMLElement],
      HTMLCanvasElement: [class HTMLCanvasElement extends HTMLElement],
      HTMLBRElement: [class HTMLBRElement extends HTMLElement],
      HTMLBodyElement: [class HTMLBodyElement extends HTMLElement],
      HTMLAreaElement: [class HTMLAreaElement extends HTMLElement],
      SVGSVGElement: [class SVGSVGElement extends SVGGraphicsElement],
      SVGAnimateElement: [class SVGAnimateElement extends SVGAnimationElement],
      SVGAnimateMotionElement: [class SVGAnimateMotionElement extends SVGAnimationElement],
      SVGAnimateTransformElement: [class SVGAnimateTransformElement extends SVGAnimationElement],
      SVGCircleElement: [class SVGCircleElement extends SVGGeometryElement],
      SVGClipPathElement: [class SVGClipPathElement extends SVGElement],
      SVGDefsElement: [class SVGDefsElement extends SVGGraphicsElement],
      SVGDescElement: [class SVGDescElement extends SVGElement],
      SVGEllipseElement: [class SVGEllipseElement extends SVGGeometryElement],
      SVGFEBlendElement: [Function],
      SVGFEColorMatrixElement: [Function],
      SVGFEComponentTransferElement: [class SVGFEComponentTransferElement extends SVGElement],
      SVGFECompositeElement: [Function],
      SVGFEConvolveMatrixElement: [Function],
      SVGFEDiffuseLightingElement: [class SVGFEDiffuseLightingElement extends SVGElement],
      SVGFEDisplacementMapElement: [Function],
      SVGFEDistantLightElement: [class SVGFEDistantLightElement extends SVGElement],
      SVGFEDropShadowElement: [class SVGFEDropShadowElement extends SVGElement],
      SVGFEFloodElement: [class SVGFEFloodElement extends SVGElement],
      SVGFEFuncAElement: [class SVGFEFuncAElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncBElement: [class SVGFEFuncBElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncGElement: [class SVGFEFuncGElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncRElement: [class SVGFEFuncRElement extends SVGComponentTransferFunctionElement],
      SVGFEGaussianBlurElement: [Function],
      SVGFEImageElement: [class SVGFEImageElement extends SVGElement],
      SVGFEMergeElement: [class SVGFEMergeElement extends SVGElement],
      SVGFEMergeNodeElement: [class SVGFEMergeNodeElement extends SVGElement],
      SVGFEMorphologyElement: [Function],
      SVGFEOffsetElement: [class SVGFEOffsetElement extends SVGElement],
      SVGFEPointLightElement: [class SVGFEPointLightElement extends SVGElement],
      SVGFESpecularLightingElement: [class SVGFESpecularLightingElement extends SVGElement],
      SVGFESpotLightElement: [class SVGFESpotLightElement extends SVGElement],
      SVGFETileElement: [class SVGFETileElement extends SVGElement],
      SVGFETurbulenceElement: [Function],
      SVGFilterElement: [class SVGFilterElement extends SVGElement],
      SVGForeignObjectElement: [class SVGForeignObjectElement extends SVGGraphicsElement],
      SVGGElement: [class SVGGElement extends SVGGraphicsElement],
      SVGImageElement: [class SVGImageElement extends SVGGraphicsElement],
      SVGLineElement: [class SVGLineElement extends SVGGeometryElement],
      SVGLinearGradientElement: [class SVGLinearGradientElement extends SVGGradientElement],
      SVGMarkerElement: [Function],
      SVGMaskElement: [class SVGMaskElement extends SVGElement],
      SVGMetadataElement: [class SVGMetadataElement extends SVGElement],
      SVGMPathElement: [class SVGMPathElement extends SVGElement],
      SVGPathElement: [class SVGPathElement extends SVGGeometryElement],
      SVGPatternElement: [class SVGPatternElement extends SVGElement],
      SVGPolygonElement: [class SVGPolygonElement extends SVGGeometryElement],
      SVGPolylineElement: [class SVGPolylineElement extends SVGGeometryElement],
      SVGRadialGradientElement: [class SVGRadialGradientElement extends SVGGradientElement],
      SVGRectElement: [class SVGRectElement extends SVGGeometryElement],
      SVGScriptElement: [class SVGScriptElement extends SVGGraphicsElement],
      SVGSetElement: [class SVGSetElement extends SVGAnimationElement],
      SVGStopElement: [class SVGStopElement extends SVGElement],
      SVGStyleElement: [class SVGStyleElement extends SVGElement],
      SVGSwitchElement: [class SVGSwitchElement extends SVGGraphicsElement],
      SVGSymbolElement: [class SVGSymbolElement extends SVGGraphicsElement],
      SVGTextElement: [class SVGTextElement extends SVGTextPositioningElement],
      SVGTextPathElement: [Function],
      SVGTitleElement: [class SVGTitleElement extends SVGElement],
      SVGTSpanElement: [class SVGTSpanElement extends SVGTextPositioningElement],
      SVGUseElement: [class SVGUseElement extends SVGGraphicsElement],
      SVGViewElement: [class SVGViewElement extends SVGElement],
      SVGElement: [class SVGElement extends Element],
      SVGAnimationElement: [class SVGAnimationElement extends SVGElement],
      SVGComponentTransferFunctionElement: [Function],
      SVGGeometryElement: [class SVGGeometryElement extends SVGGraphicsElement],
      SVGGradientElement: [Function],
      SVGTextPositioningElement: [class SVGTextPositioningElement extends SVGTextContentElement],
      SVGGraphicsElement: [class SVGGraphicsElement extends SVGElement],
      Event: [Function],
      UIEvent: [Function],
      CustomEvent: [class CustomEvent extends Event],
      CloseEvent: [class CloseEvent extends Event],
      AnimationEvent: [class AnimationEvent extends Event],
      KeyboardEvent: [Function],
      MessageEvent: [class MessageEvent extends Event],
      MouseEvent: [class MouseEvent extends UIEvent],
      PointerEvent: [class PointerEvent extends MouseEvent],
      FocusEvent: [class FocusEvent extends UIEvent],
      WheelEvent: [Function],
      InputEvent: [class InputEvent extends UIEvent],
      ErrorEvent: [class ErrorEvent extends UIEvent],
      StorageEvent: [class StorageEvent extends Event],
      SubmitEvent: [class SubmitEvent extends Event],
      ProgressEvent: [class ProgressEvent extends Event],
      MediaQueryListEvent: [class MediaQueryListEvent extends Event],
      HashChangeEvent: [class HashChangeEvent extends Event],
      ClipboardEvent: [class ClipboardEvent extends Event],
      CookieChangeEvent: [class CookieChangeEvent extends Event],
      TouchEvent: [class TouchEvent extends UIEvent],
      PopStateEvent: [class PopStateEvent extends Event],
      Touch: [class Touch],
      AudioProcessingEvent: [Function],
      BeforeInputEvent: [Function],
      BeforeUnloadEvent: [Function],
      BlobEvent: [Function],
      CompositionEvent: [Function],
      CSSFontFaceLoadEvent: [Function],
      DeviceLightEvent: [Function],
      DeviceMotionEvent: [Function],
      DeviceOrientationEvent: [Function],
      DeviceProximityEvent: [Function],
      DOMTransactionEvent: [Function],
      DragEvent: [Function],
      EditingBeforeInputEvent: [Function],
      FetchEvent: [Function],
      GamepadEvent: [Function],
      IDBVersionChangeEvent: [Function],
      MediaStreamEvent: [Function],
      MutationEvent: [Function],
      OfflineAudioCompletionEvent: [Function],
      OverconstrainedError: [Function],
      PageTransitionEvent: [Function],
      PaymentRequestUpdateEvent: [Function],
      RelatedEvent: [Function],
      RTCDataChannelEvent: [Function],
      RTCIdentityErrorEvent: [Function],
      RTCIdentityEvent: [Function],
      RTCPeerConnectionIceEvent: [Function],
      SensorEvent: [Function],
      SVGEvent: [Function],
      SVGZoomEvent: [Function],
      TimeEvent: [Function],
      TrackEvent: [Function],
      TransitionEvent: [Function],
      UserProximityEvent: [Function],
      WebGLContextEvent: [Function],
      TextEvent: [Function],
      Permissions: [class Permissions],
      History: [class History],
      Navigator: [class Navigator],
      Clipboard: [class Clipboard],
      TimeRanges: [class TimeRanges],
      TextTrackCueList: [class TextTrackCueList extends Array],
      ValidityState: [class ValidityState],
      MutationRecord: [class MutationRecord],
      IntersectionObserver: [class IntersectionObserver],
      IntersectionObserverEntry: [class IntersectionObserverEntry],
      CSSStyleDeclaration: [class CSSStyleDeclaration],
      CSSRule: [Function],
      CSSContainerRule: [class CSSContainerRule extends CSSConditionRule],
      CSSFontFaceRule: [class CSSFontFaceRule extends CSSRule],
      CSSKeyframeRule: [class CSSKeyframeRule extends CSSRule],
      CSSKeyframesRule: [class CSSKeyframesRule extends CSSRule],
      CSSMediaRule: [class CSSMediaRule extends CSSConditionRule],
      CSSStyleRule: [class CSSStyleRule extends CSSGroupingRule],
      CSSSupportsRule: [class CSSSupportsRule extends CSSConditionRule],
      CSSConditionRule: [class CSSConditionRule extends CSSGroupingRule],
      CSSGroupingRule: [class CSSGroupingRule extends CSSRule],
      CSSScopeRule: [class CSSScopeRule extends CSSGroupingRule],
      DOMRect: [class DOMRect extends DOMRectReadOnly],
      DOMRectReadOnly: [class DOMRectReadOnly],
      Plugin: [class Plugin],
      PluginArray: [class PluginArray],
      Location: [class Location],
      CustomElementRegistry: [class CustomElementRegistry],
      ResizeObserver: [class ResizeObserver],
      Blob: [class Blob],
      File: [class File extends Blob],
      Storage: [class Storage],
      CookieStore: [class CookieStore extends EventTarget],
      MimeType: [class MimeType],
      MimeTypeArray: [class MimeTypeArray],
      NodeFilter: [Object],
      ImageBitmap: [class ImageBitmap],
      HTMLCollection: [class HTMLCollection],
      HTMLFormControlCollection: [class HTMLFormControlsCollection extends HTMLCollection],
      HTMLOptionsCollection: [class HTMLOptionsCollection extends HTMLCollection],
      NodeList: [class NodeList],
      RadioNodeList: [class RadioNodeList extends NodeList],
      FileList: [class FileList extends Array],
      Screen: [class Screen extends EventTarget],
      ScreenDetails: [class ScreenDetails extends EventTarget],
      ScreenDetailed: [class ScreenDetailed extends Screen],
      NamedNodeMap: [class NamedNodeMap],
      TreeWalker: [class TreeWalker],
      NodeIterator: [class NodeIterator],
      DataTransfer: [class DataTransfer],
      DataTransferItem: [class DataTransferItem],
      DataTransferItemList: [class DataTransferItemList extends Array],
      XMLSerializer: [class XMLSerializer],
      ClipboardItem: [class ClipboardItem],
      Selection: [class Selection],
      CSSUnitValue: [class CSSUnitValue],
      SVGAngle: [Function],
      SVGAnimatedAngle: [class SVGAnimatedAngle],
      SVGAnimatedBoolean: [class SVGAnimatedBoolean],
      SVGAnimatedEnumeration: [class SVGAnimatedEnumeration],
      SVGAnimatedInteger: [class SVGAnimatedInteger],
      SVGAnimatedLength: [class SVGAnimatedLength],
      SVGAnimatedNumber: [class SVGAnimatedNumber],
      SVGAnimatedNumberList: [class SVGAnimatedNumberList],
      SVGAnimatedPreserveAspectRatio: [class SVGAnimatedPreserveAspectRatio],
      SVGAnimatedRect: [class SVGAnimatedRect],
      SVGAnimatedString: [class SVGAnimatedString],
      SVGAnimatedTransformList: [class SVGAnimatedTransformList],
      SVGLength: [Function],
      SVGLengthList: [class SVGLengthList],
      SVGMatrix: [class SVGMatrix],
      SVGNumber: [class SVGNumber],
      SVGNumberList: [class SVGNumberList],
      SVGPoint: [class SVGPoint],
      SVGPointList: [class SVGPointList],
      SVGPreserveAspectRatio: [Function],
      SVGRect: [class SVGRect],
      SVGStringList: [class SVGStringList],
      SVGTransform: [Function],
      SVGTransformList: [class SVGTransformList],
      SVGAnimatedLengthList: [class SVGAnimatedLengthList],
      SVGUnitTypes: [Function],
      StylePropertyMap: [class StylePropertyMap extends StylePropertyMapReadOnly],
      StylePropertyMapReadOnly: [class StylePropertyMapReadOnly],
      MediaList: [class MediaList],
      CSSKeywordValue: [class CSSKeywordValue],
      CSSStyleValue: [class CSSStyleValue],
      Window: [class GlobalWindow extends Window],
      URLSearchParams: [class URLSearchParams],
      WritableStream: [Function],
      ReadableStream: [Function],
      TransformStream: [Function: Transform],
      PerformanceObserver: [class PerformanceObserver],
      PerformanceEntry: [class PerformanceEntry],
      PerformanceObserverEntryList: [class PerformanceObserverEntryList],
      document: [HTMLDocument],
      customElements: [CustomElementRegistry],
      window: [Circular *3],
      globalThis: [Circular *3],
      performance: [Performance],
      screenLeft: 0,
      screenTop: 0,
      screenX: 0,
      screenY: 0,
      crypto: Crypto {},
      TextEncoder: [class TextEncoder],
      TextDecoder: [class TextDecoder],
      closed: false,
      console: [Object [console]],
      name: '',
      Buffer: [Function],
      Document: [class Document extends Document],
      HTMLDocument: [class HTMLDocument extends HTMLDocument],
      XMLDocument: [class XMLDocument extends XMLDocument],
      DocumentFragment: [class DocumentFragment extends DocumentFragment],
      Text: [class Text extends Text],
      Comment: [class Comment extends Comment],
      Image: [class Image extends Image],
      Audio: [class Audio extends Audio],
      MutationObserver: [class MutationObserver extends MutationObserver],
      MessagePort: [class MessagePort extends MessagePort],
      CSSStyleSheet: [class CSSStyleSheet extends CSSStyleSheet],
      DOMException: [class DOMException extends DOMException],
      Headers: [class Headers extends Headers],
      Request: [class Request extends Request],
      Response: [Function],
      EventTarget: [class EventTarget extends EventTarget],
      XMLHttpRequestUpload: [class XMLHttpRequestUpload extends XMLHttpRequestUpload],
      XMLHttpRequestEventTarget: [class XMLHttpRequestEventTarget extends XMLHttpRequestEventTarget],
      AbortController: [class AbortController extends AbortController],
      AbortSignal: [Function],
      FormData: [class FormData extends FormData],
      PermissionStatus: [class PermissionStatus extends PermissionStatus],
      XMLHttpRequest: [class XMLHttpRequest extends XMLHttpRequest],
      DOMParser: [class DOMParser extends DOMParser],
      Range: [class Range extends Range],
      VTTCue: [class VTTCue extends VTTCue],
      TextTrack: [class TextTrack extends TextTrack],
      TextTrackList: [class TextTrackList extends TextTrackList],
      TextTrackCue: [class TextTrackCue extends TextTrackCue],
      RemotePlayback: [class RemotePlayback extends RemotePlayback],
      FileReader: [class FileReader extends FileReader],
      MediaStream: [class MediaStream extends MediaStream],
      MediaStreamTrack: [class MediaStreamTrack extends MediaStreamTrack],
      CanvasCaptureMediaStreamTrack: [class CanvasCaptureMediaStreamTrack extends CanvasCaptureMediaStreamTrack],
      URL: [class URL extends URL],
      WebSocket: [class WebSocket extends WebSocket],
      ImageData: [class ImageData extends ImageData],
      OffscreenCanvas: [class OffscreenCanvas extends OffscreenCanvas],
      DOMMatrixReadOnly: [class DOMMatrixReadOnly extends DOMMatrixReadOnly],
      DOMMatrix: [class DOMMatrix extends DOMMatrix],
      DOMPointReadOnly: [class DOMPointReadOnly extends DOMPointReadOnly],
      DOMPoint: [class DOMPoint extends DOMPoint],
      Animation: [class Animation extends Animation],
      KeyframeEffect: [class KeyframeEffect extends KeyframeEffect],
      happyDOM: DetachedWindowAPI {},
      Array: [Function: Array],
      ArrayBuffer: [Function: ArrayBuffer],
      Boolean: [Function: Boolean],
      DataView: [Function: DataView],
      Date: [Function: Date],
      Error: [Function],
      EvalError: [Function: EvalError],
      Float32Array: [Function: Float32Array],
      Float64Array: [Function: Float64Array],
      Function: [Function: Function],
      Infinity: Infinity,
      Int16Array: [Function: Int16Array],
      Int32Array: [Function: Int32Array],
      Int8Array: [Function: Int8Array],
      Intl: Object [Intl] {},
      JSON: Object [JSON] {},
      Map: [Function: Map],
      Math: Object [Math] {},
      NaN: NaN,
      Number: [Function: Number],
      Object: [Function: Object],
      Promise: [Function: Promise],
      RangeError: [Function: RangeError],
      ReferenceError: [Function: ReferenceError],
      RegExp: [Function: RegExp],
      Set: [Function: Set],
      String: [Function: String],
      Symbol: [Function: Symbol],
      SyntaxError: [Function: SyntaxError],
      TypeError: [Function: TypeError],
      URIError: [Function: URIError],
      Uint16Array: [Function: Uint16Array],
      Uint32Array: [Function: Uint32Array],
      Uint8Array: [Function: Uint8Array],
      Uint8ClampedArray: [Function: Uint8ClampedArray],
      WeakMap: [Function: WeakMap],
      WeakSet: [Function: WeakSet],
      decodeURI: [Function: decodeURI],
      decodeURIComponent: [Function: decodeURIComponent],
      encodeURI: [Function: encodeURI],
      encodeURIComponent: [Function: encodeURIComponent],
      eval: [Function: eval],
      escape: [Function: escape],
      global: [Object [global]],
      isFinite: [Function: isFinite],
      isNaN: [Function: isNaN],
      parseFloat: [Function: parseFloat],
      parseInt: [Function: parseInt],
      undefined: undefined,
      unescape: [Function: unescape],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(mutationObservers)]: [Array],
      [Symbol(readyStateManager)]: DocumentReadyStateManager {},
      [Symbol(location)]: [Location],
      [Symbol(history)]: History {},
      [Symbol(navigator)]: Navigator {},
      [Symbol(screen)]: [Screen [EventTarget]],
      [Symbol(sessionStorage)]: [Storage],
      [Symbol(localStorage)]: [Storage],
      [Symbol(cookieStore)]: [CookieStore [EventTarget]],
      [Symbol(self)]: [Circular *3],
      [Symbol(top)]: [Circular *3],
      [Symbol(parent)]: [Circular *3],
      [Symbol(window)]: [Circular *3],
      [Symbol(frames)]: [Circular *3],
      [Symbol(internalId)]: 0,
      [Symbol(customElementReactionStack)]: [CustomElementReactionStack],
      [Symbol(modules)]: [Object],
      [Symbol(moduleImportMap)]: null,
      [Symbol(openWebSockets)]: [],
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(querySelectorCache)]: [Map]
    },
    [Symbol(classList)]: null,
    [Symbol(isValue)]: null,
    [Symbol(nodeType)]: 1,
    [Symbol(prefix)]: null,
    [Symbol(shadowRoot)]: null,
    [Symbol(scrollHeight)]: 0,
    [Symbol(scrollWidth)]: 0,
    [Symbol(scrollTop)]: 0,
    [Symbol(scrollLeft)]: 0,
    [Symbol(attributes)]: NamedNodeMap {
      [Symbol(itemsByNamespaceURI)]: [Map],
      [Symbol(itemsByName)]: [Map],
      [Symbol(items)]: [Map],
      [Symbol(ownerElement)]: [Circular *2]
    },
    [Symbol(attributesProxy)]: null,
    [Symbol(children)]: null,
    [Symbol(computedStyle)]: null,
    [Symbol(pointerCaptures)]: Set(0) {},
    [Symbol(propertyEventListeners)]: Map(0) {},
    [Symbol(animations)]: [],
    [Symbol(tagName)]: 'STYLE',
    [Symbol(localName)]: 'style',
    [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
    [Symbol(accessKey)]: '',
    [Symbol(offsetHeight)]: 0,
    [Symbol(offsetWidth)]: 0,
    [Symbol(offsetLeft)]: 0,
    [Symbol(offsetTop)]: 0,
    [Symbol(clientHeight)]: 0,
    [Symbol(clientWidth)]: 0,
    [Symbol(clientLeft)]: 0,
    [Symbol(clientTop)]: 0,
    [Symbol(style)]: null,
    [Symbol(dataset)]: null,
    [Symbol(customElementDefineCallback)]: null,
    [Symbol(sheet)]: null,
    [Symbol(disabled)]: false
  },
  className: 'tiptap',
  editorView: <ref *5> EditorView {
    _root: null,
    focused: false,
    trackWrites: null,
    mounted: false,
    markCursor: null,
    cursorWrapper: null,
    lastSelectedViewDesc: undefined,
    input: InputState {
      shiftKey: false,
      mouseDown: null,
      lastKeyCode: null,
      lastKeyCodeTime: 0,
      lastClick: [Object],
      lastSelectionOrigin: null,
      lastSelectionTime: 0,
      lastIOSEnter: 0,
      lastIOSEnterFallbackTimeout: -1,
      lastFocus: 0,
      lastTouch: 0,
      lastChromeDelete: 0,
      composing: false,
      compositionNode: null,
      composingTimeout: -1,
      compositionNodes: [],
      compositionEndedAt: -200000000,
      compositionID: 1,
      badSafariComposition: false,
      compositionPendingChanges: 0,
      domChangeCount: 0,
      eventHandlers: [Object: null prototype],
      hideSelectionGuard: null
    },
    prevDirectPlugins: [],
    pluginViews: [
      [Object],
      [Object],
      [Object],
      [Object],
      [DropCursorView],
      [Object],
      [Object],
      [Object]
    ],
    requiresGeckoHackNode: false,
    dragging: null,
    _props: {
      attributes: [Object],
      dispatchTransaction: [Function: bound dispatchTransaction],
      transformPastedHTML: [Function (anonymous)],
      state: [EditorState],
      markViews: {},
      nodeViews: [Object]
    },
    state: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    directPlugins: [],
    dispatch: [Function: bound ],
    dom: HTMLDivElement {
      pmViewDesc: [NodeViewDesc],
      editor: [Circular *4],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: [HTMLDivElement],
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [Array],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: [DOMTokenList],
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: [CSSStyleDeclaration],
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    editable: true,
    nodeViews: [Object: null prototype] { taskItem: [Function: nodeview] },
    docView: NodeViewDesc {
      parent: undefined,
      children: [Array],
      dom: [HTMLDivElement],
      contentDOM: [HTMLDivElement],
      dirty: 0,
      node: [Node],
      outerDeco: [Array],
      innerDeco: [DecorationSet],
      nodeDOM: [HTMLDivElement]
    },
    domObserver: DOMObserver {
      view: [Circular *5],
      handleDOMChange: [Function (anonymous)],
      queue: [],
      flushingSoon: -1,
      observer: MutationObserver {},
      currentSelection: [SelectionState],
      onCharData: null,
      suppressingSelectionUpdates: false,
      lastChangedTextNode: null,
      onSelectionChange: [Function: bound onSelectionChange]
    }
  },
  isFocused: false,
  destroyed: false,
  isInitialized: false,
  extensionStorage: {
    link: {},
    paragraph: {},
    listItemBranchingDeleteKeymap: {},
    taskItemBranchingDeleteKeymap: {},
    textStyle: {},
    editable: {},
    clipboardTextSerializer: {},
    commands: {},
    focusEvents: {},
    keymap: {},
    tabindex: {},
    drop: {},
    paste: {},
    delete: {},
    textDirection: {},
    starterKit: {},
    bold: {},
    blockquote: {},
    bulletList: {},
    code: {},
    doc: {},
    dropCursor: {},
    gapCursor: {},
    hardBreak: {},
    undoRedo: {},
    italic: {},
    listItem: {},
    listKeymap: {},
    orderedList: {},
    strike: {},
    text: {},
    underline: {},
    trailingNode: {},
    taskList: {},
    taskItem: {},
    color: {},
    fontSize: {},
    textAlign: {},
    typography: {},
    placeholder: {}
  },
  instanceId: 'ghfenf8',
  hasWarnedStaleDecorationRead: false,
  options: {
    element: HTMLDivElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    content: { type: 'doc', content: [Array] },
    injectCSS: true,
    injectNonce: undefined,
    extensions: [
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    autofocus: false,
    editable: true,
    textDirection: undefined,
    editorProps: { attributes: [Object] },
    parseOptions: {},
    coreExtensionOptions: {},
    enableInputRules: true,
    enablePasteRules: true,
    enableCoreExtensions: true,
    enableContentCheck: false,
    emitContentError: false,
    onBeforeCreate: [Function: onBeforeCreate],
    onCreate: [Function: onCreate],
    onMount: [Function: onMount],
    onUnmount: [Function: onUnmount],
    onUpdate: [Function: onUpdate],
    onSelectionUpdate: [Function: onSelectionUpdate],
    onTransaction: [Function: onTransaction],
    onFocus: [Function: onFocus],
    onBlur: [Function: onBlur],
    onDestroy: [Function: onDestroy],
    onContentError: [Function: onContentError],
    onPaste: [Function: onPaste],
    onDrop: [Function: onDrop],
    onDelete: [Function: onDelete],
    enableExtensionDispatchTransaction: true
  },
  isCapturingTransaction: false,
  capturedTransaction: null,
  utils: {
    getUpdatedPosition: [Function: getUpdatedPosition],
    createMappablePosition: [Function: createMappablePosition]
  },
  extensionManager: ExtensionManager {
    splittableMarks: [ 'textStyle', 'bold', 'code', 'italic', 'strike', 'underline' ],
    nonClearableMarks: [],
    decorationManager: DecorationManager {
      warnedWidgetKeys: Set(0) {},
      warnedOutOfRangeExtensions: Set(0) {},
      handleBeforeTransaction: [Function (anonymous)],
      editor: [Circular *4],
      entries: [],
      plugin: null
    },
    editor: [Circular *4],
    baseExtensions: [
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    extensions: [
      [Mark],      [Node],      [Extension],
      [Extension], [Mark],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Mark],      [Node],
      [Node],      [Mark],      [Node],
      [Extension], [Extension], [Node],
      [Extension], [Mark],      [Node],
      [Extension], [Node],      [Mark],
      [Node],      [Mark],      [Extension],
      [Node],      [Node],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension]
    ],
    schema: Schema {
      linebreakReplacement: [NodeType],
      cached: [Object: null prototype],
      spec: [Object],
      nodes: [Object: null prototype],
      marks: [Object: null prototype],
      nodeFromJSON: [Function (anonymous)],
      markFromJSON: [Function (anonymous)],
      topNodeType: [NodeType]
    }
  },
  commandManager: CommandManager {
    editor: [Circular *4],
    rawCommands: {
      setLink: [Function: setLink],
      toggleLink: [Function: toggleLink],
      unsetLink: [Function: unsetLink],
      setParagraph: [Function: setParagraph],
      toggleTextStyle: [Function: toggleTextStyle],
      removeEmptyTextStyle: [Function: removeEmptyTextStyle],
      blur: [Function: blur],
      clearContent: [Function: clearContent],
      clearNodes: [Function: clearNodes],
      command: [Function: command],
      createParagraphNear: [Function: createParagraphNear$1],
      cut: [Function: cut],
      deleteCurrentNode: [Function: deleteCurrentNode],
      deleteNode: [Function: deleteNode],
      deleteRange: [Function: deleteRange],
      deleteSelection: [Function: deleteSelection],
      enter: [Function: enter],
      exitCode: [Function: exitCode$1],
      extendMarkRange: [Function: extendMarkRange],
      first: [Function: first],
      focus: [Function: focus],
      forEach: [Function: forEach],
      insertContent: [Function: insertContent],
      insertContentAt: [Function: insertContentAt],
      insertDefaultBlock: [Function: insertDefaultBlock],
      joinBackward: [Function: joinBackward$1],
      joinDown: [Function: joinDown$1],
      joinForward: [Function: joinForward$1],
      joinItemBackward: [Function: joinItemBackward],
      joinItemForward: [Function: joinItemForward],
      joinTextblockBackward: [Function: joinTextblockBackward$1],
      joinTextblockForward: [Function: joinTextblockForward$1],
      joinUp: [Function: joinUp$1],
      keyboardShortcut: [Function: keyboardShortcut],
      lift: [Function: lift$1],
      liftEmptyBlock: [Function: liftEmptyBlock$1],
      liftListItem: [Function: liftListItem$1],
      newlineInCode: [Function: newlineInCode$1],
      resetAttributes: [Function: resetAttributes],
      scrollIntoView: [Function: scrollIntoView],
      selectAll: [Function: selectAll],
      selectNodeBackward: [Function: selectNodeBackward$1],
      selectNodeForward: [Function: selectNodeForward$1],
      selectParentNode: [Function: selectParentNode$1],
      selectTextblockEnd: [Function: selectTextblockEnd$1],
      selectTextblockStart: [Function: selectTextblockStart$1],
      setContent: [Function: setContent],
      setMark: [Function: setMark],
      setMeta: [Function: setMeta],
      setNode: [Function: setNode],
      setNodeSelection: [Function: setNodeSelection],
      setTextDirection: [Function: setTextDirection],
      setTextSelection: [Function: setTextSelection],
      sinkListItem: [Function: sinkListItem$1],
      splitBlock: [Function: splitBlock],
      splitListItem: [Function: splitListItem],
      toggleList: [Function: toggleList],
      toggleMark: [Function: toggleMark],
      toggleNode: [Function: toggleNode],
      toggleWrap: [Function: toggleWrap],
      undoInputRule: [Function: undoInputRule],
      unsetAllMarks: [Function: unsetAllMarks],
      unsetMark: [Function: unsetMark],
      unsetTextDirection: [Function: unsetTextDirection],
      updateAttributes: [Function: updateAttributes],
      updateDecorations: [Function: updateDecorations],
      wrapIn: [Function: wrapIn$1],
      wrapInList: [Function: wrapInList$1],
      setBold: [Function: setBold],
      toggleBold: [Function: toggleBold],
      unsetBold: [Function: unsetBold],
      setBlockquote: [Function: setBlockquote],
      toggleBlockquote: [Function: toggleBlockquote],
      unsetBlockquote: [Function: unsetBlockquote],
      toggleBulletList: [Function: toggleBulletList],
      setCode: [Function: setCode],
      toggleCode: [Function: toggleCode],
      unsetCode: [Function: unsetCode],
      setHardBreak: [Function: setHardBreak],
      undo: [Function: undo],
      redo: [Function: redo],
      setItalic: [Function: setItalic],
      toggleItalic: [Function: toggleItalic],
      unsetItalic: [Function: unsetItalic],
      toggleOrderedList: [Function: toggleOrderedList],
      setStrike: [Function: setStrike],
      toggleStrike: [Function: toggleStrike],
      unsetStrike: [Function: unsetStrike],
      setUnderline: [Function: setUnderline],
      toggleUnderline: [Function: toggleUnderline],
      unsetUnderline: [Function: unsetUnderline],
      toggleTaskList: [Function: toggleTaskList],
      setColor: [Function: setColor],
      unsetColor: [Function: unsetColor],
      setFontSize: [Function: setFontSize],
      unsetFontSize: [Function: unsetFontSize],
      setTextAlign: [Function: setTextAlign],
      unsetTextAlign: [Function: unsetTextAlign],
      toggleTextAlign: [Function: toggleTextAlign]
    },
    customState: undefined
  },
  schema: <ref *6> Schema {
    linebreakReplacement: NodeType {
      name: 'hardBreak',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [Array],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: false,
      isText: false
    },
    cached: [Object: null prototype] { wrappings: [Object: null prototype] {} },
    spec: { topNode: 'doc', nodes: [OrderedMap], marks: [OrderedMap] },
    nodes: [Object: null prototype] {
      paragraph: [NodeType],
      blockquote: [NodeType],
      bulletList: [NodeType],
      doc: [NodeType],
      hardBreak: [NodeType],
      listItem: [NodeType],
      orderedList: [NodeType],
      text: [NodeType],
      taskList: [NodeType],
      taskItem: [NodeType]
    },
    marks: [Object: null prototype] {
      link: [MarkType],
      textStyle: [MarkType],
      bold: [MarkType],
      code: [MarkType],
      italic: [MarkType],
      strike: [MarkType],
      underline: [MarkType]
    },
    nodeFromJSON: [Function (anonymous)],
    markFromJSON: [Function (anonymous)],
    topNodeType: NodeType {
      name: 'doc',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: true,
      isText: false
    }
  },
  editorState: EditorState {
    config: Configuration {
      schema: [Schema],
      plugins: [],
      pluginsByKey: [Object: null prototype] {},
      fields: [Array]
    },
    doc: Node {
      type: [NodeType],
      attrs: [Object: null prototype] {},
      marks: [],
      content: [Fragment]
    },
    selection: TextSelection {
      '$anchor': [ResolvedPos],
      '$head': [ResolvedPos],
      ranges: [Array]
    },
    storedMarks: null,
    scrollToSelection: 0
  },
  contentComponent: null,
  appContext: null,
  reactiveState: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  },
  reactiveExtensionStorage: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  }
} default-font-size=undefined  ... > 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[Vue warn]: Invalid prop: type check failed for prop "defaultAlignment". Expected String with value "undefined", got Undefined  
  at <SpaceCanvasTextSelectionBar key=0 editor= <ref *4> Editor {
  callbacks: {
    create: [ [Function: bound onCreate], [Function: onCreate] ],
    destroy: [ [Function: bound onDestroy], [Function: onDestroy] ],
    update: [ [Function: bound onUpdate], [Function: onUpdate] ],
    beforeCreate: [ [Function: onBeforeCreate] ],
    mount: [ [Function: onMount] ],
    unmount: [ [Function: onUnmount] ],
    contentError: [ [Function: onContentError] ],
    selectionUpdate: [ [Function: onSelectionUpdate] ],
    transaction: [ [Function: onTransaction] ],
    focus: [ [Function: onFocus] ],
    blur: [ [Function: onBlur] ],
    drop: [ [Function (anonymous)] ],
    paste: [ [Function (anonymous)] ],
    delete: [ [Function: onDelete] ],
    beforeTransaction: [ [Function (anonymous)], [Function (anonymous)] ]
  },
  css: <ref *2> HTMLStyleElement {
    [Symbol(listeners)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(listenerOptions)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(isConnected)]: true,
    [Symbol(parentNode)]: HTMLHeadElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: [HTMLHtmlElement],
      [Symbol(rootNode)]: [HTMLDocument],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'HEAD',
      [Symbol(localName)]: 'head',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    [Symbol(rootNode)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(styleNode)]: [Circular *2],
    [Symbol(textAreaNode)]: null,
    [Symbol(formNode)]: null,
    [Symbol(selectNode)]: null,
    [Symbol(mutationListeners)]: [],
    [Symbol(nodeArray)]: [ [Text] ],
    [Symbol(elementArray)]: [],
    [Symbol(childNodes)]: null,
    [Symbol(assignedToSlot)]: null,
    [Symbol(cache)]: {
      querySelector: Map(0) {},
      querySelectorAll: Map(0) {},
      matches: Map(0) {},
      elementsByTagName: Map(0) {},
      elementsByTagNameNS: Map(0) {},
      elementByTagName: Map(0) {},
      elementById: Map(0) {},
      computedStyle: null
    },
    [Symbol(affectsCache)]: [],
    [Symbol(ownerDocument)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(window)]: <ref *3> GlobalWindow [EventTarget] {
      Node: [Function],
      Attr: [class Attr extends Node],
      ShadowRoot: [class ShadowRoot extends DocumentFragment],
      ProcessingInstruction: [class ProcessingInstruction extends CharacterData],
      Element: [Function],
      CharacterData: [class CharacterData extends Node],
      DocumentType: [class DocumentType extends Node],
      AnimationTimeline: [class AnimationTimeline],
      DocumentTimeline: [class DocumentTimeline extends AnimationTimeline],
      HTMLAnchorElement: [class HTMLAnchorElement extends HTMLElement],
      HTMLButtonElement: [class HTMLButtonElement extends HTMLElement],
      HTMLOptGroupElement: [class HTMLOptGroupElement extends HTMLElement],
      HTMLOptionElement: [class HTMLOptionElement extends HTMLElement],
      HTMLElement: [Function],
      HTMLUnknownElement: [class HTMLUnknownElement extends HTMLElement],
      HTMLTemplateElement: [class HTMLTemplateElement extends HTMLElement],
      HTMLInputElement: [class HTMLInputElement extends HTMLElement],
      HTMLSelectElement: [class HTMLSelectElement extends HTMLElement],
      HTMLTextAreaElement: [class HTMLTextAreaElement extends HTMLElement],
      HTMLImageElement: [class HTMLImageElement extends HTMLElement],
      HTMLStyleElement: [class HTMLStyleElement extends HTMLElement],
      HTMLLabelElement: [class HTMLLabelElement extends HTMLElement],
      HTMLSlotElement: [class HTMLSlotElement extends HTMLElement],
      HTMLMetaElement: [class HTMLMetaElement extends HTMLElement],
      HTMLMediaElement: [class HTMLMediaElement extends HTMLElement],
      HTMLAudioElement: [class HTMLAudioElement extends HTMLMediaElement],
      HTMLVideoElement: [class HTMLVideoElement extends HTMLMediaElement],
      HTMLBaseElement: [class HTMLBaseElement extends HTMLElement],
      HTMLDialogElement: [class HTMLDialogElement extends HTMLElement],
      HTMLScriptElement: [class HTMLScriptElement extends HTMLElement],
      HTMLLinkElement: [class HTMLLinkElement extends HTMLElement],
      HTMLIFrameElement: [class HTMLIFrameElement extends HTMLElement],
      HTMLFormElement: [class HTMLFormElement extends HTMLElement],
      HTMLUListElement: [class HTMLUListElement extends HTMLElement],
      HTMLTrackElement: [class HTMLTrackElement extends HTMLElement],
      HTMLTableRowElement: [class HTMLTableRowElement extends HTMLElement],
      HTMLTitleElement: [class HTMLTitleElement extends HTMLElement],
      HTMLTimeElement: [class HTMLTimeElement extends HTMLElement],
      HTMLTableSectionElement: [class HTMLTableSectionElement extends HTMLElement],
      HTMLTableCellElement: [class HTMLTableCellElement extends HTMLElement],
      HTMLTableElement: [class HTMLTableElement extends HTMLElement],
      HTMLSpanElement: [class HTMLSpanElement extends HTMLElement],
      HTMLSourceElement: [class HTMLSourceElement extends HTMLElement],
      HTMLQuoteElement: [class HTMLQuoteElement extends HTMLElement],
      HTMLProgressElement: [class HTMLProgressElement extends HTMLElement],
      HTMLPreElement: [class HTMLPreElement extends HTMLElement],
      HTMLPictureElement: [class HTMLPictureElement extends HTMLElement],
      HTMLParamElement: [class HTMLParamElement extends HTMLElement],
      HTMLParagraphElement: [class HTMLParagraphElement extends HTMLElement],
      HTMLOutputElement: [class HTMLOutputElement extends HTMLElement],
      HTMLOListElement: [class HTMLOListElement extends HTMLElement],
      HTMLObjectElement: [class HTMLObjectElement extends HTMLElement],
      HTMLMeterElement: [class HTMLMeterElement extends HTMLElement],
      HTMLMenuElement: [class HTMLMenuElement extends HTMLElement],
      HTMLMapElement: [class HTMLMapElement extends HTMLElement],
      HTMLLIElement: [class HTMLLIElement extends HTMLElement],
      HTMLLegendElement: [class HTMLLegendElement extends HTMLElement],
      HTMLModElement: [class HTMLModElement extends HTMLElement],
      HTMLHtmlElement: [class HTMLHtmlElement extends HTMLElement],
      HTMLHRElement: [class HTMLHRElement extends HTMLElement],
      HTMLHeadElement: [class HTMLHeadElement extends HTMLElement],
      HTMLHeadingElement: [class HTMLHeadingElement extends HTMLElement],
      HTMLFieldSetElement: [class HTMLFieldSetElement extends HTMLElement],
      HTMLEmbedElement: [class HTMLEmbedElement extends HTMLElement],
      HTMLDListElement: [class HTMLDListElement extends HTMLElement],
      HTMLDivElement: [class HTMLDivElement extends HTMLElement],
      HTMLDetailsElement: [class HTMLDetailsElement extends HTMLElement],
      HTMLDataListElement: [class HTMLDataListElement extends HTMLElement],
      HTMLDataElement: [class HTMLDataElement extends HTMLElement],
      HTMLTableColElement: [class HTMLTableColElement extends HTMLElement],
      HTMLTableCaptionElement: [class HTMLTableCaptionElement extends HTMLElement],
      HTMLCanvasElement: [class HTMLCanvasElement extends HTMLElement],
      HTMLBRElement: [class HTMLBRElement extends HTMLElement],
      HTMLBodyElement: [class HTMLBodyElement extends HTMLElement],
      HTMLAreaElement: [class HTMLAreaElement extends HTMLElement],
      SVGSVGElement: [class SVGSVGElement extends SVGGraphicsElement],
      SVGAnimateElement: [class SVGAnimateElement extends SVGAnimationElement],
      SVGAnimateMotionElement: [class SVGAnimateMotionElement extends SVGAnimationElement],
      SVGAnimateTransformElement: [class SVGAnimateTransformElement extends SVGAnimationElement],
      SVGCircleElement: [class SVGCircleElement extends SVGGeometryElement],
      SVGClipPathElement: [class SVGClipPathElement extends SVGElement],
      SVGDefsElement: [class SVGDefsElement extends SVGGraphicsElement],
      SVGDescElement: [class SVGDescElement extends SVGElement],
      SVGEllipseElement: [class SVGEllipseElement extends SVGGeometryElement],
      SVGFEBlendElement: [Function],
      SVGFEColorMatrixElement: [Function],
      SVGFEComponentTransferElement: [class SVGFEComponentTransferElement extends SVGElement],
      SVGFECompositeElement: [Function],
      SVGFEConvolveMatrixElement: [Function],
      SVGFEDiffuseLightingElement: [class SVGFEDiffuseLightingElement extends SVGElement],
      SVGFEDisplacementMapElement: [Function],
      SVGFEDistantLightElement: [class SVGFEDistantLightElement extends SVGElement],
      SVGFEDropShadowElement: [class SVGFEDropShadowElement extends SVGElement],
      SVGFEFloodElement: [class SVGFEFloodElement extends SVGElement],
      SVGFEFuncAElement: [class SVGFEFuncAElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncBElement: [class SVGFEFuncBElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncGElement: [class SVGFEFuncGElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncRElement: [class SVGFEFuncRElement extends SVGComponentTransferFunctionElement],
      SVGFEGaussianBlurElement: [Function],
      SVGFEImageElement: [class SVGFEImageElement extends SVGElement],
      SVGFEMergeElement: [class SVGFEMergeElement extends SVGElement],
      SVGFEMergeNodeElement: [class SVGFEMergeNodeElement extends SVGElement],
      SVGFEMorphologyElement: [Function],
      SVGFEOffsetElement: [class SVGFEOffsetElement extends SVGElement],
      SVGFEPointLightElement: [class SVGFEPointLightElement extends SVGElement],
      SVGFESpecularLightingElement: [class SVGFESpecularLightingElement extends SVGElement],
      SVGFESpotLightElement: [class SVGFESpotLightElement extends SVGElement],
      SVGFETileElement: [class SVGFETileElement extends SVGElement],
      SVGFETurbulenceElement: [Function],
      SVGFilterElement: [class SVGFilterElement extends SVGElement],
      SVGForeignObjectElement: [class SVGForeignObjectElement extends SVGGraphicsElement],
      SVGGElement: [class SVGGElement extends SVGGraphicsElement],
      SVGImageElement: [class SVGImageElement extends SVGGraphicsElement],
      SVGLineElement: [class SVGLineElement extends SVGGeometryElement],
      SVGLinearGradientElement: [class SVGLinearGradientElement extends SVGGradientElement],
      SVGMarkerElement: [Function],
      SVGMaskElement: [class SVGMaskElement extends SVGElement],
      SVGMetadataElement: [class SVGMetadataElement extends SVGElement],
      SVGMPathElement: [class SVGMPathElement extends SVGElement],
      SVGPathElement: [class SVGPathElement extends SVGGeometryElement],
      SVGPatternElement: [class SVGPatternElement extends SVGElement],
      SVGPolygonElement: [class SVGPolygonElement extends SVGGeometryElement],
      SVGPolylineElement: [class SVGPolylineElement extends SVGGeometryElement],
      SVGRadialGradientElement: [class SVGRadialGradientElement extends SVGGradientElement],
      SVGRectElement: [class SVGRectElement extends SVGGeometryElement],
      SVGScriptElement: [class SVGScriptElement extends SVGGraphicsElement],
      SVGSetElement: [class SVGSetElement extends SVGAnimationElement],
      SVGStopElement: [class SVGStopElement extends SVGElement],
      SVGStyleElement: [class SVGStyleElement extends SVGElement],
      SVGSwitchElement: [class SVGSwitchElement extends SVGGraphicsElement],
      SVGSymbolElement: [class SVGSymbolElement extends SVGGraphicsElement],
      SVGTextElement: [class SVGTextElement extends SVGTextPositioningElement],
      SVGTextPathElement: [Function],
      SVGTitleElement: [class SVGTitleElement extends SVGElement],
      SVGTSpanElement: [class SVGTSpanElement extends SVGTextPositioningElement],
      SVGUseElement: [class SVGUseElement extends SVGGraphicsElement],
      SVGViewElement: [class SVGViewElement extends SVGElement],
      SVGElement: [class SVGElement extends Element],
      SVGAnimationElement: [class SVGAnimationElement extends SVGElement],
      SVGComponentTransferFunctionElement: [Function],
      SVGGeometryElement: [class SVGGeometryElement extends SVGGraphicsElement],
      SVGGradientElement: [Function],
      SVGTextPositioningElement: [class SVGTextPositioningElement extends SVGTextContentElement],
      SVGGraphicsElement: [class SVGGraphicsElement extends SVGElement],
      Event: [Function],
      UIEvent: [Function],
      CustomEvent: [class CustomEvent extends Event],
      CloseEvent: [class CloseEvent extends Event],
      AnimationEvent: [class AnimationEvent extends Event],
      KeyboardEvent: [Function],
      MessageEvent: [class MessageEvent extends Event],
      MouseEvent: [class MouseEvent extends UIEvent],
      PointerEvent: [class PointerEvent extends MouseEvent],
      FocusEvent: [class FocusEvent extends UIEvent],
      WheelEvent: [Function],
      InputEvent: [class InputEvent extends UIEvent],
      ErrorEvent: [class ErrorEvent extends UIEvent],
      StorageEvent: [class StorageEvent extends Event],
      SubmitEvent: [class SubmitEvent extends Event],
      ProgressEvent: [class ProgressEvent extends Event],
      MediaQueryListEvent: [class MediaQueryListEvent extends Event],
      HashChangeEvent: [class HashChangeEvent extends Event],
      ClipboardEvent: [class ClipboardEvent extends Event],
      CookieChangeEvent: [class CookieChangeEvent extends Event],
      TouchEvent: [class TouchEvent extends UIEvent],
      PopStateEvent: [class PopStateEvent extends Event],
      Touch: [class Touch],
      AudioProcessingEvent: [Function],
      BeforeInputEvent: [Function],
      BeforeUnloadEvent: [Function],
      BlobEvent: [Function],
      CompositionEvent: [Function],
      CSSFontFaceLoadEvent: [Function],
      DeviceLightEvent: [Function],
      DeviceMotionEvent: [Function],
      DeviceOrientationEvent: [Function],
      DeviceProximityEvent: [Function],
      DOMTransactionEvent: [Function],
      DragEvent: [Function],
      EditingBeforeInputEvent: [Function],
      FetchEvent: [Function],
      GamepadEvent: [Function],
      IDBVersionChangeEvent: [Function],
      MediaStreamEvent: [Function],
      MutationEvent: [Function],
      OfflineAudioCompletionEvent: [Function],
      OverconstrainedError: [Function],
      PageTransitionEvent: [Function],
      PaymentRequestUpdateEvent: [Function],
      RelatedEvent: [Function],
      RTCDataChannelEvent: [Function],
      RTCIdentityErrorEvent: [Function],
      RTCIdentityEvent: [Function],
      RTCPeerConnectionIceEvent: [Function],
      SensorEvent: [Function],
      SVGEvent: [Function],
      SVGZoomEvent: [Function],
      TimeEvent: [Function],
      TrackEvent: [Function],
      TransitionEvent: [Function],
      UserProximityEvent: [Function],
      WebGLContextEvent: [Function],
      TextEvent: [Function],
      Permissions: [class Permissions],
      History: [class History],
      Navigator: [class Navigator],
      Clipboard: [class Clipboard],
      TimeRanges: [class TimeRanges],
      TextTrackCueList: [class TextTrackCueList extends Array],
      ValidityState: [class ValidityState],
      MutationRecord: [class MutationRecord],
      IntersectionObserver: [class IntersectionObserver],
      IntersectionObserverEntry: [class IntersectionObserverEntry],
      CSSStyleDeclaration: [class CSSStyleDeclaration],
      CSSRule: [Function],
      CSSContainerRule: [class CSSContainerRule extends CSSConditionRule],
      CSSFontFaceRule: [class CSSFontFaceRule extends CSSRule],
      CSSKeyframeRule: [class CSSKeyframeRule extends CSSRule],
      CSSKeyframesRule: [class CSSKeyframesRule extends CSSRule],
      CSSMediaRule: [class CSSMediaRule extends CSSConditionRule],
      CSSStyleRule: [class CSSStyleRule extends CSSGroupingRule],
      CSSSupportsRule: [class CSSSupportsRule extends CSSConditionRule],
      CSSConditionRule: [class CSSConditionRule extends CSSGroupingRule],
      CSSGroupingRule: [class CSSGroupingRule extends CSSRule],
      CSSScopeRule: [class CSSScopeRule extends CSSGroupingRule],
      DOMRect: [class DOMRect extends DOMRectReadOnly],
      DOMRectReadOnly: [class DOMRectReadOnly],
      Plugin: [class Plugin],
      PluginArray: [class PluginArray],
      Location: [class Location],
      CustomElementRegistry: [class CustomElementRegistry],
      ResizeObserver: [class ResizeObserver],
      Blob: [class Blob],
      File: [class File extends Blob],
      Storage: [class Storage],
      CookieStore: [class CookieStore extends EventTarget],
      MimeType: [class MimeType],
      MimeTypeArray: [class MimeTypeArray],
      NodeFilter: [Object],
      ImageBitmap: [class ImageBitmap],
      HTMLCollection: [class HTMLCollection],
      HTMLFormControlCollection: [class HTMLFormControlsCollection extends HTMLCollection],
      HTMLOptionsCollection: [class HTMLOptionsCollection extends HTMLCollection],
      NodeList: [class NodeList],
      RadioNodeList: [class RadioNodeList extends NodeList],
      FileList: [class FileList extends Array],
      Screen: [class Screen extends EventTarget],
      ScreenDetails: [class ScreenDetails extends EventTarget],
      ScreenDetailed: [class ScreenDetailed extends Screen],
      NamedNodeMap: [class NamedNodeMap],
      TreeWalker: [class TreeWalker],
      NodeIterator: [class NodeIterator],
      DataTransfer: [class DataTransfer],
      DataTransferItem: [class DataTransferItem],
      DataTransferItemList: [class DataTransferItemList extends Array],
      XMLSerializer: [class XMLSerializer],
      ClipboardItem: [class ClipboardItem],
      Selection: [class Selection],
      CSSUnitValue: [class CSSUnitValue],
      SVGAngle: [Function],
      SVGAnimatedAngle: [class SVGAnimatedAngle],
      SVGAnimatedBoolean: [class SVGAnimatedBoolean],
      SVGAnimatedEnumeration: [class SVGAnimatedEnumeration],
      SVGAnimatedInteger: [class SVGAnimatedInteger],
      SVGAnimatedLength: [class SVGAnimatedLength],
      SVGAnimatedNumber: [class SVGAnimatedNumber],
      SVGAnimatedNumberList: [class SVGAnimatedNumberList],
      SVGAnimatedPreserveAspectRatio: [class SVGAnimatedPreserveAspectRatio],
      SVGAnimatedRect: [class SVGAnimatedRect],
      SVGAnimatedString: [class SVGAnimatedString],
      SVGAnimatedTransformList: [class SVGAnimatedTransformList],
      SVGLength: [Function],
      SVGLengthList: [class SVGLengthList],
      SVGMatrix: [class SVGMatrix],
      SVGNumber: [class SVGNumber],
      SVGNumberList: [class SVGNumberList],
      SVGPoint: [class SVGPoint],
      SVGPointList: [class SVGPointList],
      SVGPreserveAspectRatio: [Function],
      SVGRect: [class SVGRect],
      SVGStringList: [class SVGStringList],
      SVGTransform: [Function],
      SVGTransformList: [class SVGTransformList],
      SVGAnimatedLengthList: [class SVGAnimatedLengthList],
      SVGUnitTypes: [Function],
      StylePropertyMap: [class StylePropertyMap extends StylePropertyMapReadOnly],
      StylePropertyMapReadOnly: [class StylePropertyMapReadOnly],
      MediaList: [class MediaList],
      CSSKeywordValue: [class CSSKeywordValue],
      CSSStyleValue: [class CSSStyleValue],
      Window: [class GlobalWindow extends Window],
      URLSearchParams: [class URLSearchParams],
      WritableStream: [Function],
      ReadableStream: [Function],
      TransformStream: [Function: Transform],
      PerformanceObserver: [class PerformanceObserver],
      PerformanceEntry: [class PerformanceEntry],
      PerformanceObserverEntryList: [class PerformanceObserverEntryList],
      document: [HTMLDocument],
      customElements: [CustomElementRegistry],
      window: [Circular *3],
      globalThis: [Circular *3],
      performance: [Performance],
      screenLeft: 0,
      screenTop: 0,
      screenX: 0,
      screenY: 0,
      crypto: Crypto {},
      TextEncoder: [class TextEncoder],
      TextDecoder: [class TextDecoder],
      closed: false,
      console: [Object [console]],
      name: '',
      Buffer: [Function],
      Document: [class Document extends Document],
      HTMLDocument: [class HTMLDocument extends HTMLDocument],
      XMLDocument: [class XMLDocument extends XMLDocument],
      DocumentFragment: [class DocumentFragment extends DocumentFragment],
      Text: [class Text extends Text],
      Comment: [class Comment extends Comment],
      Image: [class Image extends Image],
      Audio: [class Audio extends Audio],
      MutationObserver: [class MutationObserver extends MutationObserver],
      MessagePort: [class MessagePort extends MessagePort],
      CSSStyleSheet: [class CSSStyleSheet extends CSSStyleSheet],
      DOMException: [class DOMException extends DOMException],
      Headers: [class Headers extends Headers],
      Request: [class Request extends Request],
      Response: [Function],
      EventTarget: [class EventTarget extends EventTarget],
      XMLHttpRequestUpload: [class XMLHttpRequestUpload extends XMLHttpRequestUpload],
      XMLHttpRequestEventTarget: [class XMLHttpRequestEventTarget extends XMLHttpRequestEventTarget],
      AbortController: [class AbortController extends AbortController],
      AbortSignal: [Function],
      FormData: [class FormData extends FormData],
      PermissionStatus: [class PermissionStatus extends PermissionStatus],
      XMLHttpRequest: [class XMLHttpRequest extends XMLHttpRequest],
      DOMParser: [class DOMParser extends DOMParser],
      Range: [class Range extends Range],
      VTTCue: [class VTTCue extends VTTCue],
      TextTrack: [class TextTrack extends TextTrack],
      TextTrackList: [class TextTrackList extends TextTrackList],
      TextTrackCue: [class TextTrackCue extends TextTrackCue],
      RemotePlayback: [class RemotePlayback extends RemotePlayback],
      FileReader: [class FileReader extends FileReader],
      MediaStream: [class MediaStream extends MediaStream],
      MediaStreamTrack: [class MediaStreamTrack extends MediaStreamTrack],
      CanvasCaptureMediaStreamTrack: [class CanvasCaptureMediaStreamTrack extends CanvasCaptureMediaStreamTrack],
      URL: [class URL extends URL],
      WebSocket: [class WebSocket extends WebSocket],
      ImageData: [class ImageData extends ImageData],
      OffscreenCanvas: [class OffscreenCanvas extends OffscreenCanvas],
      DOMMatrixReadOnly: [class DOMMatrixReadOnly extends DOMMatrixReadOnly],
      DOMMatrix: [class DOMMatrix extends DOMMatrix],
      DOMPointReadOnly: [class DOMPointReadOnly extends DOMPointReadOnly],
      DOMPoint: [class DOMPoint extends DOMPoint],
      Animation: [class Animation extends Animation],
      KeyframeEffect: [class KeyframeEffect extends KeyframeEffect],
      happyDOM: DetachedWindowAPI {},
      Array: [Function: Array],
      ArrayBuffer: [Function: ArrayBuffer],
      Boolean: [Function: Boolean],
      DataView: [Function: DataView],
      Date: [Function: Date],
      Error: [Function],
      EvalError: [Function: EvalError],
      Float32Array: [Function: Float32Array],
      Float64Array: [Function: Float64Array],
      Function: [Function: Function],
      Infinity: Infinity,
      Int16Array: [Function: Int16Array],
      Int32Array: [Function: Int32Array],
      Int8Array: [Function: Int8Array],
      Intl: Object [Intl] {},
      JSON: Object [JSON] {},
      Map: [Function: Map],
      Math: Object [Math] {},
      NaN: NaN,
      Number: [Function: Number],
      Object: [Function: Object],
      Promise: [Function: Promise],
      RangeError: [Function: RangeError],
      ReferenceError: [Function: ReferenceError],
      RegExp: [Function: RegExp],
      Set: [Function: Set],
      String: [Function: String],
      Symbol: [Function: Symbol],
      SyntaxError: [Function: SyntaxError],
      TypeError: [Function: TypeError],
      URIError: [Function: URIError],
      Uint16Array: [Function: Uint16Array],
      Uint32Array: [Function: Uint32Array],
      Uint8Array: [Function: Uint8Array],
      Uint8ClampedArray: [Function: Uint8ClampedArray],
      WeakMap: [Function: WeakMap],
      WeakSet: [Function: WeakSet],
      decodeURI: [Function: decodeURI],
      decodeURIComponent: [Function: decodeURIComponent],
      encodeURI: [Function: encodeURI],
      encodeURIComponent: [Function: encodeURIComponent],
      eval: [Function: eval],
      escape: [Function: escape],
      global: [Object [global]],
      isFinite: [Function: isFinite],
      isNaN: [Function: isNaN],
      parseFloat: [Function: parseFloat],
      parseInt: [Function: parseInt],
      undefined: undefined,
      unescape: [Function: unescape],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(mutationObservers)]: [Array],
      [Symbol(readyStateManager)]: DocumentReadyStateManager {},
      [Symbol(location)]: [Location],
      [Symbol(history)]: History {},
      [Symbol(navigator)]: Navigator {},
      [Symbol(screen)]: [Screen [EventTarget]],
      [Symbol(sessionStorage)]: [Storage],
      [Symbol(localStorage)]: [Storage],
      [Symbol(cookieStore)]: [CookieStore [EventTarget]],
      [Symbol(self)]: [Circular *3],
      [Symbol(top)]: [Circular *3],
      [Symbol(parent)]: [Circular *3],
      [Symbol(window)]: [Circular *3],
      [Symbol(frames)]: [Circular *3],
      [Symbol(internalId)]: 0,
      [Symbol(customElementReactionStack)]: [CustomElementReactionStack],
      [Symbol(modules)]: [Object],
      [Symbol(moduleImportMap)]: null,
      [Symbol(openWebSockets)]: [],
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(querySelectorCache)]: [Map]
    },
    [Symbol(classList)]: null,
    [Symbol(isValue)]: null,
    [Symbol(nodeType)]: 1,
    [Symbol(prefix)]: null,
    [Symbol(shadowRoot)]: null,
    [Symbol(scrollHeight)]: 0,
    [Symbol(scrollWidth)]: 0,
    [Symbol(scrollTop)]: 0,
    [Symbol(scrollLeft)]: 0,
    [Symbol(attributes)]: NamedNodeMap {
      [Symbol(itemsByNamespaceURI)]: [Map],
      [Symbol(itemsByName)]: [Map],
      [Symbol(items)]: [Map],
      [Symbol(ownerElement)]: [Circular *2]
    },
    [Symbol(attributesProxy)]: null,
    [Symbol(children)]: null,
    [Symbol(computedStyle)]: null,
    [Symbol(pointerCaptures)]: Set(0) {},
    [Symbol(propertyEventListeners)]: Map(0) {},
    [Symbol(animations)]: [],
    [Symbol(tagName)]: 'STYLE',
    [Symbol(localName)]: 'style',
    [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
    [Symbol(accessKey)]: '',
    [Symbol(offsetHeight)]: 0,
    [Symbol(offsetWidth)]: 0,
    [Symbol(offsetLeft)]: 0,
    [Symbol(offsetTop)]: 0,
    [Symbol(clientHeight)]: 0,
    [Symbol(clientWidth)]: 0,
    [Symbol(clientLeft)]: 0,
    [Symbol(clientTop)]: 0,
    [Symbol(style)]: null,
    [Symbol(dataset)]: null,
    [Symbol(customElementDefineCallback)]: null,
    [Symbol(sheet)]: null,
    [Symbol(disabled)]: false
  },
  className: 'tiptap',
  editorView: <ref *5> EditorView {
    _root: null,
    focused: false,
    trackWrites: null,
    mounted: false,
    markCursor: null,
    cursorWrapper: null,
    lastSelectedViewDesc: undefined,
    input: InputState {
      shiftKey: false,
      mouseDown: null,
      lastKeyCode: null,
      lastKeyCodeTime: 0,
      lastClick: [Object],
      lastSelectionOrigin: null,
      lastSelectionTime: 0,
      lastIOSEnter: 0,
      lastIOSEnterFallbackTimeout: -1,
      lastFocus: 0,
      lastTouch: 0,
      lastChromeDelete: 0,
      composing: false,
      compositionNode: null,
      composingTimeout: -1,
      compositionNodes: [],
      compositionEndedAt: -200000000,
      compositionID: 1,
      badSafariComposition: false,
      compositionPendingChanges: 0,
      domChangeCount: 0,
      eventHandlers: [Object: null prototype],
      hideSelectionGuard: null
    },
    prevDirectPlugins: [],
    pluginViews: [
      [Object],
      [Object],
      [Object],
      [Object],
      [DropCursorView],
      [Object],
      [Object],
      [Object]
    ],
    requiresGeckoHackNode: false,
    dragging: null,
    _props: {
      attributes: [Object],
      dispatchTransaction: [Function: bound dispatchTransaction],
      transformPastedHTML: [Function (anonymous)],
      state: [EditorState],
      markViews: {},
      nodeViews: [Object]
    },
    state: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    directPlugins: [],
    dispatch: [Function: bound ],
    dom: HTMLDivElement {
      pmViewDesc: [NodeViewDesc],
      editor: [Circular *4],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: [HTMLDivElement],
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [Array],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: [DOMTokenList],
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: [CSSStyleDeclaration],
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    editable: true,
    nodeViews: [Object: null prototype] { taskItem: [Function: nodeview] },
    docView: NodeViewDesc {
      parent: undefined,
      children: [Array],
      dom: [HTMLDivElement],
      contentDOM: [HTMLDivElement],
      dirty: 0,
      node: [Node],
      outerDeco: [Array],
      innerDeco: [DecorationSet],
      nodeDOM: [HTMLDivElement]
    },
    domObserver: DOMObserver {
      view: [Circular *5],
      handleDOMChange: [Function (anonymous)],
      queue: [],
      flushingSoon: -1,
      observer: MutationObserver {},
      currentSelection: [SelectionState],
      onCharData: null,
      suppressingSelectionUpdates: false,
      lastChangedTextNode: null,
      onSelectionChange: [Function: bound onSelectionChange]
    }
  },
  isFocused: false,
  destroyed: false,
  isInitialized: false,
  extensionStorage: {
    link: {},
    paragraph: {},
    listItemBranchingDeleteKeymap: {},
    taskItemBranchingDeleteKeymap: {},
    textStyle: {},
    editable: {},
    clipboardTextSerializer: {},
    commands: {},
    focusEvents: {},
    keymap: {},
    tabindex: {},
    drop: {},
    paste: {},
    delete: {},
    textDirection: {},
    starterKit: {},
    bold: {},
    blockquote: {},
    bulletList: {},
    code: {},
    doc: {},
    dropCursor: {},
    gapCursor: {},
    hardBreak: {},
    undoRedo: {},
    italic: {},
    listItem: {},
    listKeymap: {},
    orderedList: {},
    strike: {},
    text: {},
    underline: {},
    trailingNode: {},
    taskList: {},
    taskItem: {},
    color: {},
    fontSize: {},
    textAlign: {},
    typography: {},
    placeholder: {}
  },
  instanceId: 'ghfenf8',
  hasWarnedStaleDecorationRead: false,
  options: {
    element: HTMLDivElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    content: { type: 'doc', content: [Array] },
    injectCSS: true,
    injectNonce: undefined,
    extensions: [
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    autofocus: false,
    editable: true,
    textDirection: undefined,
    editorProps: { attributes: [Object] },
    parseOptions: {},
    coreExtensionOptions: {},
    enableInputRules: true,
    enablePasteRules: true,
    enableCoreExtensions: true,
    enableContentCheck: false,
    emitContentError: false,
    onBeforeCreate: [Function: onBeforeCreate],
    onCreate: [Function: onCreate],
    onMount: [Function: onMount],
    onUnmount: [Function: onUnmount],
    onUpdate: [Function: onUpdate],
    onSelectionUpdate: [Function: onSelectionUpdate],
    onTransaction: [Function: onTransaction],
    onFocus: [Function: onFocus],
    onBlur: [Function: onBlur],
    onDestroy: [Function: onDestroy],
    onContentError: [Function: onContentError],
    onPaste: [Function: onPaste],
    onDrop: [Function: onDrop],
    onDelete: [Function: onDelete],
    enableExtensionDispatchTransaction: true
  },
  isCapturingTransaction: false,
  capturedTransaction: null,
  utils: {
    getUpdatedPosition: [Function: getUpdatedPosition],
    createMappablePosition: [Function: createMappablePosition]
  },
  extensionManager: ExtensionManager {
    splittableMarks: [ 'textStyle', 'bold', 'code', 'italic', 'strike', 'underline' ],
    nonClearableMarks: [],
    decorationManager: DecorationManager {
      warnedWidgetKeys: Set(0) {},
      warnedOutOfRangeExtensions: Set(0) {},
      handleBeforeTransaction: [Function (anonymous)],
      editor: [Circular *4],
      entries: [],
      plugin: null
    },
    editor: [Circular *4],
    baseExtensions: [
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    extensions: [
      [Mark],      [Node],      [Extension],
      [Extension], [Mark],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Mark],      [Node],
      [Node],      [Mark],      [Node],
      [Extension], [Extension], [Node],
      [Extension], [Mark],      [Node],
      [Extension], [Node],      [Mark],
      [Node],      [Mark],      [Extension],
      [Node],      [Node],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension]
    ],
    schema: Schema {
      linebreakReplacement: [NodeType],
      cached: [Object: null prototype],
      spec: [Object],
      nodes: [Object: null prototype],
      marks: [Object: null prototype],
      nodeFromJSON: [Function (anonymous)],
      markFromJSON: [Function (anonymous)],
      topNodeType: [NodeType]
    }
  },
  commandManager: CommandManager {
    editor: [Circular *4],
    rawCommands: {
      setLink: [Function: setLink],
      toggleLink: [Function: toggleLink],
      unsetLink: [Function: unsetLink],
      setParagraph: [Function: setParagraph],
      toggleTextStyle: [Function: toggleTextStyle],
      removeEmptyTextStyle: [Function: removeEmptyTextStyle],
      blur: [Function: blur],
      clearContent: [Function: clearContent],
      clearNodes: [Function: clearNodes],
      command: [Function: command],
      createParagraphNear: [Function: createParagraphNear$1],
      cut: [Function: cut],
      deleteCurrentNode: [Function: deleteCurrentNode],
      deleteNode: [Function: deleteNode],
      deleteRange: [Function: deleteRange],
      deleteSelection: [Function: deleteSelection],
      enter: [Function: enter],
      exitCode: [Function: exitCode$1],
      extendMarkRange: [Function: extendMarkRange],
      first: [Function: first],
      focus: [Function: focus],
      forEach: [Function: forEach],
      insertContent: [Function: insertContent],
      insertContentAt: [Function: insertContentAt],
      insertDefaultBlock: [Function: insertDefaultBlock],
      joinBackward: [Function: joinBackward$1],
      joinDown: [Function: joinDown$1],
      joinForward: [Function: joinForward$1],
      joinItemBackward: [Function: joinItemBackward],
      joinItemForward: [Function: joinItemForward],
      joinTextblockBackward: [Function: joinTextblockBackward$1],
      joinTextblockForward: [Function: joinTextblockForward$1],
      joinUp: [Function: joinUp$1],
      keyboardShortcut: [Function: keyboardShortcut],
      lift: [Function: lift$1],
      liftEmptyBlock: [Function: liftEmptyBlock$1],
      liftListItem: [Function: liftListItem$1],
      newlineInCode: [Function: newlineInCode$1],
      resetAttributes: [Function: resetAttributes],
      scrollIntoView: [Function: scrollIntoView],
      selectAll: [Function: selectAll],
      selectNodeBackward: [Function: selectNodeBackward$1],
      selectNodeForward: [Function: selectNodeForward$1],
      selectParentNode: [Function: selectParentNode$1],
      selectTextblockEnd: [Function: selectTextblockEnd$1],
      selectTextblockStart: [Function: selectTextblockStart$1],
      setContent: [Function: setContent],
      setMark: [Function: setMark],
      setMeta: [Function: setMeta],
      setNode: [Function: setNode],
      setNodeSelection: [Function: setNodeSelection],
      setTextDirection: [Function: setTextDirection],
      setTextSelection: [Function: setTextSelection],
      sinkListItem: [Function: sinkListItem$1],
      splitBlock: [Function: splitBlock],
      splitListItem: [Function: splitListItem],
      toggleList: [Function: toggleList],
      toggleMark: [Function: toggleMark],
      toggleNode: [Function: toggleNode],
      toggleWrap: [Function: toggleWrap],
      undoInputRule: [Function: undoInputRule],
      unsetAllMarks: [Function: unsetAllMarks],
      unsetMark: [Function: unsetMark],
      unsetTextDirection: [Function: unsetTextDirection],
      updateAttributes: [Function: updateAttributes],
      updateDecorations: [Function: updateDecorations],
      wrapIn: [Function: wrapIn$1],
      wrapInList: [Function: wrapInList$1],
      setBold: [Function: setBold],
      toggleBold: [Function: toggleBold],
      unsetBold: [Function: unsetBold],
      setBlockquote: [Function: setBlockquote],
      toggleBlockquote: [Function: toggleBlockquote],
      unsetBlockquote: [Function: unsetBlockquote],
      toggleBulletList: [Function: toggleBulletList],
      setCode: [Function: setCode],
      toggleCode: [Function: toggleCode],
      unsetCode: [Function: unsetCode],
      setHardBreak: [Function: setHardBreak],
      undo: [Function: undo],
      redo: [Function: redo],
      setItalic: [Function: setItalic],
      toggleItalic: [Function: toggleItalic],
      unsetItalic: [Function: unsetItalic],
      toggleOrderedList: [Function: toggleOrderedList],
      setStrike: [Function: setStrike],
      toggleStrike: [Function: toggleStrike],
      unsetStrike: [Function: unsetStrike],
      setUnderline: [Function: setUnderline],
      toggleUnderline: [Function: toggleUnderline],
      unsetUnderline: [Function: unsetUnderline],
      toggleTaskList: [Function: toggleTaskList],
      setColor: [Function: setColor],
      unsetColor: [Function: unsetColor],
      setFontSize: [Function: setFontSize],
      unsetFontSize: [Function: unsetFontSize],
      setTextAlign: [Function: setTextAlign],
      unsetTextAlign: [Function: unsetTextAlign],
      toggleTextAlign: [Function: toggleTextAlign]
    },
    customState: undefined
  },
  schema: <ref *6> Schema {
    linebreakReplacement: NodeType {
      name: 'hardBreak',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [Array],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: false,
      isText: false
    },
    cached: [Object: null prototype] { wrappings: [Object: null prototype] {} },
    spec: { topNode: 'doc', nodes: [OrderedMap], marks: [OrderedMap] },
    nodes: [Object: null prototype] {
      paragraph: [NodeType],
      blockquote: [NodeType],
      bulletList: [NodeType],
      doc: [NodeType],
      hardBreak: [NodeType],
      listItem: [NodeType],
      orderedList: [NodeType],
      text: [NodeType],
      taskList: [NodeType],
      taskItem: [NodeType]
    },
    marks: [Object: null prototype] {
      link: [MarkType],
      textStyle: [MarkType],
      bold: [MarkType],
      code: [MarkType],
      italic: [MarkType],
      strike: [MarkType],
      underline: [MarkType]
    },
    nodeFromJSON: [Function (anonymous)],
    markFromJSON: [Function (anonymous)],
    topNodeType: NodeType {
      name: 'doc',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: true,
      isText: false
    }
  },
  editorState: EditorState {
    config: Configuration {
      schema: [Schema],
      plugins: [],
      pluginsByKey: [Object: null prototype] {},
      fields: [Array]
    },
    doc: Node {
      type: [NodeType],
      attrs: [Object: null prototype] {},
      marks: [],
      content: [Fragment]
    },
    selection: TextSelection {
      '$anchor': [ResolvedPos],
      '$head': [ResolvedPos],
      ranges: [Array]
    },
    storedMarks: null,
    scrollToSelection: 0
  },
  contentComponent: null,
  appContext: null,
  reactiveState: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  },
  reactiveExtensionStorage: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  }
} default-font-size=undefined  ... > 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[Vue warn]: Invalid prop: type check failed for prop "selected". Expected String with value "undefined", got Undefined  
  at <SpaceCanvasTextAlignmentMenu editor= <ref *4> Editor {
  callbacks: {
    create: [ [Function: bound onCreate], [Function: onCreate] ],
    destroy: [ [Function: bound onDestroy], [Function: onDestroy] ],
    update: [ [Function: bound onUpdate], [Function: onUpdate] ],
    beforeCreate: [ [Function: onBeforeCreate] ],
    mount: [ [Function: onMount] ],
    unmount: [ [Function: onUnmount] ],
    contentError: [ [Function: onContentError] ],
    selectionUpdate: [ [Function: onSelectionUpdate] ],
    transaction: [ [Function: onTransaction] ],
    focus: [ [Function: onFocus] ],
    blur: [ [Function: onBlur] ],
    drop: [ [Function (anonymous)] ],
    paste: [ [Function (anonymous)] ],
    delete: [ [Function: onDelete] ],
    beforeTransaction: [ [Function (anonymous)], [Function (anonymous)] ]
  },
  css: <ref *2> HTMLStyleElement {
    [Symbol(listeners)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(listenerOptions)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(isConnected)]: true,
    [Symbol(parentNode)]: HTMLHeadElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: [HTMLHtmlElement],
      [Symbol(rootNode)]: [HTMLDocument],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'HEAD',
      [Symbol(localName)]: 'head',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    [Symbol(rootNode)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(styleNode)]: [Circular *2],
    [Symbol(textAreaNode)]: null,
    [Symbol(formNode)]: null,
    [Symbol(selectNode)]: null,
    [Symbol(mutationListeners)]: [],
    [Symbol(nodeArray)]: [ [Text] ],
    [Symbol(elementArray)]: [],
    [Symbol(childNodes)]: null,
    [Symbol(assignedToSlot)]: null,
    [Symbol(cache)]: {
      querySelector: Map(0) {},
      querySelectorAll: Map(0) {},
      matches: Map(0) {},
      elementsByTagName: Map(0) {},
      elementsByTagNameNS: Map(0) {},
      elementByTagName: Map(0) {},
      elementById: Map(0) {},
      computedStyle: null
    },
    [Symbol(affectsCache)]: [],
    [Symbol(ownerDocument)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(window)]: <ref *3> GlobalWindow [EventTarget] {
      Node: [Function],
      Attr: [class Attr extends Node],
      ShadowRoot: [class ShadowRoot extends DocumentFragment],
      ProcessingInstruction: [class ProcessingInstruction extends CharacterData],
      Element: [Function],
      CharacterData: [class CharacterData extends Node],
      DocumentType: [class DocumentType extends Node],
      AnimationTimeline: [class AnimationTimeline],
      DocumentTimeline: [class DocumentTimeline extends AnimationTimeline],
      HTMLAnchorElement: [class HTMLAnchorElement extends HTMLElement],
      HTMLButtonElement: [class HTMLButtonElement extends HTMLElement],
      HTMLOptGroupElement: [class HTMLOptGroupElement extends HTMLElement],
      HTMLOptionElement: [class HTMLOptionElement extends HTMLElement],
      HTMLElement: [Function],
      HTMLUnknownElement: [class HTMLUnknownElement extends HTMLElement],
      HTMLTemplateElement: [class HTMLTemplateElement extends HTMLElement],
      HTMLInputElement: [class HTMLInputElement extends HTMLElement],
      HTMLSelectElement: [class HTMLSelectElement extends HTMLElement],
      HTMLTextAreaElement: [class HTMLTextAreaElement extends HTMLElement],
      HTMLImageElement: [class HTMLImageElement extends HTMLElement],
      HTMLStyleElement: [class HTMLStyleElement extends HTMLElement],
      HTMLLabelElement: [class HTMLLabelElement extends HTMLElement],
      HTMLSlotElement: [class HTMLSlotElement extends HTMLElement],
      HTMLMetaElement: [class HTMLMetaElement extends HTMLElement],
      HTMLMediaElement: [class HTMLMediaElement extends HTMLElement],
      HTMLAudioElement: [class HTMLAudioElement extends HTMLMediaElement],
      HTMLVideoElement: [class HTMLVideoElement extends HTMLMediaElement],
      HTMLBaseElement: [class HTMLBaseElement extends HTMLElement],
      HTMLDialogElement: [class HTMLDialogElement extends HTMLElement],
      HTMLScriptElement: [class HTMLScriptElement extends HTMLElement],
      HTMLLinkElement: [class HTMLLinkElement extends HTMLElement],
      HTMLIFrameElement: [class HTMLIFrameElement extends HTMLElement],
      HTMLFormElement: [class HTMLFormElement extends HTMLElement],
      HTMLUListElement: [class HTMLUListElement extends HTMLElement],
      HTMLTrackElement: [class HTMLTrackElement extends HTMLElement],
      HTMLTableRowElement: [class HTMLTableRowElement extends HTMLElement],
      HTMLTitleElement: [class HTMLTitleElement extends HTMLElement],
      HTMLTimeElement: [class HTMLTimeElement extends HTMLElement],
      HTMLTableSectionElement: [class HTMLTableSectionElement extends HTMLElement],
      HTMLTableCellElement: [class HTMLTableCellElement extends HTMLElement],
      HTMLTableElement: [class HTMLTableElement extends HTMLElement],
      HTMLSpanElement: [class HTMLSpanElement extends HTMLElement],
      HTMLSourceElement: [class HTMLSourceElement extends HTMLElement],
      HTMLQuoteElement: [class HTMLQuoteElement extends HTMLElement],
      HTMLProgressElement: [class HTMLProgressElement extends HTMLElement],
      HTMLPreElement: [class HTMLPreElement extends HTMLElement],
      HTMLPictureElement: [class HTMLPictureElement extends HTMLElement],
      HTMLParamElement: [class HTMLParamElement extends HTMLElement],
      HTMLParagraphElement: [class HTMLParagraphElement extends HTMLElement],
      HTMLOutputElement: [class HTMLOutputElement extends HTMLElement],
      HTMLOListElement: [class HTMLOListElement extends HTMLElement],
      HTMLObjectElement: [class HTMLObjectElement extends HTMLElement],
      HTMLMeterElement: [class HTMLMeterElement extends HTMLElement],
      HTMLMenuElement: [class HTMLMenuElement extends HTMLElement],
      HTMLMapElement: [class HTMLMapElement extends HTMLElement],
      HTMLLIElement: [class HTMLLIElement extends HTMLElement],
      HTMLLegendElement: [class HTMLLegendElement extends HTMLElement],
      HTMLModElement: [class HTMLModElement extends HTMLElement],
      HTMLHtmlElement: [class HTMLHtmlElement extends HTMLElement],
      HTMLHRElement: [class HTMLHRElement extends HTMLElement],
      HTMLHeadElement: [class HTMLHeadElement extends HTMLElement],
      HTMLHeadingElement: [class HTMLHeadingElement extends HTMLElement],
      HTMLFieldSetElement: [class HTMLFieldSetElement extends HTMLElement],
      HTMLEmbedElement: [class HTMLEmbedElement extends HTMLElement],
      HTMLDListElement: [class HTMLDListElement extends HTMLElement],
      HTMLDivElement: [class HTMLDivElement extends HTMLElement],
      HTMLDetailsElement: [class HTMLDetailsElement extends HTMLElement],
      HTMLDataListElement: [class HTMLDataListElement extends HTMLElement],
      HTMLDataElement: [class HTMLDataElement extends HTMLElement],
      HTMLTableColElement: [class HTMLTableColElement extends HTMLElement],
      HTMLTableCaptionElement: [class HTMLTableCaptionElement extends HTMLElement],
      HTMLCanvasElement: [class HTMLCanvasElement extends HTMLElement],
      HTMLBRElement: [class HTMLBRElement extends HTMLElement],
      HTMLBodyElement: [class HTMLBodyElement extends HTMLElement],
      HTMLAreaElement: [class HTMLAreaElement extends HTMLElement],
      SVGSVGElement: [class SVGSVGElement extends SVGGraphicsElement],
      SVGAnimateElement: [class SVGAnimateElement extends SVGAnimationElement],
      SVGAnimateMotionElement: [class SVGAnimateMotionElement extends SVGAnimationElement],
      SVGAnimateTransformElement: [class SVGAnimateTransformElement extends SVGAnimationElement],
      SVGCircleElement: [class SVGCircleElement extends SVGGeometryElement],
      SVGClipPathElement: [class SVGClipPathElement extends SVGElement],
      SVGDefsElement: [class SVGDefsElement extends SVGGraphicsElement],
      SVGDescElement: [class SVGDescElement extends SVGElement],
      SVGEllipseElement: [class SVGEllipseElement extends SVGGeometryElement],
      SVGFEBlendElement: [Function],
      SVGFEColorMatrixElement: [Function],
      SVGFEComponentTransferElement: [class SVGFEComponentTransferElement extends SVGElement],
      SVGFECompositeElement: [Function],
      SVGFEConvolveMatrixElement: [Function],
      SVGFEDiffuseLightingElement: [class SVGFEDiffuseLightingElement extends SVGElement],
      SVGFEDisplacementMapElement: [Function],
      SVGFEDistantLightElement: [class SVGFEDistantLightElement extends SVGElement],
      SVGFEDropShadowElement: [class SVGFEDropShadowElement extends SVGElement],
      SVGFEFloodElement: [class SVGFEFloodElement extends SVGElement],
      SVGFEFuncAElement: [class SVGFEFuncAElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncBElement: [class SVGFEFuncBElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncGElement: [class SVGFEFuncGElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncRElement: [class SVGFEFuncRElement extends SVGComponentTransferFunctionElement],
      SVGFEGaussianBlurElement: [Function],
      SVGFEImageElement: [class SVGFEImageElement extends SVGElement],
      SVGFEMergeElement: [class SVGFEMergeElement extends SVGElement],
      SVGFEMergeNodeElement: [class SVGFEMergeNodeElement extends SVGElement],
      SVGFEMorphologyElement: [Function],
      SVGFEOffsetElement: [class SVGFEOffsetElement extends SVGElement],
      SVGFEPointLightElement: [class SVGFEPointLightElement extends SVGElement],
      SVGFESpecularLightingElement: [class SVGFESpecularLightingElement extends SVGElement],
      SVGFESpotLightElement: [class SVGFESpotLightElement extends SVGElement],
      SVGFETileElement: [class SVGFETileElement extends SVGElement],
      SVGFETurbulenceElement: [Function],
      SVGFilterElement: [class SVGFilterElement extends SVGElement],
      SVGForeignObjectElement: [class SVGForeignObjectElement extends SVGGraphicsElement],
      SVGGElement: [class SVGGElement extends SVGGraphicsElement],
      SVGImageElement: [class SVGImageElement extends SVGGraphicsElement],
      SVGLineElement: [class SVGLineElement extends SVGGeometryElement],
      SVGLinearGradientElement: [class SVGLinearGradientElement extends SVGGradientElement],
      SVGMarkerElement: [Function],
      SVGMaskElement: [class SVGMaskElement extends SVGElement],
      SVGMetadataElement: [class SVGMetadataElement extends SVGElement],
      SVGMPathElement: [class SVGMPathElement extends SVGElement],
      SVGPathElement: [class SVGPathElement extends SVGGeometryElement],
      SVGPatternElement: [class SVGPatternElement extends SVGElement],
      SVGPolygonElement: [class SVGPolygonElement extends SVGGeometryElement],
      SVGPolylineElement: [class SVGPolylineElement extends SVGGeometryElement],
      SVGRadialGradientElement: [class SVGRadialGradientElement extends SVGGradientElement],
      SVGRectElement: [class SVGRectElement extends SVGGeometryElement],
      SVGScriptElement: [class SVGScriptElement extends SVGGraphicsElement],
      SVGSetElement: [class SVGSetElement extends SVGAnimationElement],
      SVGStopElement: [class SVGStopElement extends SVGElement],
      SVGStyleElement: [class SVGStyleElement extends SVGElement],
      SVGSwitchElement: [class SVGSwitchElement extends SVGGraphicsElement],
      SVGSymbolElement: [class SVGSymbolElement extends SVGGraphicsElement],
      SVGTextElement: [class SVGTextElement extends SVGTextPositioningElement],
      SVGTextPathElement: [Function],
      SVGTitleElement: [class SVGTitleElement extends SVGElement],
      SVGTSpanElement: [class SVGTSpanElement extends SVGTextPositioningElement],
      SVGUseElement: [class SVGUseElement extends SVGGraphicsElement],
      SVGViewElement: [class SVGViewElement extends SVGElement],
      SVGElement: [class SVGElement extends Element],
      SVGAnimationElement: [class SVGAnimationElement extends SVGElement],
      SVGComponentTransferFunctionElement: [Function],
      SVGGeometryElement: [class SVGGeometryElement extends SVGGraphicsElement],
      SVGGradientElement: [Function],
      SVGTextPositioningElement: [class SVGTextPositioningElement extends SVGTextContentElement],
      SVGGraphicsElement: [class SVGGraphicsElement extends SVGElement],
      Event: [Function],
      UIEvent: [Function],
      CustomEvent: [class CustomEvent extends Event],
      CloseEvent: [class CloseEvent extends Event],
      AnimationEvent: [class AnimationEvent extends Event],
      KeyboardEvent: [Function],
      MessageEvent: [class MessageEvent extends Event],
      MouseEvent: [class MouseEvent extends UIEvent],
      PointerEvent: [class PointerEvent extends MouseEvent],
      FocusEvent: [class FocusEvent extends UIEvent],
      WheelEvent: [Function],
      InputEvent: [class InputEvent extends UIEvent],
      ErrorEvent: [class ErrorEvent extends UIEvent],
      StorageEvent: [class StorageEvent extends Event],
      SubmitEvent: [class SubmitEvent extends Event],
      ProgressEvent: [class ProgressEvent extends Event],
      MediaQueryListEvent: [class MediaQueryListEvent extends Event],
      HashChangeEvent: [class HashChangeEvent extends Event],
      ClipboardEvent: [class ClipboardEvent extends Event],
      CookieChangeEvent: [class CookieChangeEvent extends Event],
      TouchEvent: [class TouchEvent extends UIEvent],
      PopStateEvent: [class PopStateEvent extends Event],
      Touch: [class Touch],
      AudioProcessingEvent: [Function],
      BeforeInputEvent: [Function],
      BeforeUnloadEvent: [Function],
      BlobEvent: [Function],
      CompositionEvent: [Function],
      CSSFontFaceLoadEvent: [Function],
      DeviceLightEvent: [Function],
      DeviceMotionEvent: [Function],
      DeviceOrientationEvent: [Function],
      DeviceProximityEvent: [Function],
      DOMTransactionEvent: [Function],
      DragEvent: [Function],
      EditingBeforeInputEvent: [Function],
      FetchEvent: [Function],
      GamepadEvent: [Function],
      IDBVersionChangeEvent: [Function],
      MediaStreamEvent: [Function],
      MutationEvent: [Function],
      OfflineAudioCompletionEvent: [Function],
      OverconstrainedError: [Function],
      PageTransitionEvent: [Function],
      PaymentRequestUpdateEvent: [Function],
      RelatedEvent: [Function],
      RTCDataChannelEvent: [Function],
      RTCIdentityErrorEvent: [Function],
      RTCIdentityEvent: [Function],
      RTCPeerConnectionIceEvent: [Function],
      SensorEvent: [Function],
      SVGEvent: [Function],
      SVGZoomEvent: [Function],
      TimeEvent: [Function],
      TrackEvent: [Function],
      TransitionEvent: [Function],
      UserProximityEvent: [Function],
      WebGLContextEvent: [Function],
      TextEvent: [Function],
      Permissions: [class Permissions],
      History: [class History],
      Navigator: [class Navigator],
      Clipboard: [class Clipboard],
      TimeRanges: [class TimeRanges],
      TextTrackCueList: [class TextTrackCueList extends Array],
      ValidityState: [class ValidityState],
      MutationRecord: [class MutationRecord],
      IntersectionObserver: [class IntersectionObserver],
      IntersectionObserverEntry: [class IntersectionObserverEntry],
      CSSStyleDeclaration: [class CSSStyleDeclaration],
      CSSRule: [Function],
      CSSContainerRule: [class CSSContainerRule extends CSSConditionRule],
      CSSFontFaceRule: [class CSSFontFaceRule extends CSSRule],
      CSSKeyframeRule: [class CSSKeyframeRule extends CSSRule],
      CSSKeyframesRule: [class CSSKeyframesRule extends CSSRule],
      CSSMediaRule: [class CSSMediaRule extends CSSConditionRule],
      CSSStyleRule: [class CSSStyleRule extends CSSGroupingRule],
      CSSSupportsRule: [class CSSSupportsRule extends CSSConditionRule],
      CSSConditionRule: [class CSSConditionRule extends CSSGroupingRule],
      CSSGroupingRule: [class CSSGroupingRule extends CSSRule],
      CSSScopeRule: [class CSSScopeRule extends CSSGroupingRule],
      DOMRect: [class DOMRect extends DOMRectReadOnly],
      DOMRectReadOnly: [class DOMRectReadOnly],
      Plugin: [class Plugin],
      PluginArray: [class PluginArray],
      Location: [class Location],
      CustomElementRegistry: [class CustomElementRegistry],
      ResizeObserver: [class ResizeObserver],
      Blob: [class Blob],
      File: [class File extends Blob],
      Storage: [class Storage],
      CookieStore: [class CookieStore extends EventTarget],
      MimeType: [class MimeType],
      MimeTypeArray: [class MimeTypeArray],
      NodeFilter: [Object],
      ImageBitmap: [class ImageBitmap],
      HTMLCollection: [class HTMLCollection],
      HTMLFormControlCollection: [class HTMLFormControlsCollection extends HTMLCollection],
      HTMLOptionsCollection: [class HTMLOptionsCollection extends HTMLCollection],
      NodeList: [class NodeList],
      RadioNodeList: [class RadioNodeList extends NodeList],
      FileList: [class FileList extends Array],
      Screen: [class Screen extends EventTarget],
      ScreenDetails: [class ScreenDetails extends EventTarget],
      ScreenDetailed: [class ScreenDetailed extends Screen],
      NamedNodeMap: [class NamedNodeMap],
      TreeWalker: [class TreeWalker],
      NodeIterator: [class NodeIterator],
      DataTransfer: [class DataTransfer],
      DataTransferItem: [class DataTransferItem],
      DataTransferItemList: [class DataTransferItemList extends Array],
      XMLSerializer: [class XMLSerializer],
      ClipboardItem: [class ClipboardItem],
      Selection: [class Selection],
      CSSUnitValue: [class CSSUnitValue],
      SVGAngle: [Function],
      SVGAnimatedAngle: [class SVGAnimatedAngle],
      SVGAnimatedBoolean: [class SVGAnimatedBoolean],
      SVGAnimatedEnumeration: [class SVGAnimatedEnumeration],
      SVGAnimatedInteger: [class SVGAnimatedInteger],
      SVGAnimatedLength: [class SVGAnimatedLength],
      SVGAnimatedNumber: [class SVGAnimatedNumber],
      SVGAnimatedNumberList: [class SVGAnimatedNumberList],
      SVGAnimatedPreserveAspectRatio: [class SVGAnimatedPreserveAspectRatio],
      SVGAnimatedRect: [class SVGAnimatedRect],
      SVGAnimatedString: [class SVGAnimatedString],
      SVGAnimatedTransformList: [class SVGAnimatedTransformList],
      SVGLength: [Function],
      SVGLengthList: [class SVGLengthList],
      SVGMatrix: [class SVGMatrix],
      SVGNumber: [class SVGNumber],
      SVGNumberList: [class SVGNumberList],
      SVGPoint: [class SVGPoint],
      SVGPointList: [class SVGPointList],
      SVGPreserveAspectRatio: [Function],
      SVGRect: [class SVGRect],
      SVGStringList: [class SVGStringList],
      SVGTransform: [Function],
      SVGTransformList: [class SVGTransformList],
      SVGAnimatedLengthList: [class SVGAnimatedLengthList],
      SVGUnitTypes: [Function],
      StylePropertyMap: [class StylePropertyMap extends StylePropertyMapReadOnly],
      StylePropertyMapReadOnly: [class StylePropertyMapReadOnly],
      MediaList: [class MediaList],
      CSSKeywordValue: [class CSSKeywordValue],
      CSSStyleValue: [class CSSStyleValue],
      Window: [class GlobalWindow extends Window],
      URLSearchParams: [class URLSearchParams],
      WritableStream: [Function],
      ReadableStream: [Function],
      TransformStream: [Function: Transform],
      PerformanceObserver: [class PerformanceObserver],
      PerformanceEntry: [class PerformanceEntry],
      PerformanceObserverEntryList: [class PerformanceObserverEntryList],
      document: [HTMLDocument],
      customElements: [CustomElementRegistry],
      window: [Circular *3],
      globalThis: [Circular *3],
      performance: [Performance],
      screenLeft: 0,
      screenTop: 0,
      screenX: 0,
      screenY: 0,
      crypto: Crypto {},
      TextEncoder: [class TextEncoder],
      TextDecoder: [class TextDecoder],
      closed: false,
      console: [Object [console]],
      name: '',
      Buffer: [Function],
      Document: [class Document extends Document],
      HTMLDocument: [class HTMLDocument extends HTMLDocument],
      XMLDocument: [class XMLDocument extends XMLDocument],
      DocumentFragment: [class DocumentFragment extends DocumentFragment],
      Text: [class Text extends Text],
      Comment: [class Comment extends Comment],
      Image: [class Image extends Image],
      Audio: [class Audio extends Audio],
      MutationObserver: [class MutationObserver extends MutationObserver],
      MessagePort: [class MessagePort extends MessagePort],
      CSSStyleSheet: [class CSSStyleSheet extends CSSStyleSheet],
      DOMException: [class DOMException extends DOMException],
      Headers: [class Headers extends Headers],
      Request: [class Request extends Request],
      Response: [Function],
      EventTarget: [class EventTarget extends EventTarget],
      XMLHttpRequestUpload: [class XMLHttpRequestUpload extends XMLHttpRequestUpload],
      XMLHttpRequestEventTarget: [class XMLHttpRequestEventTarget extends XMLHttpRequestEventTarget],
      AbortController: [class AbortController extends AbortController],
      AbortSignal: [Function],
      FormData: [class FormData extends FormData],
      PermissionStatus: [class PermissionStatus extends PermissionStatus],
      XMLHttpRequest: [class XMLHttpRequest extends XMLHttpRequest],
      DOMParser: [class DOMParser extends DOMParser],
      Range: [class Range extends Range],
      VTTCue: [class VTTCue extends VTTCue],
      TextTrack: [class TextTrack extends TextTrack],
      TextTrackList: [class TextTrackList extends TextTrackList],
      TextTrackCue: [class TextTrackCue extends TextTrackCue],
      RemotePlayback: [class RemotePlayback extends RemotePlayback],
      FileReader: [class FileReader extends FileReader],
      MediaStream: [class MediaStream extends MediaStream],
      MediaStreamTrack: [class MediaStreamTrack extends MediaStreamTrack],
      CanvasCaptureMediaStreamTrack: [class CanvasCaptureMediaStreamTrack extends CanvasCaptureMediaStreamTrack],
      URL: [class URL extends URL],
      WebSocket: [class WebSocket extends WebSocket],
      ImageData: [class ImageData extends ImageData],
      OffscreenCanvas: [class OffscreenCanvas extends OffscreenCanvas],
      DOMMatrixReadOnly: [class DOMMatrixReadOnly extends DOMMatrixReadOnly],
      DOMMatrix: [class DOMMatrix extends DOMMatrix],
      DOMPointReadOnly: [class DOMPointReadOnly extends DOMPointReadOnly],
      DOMPoint: [class DOMPoint extends DOMPoint],
      Animation: [class Animation extends Animation],
      KeyframeEffect: [class KeyframeEffect extends KeyframeEffect],
      happyDOM: DetachedWindowAPI {},
      Array: [Function: Array],
      ArrayBuffer: [Function: ArrayBuffer],
      Boolean: [Function: Boolean],
      DataView: [Function: DataView],
      Date: [Function: Date],
      Error: [Function],
      EvalError: [Function: EvalError],
      Float32Array: [Function: Float32Array],
      Float64Array: [Function: Float64Array],
      Function: [Function: Function],
      Infinity: Infinity,
      Int16Array: [Function: Int16Array],
      Int32Array: [Function: Int32Array],
      Int8Array: [Function: Int8Array],
      Intl: Object [Intl] {},
      JSON: Object [JSON] {},
      Map: [Function: Map],
      Math: Object [Math] {},
      NaN: NaN,
      Number: [Function: Number],
      Object: [Function: Object],
      Promise: [Function: Promise],
      RangeError: [Function: RangeError],
      ReferenceError: [Function: ReferenceError],
      RegExp: [Function: RegExp],
      Set: [Function: Set],
      String: [Function: String],
      Symbol: [Function: Symbol],
      SyntaxError: [Function: SyntaxError],
      TypeError: [Function: TypeError],
      URIError: [Function: URIError],
      Uint16Array: [Function: Uint16Array],
      Uint32Array: [Function: Uint32Array],
      Uint8Array: [Function: Uint8Array],
      Uint8ClampedArray: [Function: Uint8ClampedArray],
      WeakMap: [Function: WeakMap],
      WeakSet: [Function: WeakSet],
      decodeURI: [Function: decodeURI],
      decodeURIComponent: [Function: decodeURIComponent],
      encodeURI: [Function: encodeURI],
      encodeURIComponent: [Function: encodeURIComponent],
      eval: [Function: eval],
      escape: [Function: escape],
      global: [Object [global]],
      isFinite: [Function: isFinite],
      isNaN: [Function: isNaN],
      parseFloat: [Function: parseFloat],
      parseInt: [Function: parseInt],
      undefined: undefined,
      unescape: [Function: unescape],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(mutationObservers)]: [Array],
      [Symbol(readyStateManager)]: DocumentReadyStateManager {},
      [Symbol(location)]: [Location],
      [Symbol(history)]: History {},
      [Symbol(navigator)]: Navigator {},
      [Symbol(screen)]: [Screen [EventTarget]],
      [Symbol(sessionStorage)]: [Storage],
      [Symbol(localStorage)]: [Storage],
      [Symbol(cookieStore)]: [CookieStore [EventTarget]],
      [Symbol(self)]: [Circular *3],
      [Symbol(top)]: [Circular *3],
      [Symbol(parent)]: [Circular *3],
      [Symbol(window)]: [Circular *3],
      [Symbol(frames)]: [Circular *3],
      [Symbol(internalId)]: 0,
      [Symbol(customElementReactionStack)]: [CustomElementReactionStack],
      [Symbol(modules)]: [Object],
      [Symbol(moduleImportMap)]: null,
      [Symbol(openWebSockets)]: [],
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(querySelectorCache)]: [Map]
    },
    [Symbol(classList)]: null,
    [Symbol(isValue)]: null,
    [Symbol(nodeType)]: 1,
    [Symbol(prefix)]: null,
    [Symbol(shadowRoot)]: null,
    [Symbol(scrollHeight)]: 0,
    [Symbol(scrollWidth)]: 0,
    [Symbol(scrollTop)]: 0,
    [Symbol(scrollLeft)]: 0,
    [Symbol(attributes)]: NamedNodeMap {
      [Symbol(itemsByNamespaceURI)]: [Map],
      [Symbol(itemsByName)]: [Map],
      [Symbol(items)]: [Map],
      [Symbol(ownerElement)]: [Circular *2]
    },
    [Symbol(attributesProxy)]: null,
    [Symbol(children)]: null,
    [Symbol(computedStyle)]: null,
    [Symbol(pointerCaptures)]: Set(0) {},
    [Symbol(propertyEventListeners)]: Map(0) {},
    [Symbol(animations)]: [],
    [Symbol(tagName)]: 'STYLE',
    [Symbol(localName)]: 'style',
    [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
    [Symbol(accessKey)]: '',
    [Symbol(offsetHeight)]: 0,
    [Symbol(offsetWidth)]: 0,
    [Symbol(offsetLeft)]: 0,
    [Symbol(offsetTop)]: 0,
    [Symbol(clientHeight)]: 0,
    [Symbol(clientWidth)]: 0,
    [Symbol(clientLeft)]: 0,
    [Symbol(clientTop)]: 0,
    [Symbol(style)]: null,
    [Symbol(dataset)]: null,
    [Symbol(customElementDefineCallback)]: null,
    [Symbol(sheet)]: null,
    [Symbol(disabled)]: false
  },
  className: 'tiptap',
  editorView: <ref *5> EditorView {
    _root: null,
    focused: false,
    trackWrites: null,
    mounted: false,
    markCursor: null,
    cursorWrapper: null,
    lastSelectedViewDesc: undefined,
    input: InputState {
      shiftKey: false,
      mouseDown: null,
      lastKeyCode: null,
      lastKeyCodeTime: 0,
      lastClick: [Object],
      lastSelectionOrigin: null,
      lastSelectionTime: 0,
      lastIOSEnter: 0,
      lastIOSEnterFallbackTimeout: -1,
      lastFocus: 0,
      lastTouch: 0,
      lastChromeDelete: 0,
      composing: false,
      compositionNode: null,
      composingTimeout: -1,
      compositionNodes: [],
      compositionEndedAt: -200000000,
      compositionID: 1,
      badSafariComposition: false,
      compositionPendingChanges: 0,
      domChangeCount: 0,
      eventHandlers: [Object: null prototype],
      hideSelectionGuard: null
    },
    prevDirectPlugins: [],
    pluginViews: [
      [Object],
      [Object],
      [Object],
      [Object],
      [DropCursorView],
      [Object],
      [Object],
      [Object]
    ],
    requiresGeckoHackNode: false,
    dragging: null,
    _props: {
      attributes: [Object],
      dispatchTransaction: [Function: bound dispatchTransaction],
      transformPastedHTML: [Function (anonymous)],
      state: [EditorState],
      markViews: {},
      nodeViews: [Object]
    },
    state: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    directPlugins: [],
    dispatch: [Function: bound ],
    dom: HTMLDivElement {
      pmViewDesc: [NodeViewDesc],
      editor: [Circular *4],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: [HTMLDivElement],
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [Array],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: [DOMTokenList],
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: [CSSStyleDeclaration],
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    editable: true,
    nodeViews: [Object: null prototype] { taskItem: [Function: nodeview] },
    docView: NodeViewDesc {
      parent: undefined,
      children: [Array],
      dom: [HTMLDivElement],
      contentDOM: [HTMLDivElement],
      dirty: 0,
      node: [Node],
      outerDeco: [Array],
      innerDeco: [DecorationSet],
      nodeDOM: [HTMLDivElement]
    },
    domObserver: DOMObserver {
      view: [Circular *5],
      handleDOMChange: [Function (anonymous)],
      queue: [],
      flushingSoon: -1,
      observer: MutationObserver {},
      currentSelection: [SelectionState],
      onCharData: null,
      suppressingSelectionUpdates: false,
      lastChangedTextNode: null,
      onSelectionChange: [Function: bound onSelectionChange]
    }
  },
  isFocused: false,
  destroyed: false,
  isInitialized: false,
  extensionStorage: {
    link: {},
    paragraph: {},
    listItemBranchingDeleteKeymap: {},
    taskItemBranchingDeleteKeymap: {},
    textStyle: {},
    editable: {},
    clipboardTextSerializer: {},
    commands: {},
    focusEvents: {},
    keymap: {},
    tabindex: {},
    drop: {},
    paste: {},
    delete: {},
    textDirection: {},
    starterKit: {},
    bold: {},
    blockquote: {},
    bulletList: {},
    code: {},
    doc: {},
    dropCursor: {},
    gapCursor: {},
    hardBreak: {},
    undoRedo: {},
    italic: {},
    listItem: {},
    listKeymap: {},
    orderedList: {},
    strike: {},
    text: {},
    underline: {},
    trailingNode: {},
    taskList: {},
    taskItem: {},
    color: {},
    fontSize: {},
    textAlign: {},
    typography: {},
    placeholder: {}
  },
  instanceId: 'ghfenf8',
  hasWarnedStaleDecorationRead: false,
  options: {
    element: HTMLDivElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    content: { type: 'doc', content: [Array] },
    injectCSS: true,
    injectNonce: undefined,
    extensions: [
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    autofocus: false,
    editable: true,
    textDirection: undefined,
    editorProps: { attributes: [Object] },
    parseOptions: {},
    coreExtensionOptions: {},
    enableInputRules: true,
    enablePasteRules: true,
    enableCoreExtensions: true,
    enableContentCheck: false,
    emitContentError: false,
    onBeforeCreate: [Function: onBeforeCreate],
    onCreate: [Function: onCreate],
    onMount: [Function: onMount],
    onUnmount: [Function: onUnmount],
    onUpdate: [Function: onUpdate],
    onSelectionUpdate: [Function: onSelectionUpdate],
    onTransaction: [Function: onTransaction],
    onFocus: [Function: onFocus],
    onBlur: [Function: onBlur],
    onDestroy: [Function: onDestroy],
    onContentError: [Function: onContentError],
    onPaste: [Function: onPaste],
    onDrop: [Function: onDrop],
    onDelete: [Function: onDelete],
    enableExtensionDispatchTransaction: true
  },
  isCapturingTransaction: false,
  capturedTransaction: null,
  utils: {
    getUpdatedPosition: [Function: getUpdatedPosition],
    createMappablePosition: [Function: createMappablePosition]
  },
  extensionManager: ExtensionManager {
    splittableMarks: [ 'textStyle', 'bold', 'code', 'italic', 'strike', 'underline' ],
    nonClearableMarks: [],
    decorationManager: DecorationManager {
      warnedWidgetKeys: Set(0) {},
      warnedOutOfRangeExtensions: Set(0) {},
      handleBeforeTransaction: [Function (anonymous)],
      editor: [Circular *4],
      entries: [],
      plugin: null
    },
    editor: [Circular *4],
    baseExtensions: [
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    extensions: [
      [Mark],      [Node],      [Extension],
      [Extension], [Mark],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Mark],      [Node],
      [Node],      [Mark],      [Node],
      [Extension], [Extension], [Node],
      [Extension], [Mark],      [Node],
      [Extension], [Node],      [Mark],
      [Node],      [Mark],      [Extension],
      [Node],      [Node],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension]
    ],
    schema: Schema {
      linebreakReplacement: [NodeType],
      cached: [Object: null prototype],
      spec: [Object],
      nodes: [Object: null prototype],
      marks: [Object: null prototype],
      nodeFromJSON: [Function (anonymous)],
      markFromJSON: [Function (anonymous)],
      topNodeType: [NodeType]
    }
  },
  commandManager: CommandManager {
    editor: [Circular *4],
    rawCommands: {
      setLink: [Function: setLink],
      toggleLink: [Function: toggleLink],
      unsetLink: [Function: unsetLink],
      setParagraph: [Function: setParagraph],
      toggleTextStyle: [Function: toggleTextStyle],
      removeEmptyTextStyle: [Function: removeEmptyTextStyle],
      blur: [Function: blur],
      clearContent: [Function: clearContent],
      clearNodes: [Function: clearNodes],
      command: [Function: command],
      createParagraphNear: [Function: createParagraphNear$1],
      cut: [Function: cut],
      deleteCurrentNode: [Function: deleteCurrentNode],
      deleteNode: [Function: deleteNode],
      deleteRange: [Function: deleteRange],
      deleteSelection: [Function: deleteSelection],
      enter: [Function: enter],
      exitCode: [Function: exitCode$1],
      extendMarkRange: [Function: extendMarkRange],
      first: [Function: first],
      focus: [Function: focus],
      forEach: [Function: forEach],
      insertContent: [Function: insertContent],
      insertContentAt: [Function: insertContentAt],
      insertDefaultBlock: [Function: insertDefaultBlock],
      joinBackward: [Function: joinBackward$1],
      joinDown: [Function: joinDown$1],
      joinForward: [Function: joinForward$1],
      joinItemBackward: [Function: joinItemBackward],
      joinItemForward: [Function: joinItemForward],
      joinTextblockBackward: [Function: joinTextblockBackward$1],
      joinTextblockForward: [Function: joinTextblockForward$1],
      joinUp: [Function: joinUp$1],
      keyboardShortcut: [Function: keyboardShortcut],
      lift: [Function: lift$1],
      liftEmptyBlock: [Function: liftEmptyBlock$1],
      liftListItem: [Function: liftListItem$1],
      newlineInCode: [Function: newlineInCode$1],
      resetAttributes: [Function: resetAttributes],
      scrollIntoView: [Function: scrollIntoView],
      selectAll: [Function: selectAll],
      selectNodeBackward: [Function: selectNodeBackward$1],
      selectNodeForward: [Function: selectNodeForward$1],
      selectParentNode: [Function: selectParentNode$1],
      selectTextblockEnd: [Function: selectTextblockEnd$1],
      selectTextblockStart: [Function: selectTextblockStart$1],
      setContent: [Function: setContent],
      setMark: [Function: setMark],
      setMeta: [Function: setMeta],
      setNode: [Function: setNode],
      setNodeSelection: [Function: setNodeSelection],
      setTextDirection: [Function: setTextDirection],
      setTextSelection: [Function: setTextSelection],
      sinkListItem: [Function: sinkListItem$1],
      splitBlock: [Function: splitBlock],
      splitListItem: [Function: splitListItem],
      toggleList: [Function: toggleList],
      toggleMark: [Function: toggleMark],
      toggleNode: [Function: toggleNode],
      toggleWrap: [Function: toggleWrap],
      undoInputRule: [Function: undoInputRule],
      unsetAllMarks: [Function: unsetAllMarks],
      unsetMark: [Function: unsetMark],
      unsetTextDirection: [Function: unsetTextDirection],
      updateAttributes: [Function: updateAttributes],
      updateDecorations: [Function: updateDecorations],
      wrapIn: [Function: wrapIn$1],
      wrapInList: [Function: wrapInList$1],
      setBold: [Function: setBold],
      toggleBold: [Function: toggleBold],
      unsetBold: [Function: unsetBold],
      setBlockquote: [Function: setBlockquote],
      toggleBlockquote: [Function: toggleBlockquote],
      unsetBlockquote: [Function: unsetBlockquote],
      toggleBulletList: [Function: toggleBulletList],
      setCode: [Function: setCode],
      toggleCode: [Function: toggleCode],
      unsetCode: [Function: unsetCode],
      setHardBreak: [Function: setHardBreak],
      undo: [Function: undo],
      redo: [Function: redo],
      setItalic: [Function: setItalic],
      toggleItalic: [Function: toggleItalic],
      unsetItalic: [Function: unsetItalic],
      toggleOrderedList: [Function: toggleOrderedList],
      setStrike: [Function: setStrike],
      toggleStrike: [Function: toggleStrike],
      unsetStrike: [Function: unsetStrike],
      setUnderline: [Function: setUnderline],
      toggleUnderline: [Function: toggleUnderline],
      unsetUnderline: [Function: unsetUnderline],
      toggleTaskList: [Function: toggleTaskList],
      setColor: [Function: setColor],
      unsetColor: [Function: unsetColor],
      setFontSize: [Function: setFontSize],
      unsetFontSize: [Function: unsetFontSize],
      setTextAlign: [Function: setTextAlign],
      unsetTextAlign: [Function: unsetTextAlign],
      toggleTextAlign: [Function: toggleTextAlign]
    },
    customState: undefined
  },
  schema: <ref *6> Schema {
    linebreakReplacement: NodeType {
      name: 'hardBreak',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [Array],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: false,
      isText: false
    },
    cached: [Object: null prototype] { wrappings: [Object: null prototype] {} },
    spec: { topNode: 'doc', nodes: [OrderedMap], marks: [OrderedMap] },
    nodes: [Object: null prototype] {
      paragraph: [NodeType],
      blockquote: [NodeType],
      bulletList: [NodeType],
      doc: [NodeType],
      hardBreak: [NodeType],
      listItem: [NodeType],
      orderedList: [NodeType],
      text: [NodeType],
      taskList: [NodeType],
      taskItem: [NodeType]
    },
    marks: [Object: null prototype] {
      link: [MarkType],
      textStyle: [MarkType],
      bold: [MarkType],
      code: [MarkType],
      italic: [MarkType],
      strike: [MarkType],
      underline: [MarkType]
    },
    nodeFromJSON: [Function (anonymous)],
    markFromJSON: [Function (anonymous)],
    topNodeType: NodeType {
      name: 'doc',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: true,
      isText: false
    }
  },
  editorState: EditorState {
    config: Configuration {
      schema: [Schema],
      plugins: [],
      pluginsByKey: [Object: null prototype] {},
      fields: [Array]
    },
    doc: Node {
      type: [NodeType],
      attrs: [Object: null prototype] {},
      marks: [],
      content: [Fragment]
    },
    selection: TextSelection {
      '$anchor': [ResolvedPos],
      '$head': [ResolvedPos],
      ranges: [Array]
    },
    storedMarks: null,
    scrollToSelection: 0
  },
  contentComponent: null,
  appContext: null,
  reactiveState: CustomRefImpl {
    __v_isRef: true,
    _value: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: [Link],
      map: undefined,
      key: undefined,
      sc: 2,
      __v_skip: true,
      subsHead: [Link]
    },
    _get: [Function: get],
    _set: [Function: set]
  },
  reactiveExtensionStorage: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  }
} selected=undefined > 
  at <BubbleMenu editor= <ref *4> Editor {
  callbacks: {
    create: [ [Function: bound onCreate], [Function: onCreate] ],
    destroy: [ [Function: bound onDestroy], [Function: onDestroy] ],
    update: [ [Function: bound onUpdate], [Function: onUpdate] ],
    beforeCreate: [ [Function: onBeforeCreate] ],
    mount: [ [Function: onMount] ],
    unmount: [ [Function: onUnmount] ],
    contentError: [ [Function: onContentError] ],
    selectionUpdate: [ [Function: onSelectionUpdate] ],
    transaction: [ [Function: onTransaction] ],
    focus: [ [Function: onFocus] ],
    blur: [ [Function: onBlur] ],
    drop: [ [Function (anonymous)] ],
    paste: [ [Function (anonymous)] ],
    delete: [ [Function: onDelete] ],
    beforeTransaction: [ [Function (anonymous)], [Function (anonymous)] ]
  },
  css: <ref *2> HTMLStyleElement {
    [Symbol(listeners)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(listenerOptions)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(isConnected)]: true,
    [Symbol(parentNode)]: HTMLHeadElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: [HTMLHtmlElement],
      [Symbol(rootNode)]: [HTMLDocument],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'HEAD',
      [Symbol(localName)]: 'head',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    [Symbol(rootNode)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(styleNode)]: [Circular *2],
    [Symbol(textAreaNode)]: null,
    [Symbol(formNode)]: null,
    [Symbol(selectNode)]: null,
    [Symbol(mutationListeners)]: [],
    [Symbol(nodeArray)]: [ [Text] ],
    [Symbol(elementArray)]: [],
    [Symbol(childNodes)]: null,
    [Symbol(assignedToSlot)]: null,
    [Symbol(cache)]: {
      querySelector: Map(0) {},
      querySelectorAll: Map(0) {},
      matches: Map(0) {},
      elementsByTagName: Map(0) {},
      elementsByTagNameNS: Map(0) {},
      elementByTagName: Map(0) {},
      elementById: Map(0) {},
      computedStyle: null
    },
    [Symbol(affectsCache)]: [],
    [Symbol(ownerDocument)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(window)]: <ref *3> GlobalWindow [EventTarget] {
      Node: [Function],
      Attr: [class Attr extends Node],
      ShadowRoot: [class ShadowRoot extends DocumentFragment],
      ProcessingInstruction: [class ProcessingInstruction extends CharacterData],
      Element: [Function],
      CharacterData: [class CharacterData extends Node],
      DocumentType: [class DocumentType extends Node],
      AnimationTimeline: [class AnimationTimeline],
      DocumentTimeline: [class DocumentTimeline extends AnimationTimeline],
      HTMLAnchorElement: [class HTMLAnchorElement extends HTMLElement],
      HTMLButtonElement: [class HTMLButtonElement extends HTMLElement],
      HTMLOptGroupElement: [class HTMLOptGroupElement extends HTMLElement],
      HTMLOptionElement: [class HTMLOptionElement extends HTMLElement],
      HTMLElement: [Function],
      HTMLUnknownElement: [class HTMLUnknownElement extends HTMLElement],
      HTMLTemplateElement: [class HTMLTemplateElement extends HTMLElement],
      HTMLInputElement: [class HTMLInputElement extends HTMLElement],
      HTMLSelectElement: [class HTMLSelectElement extends HTMLElement],
      HTMLTextAreaElement: [class HTMLTextAreaElement extends HTMLElement],
      HTMLImageElement: [class HTMLImageElement extends HTMLElement],
      HTMLStyleElement: [class HTMLStyleElement extends HTMLElement],
      HTMLLabelElement: [class HTMLLabelElement extends HTMLElement],
      HTMLSlotElement: [class HTMLSlotElement extends HTMLElement],
      HTMLMetaElement: [class HTMLMetaElement extends HTMLElement],
      HTMLMediaElement: [class HTMLMediaElement extends HTMLElement],
      HTMLAudioElement: [class HTMLAudioElement extends HTMLMediaElement],
      HTMLVideoElement: [class HTMLVideoElement extends HTMLMediaElement],
      HTMLBaseElement: [class HTMLBaseElement extends HTMLElement],
      HTMLDialogElement: [class HTMLDialogElement extends HTMLElement],
      HTMLScriptElement: [class HTMLScriptElement extends HTMLElement],
      HTMLLinkElement: [class HTMLLinkElement extends HTMLElement],
      HTMLIFrameElement: [class HTMLIFrameElement extends HTMLElement],
      HTMLFormElement: [class HTMLFormElement extends HTMLElement],
      HTMLUListElement: [class HTMLUListElement extends HTMLElement],
      HTMLTrackElement: [class HTMLTrackElement extends HTMLElement],
      HTMLTableRowElement: [class HTMLTableRowElement extends HTMLElement],
      HTMLTitleElement: [class HTMLTitleElement extends HTMLElement],
      HTMLTimeElement: [class HTMLTimeElement extends HTMLElement],
      HTMLTableSectionElement: [class HTMLTableSectionElement extends HTMLElement],
      HTMLTableCellElement: [class HTMLTableCellElement extends HTMLElement],
      HTMLTableElement: [class HTMLTableElement extends HTMLElement],
      HTMLSpanElement: [class HTMLSpanElement extends HTMLElement],
      HTMLSourceElement: [class HTMLSourceElement extends HTMLElement],
      HTMLQuoteElement: [class HTMLQuoteElement extends HTMLElement],
      HTMLProgressElement: [class HTMLProgressElement extends HTMLElement],
      HTMLPreElement: [class HTMLPreElement extends HTMLElement],
      HTMLPictureElement: [class HTMLPictureElement extends HTMLElement],
      HTMLParamElement: [class HTMLParamElement extends HTMLElement],
      HTMLParagraphElement: [class HTMLParagraphElement extends HTMLElement],
      HTMLOutputElement: [class HTMLOutputElement extends HTMLElement],
      HTMLOListElement: [class HTMLOListElement extends HTMLElement],
      HTMLObjectElement: [class HTMLObjectElement extends HTMLElement],
      HTMLMeterElement: [class HTMLMeterElement extends HTMLElement],
      HTMLMenuElement: [class HTMLMenuElement extends HTMLElement],
      HTMLMapElement: [class HTMLMapElement extends HTMLElement],
      HTMLLIElement: [class HTMLLIElement extends HTMLElement],
      HTMLLegendElement: [class HTMLLegendElement extends HTMLElement],
      HTMLModElement: [class HTMLModElement extends HTMLElement],
      HTMLHtmlElement: [class HTMLHtmlElement extends HTMLElement],
      HTMLHRElement: [class HTMLHRElement extends HTMLElement],
      HTMLHeadElement: [class HTMLHeadElement extends HTMLElement],
      HTMLHeadingElement: [class HTMLHeadingElement extends HTMLElement],
      HTMLFieldSetElement: [class HTMLFieldSetElement extends HTMLElement],
      HTMLEmbedElement: [class HTMLEmbedElement extends HTMLElement],
      HTMLDListElement: [class HTMLDListElement extends HTMLElement],
      HTMLDivElement: [class HTMLDivElement extends HTMLElement],
      HTMLDetailsElement: [class HTMLDetailsElement extends HTMLElement],
      HTMLDataListElement: [class HTMLDataListElement extends HTMLElement],
      HTMLDataElement: [class HTMLDataElement extends HTMLElement],
      HTMLTableColElement: [class HTMLTableColElement extends HTMLElement],
      HTMLTableCaptionElement: [class HTMLTableCaptionElement extends HTMLElement],
      HTMLCanvasElement: [class HTMLCanvasElement extends HTMLElement],
      HTMLBRElement: [class HTMLBRElement extends HTMLElement],
      HTMLBodyElement: [class HTMLBodyElement extends HTMLElement],
      HTMLAreaElement: [class HTMLAreaElement extends HTMLElement],
      SVGSVGElement: [class SVGSVGElement extends SVGGraphicsElement],
      SVGAnimateElement: [class SVGAnimateElement extends SVGAnimationElement],
      SVGAnimateMotionElement: [class SVGAnimateMotionElement extends SVGAnimationElement],
      SVGAnimateTransformElement: [class SVGAnimateTransformElement extends SVGAnimationElement],
      SVGCircleElement: [class SVGCircleElement extends SVGGeometryElement],
      SVGClipPathElement: [class SVGClipPathElement extends SVGElement],
      SVGDefsElement: [class SVGDefsElement extends SVGGraphicsElement],
      SVGDescElement: [class SVGDescElement extends SVGElement],
      SVGEllipseElement: [class SVGEllipseElement extends SVGGeometryElement],
      SVGFEBlendElement: [Function],
      SVGFEColorMatrixElement: [Function],
      SVGFEComponentTransferElement: [class SVGFEComponentTransferElement extends SVGElement],
      SVGFECompositeElement: [Function],
      SVGFEConvolveMatrixElement: [Function],
      SVGFEDiffuseLightingElement: [class SVGFEDiffuseLightingElement extends SVGElement],
      SVGFEDisplacementMapElement: [Function],
      SVGFEDistantLightElement: [class SVGFEDistantLightElement extends SVGElement],
      SVGFEDropShadowElement: [class SVGFEDropShadowElement extends SVGElement],
      SVGFEFloodElement: [class SVGFEFloodElement extends SVGElement],
      SVGFEFuncAElement: [class SVGFEFuncAElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncBElement: [class SVGFEFuncBElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncGElement: [class SVGFEFuncGElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncRElement: [class SVGFEFuncRElement extends SVGComponentTransferFunctionElement],
      SVGFEGaussianBlurElement: [Function],
      SVGFEImageElement: [class SVGFEImageElement extends SVGElement],
      SVGFEMergeElement: [class SVGFEMergeElement extends SVGElement],
      SVGFEMergeNodeElement: [class SVGFEMergeNodeElement extends SVGElement],
      SVGFEMorphologyElement: [Function],
      SVGFEOffsetElement: [class SVGFEOffsetElement extends SVGElement],
      SVGFEPointLightElement: [class SVGFEPointLightElement extends SVGElement],
      SVGFESpecularLightingElement: [class SVGFESpecularLightingElement extends SVGElement],
      SVGFESpotLightElement: [class SVGFESpotLightElement extends SVGElement],
      SVGFETileElement: [class SVGFETileElement extends SVGElement],
      SVGFETurbulenceElement: [Function],
      SVGFilterElement: [class SVGFilterElement extends SVGElement],
      SVGForeignObjectElement: [class SVGForeignObjectElement extends SVGGraphicsElement],
      SVGGElement: [class SVGGElement extends SVGGraphicsElement],
      SVGImageElement: [class SVGImageElement extends SVGGraphicsElement],
      SVGLineElement: [class SVGLineElement extends SVGGeometryElement],
      SVGLinearGradientElement: [class SVGLinearGradientElement extends SVGGradientElement],
      SVGMarkerElement: [Function],
      SVGMaskElement: [class SVGMaskElement extends SVGElement],
      SVGMetadataElement: [class SVGMetadataElement extends SVGElement],
      SVGMPathElement: [class SVGMPathElement extends SVGElement],
      SVGPathElement: [class SVGPathElement extends SVGGeometryElement],
      SVGPatternElement: [class SVGPatternElement extends SVGElement],
      SVGPolygonElement: [class SVGPolygonElement extends SVGGeometryElement],
      SVGPolylineElement: [class SVGPolylineElement extends SVGGeometryElement],
      SVGRadialGradientElement: [class SVGRadialGradientElement extends SVGGradientElement],
      SVGRectElement: [class SVGRectElement extends SVGGeometryElement],
      SVGScriptElement: [class SVGScriptElement extends SVGGraphicsElement],
      SVGSetElement: [class SVGSetElement extends SVGAnimationElement],
      SVGStopElement: [class SVGStopElement extends SVGElement],
      SVGStyleElement: [class SVGStyleElement extends SVGElement],
      SVGSwitchElement: [class SVGSwitchElement extends SVGGraphicsElement],
      SVGSymbolElement: [class SVGSymbolElement extends SVGGraphicsElement],
      SVGTextElement: [class SVGTextElement extends SVGTextPositioningElement],
      SVGTextPathElement: [Function],
      SVGTitleElement: [class SVGTitleElement extends SVGElement],
      SVGTSpanElement: [class SVGTSpanElement extends SVGTextPositioningElement],
      SVGUseElement: [class SVGUseElement extends SVGGraphicsElement],
      SVGViewElement: [class SVGViewElement extends SVGElement],
      SVGElement: [class SVGElement extends Element],
      SVGAnimationElement: [class SVGAnimationElement extends SVGElement],
      SVGComponentTransferFunctionElement: [Function],
      SVGGeometryElement: [class SVGGeometryElement extends SVGGraphicsElement],
      SVGGradientElement: [Function],
      SVGTextPositioningElement: [class SVGTextPositioningElement extends SVGTextContentElement],
      SVGGraphicsElement: [class SVGGraphicsElement extends SVGElement],
      Event: [Function],
      UIEvent: [Function],
      CustomEvent: [class CustomEvent extends Event],
      CloseEvent: [class CloseEvent extends Event],
      AnimationEvent: [class AnimationEvent extends Event],
      KeyboardEvent: [Function],
      MessageEvent: [class MessageEvent extends Event],
      MouseEvent: [class MouseEvent extends UIEvent],
      PointerEvent: [class PointerEvent extends MouseEvent],
      FocusEvent: [class FocusEvent extends UIEvent],
      WheelEvent: [Function],
      InputEvent: [class InputEvent extends UIEvent],
      ErrorEvent: [class ErrorEvent extends UIEvent],
      StorageEvent: [class StorageEvent extends Event],
      SubmitEvent: [class SubmitEvent extends Event],
      ProgressEvent: [class ProgressEvent extends Event],
      MediaQueryListEvent: [class MediaQueryListEvent extends Event],
      HashChangeEvent: [class HashChangeEvent extends Event],
      ClipboardEvent: [class ClipboardEvent extends Event],
      CookieChangeEvent: [class CookieChangeEvent extends Event],
      TouchEvent: [class TouchEvent extends UIEvent],
      PopStateEvent: [class PopStateEvent extends Event],
      Touch: [class Touch],
      AudioProcessingEvent: [Function],
      BeforeInputEvent: [Function],
      BeforeUnloadEvent: [Function],
      BlobEvent: [Function],
      CompositionEvent: [Function],
      CSSFontFaceLoadEvent: [Function],
      DeviceLightEvent: [Function],
      DeviceMotionEvent: [Function],
      DeviceOrientationEvent: [Function],
      DeviceProximityEvent: [Function],
      DOMTransactionEvent: [Function],
      DragEvent: [Function],
      EditingBeforeInputEvent: [Function],
      FetchEvent: [Function],
      GamepadEvent: [Function],
      IDBVersionChangeEvent: [Function],
      MediaStreamEvent: [Function],
      MutationEvent: [Function],
      OfflineAudioCompletionEvent: [Function],
      OverconstrainedError: [Function],
      PageTransitionEvent: [Function],
      PaymentRequestUpdateEvent: [Function],
      RelatedEvent: [Function],
      RTCDataChannelEvent: [Function],
      RTCIdentityErrorEvent: [Function],
      RTCIdentityEvent: [Function],
      RTCPeerConnectionIceEvent: [Function],
      SensorEvent: [Function],
      SVGEvent: [Function],
      SVGZoomEvent: [Function],
      TimeEvent: [Function],
      TrackEvent: [Function],
      TransitionEvent: [Function],
      UserProximityEvent: [Function],
      WebGLContextEvent: [Function],
      TextEvent: [Function],
      Permissions: [class Permissions],
      History: [class History],
      Navigator: [class Navigator],
      Clipboard: [class Clipboard],
      TimeRanges: [class TimeRanges],
      TextTrackCueList: [class TextTrackCueList extends Array],
      ValidityState: [class ValidityState],
      MutationRecord: [class MutationRecord],
      IntersectionObserver: [class IntersectionObserver],
      IntersectionObserverEntry: [class IntersectionObserverEntry],
      CSSStyleDeclaration: [class CSSStyleDeclaration],
      CSSRule: [Function],
      CSSContainerRule: [class CSSContainerRule extends CSSConditionRule],
      CSSFontFaceRule: [class CSSFontFaceRule extends CSSRule],
      CSSKeyframeRule: [class CSSKeyframeRule extends CSSRule],
      CSSKeyframesRule: [class CSSKeyframesRule extends CSSRule],
      CSSMediaRule: [class CSSMediaRule extends CSSConditionRule],
      CSSStyleRule: [class CSSStyleRule extends CSSGroupingRule],
      CSSSupportsRule: [class CSSSupportsRule extends CSSConditionRule],
      CSSConditionRule: [class CSSConditionRule extends CSSGroupingRule],
      CSSGroupingRule: [class CSSGroupingRule extends CSSRule],
      CSSScopeRule: [class CSSScopeRule extends CSSGroupingRule],
      DOMRect: [class DOMRect extends DOMRectReadOnly],
      DOMRectReadOnly: [class DOMRectReadOnly],
      Plugin: [class Plugin],
      PluginArray: [class PluginArray],
      Location: [class Location],
      CustomElementRegistry: [class CustomElementRegistry],
      ResizeObserver: [class ResizeObserver],
      Blob: [class Blob],
      File: [class File extends Blob],
      Storage: [class Storage],
      CookieStore: [class CookieStore extends EventTarget],
      MimeType: [class MimeType],
      MimeTypeArray: [class MimeTypeArray],
      NodeFilter: [Object],
      ImageBitmap: [class ImageBitmap],
      HTMLCollection: [class HTMLCollection],
      HTMLFormControlCollection: [class HTMLFormControlsCollection extends HTMLCollection],
      HTMLOptionsCollection: [class HTMLOptionsCollection extends HTMLCollection],
      NodeList: [class NodeList],
      RadioNodeList: [class RadioNodeList extends NodeList],
      FileList: [class FileList extends Array],
      Screen: [class Screen extends EventTarget],
      ScreenDetails: [class ScreenDetails extends EventTarget],
      ScreenDetailed: [class ScreenDetailed extends Screen],
      NamedNodeMap: [class NamedNodeMap],
      TreeWalker: [class TreeWalker],
      NodeIterator: [class NodeIterator],
      DataTransfer: [class DataTransfer],
      DataTransferItem: [class DataTransferItem],
      DataTransferItemList: [class DataTransferItemList extends Array],
      XMLSerializer: [class XMLSerializer],
      ClipboardItem: [class ClipboardItem],
      Selection: [class Selection],
      CSSUnitValue: [class CSSUnitValue],
      SVGAngle: [Function],
      SVGAnimatedAngle: [class SVGAnimatedAngle],
      SVGAnimatedBoolean: [class SVGAnimatedBoolean],
      SVGAnimatedEnumeration: [class SVGAnimatedEnumeration],
      SVGAnimatedInteger: [class SVGAnimatedInteger],
      SVGAnimatedLength: [class SVGAnimatedLength],
      SVGAnimatedNumber: [class SVGAnimatedNumber],
      SVGAnimatedNumberList: [class SVGAnimatedNumberList],
      SVGAnimatedPreserveAspectRatio: [class SVGAnimatedPreserveAspectRatio],
      SVGAnimatedRect: [class SVGAnimatedRect],
      SVGAnimatedString: [class SVGAnimatedString],
      SVGAnimatedTransformList: [class SVGAnimatedTransformList],
      SVGLength: [Function],
      SVGLengthList: [class SVGLengthList],
      SVGMatrix: [class SVGMatrix],
      SVGNumber: [class SVGNumber],
      SVGNumberList: [class SVGNumberList],
      SVGPoint: [class SVGPoint],
      SVGPointList: [class SVGPointList],
      SVGPreserveAspectRatio: [Function],
      SVGRect: [class SVGRect],
      SVGStringList: [class SVGStringList],
      SVGTransform: [Function],
      SVGTransformList: [class SVGTransformList],
      SVGAnimatedLengthList: [class SVGAnimatedLengthList],
      SVGUnitTypes: [Function],
      StylePropertyMap: [class StylePropertyMap extends StylePropertyMapReadOnly],
      StylePropertyMapReadOnly: [class StylePropertyMapReadOnly],
      MediaList: [class MediaList],
      CSSKeywordValue: [class CSSKeywordValue],
      CSSStyleValue: [class CSSStyleValue],
      Window: [class GlobalWindow extends Window],
      URLSearchParams: [class URLSearchParams],
      WritableStream: [Function],
      ReadableStream: [Function],
      TransformStream: [Function: Transform],
      PerformanceObserver: [class PerformanceObserver],
      PerformanceEntry: [class PerformanceEntry],
      PerformanceObserverEntryList: [class PerformanceObserverEntryList],
      document: [HTMLDocument],
      customElements: [CustomElementRegistry],
      window: [Circular *3],
      globalThis: [Circular *3],
      performance: [Performance],
      screenLeft: 0,
      screenTop: 0,
      screenX: 0,
      screenY: 0,
      crypto: Crypto {},
      TextEncoder: [class TextEncoder],
      TextDecoder: [class TextDecoder],
      closed: false,
      console: [Object [console]],
      name: '',
      Buffer: [Function],
      Document: [class Document extends Document],
      HTMLDocument: [class HTMLDocument extends HTMLDocument],
      XMLDocument: [class XMLDocument extends XMLDocument],
      DocumentFragment: [class DocumentFragment extends DocumentFragment],
      Text: [class Text extends Text],
      Comment: [class Comment extends Comment],
      Image: [class Image extends Image],
      Audio: [class Audio extends Audio],
      MutationObserver: [class MutationObserver extends MutationObserver],
      MessagePort: [class MessagePort extends MessagePort],
      CSSStyleSheet: [class CSSStyleSheet extends CSSStyleSheet],
      DOMException: [class DOMException extends DOMException],
      Headers: [class Headers extends Headers],
      Request: [class Request extends Request],
      Response: [Function],
      EventTarget: [class EventTarget extends EventTarget],
      XMLHttpRequestUpload: [class XMLHttpRequestUpload extends XMLHttpRequestUpload],
      XMLHttpRequestEventTarget: [class XMLHttpRequestEventTarget extends XMLHttpRequestEventTarget],
      AbortController: [class AbortController extends AbortController],
      AbortSignal: [Function],
      FormData: [class FormData extends FormData],
      PermissionStatus: [class PermissionStatus extends PermissionStatus],
      XMLHttpRequest: [class XMLHttpRequest extends XMLHttpRequest],
      DOMParser: [class DOMParser extends DOMParser],
      Range: [class Range extends Range],
      VTTCue: [class VTTCue extends VTTCue],
      TextTrack: [class TextTrack extends TextTrack],
      TextTrackList: [class TextTrackList extends TextTrackList],
      TextTrackCue: [class TextTrackCue extends TextTrackCue],
      RemotePlayback: [class RemotePlayback extends RemotePlayback],
      FileReader: [class FileReader extends FileReader],
      MediaStream: [class MediaStream extends MediaStream],
      MediaStreamTrack: [class MediaStreamTrack extends MediaStreamTrack],
      CanvasCaptureMediaStreamTrack: [class CanvasCaptureMediaStreamTrack extends CanvasCaptureMediaStreamTrack],
      URL: [class URL extends URL],
      WebSocket: [class WebSocket extends WebSocket],
      ImageData: [class ImageData extends ImageData],
      OffscreenCanvas: [class OffscreenCanvas extends OffscreenCanvas],
      DOMMatrixReadOnly: [class DOMMatrixReadOnly extends DOMMatrixReadOnly],
      DOMMatrix: [class DOMMatrix extends DOMMatrix],
      DOMPointReadOnly: [class DOMPointReadOnly extends DOMPointReadOnly],
      DOMPoint: [class DOMPoint extends DOMPoint],
      Animation: [class Animation extends Animation],
      KeyframeEffect: [class KeyframeEffect extends KeyframeEffect],
      happyDOM: DetachedWindowAPI {},
      Array: [Function: Array],
      ArrayBuffer: [Function: ArrayBuffer],
      Boolean: [Function: Boolean],
      DataView: [Function: DataView],
      Date: [Function: Date],
      Error: [Function],
      EvalError: [Function: EvalError],
      Float32Array: [Function: Float32Array],
      Float64Array: [Function: Float64Array],
      Function: [Function: Function],
      Infinity: Infinity,
      Int16Array: [Function: Int16Array],
      Int32Array: [Function: Int32Array],
      Int8Array: [Function: Int8Array],
      Intl: Object [Intl] {},
      JSON: Object [JSON] {},
      Map: [Function: Map],
      Math: Object [Math] {},
      NaN: NaN,
      Number: [Function: Number],
      Object: [Function: Object],
      Promise: [Function: Promise],
      RangeError: [Function: RangeError],
      ReferenceError: [Function: ReferenceError],
      RegExp: [Function: RegExp],
      Set: [Function: Set],
      String: [Function: String],
      Symbol: [Function: Symbol],
      SyntaxError: [Function: SyntaxError],
      TypeError: [Function: TypeError],
      URIError: [Function: URIError],
      Uint16Array: [Function: Uint16Array],
      Uint32Array: [Function: Uint32Array],
      Uint8Array: [Function: Uint8Array],
      Uint8ClampedArray: [Function: Uint8ClampedArray],
      WeakMap: [Function: WeakMap],
      WeakSet: [Function: WeakSet],
      decodeURI: [Function: decodeURI],
      decodeURIComponent: [Function: decodeURIComponent],
      encodeURI: [Function: encodeURI],
      encodeURIComponent: [Function: encodeURIComponent],
      eval: [Function: eval],
      escape: [Function: escape],
      global: [Object [global]],
      isFinite: [Function: isFinite],
      isNaN: [Function: isNaN],
      parseFloat: [Function: parseFloat],
      parseInt: [Function: parseInt],
      undefined: undefined,
      unescape: [Function: unescape],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(mutationObservers)]: [Array],
      [Symbol(readyStateManager)]: DocumentReadyStateManager {},
      [Symbol(location)]: [Location],
      [Symbol(history)]: History {},
      [Symbol(navigator)]: Navigator {},
      [Symbol(screen)]: [Screen [EventTarget]],
      [Symbol(sessionStorage)]: [Storage],
      [Symbol(localStorage)]: [Storage],
      [Symbol(cookieStore)]: [CookieStore [EventTarget]],
      [Symbol(self)]: [Circular *3],
      [Symbol(top)]: [Circular *3],
      [Symbol(parent)]: [Circular *3],
      [Symbol(window)]: [Circular *3],
      [Symbol(frames)]: [Circular *3],
      [Symbol(internalId)]: 0,
      [Symbol(customElementReactionStack)]: [CustomElementReactionStack],
      [Symbol(modules)]: [Object],
      [Symbol(moduleImportMap)]: null,
      [Symbol(openWebSockets)]: [],
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(querySelectorCache)]: [Map]
    },
    [Symbol(classList)]: null,
    [Symbol(isValue)]: null,
    [Symbol(nodeType)]: 1,
    [Symbol(prefix)]: null,
    [Symbol(shadowRoot)]: null,
    [Symbol(scrollHeight)]: 0,
    [Symbol(scrollWidth)]: 0,
    [Symbol(scrollTop)]: 0,
    [Symbol(scrollLeft)]: 0,
    [Symbol(attributes)]: NamedNodeMap {
      [Symbol(itemsByNamespaceURI)]: [Map],
      [Symbol(itemsByName)]: [Map],
      [Symbol(items)]: [Map],
      [Symbol(ownerElement)]: [Circular *2]
    },
    [Symbol(attributesProxy)]: null,
    [Symbol(children)]: null,
    [Symbol(computedStyle)]: null,
    [Symbol(pointerCaptures)]: Set(0) {},
    [Symbol(propertyEventListeners)]: Map(0) {},
    [Symbol(animations)]: [],
    [Symbol(tagName)]: 'STYLE',
    [Symbol(localName)]: 'style',
    [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
    [Symbol(accessKey)]: '',
    [Symbol(offsetHeight)]: 0,
    [Symbol(offsetWidth)]: 0,
    [Symbol(offsetLeft)]: 0,
    [Symbol(offsetTop)]: 0,
    [Symbol(clientHeight)]: 0,
    [Symbol(clientWidth)]: 0,
    [Symbol(clientLeft)]: 0,
    [Symbol(clientTop)]: 0,
    [Symbol(style)]: null,
    [Symbol(dataset)]: null,
    [Symbol(customElementDefineCallback)]: null,
    [Symbol(sheet)]: null,
    [Symbol(disabled)]: false
  },
  className: 'tiptap',
  editorView: <ref *5> EditorView {
    _root: null,
    focused: false,
    trackWrites: null,
    mounted: false,
    markCursor: null,
    cursorWrapper: null,
    lastSelectedViewDesc: undefined,
    input: InputState {
      shiftKey: false,
      mouseDown: null,
      lastKeyCode: null,
      lastKeyCodeTime: 0,
      lastClick: [Object],
      lastSelectionOrigin: null,
      lastSelectionTime: 0,
      lastIOSEnter: 0,
      lastIOSEnterFallbackTimeout: -1,
      lastFocus: 0,
      lastTouch: 0,
      lastChromeDelete: 0,
      composing: false,
      compositionNode: null,
      composingTimeout: -1,
      compositionNodes: [],
      compositionEndedAt: -200000000,
      compositionID: 1,
      badSafariComposition: false,
      compositionPendingChanges: 0,
      domChangeCount: 0,
      eventHandlers: [Object: null prototype],
      hideSelectionGuard: null
    },
    prevDirectPlugins: [],
    pluginViews: [
      [Object],
      [Object],
      [Object],
      [Object],
      [DropCursorView],
      [Object],
      [Object],
      [Object]
    ],
    requiresGeckoHackNode: false,
    dragging: null,
    _props: {
      attributes: [Object],
      dispatchTransaction: [Function: bound dispatchTransaction],
      transformPastedHTML: [Function (anonymous)],
      state: [EditorState],
      markViews: {},
      nodeViews: [Object]
    },
    state: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    directPlugins: [],
    dispatch: [Function: bound ],
    dom: HTMLDivElement {
      pmViewDesc: [NodeViewDesc],
      editor: [Circular *4],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: [HTMLDivElement],
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [Array],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: [DOMTokenList],
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: [CSSStyleDeclaration],
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    editable: true,
    nodeViews: [Object: null prototype] { taskItem: [Function: nodeview] },
    docView: NodeViewDesc {
      parent: undefined,
      children: [Array],
      dom: [HTMLDivElement],
      contentDOM: [HTMLDivElement],
      dirty: 0,
      node: [Node],
      outerDeco: [Array],
      innerDeco: [DecorationSet],
      nodeDOM: [HTMLDivElement]
    },
    domObserver: DOMObserver {
      view: [Circular *5],
      handleDOMChange: [Function (anonymous)],
      queue: [],
      flushingSoon: -1,
      observer: MutationObserver {},
      currentSelection: [SelectionState],
      onCharData: null,
      suppressingSelectionUpdates: false,
      lastChangedTextNode: null,
      onSelectionChange: [Function: bound onSelectionChange]
    }
  },
  isFocused: false,
  destroyed: false,
  isInitialized: false,
  extensionStorage: {
    link: {},
    paragraph: {},
    listItemBranchingDeleteKeymap: {},
    taskItemBranchingDeleteKeymap: {},
    textStyle: {},
    editable: {},
    clipboardTextSerializer: {},
    commands: {},
    focusEvents: {},
    keymap: {},
    tabindex: {},
    drop: {},
    paste: {},
    delete: {},
    textDirection: {},
    starterKit: {},
    bold: {},
    blockquote: {},
    bulletList: {},
    code: {},
    doc: {},
    dropCursor: {},
    gapCursor: {},
    hardBreak: {},
    undoRedo: {},
    italic: {},
    listItem: {},
    listKeymap: {},
    orderedList: {},
    strike: {},
    text: {},
    underline: {},
    trailingNode: {},
    taskList: {},
    taskItem: {},
    color: {},
    fontSize: {},
    textAlign: {},
    typography: {},
    placeholder: {}
  },
  instanceId: 'ghfenf8',
  hasWarnedStaleDecorationRead: false,
  options: {
    element: HTMLDivElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    content: { type: 'doc', content: [Array] },
    injectCSS: true,
    injectNonce: undefined,
    extensions: [
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    autofocus: false,
    editable: true,
    textDirection: undefined,
    editorProps: { attributes: [Object] },
    parseOptions: {},
    coreExtensionOptions: {},
    enableInputRules: true,
    enablePasteRules: true,
    enableCoreExtensions: true,
    enableContentCheck: false,
    emitContentError: false,
    onBeforeCreate: [Function: onBeforeCreate],
    onCreate: [Function: onCreate],
    onMount: [Function: onMount],
    onUnmount: [Function: onUnmount],
    onUpdate: [Function: onUpdate],
    onSelectionUpdate: [Function: onSelectionUpdate],
    onTransaction: [Function: onTransaction],
    onFocus: [Function: onFocus],
    onBlur: [Function: onBlur],
    onDestroy: [Function: onDestroy],
    onContentError: [Function: onContentError],
    onPaste: [Function: onPaste],
    onDrop: [Function: onDrop],
    onDelete: [Function: onDelete],
    enableExtensionDispatchTransaction: true
  },
  isCapturingTransaction: false,
  capturedTransaction: null,
  utils: {
    getUpdatedPosition: [Function: getUpdatedPosition],
    createMappablePosition: [Function: createMappablePosition]
  },
  extensionManager: ExtensionManager {
    splittableMarks: [ 'textStyle', 'bold', 'code', 'italic', 'strike', 'underline' ],
    nonClearableMarks: [],
    decorationManager: DecorationManager {
      warnedWidgetKeys: Set(0) {},
      warnedOutOfRangeExtensions: Set(0) {},
      handleBeforeTransaction: [Function (anonymous)],
      editor: [Circular *4],
      entries: [],
      plugin: null
    },
    editor: [Circular *4],
    baseExtensions: [
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    extensions: [
      [Mark],      [Node],      [Extension],
      [Extension], [Mark],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Mark],      [Node],
      [Node],      [Mark],      [Node],
      [Extension], [Extension], [Node],
      [Extension], [Mark],      [Node],
      [Extension], [Node],      [Mark],
      [Node],      [Mark],      [Extension],
      [Node],      [Node],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension]
    ],
    schema: Schema {
      linebreakReplacement: [NodeType],
      cached: [Object: null prototype],
      spec: [Object],
      nodes: [Object: null prototype],
      marks: [Object: null prototype],
      nodeFromJSON: [Function (anonymous)],
      markFromJSON: [Function (anonymous)],
      topNodeType: [NodeType]
    }
  },
  commandManager: CommandManager {
    editor: [Circular *4],
    rawCommands: {
      setLink: [Function: setLink],
      toggleLink: [Function: toggleLink],
      unsetLink: [Function: unsetLink],
      setParagraph: [Function: setParagraph],
      toggleTextStyle: [Function: toggleTextStyle],
      removeEmptyTextStyle: [Function: removeEmptyTextStyle],
      blur: [Function: blur],
      clearContent: [Function: clearContent],
      clearNodes: [Function: clearNodes],
      command: [Function: command],
      createParagraphNear: [Function: createParagraphNear$1],
      cut: [Function: cut],
      deleteCurrentNode: [Function: deleteCurrentNode],
      deleteNode: [Function: deleteNode],
      deleteRange: [Function: deleteRange],
      deleteSelection: [Function: deleteSelection],
      enter: [Function: enter],
      exitCode: [Function: exitCode$1],
      extendMarkRange: [Function: extendMarkRange],
      first: [Function: first],
      focus: [Function: focus],
      forEach: [Function: forEach],
      insertContent: [Function: insertContent],
      insertContentAt: [Function: insertContentAt],
      insertDefaultBlock: [Function: insertDefaultBlock],
      joinBackward: [Function: joinBackward$1],
      joinDown: [Function: joinDown$1],
      joinForward: [Function: joinForward$1],
      joinItemBackward: [Function: joinItemBackward],
      joinItemForward: [Function: joinItemForward],
      joinTextblockBackward: [Function: joinTextblockBackward$1],
      joinTextblockForward: [Function: joinTextblockForward$1],
      joinUp: [Function: joinUp$1],
      keyboardShortcut: [Function: keyboardShortcut],
      lift: [Function: lift$1],
      liftEmptyBlock: [Function: liftEmptyBlock$1],
      liftListItem: [Function: liftListItem$1],
      newlineInCode: [Function: newlineInCode$1],
      resetAttributes: [Function: resetAttributes],
      scrollIntoView: [Function: scrollIntoView],
      selectAll: [Function: selectAll],
      selectNodeBackward: [Function: selectNodeBackward$1],
      selectNodeForward: [Function: selectNodeForward$1],
      selectParentNode: [Function: selectParentNode$1],
      selectTextblockEnd: [Function: selectTextblockEnd$1],
      selectTextblockStart: [Function: selectTextblockStart$1],
      setContent: [Function: setContent],
      setMark: [Function: setMark],
      setMeta: [Function: setMeta],
      setNode: [Function: setNode],
      setNodeSelection: [Function: setNodeSelection],
      setTextDirection: [Function: setTextDirection],
      setTextSelection: [Function: setTextSelection],
      sinkListItem: [Function: sinkListItem$1],
      splitBlock: [Function: splitBlock],
      splitListItem: [Function: splitListItem],
      toggleList: [Function: toggleList],
      toggleMark: [Function: toggleMark],
      toggleNode: [Function: toggleNode],
      toggleWrap: [Function: toggleWrap],
      undoInputRule: [Function: undoInputRule],
      unsetAllMarks: [Function: unsetAllMarks],
      unsetMark: [Function: unsetMark],
      unsetTextDirection: [Function: unsetTextDirection],
      updateAttributes: [Function: updateAttributes],
      updateDecorations: [Function: updateDecorations],
      wrapIn: [Function: wrapIn$1],
      wrapInList: [Function: wrapInList$1],
      setBold: [Function: setBold],
      toggleBold: [Function: toggleBold],
      unsetBold: [Function: unsetBold],
      setBlockquote: [Function: setBlockquote],
      toggleBlockquote: [Function: toggleBlockquote],
      unsetBlockquote: [Function: unsetBlockquote],
      toggleBulletList: [Function: toggleBulletList],
      setCode: [Function: setCode],
      toggleCode: [Function: toggleCode],
      unsetCode: [Function: unsetCode],
      setHardBreak: [Function: setHardBreak],
      undo: [Function: undo],
      redo: [Function: redo],
      setItalic: [Function: setItalic],
      toggleItalic: [Function: toggleItalic],
      unsetItalic: [Function: unsetItalic],
      toggleOrderedList: [Function: toggleOrderedList],
      setStrike: [Function: setStrike],
      toggleStrike: [Function: toggleStrike],
      unsetStrike: [Function: unsetStrike],
      setUnderline: [Function: setUnderline],
      toggleUnderline: [Function: toggleUnderline],
      unsetUnderline: [Function: unsetUnderline],
      toggleTaskList: [Function: toggleTaskList],
      setColor: [Function: setColor],
      unsetColor: [Function: unsetColor],
      setFontSize: [Function: setFontSize],
      unsetFontSize: [Function: unsetFontSize],
      setTextAlign: [Function: setTextAlign],
      unsetTextAlign: [Function: unsetTextAlign],
      toggleTextAlign: [Function: toggleTextAlign]
    },
    customState: undefined
  },
  schema: <ref *6> Schema {
    linebreakReplacement: NodeType {
      name: 'hardBreak',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [Array],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: false,
      isText: false
    },
    cached: [Object: null prototype] { wrappings: [Object: null prototype] {} },
    spec: { topNode: 'doc', nodes: [OrderedMap], marks: [OrderedMap] },
    nodes: [Object: null prototype] {
      paragraph: [NodeType],
      blockquote: [NodeType],
      bulletList: [NodeType],
      doc: [NodeType],
      hardBreak: [NodeType],
      listItem: [NodeType],
      orderedList: [NodeType],
      text: [NodeType],
      taskList: [NodeType],
      taskItem: [NodeType]
    },
    marks: [Object: null prototype] {
      link: [MarkType],
      textStyle: [MarkType],
      bold: [MarkType],
      code: [MarkType],
      italic: [MarkType],
      strike: [MarkType],
      underline: [MarkType]
    },
    nodeFromJSON: [Function (anonymous)],
    markFromJSON: [Function (anonymous)],
    topNodeType: NodeType {
      name: 'doc',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: true,
      isText: false
    }
  },
  editorState: EditorState {
    config: Configuration {
      schema: [Schema],
      plugins: [],
      pluginsByKey: [Object: null prototype] {},
      fields: [Array]
    },
    doc: Node {
      type: [NodeType],
      attrs: [Object: null prototype] {},
      marks: [],
      content: [Fragment]
    },
    selection: TextSelection {
      '$anchor': [ResolvedPos],
      '$head': [ResolvedPos],
      ranges: [Array]
    },
    storedMarks: null,
    scrollToSelection: 0
  },
  contentComponent: null,
  appContext: null,
  reactiveState: CustomRefImpl {
    __v_isRef: true,
    _value: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: [Link],
      map: undefined,
      key: undefined,
      sc: 2,
      __v_skip: true,
      subsHead: [Link]
    },
    _get: [Function: get],
    _set: [Function: set]
  },
  reactiveExtensionStorage: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  }
} append-to=fn<appendToCanvasOverlay> should-show=fn<shouldShow>  ... > 
  at <SpaceCanvasTextSelectionBar key=0 editor= <ref *4> Editor {
  callbacks: {
    create: [ [Function: bound onCreate], [Function: onCreate] ],
    destroy: [ [Function: bound onDestroy], [Function: onDestroy] ],
    update: [ [Function: bound onUpdate], [Function: onUpdate] ],
    beforeCreate: [ [Function: onBeforeCreate] ],
    mount: [ [Function: onMount] ],
    unmount: [ [Function: onUnmount] ],
    contentError: [ [Function: onContentError] ],
    selectionUpdate: [ [Function: onSelectionUpdate] ],
    transaction: [ [Function: onTransaction] ],
    focus: [ [Function: onFocus] ],
    blur: [ [Function: onBlur] ],
    drop: [ [Function (anonymous)] ],
    paste: [ [Function (anonymous)] ],
    delete: [ [Function: onDelete] ],
    beforeTransaction: [ [Function (anonymous)], [Function (anonymous)] ]
  },
  css: <ref *2> HTMLStyleElement {
    [Symbol(listeners)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(listenerOptions)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(isConnected)]: true,
    [Symbol(parentNode)]: HTMLHeadElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: [HTMLHtmlElement],
      [Symbol(rootNode)]: [HTMLDocument],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'HEAD',
      [Symbol(localName)]: 'head',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    [Symbol(rootNode)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(styleNode)]: [Circular *2],
    [Symbol(textAreaNode)]: null,
    [Symbol(formNode)]: null,
    [Symbol(selectNode)]: null,
    [Symbol(mutationListeners)]: [],
    [Symbol(nodeArray)]: [ [Text] ],
    [Symbol(elementArray)]: [],
    [Symbol(childNodes)]: null,
    [Symbol(assignedToSlot)]: null,
    [Symbol(cache)]: {
      querySelector: Map(0) {},
      querySelectorAll: Map(0) {},
      matches: Map(0) {},
      elementsByTagName: Map(0) {},
      elementsByTagNameNS: Map(0) {},
      elementByTagName: Map(0) {},
      elementById: Map(0) {},
      computedStyle: null
    },
    [Symbol(affectsCache)]: [],
    [Symbol(ownerDocument)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(window)]: <ref *3> GlobalWindow [EventTarget] {
      Node: [Function],
      Attr: [class Attr extends Node],
      ShadowRoot: [class ShadowRoot extends DocumentFragment],
      ProcessingInstruction: [class ProcessingInstruction extends CharacterData],
      Element: [Function],
      CharacterData: [class CharacterData extends Node],
      DocumentType: [class DocumentType extends Node],
      AnimationTimeline: [class AnimationTimeline],
      DocumentTimeline: [class DocumentTimeline extends AnimationTimeline],
      HTMLAnchorElement: [class HTMLAnchorElement extends HTMLElement],
      HTMLButtonElement: [class HTMLButtonElement extends HTMLElement],
      HTMLOptGroupElement: [class HTMLOptGroupElement extends HTMLElement],
      HTMLOptionElement: [class HTMLOptionElement extends HTMLElement],
      HTMLElement: [Function],
      HTMLUnknownElement: [class HTMLUnknownElement extends HTMLElement],
      HTMLTemplateElement: [class HTMLTemplateElement extends HTMLElement],
      HTMLInputElement: [class HTMLInputElement extends HTMLElement],
      HTMLSelectElement: [class HTMLSelectElement extends HTMLElement],
      HTMLTextAreaElement: [class HTMLTextAreaElement extends HTMLElement],
      HTMLImageElement: [class HTMLImageElement extends HTMLElement],
      HTMLStyleElement: [class HTMLStyleElement extends HTMLElement],
      HTMLLabelElement: [class HTMLLabelElement extends HTMLElement],
      HTMLSlotElement: [class HTMLSlotElement extends HTMLElement],
      HTMLMetaElement: [class HTMLMetaElement extends HTMLElement],
      HTMLMediaElement: [class HTMLMediaElement extends HTMLElement],
      HTMLAudioElement: [class HTMLAudioElement extends HTMLMediaElement],
      HTMLVideoElement: [class HTMLVideoElement extends HTMLMediaElement],
      HTMLBaseElement: [class HTMLBaseElement extends HTMLElement],
      HTMLDialogElement: [class HTMLDialogElement extends HTMLElement],
      HTMLScriptElement: [class HTMLScriptElement extends HTMLElement],
      HTMLLinkElement: [class HTMLLinkElement extends HTMLElement],
      HTMLIFrameElement: [class HTMLIFrameElement extends HTMLElement],
      HTMLFormElement: [class HTMLFormElement extends HTMLElement],
      HTMLUListElement: [class HTMLUListElement extends HTMLElement],
      HTMLTrackElement: [class HTMLTrackElement extends HTMLElement],
      HTMLTableRowElement: [class HTMLTableRowElement extends HTMLElement],
      HTMLTitleElement: [class HTMLTitleElement extends HTMLElement],
      HTMLTimeElement: [class HTMLTimeElement extends HTMLElement],
      HTMLTableSectionElement: [class HTMLTableSectionElement extends HTMLElement],
      HTMLTableCellElement: [class HTMLTableCellElement extends HTMLElement],
      HTMLTableElement: [class HTMLTableElement extends HTMLElement],
      HTMLSpanElement: [class HTMLSpanElement extends HTMLElement],
      HTMLSourceElement: [class HTMLSourceElement extends HTMLElement],
      HTMLQuoteElement: [class HTMLQuoteElement extends HTMLElement],
      HTMLProgressElement: [class HTMLProgressElement extends HTMLElement],
      HTMLPreElement: [class HTMLPreElement extends HTMLElement],
      HTMLPictureElement: [class HTMLPictureElement extends HTMLElement],
      HTMLParamElement: [class HTMLParamElement extends HTMLElement],
      HTMLParagraphElement: [class HTMLParagraphElement extends HTMLElement],
      HTMLOutputElement: [class HTMLOutputElement extends HTMLElement],
      HTMLOListElement: [class HTMLOListElement extends HTMLElement],
      HTMLObjectElement: [class HTMLObjectElement extends HTMLElement],
      HTMLMeterElement: [class HTMLMeterElement extends HTMLElement],
      HTMLMenuElement: [class HTMLMenuElement extends HTMLElement],
      HTMLMapElement: [class HTMLMapElement extends HTMLElement],
      HTMLLIElement: [class HTMLLIElement extends HTMLElement],
      HTMLLegendElement: [class HTMLLegendElement extends HTMLElement],
      HTMLModElement: [class HTMLModElement extends HTMLElement],
      HTMLHtmlElement: [class HTMLHtmlElement extends HTMLElement],
      HTMLHRElement: [class HTMLHRElement extends HTMLElement],
      HTMLHeadElement: [class HTMLHeadElement extends HTMLElement],
      HTMLHeadingElement: [class HTMLHeadingElement extends HTMLElement],
      HTMLFieldSetElement: [class HTMLFieldSetElement extends HTMLElement],
      HTMLEmbedElement: [class HTMLEmbedElement extends HTMLElement],
      HTMLDListElement: [class HTMLDListElement extends HTMLElement],
      HTMLDivElement: [class HTMLDivElement extends HTMLElement],
      HTMLDetailsElement: [class HTMLDetailsElement extends HTMLElement],
      HTMLDataListElement: [class HTMLDataListElement extends HTMLElement],
      HTMLDataElement: [class HTMLDataElement extends HTMLElement],
      HTMLTableColElement: [class HTMLTableColElement extends HTMLElement],
      HTMLTableCaptionElement: [class HTMLTableCaptionElement extends HTMLElement],
      HTMLCanvasElement: [class HTMLCanvasElement extends HTMLElement],
      HTMLBRElement: [class HTMLBRElement extends HTMLElement],
      HTMLBodyElement: [class HTMLBodyElement extends HTMLElement],
      HTMLAreaElement: [class HTMLAreaElement extends HTMLElement],
      SVGSVGElement: [class SVGSVGElement extends SVGGraphicsElement],
      SVGAnimateElement: [class SVGAnimateElement extends SVGAnimationElement],
      SVGAnimateMotionElement: [class SVGAnimateMotionElement extends SVGAnimationElement],
      SVGAnimateTransformElement: [class SVGAnimateTransformElement extends SVGAnimationElement],
      SVGCircleElement: [class SVGCircleElement extends SVGGeometryElement],
      SVGClipPathElement: [class SVGClipPathElement extends SVGElement],
      SVGDefsElement: [class SVGDefsElement extends SVGGraphicsElement],
      SVGDescElement: [class SVGDescElement extends SVGElement],
      SVGEllipseElement: [class SVGEllipseElement extends SVGGeometryElement],
      SVGFEBlendElement: [Function],
      SVGFEColorMatrixElement: [Function],
      SVGFEComponentTransferElement: [class SVGFEComponentTransferElement extends SVGElement],
      SVGFECompositeElement: [Function],
      SVGFEConvolveMatrixElement: [Function],
      SVGFEDiffuseLightingElement: [class SVGFEDiffuseLightingElement extends SVGElement],
      SVGFEDisplacementMapElement: [Function],
      SVGFEDistantLightElement: [class SVGFEDistantLightElement extends SVGElement],
      SVGFEDropShadowElement: [class SVGFEDropShadowElement extends SVGElement],
      SVGFEFloodElement: [class SVGFEFloodElement extends SVGElement],
      SVGFEFuncAElement: [class SVGFEFuncAElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncBElement: [class SVGFEFuncBElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncGElement: [class SVGFEFuncGElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncRElement: [class SVGFEFuncRElement extends SVGComponentTransferFunctionElement],
      SVGFEGaussianBlurElement: [Function],
      SVGFEImageElement: [class SVGFEImageElement extends SVGElement],
      SVGFEMergeElement: [class SVGFEMergeElement extends SVGElement],
      SVGFEMergeNodeElement: [class SVGFEMergeNodeElement extends SVGElement],
      SVGFEMorphologyElement: [Function],
      SVGFEOffsetElement: [class SVGFEOffsetElement extends SVGElement],
      SVGFEPointLightElement: [class SVGFEPointLightElement extends SVGElement],
      SVGFESpecularLightingElement: [class SVGFESpecularLightingElement extends SVGElement],
      SVGFESpotLightElement: [class SVGFESpotLightElement extends SVGElement],
      SVGFETileElement: [class SVGFETileElement extends SVGElement],
      SVGFETurbulenceElement: [Function],
      SVGFilterElement: [class SVGFilterElement extends SVGElement],
      SVGForeignObjectElement: [class SVGForeignObjectElement extends SVGGraphicsElement],
      SVGGElement: [class SVGGElement extends SVGGraphicsElement],
      SVGImageElement: [class SVGImageElement extends SVGGraphicsElement],
      SVGLineElement: [class SVGLineElement extends SVGGeometryElement],
      SVGLinearGradientElement: [class SVGLinearGradientElement extends SVGGradientElement],
      SVGMarkerElement: [Function],
      SVGMaskElement: [class SVGMaskElement extends SVGElement],
      SVGMetadataElement: [class SVGMetadataElement extends SVGElement],
      SVGMPathElement: [class SVGMPathElement extends SVGElement],
      SVGPathElement: [class SVGPathElement extends SVGGeometryElement],
      SVGPatternElement: [class SVGPatternElement extends SVGElement],
      SVGPolygonElement: [class SVGPolygonElement extends SVGGeometryElement],
      SVGPolylineElement: [class SVGPolylineElement extends SVGGeometryElement],
      SVGRadialGradientElement: [class SVGRadialGradientElement extends SVGGradientElement],
      SVGRectElement: [class SVGRectElement extends SVGGeometryElement],
      SVGScriptElement: [class SVGScriptElement extends SVGGraphicsElement],
      SVGSetElement: [class SVGSetElement extends SVGAnimationElement],
      SVGStopElement: [class SVGStopElement extends SVGElement],
      SVGStyleElement: [class SVGStyleElement extends SVGElement],
      SVGSwitchElement: [class SVGSwitchElement extends SVGGraphicsElement],
      SVGSymbolElement: [class SVGSymbolElement extends SVGGraphicsElement],
      SVGTextElement: [class SVGTextElement extends SVGTextPositioningElement],
      SVGTextPathElement: [Function],
      SVGTitleElement: [class SVGTitleElement extends SVGElement],
      SVGTSpanElement: [class SVGTSpanElement extends SVGTextPositioningElement],
      SVGUseElement: [class SVGUseElement extends SVGGraphicsElement],
      SVGViewElement: [class SVGViewElement extends SVGElement],
      SVGElement: [class SVGElement extends Element],
      SVGAnimationElement: [class SVGAnimationElement extends SVGElement],
      SVGComponentTransferFunctionElement: [Function],
      SVGGeometryElement: [class SVGGeometryElement extends SVGGraphicsElement],
      SVGGradientElement: [Function],
      SVGTextPositioningElement: [class SVGTextPositioningElement extends SVGTextContentElement],
      SVGGraphicsElement: [class SVGGraphicsElement extends SVGElement],
      Event: [Function],
      UIEvent: [Function],
      CustomEvent: [class CustomEvent extends Event],
      CloseEvent: [class CloseEvent extends Event],
      AnimationEvent: [class AnimationEvent extends Event],
      KeyboardEvent: [Function],
      MessageEvent: [class MessageEvent extends Event],
      MouseEvent: [class MouseEvent extends UIEvent],
      PointerEvent: [class PointerEvent extends MouseEvent],
      FocusEvent: [class FocusEvent extends UIEvent],
      WheelEvent: [Function],
      InputEvent: [class InputEvent extends UIEvent],
      ErrorEvent: [class ErrorEvent extends UIEvent],
      StorageEvent: [class StorageEvent extends Event],
      SubmitEvent: [class SubmitEvent extends Event],
      ProgressEvent: [class ProgressEvent extends Event],
      MediaQueryListEvent: [class MediaQueryListEvent extends Event],
      HashChangeEvent: [class HashChangeEvent extends Event],
      ClipboardEvent: [class ClipboardEvent extends Event],
      CookieChangeEvent: [class CookieChangeEvent extends Event],
      TouchEvent: [class TouchEvent extends UIEvent],
      PopStateEvent: [class PopStateEvent extends Event],
      Touch: [class Touch],
      AudioProcessingEvent: [Function],
      BeforeInputEvent: [Function],
      BeforeUnloadEvent: [Function],
      BlobEvent: [Function],
      CompositionEvent: [Function],
      CSSFontFaceLoadEvent: [Function],
      DeviceLightEvent: [Function],
      DeviceMotionEvent: [Function],
      DeviceOrientationEvent: [Function],
      DeviceProximityEvent: [Function],
      DOMTransactionEvent: [Function],
      DragEvent: [Function],
      EditingBeforeInputEvent: [Function],
      FetchEvent: [Function],
      GamepadEvent: [Function],
      IDBVersionChangeEvent: [Function],
      MediaStreamEvent: [Function],
      MutationEvent: [Function],
      OfflineAudioCompletionEvent: [Function],
      OverconstrainedError: [Function],
      PageTransitionEvent: [Function],
      PaymentRequestUpdateEvent: [Function],
      RelatedEvent: [Function],
      RTCDataChannelEvent: [Function],
      RTCIdentityErrorEvent: [Function],
      RTCIdentityEvent: [Function],
      RTCPeerConnectionIceEvent: [Function],
      SensorEvent: [Function],
      SVGEvent: [Function],
      SVGZoomEvent: [Function],
      TimeEvent: [Function],
      TrackEvent: [Function],
      TransitionEvent: [Function],
      UserProximityEvent: [Function],
      WebGLContextEvent: [Function],
      TextEvent: [Function],
      Permissions: [class Permissions],
      History: [class History],
      Navigator: [class Navigator],
      Clipboard: [class Clipboard],
      TimeRanges: [class TimeRanges],
      TextTrackCueList: [class TextTrackCueList extends Array],
      ValidityState: [class ValidityState],
      MutationRecord: [class MutationRecord],
      IntersectionObserver: [class IntersectionObserver],
      IntersectionObserverEntry: [class IntersectionObserverEntry],
      CSSStyleDeclaration: [class CSSStyleDeclaration],
      CSSRule: [Function],
      CSSContainerRule: [class CSSContainerRule extends CSSConditionRule],
      CSSFontFaceRule: [class CSSFontFaceRule extends CSSRule],
      CSSKeyframeRule: [class CSSKeyframeRule extends CSSRule],
      CSSKeyframesRule: [class CSSKeyframesRule extends CSSRule],
      CSSMediaRule: [class CSSMediaRule extends CSSConditionRule],
      CSSStyleRule: [class CSSStyleRule extends CSSGroupingRule],
      CSSSupportsRule: [class CSSSupportsRule extends CSSConditionRule],
      CSSConditionRule: [class CSSConditionRule extends CSSGroupingRule],
      CSSGroupingRule: [class CSSGroupingRule extends CSSRule],
      CSSScopeRule: [class CSSScopeRule extends CSSGroupingRule],
      DOMRect: [class DOMRect extends DOMRectReadOnly],
      DOMRectReadOnly: [class DOMRectReadOnly],
      Plugin: [class Plugin],
      PluginArray: [class PluginArray],
      Location: [class Location],
      CustomElementRegistry: [class CustomElementRegistry],
      ResizeObserver: [class ResizeObserver],
      Blob: [class Blob],
      File: [class File extends Blob],
      Storage: [class Storage],
      CookieStore: [class CookieStore extends EventTarget],
      MimeType: [class MimeType],
      MimeTypeArray: [class MimeTypeArray],
      NodeFilter: [Object],
      ImageBitmap: [class ImageBitmap],
      HTMLCollection: [class HTMLCollection],
      HTMLFormControlCollection: [class HTMLFormControlsCollection extends HTMLCollection],
      HTMLOptionsCollection: [class HTMLOptionsCollection extends HTMLCollection],
      NodeList: [class NodeList],
      RadioNodeList: [class RadioNodeList extends NodeList],
      FileList: [class FileList extends Array],
      Screen: [class Screen extends EventTarget],
      ScreenDetails: [class ScreenDetails extends EventTarget],
      ScreenDetailed: [class ScreenDetailed extends Screen],
      NamedNodeMap: [class NamedNodeMap],
      TreeWalker: [class TreeWalker],
      NodeIterator: [class NodeIterator],
      DataTransfer: [class DataTransfer],
      DataTransferItem: [class DataTransferItem],
      DataTransferItemList: [class DataTransferItemList extends Array],
      XMLSerializer: [class XMLSerializer],
      ClipboardItem: [class ClipboardItem],
      Selection: [class Selection],
      CSSUnitValue: [class CSSUnitValue],
      SVGAngle: [Function],
      SVGAnimatedAngle: [class SVGAnimatedAngle],
      SVGAnimatedBoolean: [class SVGAnimatedBoolean],
      SVGAnimatedEnumeration: [class SVGAnimatedEnumeration],
      SVGAnimatedInteger: [class SVGAnimatedInteger],
      SVGAnimatedLength: [class SVGAnimatedLength],
      SVGAnimatedNumber: [class SVGAnimatedNumber],
      SVGAnimatedNumberList: [class SVGAnimatedNumberList],
      SVGAnimatedPreserveAspectRatio: [class SVGAnimatedPreserveAspectRatio],
      SVGAnimatedRect: [class SVGAnimatedRect],
      SVGAnimatedString: [class SVGAnimatedString],
      SVGAnimatedTransformList: [class SVGAnimatedTransformList],
      SVGLength: [Function],
      SVGLengthList: [class SVGLengthList],
      SVGMatrix: [class SVGMatrix],
      SVGNumber: [class SVGNumber],
      SVGNumberList: [class SVGNumberList],
      SVGPoint: [class SVGPoint],
      SVGPointList: [class SVGPointList],
      SVGPreserveAspectRatio: [Function],
      SVGRect: [class SVGRect],
      SVGStringList: [class SVGStringList],
      SVGTransform: [Function],
      SVGTransformList: [class SVGTransformList],
      SVGAnimatedLengthList: [class SVGAnimatedLengthList],
      SVGUnitTypes: [Function],
      StylePropertyMap: [class StylePropertyMap extends StylePropertyMapReadOnly],
      StylePropertyMapReadOnly: [class StylePropertyMapReadOnly],
      MediaList: [class MediaList],
      CSSKeywordValue: [class CSSKeywordValue],
      CSSStyleValue: [class CSSStyleValue],
      Window: [class GlobalWindow extends Window],
      URLSearchParams: [class URLSearchParams],
      WritableStream: [Function],
      ReadableStream: [Function],
      TransformStream: [Function: Transform],
      PerformanceObserver: [class PerformanceObserver],
      PerformanceEntry: [class PerformanceEntry],
      PerformanceObserverEntryList: [class PerformanceObserverEntryList],
      document: [HTMLDocument],
      customElements: [CustomElementRegistry],
      window: [Circular *3],
      globalThis: [Circular *3],
      performance: [Performance],
      screenLeft: 0,
      screenTop: 0,
      screenX: 0,
      screenY: 0,
      crypto: Crypto {},
      TextEncoder: [class TextEncoder],
      TextDecoder: [class TextDecoder],
      closed: false,
      console: [Object [console]],
      name: '',
      Buffer: [Function],
      Document: [class Document extends Document],
      HTMLDocument: [class HTMLDocument extends HTMLDocument],
      XMLDocument: [class XMLDocument extends XMLDocument],
      DocumentFragment: [class DocumentFragment extends DocumentFragment],
      Text: [class Text extends Text],
      Comment: [class Comment extends Comment],
      Image: [class Image extends Image],
      Audio: [class Audio extends Audio],
      MutationObserver: [class MutationObserver extends MutationObserver],
      MessagePort: [class MessagePort extends MessagePort],
      CSSStyleSheet: [class CSSStyleSheet extends CSSStyleSheet],
      DOMException: [class DOMException extends DOMException],
      Headers: [class Headers extends Headers],
      Request: [class Request extends Request],
      Response: [Function],
      EventTarget: [class EventTarget extends EventTarget],
      XMLHttpRequestUpload: [class XMLHttpRequestUpload extends XMLHttpRequestUpload],
      XMLHttpRequestEventTarget: [class XMLHttpRequestEventTarget extends XMLHttpRequestEventTarget],
      AbortController: [class AbortController extends AbortController],
      AbortSignal: [Function],
      FormData: [class FormData extends FormData],
      PermissionStatus: [class PermissionStatus extends PermissionStatus],
      XMLHttpRequest: [class XMLHttpRequest extends XMLHttpRequest],
      DOMParser: [class DOMParser extends DOMParser],
      Range: [class Range extends Range],
      VTTCue: [class VTTCue extends VTTCue],
      TextTrack: [class TextTrack extends TextTrack],
      TextTrackList: [class TextTrackList extends TextTrackList],
      TextTrackCue: [class TextTrackCue extends TextTrackCue],
      RemotePlayback: [class RemotePlayback extends RemotePlayback],
      FileReader: [class FileReader extends FileReader],
      MediaStream: [class MediaStream extends MediaStream],
      MediaStreamTrack: [class MediaStreamTrack extends MediaStreamTrack],
      CanvasCaptureMediaStreamTrack: [class CanvasCaptureMediaStreamTrack extends CanvasCaptureMediaStreamTrack],
      URL: [class URL extends URL],
      WebSocket: [class WebSocket extends WebSocket],
      ImageData: [class ImageData extends ImageData],
      OffscreenCanvas: [class OffscreenCanvas extends OffscreenCanvas],
      DOMMatrixReadOnly: [class DOMMatrixReadOnly extends DOMMatrixReadOnly],
      DOMMatrix: [class DOMMatrix extends DOMMatrix],
      DOMPointReadOnly: [class DOMPointReadOnly extends DOMPointReadOnly],
      DOMPoint: [class DOMPoint extends DOMPoint],
      Animation: [class Animation extends Animation],
      KeyframeEffect: [class KeyframeEffect extends KeyframeEffect],
      happyDOM: DetachedWindowAPI {},
      Array: [Function: Array],
      ArrayBuffer: [Function: ArrayBuffer],
      Boolean: [Function: Boolean],
      DataView: [Function: DataView],
      Date: [Function: Date],
      Error: [Function],
      EvalError: [Function: EvalError],
      Float32Array: [Function: Float32Array],
      Float64Array: [Function: Float64Array],
      Function: [Function: Function],
      Infinity: Infinity,
      Int16Array: [Function: Int16Array],
      Int32Array: [Function: Int32Array],
      Int8Array: [Function: Int8Array],
      Intl: Object [Intl] {},
      JSON: Object [JSON] {},
      Map: [Function: Map],
      Math: Object [Math] {},
      NaN: NaN,
      Number: [Function: Number],
      Object: [Function: Object],
      Promise: [Function: Promise],
      RangeError: [Function: RangeError],
      ReferenceError: [Function: ReferenceError],
      RegExp: [Function: RegExp],
      Set: [Function: Set],
      String: [Function: String],
      Symbol: [Function: Symbol],
      SyntaxError: [Function: SyntaxError],
      TypeError: [Function: TypeError],
      URIError: [Function: URIError],
      Uint16Array: [Function: Uint16Array],
      Uint32Array: [Function: Uint32Array],
      Uint8Array: [Function: Uint8Array],
      Uint8ClampedArray: [Function: Uint8ClampedArray],
      WeakMap: [Function: WeakMap],
      WeakSet: [Function: WeakSet],
      decodeURI: [Function: decodeURI],
      decodeURIComponent: [Function: decodeURIComponent],
      encodeURI: [Function: encodeURI],
      encodeURIComponent: [Function: encodeURIComponent],
      eval: [Function: eval],
      escape: [Function: escape],
      global: [Object [global]],
      isFinite: [Function: isFinite],
      isNaN: [Function: isNaN],
      parseFloat: [Function: parseFloat],
      parseInt: [Function: parseInt],
      undefined: undefined,
      unescape: [Function: unescape],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(mutationObservers)]: [Array],
      [Symbol(readyStateManager)]: DocumentReadyStateManager {},
      [Symbol(location)]: [Location],
      [Symbol(history)]: History {},
      [Symbol(navigator)]: Navigator {},
      [Symbol(screen)]: [Screen [EventTarget]],
      [Symbol(sessionStorage)]: [Storage],
      [Symbol(localStorage)]: [Storage],
      [Symbol(cookieStore)]: [CookieStore [EventTarget]],
      [Symbol(self)]: [Circular *3],
      [Symbol(top)]: [Circular *3],
      [Symbol(parent)]: [Circular *3],
      [Symbol(window)]: [Circular *3],
      [Symbol(frames)]: [Circular *3],
      [Symbol(internalId)]: 0,
      [Symbol(customElementReactionStack)]: [CustomElementReactionStack],
      [Symbol(modules)]: [Object],
      [Symbol(moduleImportMap)]: null,
      [Symbol(openWebSockets)]: [],
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(querySelectorCache)]: [Map]
    },
    [Symbol(classList)]: null,
    [Symbol(isValue)]: null,
    [Symbol(nodeType)]: 1,
    [Symbol(prefix)]: null,
    [Symbol(shadowRoot)]: null,
    [Symbol(scrollHeight)]: 0,
    [Symbol(scrollWidth)]: 0,
    [Symbol(scrollTop)]: 0,
    [Symbol(scrollLeft)]: 0,
    [Symbol(attributes)]: NamedNodeMap {
      [Symbol(itemsByNamespaceURI)]: [Map],
      [Symbol(itemsByName)]: [Map],
      [Symbol(items)]: [Map],
      [Symbol(ownerElement)]: [Circular *2]
    },
    [Symbol(attributesProxy)]: null,
    [Symbol(children)]: null,
    [Symbol(computedStyle)]: null,
    [Symbol(pointerCaptures)]: Set(0) {},
    [Symbol(propertyEventListeners)]: Map(0) {},
    [Symbol(animations)]: [],
    [Symbol(tagName)]: 'STYLE',
    [Symbol(localName)]: 'style',
    [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
    [Symbol(accessKey)]: '',
    [Symbol(offsetHeight)]: 0,
    [Symbol(offsetWidth)]: 0,
    [Symbol(offsetLeft)]: 0,
    [Symbol(offsetTop)]: 0,
    [Symbol(clientHeight)]: 0,
    [Symbol(clientWidth)]: 0,
    [Symbol(clientLeft)]: 0,
    [Symbol(clientTop)]: 0,
    [Symbol(style)]: null,
    [Symbol(dataset)]: null,
    [Symbol(customElementDefineCallback)]: null,
    [Symbol(sheet)]: null,
    [Symbol(disabled)]: false
  },
  className: 'tiptap',
  editorView: <ref *5> EditorView {
    _root: null,
    focused: false,
    trackWrites: null,
    mounted: false,
    markCursor: null,
    cursorWrapper: null,
    lastSelectedViewDesc: undefined,
    input: InputState {
      shiftKey: false,
      mouseDown: null,
      lastKeyCode: null,
      lastKeyCodeTime: 0,
      lastClick: [Object],
      lastSelectionOrigin: null,
      lastSelectionTime: 0,
      lastIOSEnter: 0,
      lastIOSEnterFallbackTimeout: -1,
      lastFocus: 0,
      lastTouch: 0,
      lastChromeDelete: 0,
      composing: false,
      compositionNode: null,
      composingTimeout: -1,
      compositionNodes: [],
      compositionEndedAt: -200000000,
      compositionID: 1,
      badSafariComposition: false,
      compositionPendingChanges: 0,
      domChangeCount: 0,
      eventHandlers: [Object: null prototype],
      hideSelectionGuard: null
    },
    prevDirectPlugins: [],
    pluginViews: [
      [Object],
      [Object],
      [Object],
      [Object],
      [DropCursorView],
      [Object],
      [Object],
      [Object]
    ],
    requiresGeckoHackNode: false,
    dragging: null,
    _props: {
      attributes: [Object],
      dispatchTransaction: [Function: bound dispatchTransaction],
      transformPastedHTML: [Function (anonymous)],
      state: [EditorState],
      markViews: {},
      nodeViews: [Object]
    },
    state: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    directPlugins: [],
    dispatch: [Function: bound ],
    dom: HTMLDivElement {
      pmViewDesc: [NodeViewDesc],
      editor: [Circular *4],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: [HTMLDivElement],
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [Array],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: [DOMTokenList],
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: [CSSStyleDeclaration],
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    editable: true,
    nodeViews: [Object: null prototype] { taskItem: [Function: nodeview] },
    docView: NodeViewDesc {
      parent: undefined,
      children: [Array],
      dom: [HTMLDivElement],
      contentDOM: [HTMLDivElement],
      dirty: 0,
      node: [Node],
      outerDeco: [Array],
      innerDeco: [DecorationSet],
      nodeDOM: [HTMLDivElement]
    },
    domObserver: DOMObserver {
      view: [Circular *5],
      handleDOMChange: [Function (anonymous)],
      queue: [],
      flushingSoon: -1,
      observer: MutationObserver {},
      currentSelection: [SelectionState],
      onCharData: null,
      suppressingSelectionUpdates: false,
      lastChangedTextNode: null,
      onSelectionChange: [Function: bound onSelectionChange]
    }
  },
  isFocused: false,
  destroyed: false,
  isInitialized: false,
  extensionStorage: {
    link: {},
    paragraph: {},
    listItemBranchingDeleteKeymap: {},
    taskItemBranchingDeleteKeymap: {},
    textStyle: {},
    editable: {},
    clipboardTextSerializer: {},
    commands: {},
    focusEvents: {},
    keymap: {},
    tabindex: {},
    drop: {},
    paste: {},
    delete: {},
    textDirection: {},
    starterKit: {},
    bold: {},
    blockquote: {},
    bulletList: {},
    code: {},
    doc: {},
    dropCursor: {},
    gapCursor: {},
    hardBreak: {},
    undoRedo: {},
    italic: {},
    listItem: {},
    listKeymap: {},
    orderedList: {},
    strike: {},
    text: {},
    underline: {},
    trailingNode: {},
    taskList: {},
    taskItem: {},
    color: {},
    fontSize: {},
    textAlign: {},
    typography: {},
    placeholder: {}
  },
  instanceId: 'ghfenf8',
  hasWarnedStaleDecorationRead: false,
  options: {
    element: HTMLDivElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    content: { type: 'doc', content: [Array] },
    injectCSS: true,
    injectNonce: undefined,
    extensions: [
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    autofocus: false,
    editable: true,
    textDirection: undefined,
    editorProps: { attributes: [Object] },
    parseOptions: {},
    coreExtensionOptions: {},
    enableInputRules: true,
    enablePasteRules: true,
    enableCoreExtensions: true,
    enableContentCheck: false,
    emitContentError: false,
    onBeforeCreate: [Function: onBeforeCreate],
    onCreate: [Function: onCreate],
    onMount: [Function: onMount],
    onUnmount: [Function: onUnmount],
    onUpdate: [Function: onUpdate],
    onSelectionUpdate: [Function: onSelectionUpdate],
    onTransaction: [Function: onTransaction],
    onFocus: [Function: onFocus],
    onBlur: [Function: onBlur],
    onDestroy: [Function: onDestroy],
    onContentError: [Function: onContentError],
    onPaste: [Function: onPaste],
    onDrop: [Function: onDrop],
    onDelete: [Function: onDelete],
    enableExtensionDispatchTransaction: true
  },
  isCapturingTransaction: false,
  capturedTransaction: null,
  utils: {
    getUpdatedPosition: [Function: getUpdatedPosition],
    createMappablePosition: [Function: createMappablePosition]
  },
  extensionManager: ExtensionManager {
    splittableMarks: [ 'textStyle', 'bold', 'code', 'italic', 'strike', 'underline' ],
    nonClearableMarks: [],
    decorationManager: DecorationManager {
      warnedWidgetKeys: Set(0) {},
      warnedOutOfRangeExtensions: Set(0) {},
      handleBeforeTransaction: [Function (anonymous)],
      editor: [Circular *4],
      entries: [],
      plugin: null
    },
    editor: [Circular *4],
    baseExtensions: [
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    extensions: [
      [Mark],      [Node],      [Extension],
      [Extension], [Mark],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Mark],      [Node],
      [Node],      [Mark],      [Node],
      [Extension], [Extension], [Node],
      [Extension], [Mark],      [Node],
      [Extension], [Node],      [Mark],
      [Node],      [Mark],      [Extension],
      [Node],      [Node],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension]
    ],
    schema: Schema {
      linebreakReplacement: [NodeType],
      cached: [Object: null prototype],
      spec: [Object],
      nodes: [Object: null prototype],
      marks: [Object: null prototype],
      nodeFromJSON: [Function (anonymous)],
      markFromJSON: [Function (anonymous)],
      topNodeType: [NodeType]
    }
  },
  commandManager: CommandManager {
    editor: [Circular *4],
    rawCommands: {
      setLink: [Function: setLink],
      toggleLink: [Function: toggleLink],
      unsetLink: [Function: unsetLink],
      setParagraph: [Function: setParagraph],
      toggleTextStyle: [Function: toggleTextStyle],
      removeEmptyTextStyle: [Function: removeEmptyTextStyle],
      blur: [Function: blur],
      clearContent: [Function: clearContent],
      clearNodes: [Function: clearNodes],
      command: [Function: command],
      createParagraphNear: [Function: createParagraphNear$1],
      cut: [Function: cut],
      deleteCurrentNode: [Function: deleteCurrentNode],
      deleteNode: [Function: deleteNode],
      deleteRange: [Function: deleteRange],
      deleteSelection: [Function: deleteSelection],
      enter: [Function: enter],
      exitCode: [Function: exitCode$1],
      extendMarkRange: [Function: extendMarkRange],
      first: [Function: first],
      focus: [Function: focus],
      forEach: [Function: forEach],
      insertContent: [Function: insertContent],
      insertContentAt: [Function: insertContentAt],
      insertDefaultBlock: [Function: insertDefaultBlock],
      joinBackward: [Function: joinBackward$1],
      joinDown: [Function: joinDown$1],
      joinForward: [Function: joinForward$1],
      joinItemBackward: [Function: joinItemBackward],
      joinItemForward: [Function: joinItemForward],
      joinTextblockBackward: [Function: joinTextblockBackward$1],
      joinTextblockForward: [Function: joinTextblockForward$1],
      joinUp: [Function: joinUp$1],
      keyboardShortcut: [Function: keyboardShortcut],
      lift: [Function: lift$1],
      liftEmptyBlock: [Function: liftEmptyBlock$1],
      liftListItem: [Function: liftListItem$1],
      newlineInCode: [Function: newlineInCode$1],
      resetAttributes: [Function: resetAttributes],
      scrollIntoView: [Function: scrollIntoView],
      selectAll: [Function: selectAll],
      selectNodeBackward: [Function: selectNodeBackward$1],
      selectNodeForward: [Function: selectNodeForward$1],
      selectParentNode: [Function: selectParentNode$1],
      selectTextblockEnd: [Function: selectTextblockEnd$1],
      selectTextblockStart: [Function: selectTextblockStart$1],
      setContent: [Function: setContent],
      setMark: [Function: setMark],
      setMeta: [Function: setMeta],
      setNode: [Function: setNode],
      setNodeSelection: [Function: setNodeSelection],
      setTextDirection: [Function: setTextDirection],
      setTextSelection: [Function: setTextSelection],
      sinkListItem: [Function: sinkListItem$1],
      splitBlock: [Function: splitBlock],
      splitListItem: [Function: splitListItem],
      toggleList: [Function: toggleList],
      toggleMark: [Function: toggleMark],
      toggleNode: [Function: toggleNode],
      toggleWrap: [Function: toggleWrap],
      undoInputRule: [Function: undoInputRule],
      unsetAllMarks: [Function: unsetAllMarks],
      unsetMark: [Function: unsetMark],
      unsetTextDirection: [Function: unsetTextDirection],
      updateAttributes: [Function: updateAttributes],
      updateDecorations: [Function: updateDecorations],
      wrapIn: [Function: wrapIn$1],
      wrapInList: [Function: wrapInList$1],
      setBold: [Function: setBold],
      toggleBold: [Function: toggleBold],
      unsetBold: [Function: unsetBold],
      setBlockquote: [Function: setBlockquote],
      toggleBlockquote: [Function: toggleBlockquote],
      unsetBlockquote: [Function: unsetBlockquote],
      toggleBulletList: [Function: toggleBulletList],
      setCode: [Function: setCode],
      toggleCode: [Function: toggleCode],
      unsetCode: [Function: unsetCode],
      setHardBreak: [Function: setHardBreak],
      undo: [Function: undo],
      redo: [Function: redo],
      setItalic: [Function: setItalic],
      toggleItalic: [Function: toggleItalic],
      unsetItalic: [Function: unsetItalic],
      toggleOrderedList: [Function: toggleOrderedList],
      setStrike: [Function: setStrike],
      toggleStrike: [Function: toggleStrike],
      unsetStrike: [Function: unsetStrike],
      setUnderline: [Function: setUnderline],
      toggleUnderline: [Function: toggleUnderline],
      unsetUnderline: [Function: unsetUnderline],
      toggleTaskList: [Function: toggleTaskList],
      setColor: [Function: setColor],
      unsetColor: [Function: unsetColor],
      setFontSize: [Function: setFontSize],
      unsetFontSize: [Function: unsetFontSize],
      setTextAlign: [Function: setTextAlign],
      unsetTextAlign: [Function: unsetTextAlign],
      toggleTextAlign: [Function: toggleTextAlign]
    },
    customState: undefined
  },
  schema: <ref *6> Schema {
    linebreakReplacement: NodeType {
      name: 'hardBreak',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [Array],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: false,
      isText: false
    },
    cached: [Object: null prototype] { wrappings: [Object: null prototype] {} },
    spec: { topNode: 'doc', nodes: [OrderedMap], marks: [OrderedMap] },
    nodes: [Object: null prototype] {
      paragraph: [NodeType],
      blockquote: [NodeType],
      bulletList: [NodeType],
      doc: [NodeType],
      hardBreak: [NodeType],
      listItem: [NodeType],
      orderedList: [NodeType],
      text: [NodeType],
      taskList: [NodeType],
      taskItem: [NodeType]
    },
    marks: [Object: null prototype] {
      link: [MarkType],
      textStyle: [MarkType],
      bold: [MarkType],
      code: [MarkType],
      italic: [MarkType],
      strike: [MarkType],
      underline: [MarkType]
    },
    nodeFromJSON: [Function (anonymous)],
    markFromJSON: [Function (anonymous)],
    topNodeType: NodeType {
      name: 'doc',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: true,
      isText: false
    }
  },
  editorState: EditorState {
    config: Configuration {
      schema: [Schema],
      plugins: [],
      pluginsByKey: [Object: null prototype] {},
      fields: [Array]
    },
    doc: Node {
      type: [NodeType],
      attrs: [Object: null prototype] {},
      marks: [],
      content: [Fragment]
    },
    selection: TextSelection {
      '$anchor': [ResolvedPos],
      '$head': [ResolvedPos],
      ranges: [Array]
    },
    storedMarks: null,
    scrollToSelection: 0
  },
  contentComponent: null,
  appContext: null,
  reactiveState: CustomRefImpl {
    __v_isRef: true,
    _value: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: [Link],
      map: undefined,
      key: undefined,
      sc: 2,
      __v_skip: true,
      subsHead: [Link]
    },
    _get: [Function: get],
    _set: [Function: set]
  },
  reactiveExtensionStorage: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  }
} default-font-size=undefined  ... > 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[intlify] Not found 'spaces.canvas.textAlignments.undefined' key in 'en' locale messages.
[Vue warn]: Unhandled error during execution of render function 
  at <PrimitiveSlot aria-describedby=undefined data-state="closed" data-grace-area-trigger=""  ... > 
  at <Primitive aria-describedby=undefined data-state="closed" as="button"  ... > 
  at <PrimitiveSlot > 
  at <Primitive ref=fn<forwardRef> as=undefined as-child=true > 
  at <PopperAnchor as-child="" reference=undefined > 
  at <TooltipTrigger as-child="" > 
  at <PopperRoot> 
  at <TooltipRoot disabled=false ignore-non-keyboard-focus=true > 
  at <TooltipProvider delay-duration=300 skip-delay-duration=100 > 
  at <UiTooltip text="spaces.canvas.textAlignments.undefined" shortcut=undefined side="top"  ... > 
  at <UiIconButton id="reka-dropdown-menu-trigger-v-8" type="button" aria-haspopup="menu"  ... > 
  at <PrimitiveSlot id="reka-dropdown-menu-trigger-v-8" type="button" aria-haspopup="menu"  ... > 
  at <Primitive id="reka-dropdown-menu-trigger-v-8" type="button" as-child=true  ... > 
  at <PrimitiveSlot > 
  at <Primitive ref=fn<forwardRef> as=undefined as-child=true > 
  at <PopperAnchor asChild=true reference=undefined as=undefined > 
  at <MenuAnchor as-child="" > 
  at <DropdownMenuTrigger key=0 as-child="" > 
  at <PopperRoot> 
  at <MenuRoot open=false onUpdate:open=fn dir="ltr"  ... > 
  at <DropdownMenuRoot open=false onUpdate:open=fn modal=false > 
  at <UiDropdownMenu side="top" align="end" side-offset=5  ... > 
  at <SpaceCanvasTextAlignmentMenu editor= <ref *4> Editor {
  callbacks: {
    create: [ [Function: bound onCreate], [Function: onCreate] ],
    destroy: [ [Function: bound onDestroy], [Function: onDestroy] ],
    update: [ [Function: bound onUpdate], [Function: onUpdate] ],
    beforeCreate: [ [Function: onBeforeCreate] ],
    mount: [ [Function: onMount] ],
    unmount: [ [Function: onUnmount] ],
    contentError: [ [Function: onContentError] ],
    selectionUpdate: [ [Function: onSelectionUpdate] ],
    transaction: [ [Function: onTransaction] ],
    focus: [ [Function: onFocus] ],
    blur: [ [Function: onBlur] ],
    drop: [ [Function (anonymous)] ],
    paste: [ [Function (anonymous)] ],
    delete: [ [Function: onDelete] ],
    beforeTransaction: [ [Function (anonymous)], [Function (anonymous)] ]
  },
  css: <ref *2> HTMLStyleElement {
    [Symbol(listeners)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(listenerOptions)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(isConnected)]: true,
    [Symbol(parentNode)]: HTMLHeadElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: [HTMLHtmlElement],
      [Symbol(rootNode)]: [HTMLDocument],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'HEAD',
      [Symbol(localName)]: 'head',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    [Symbol(rootNode)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(styleNode)]: [Circular *2],
    [Symbol(textAreaNode)]: null,
    [Symbol(formNode)]: null,
    [Symbol(selectNode)]: null,
    [Symbol(mutationListeners)]: [],
    [Symbol(nodeArray)]: [ [Text] ],
    [Symbol(elementArray)]: [],
    [Symbol(childNodes)]: null,
    [Symbol(assignedToSlot)]: null,
    [Symbol(cache)]: {
      querySelector: Map(0) {},
      querySelectorAll: Map(0) {},
      matches: Map(0) {},
      elementsByTagName: Map(0) {},
      elementsByTagNameNS: Map(0) {},
      elementByTagName: Map(0) {},
      elementById: Map(0) {},
      computedStyle: null
    },
    [Symbol(affectsCache)]: [],
    [Symbol(ownerDocument)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(window)]: <ref *3> GlobalWindow [EventTarget] {
      Node: [Function],
      Attr: [class Attr extends Node],
      ShadowRoot: [class ShadowRoot extends DocumentFragment],
      ProcessingInstruction: [class ProcessingInstruction extends CharacterData],
      Element: [Function],
      CharacterData: [class CharacterData extends Node],
      DocumentType: [class DocumentType extends Node],
      AnimationTimeline: [class AnimationTimeline],
      DocumentTimeline: [class DocumentTimeline extends AnimationTimeline],
      HTMLAnchorElement: [class HTMLAnchorElement extends HTMLElement],
      HTMLButtonElement: [class HTMLButtonElement extends HTMLElement],
      HTMLOptGroupElement: [class HTMLOptGroupElement extends HTMLElement],
      HTMLOptionElement: [class HTMLOptionElement extends HTMLElement],
      HTMLElement: [Function],
      HTMLUnknownElement: [class HTMLUnknownElement extends HTMLElement],
      HTMLTemplateElement: [class HTMLTemplateElement extends HTMLElement],
      HTMLInputElement: [class HTMLInputElement extends HTMLElement],
      HTMLSelectElement: [class HTMLSelectElement extends HTMLElement],
      HTMLTextAreaElement: [class HTMLTextAreaElement extends HTMLElement],
      HTMLImageElement: [class HTMLImageElement extends HTMLElement],
      HTMLStyleElement: [class HTMLStyleElement extends HTMLElement],
      HTMLLabelElement: [class HTMLLabelElement extends HTMLElement],
      HTMLSlotElement: [class HTMLSlotElement extends HTMLElement],
      HTMLMetaElement: [class HTMLMetaElement extends HTMLElement],
      HTMLMediaElement: [class HTMLMediaElement extends HTMLElement],
      HTMLAudioElement: [class HTMLAudioElement extends HTMLMediaElement],
      HTMLVideoElement: [class HTMLVideoElement extends HTMLMediaElement],
      HTMLBaseElement: [class HTMLBaseElement extends HTMLElement],
      HTMLDialogElement: [class HTMLDialogElement extends HTMLElement],
      HTMLScriptElement: [class HTMLScriptElement extends HTMLElement],
      HTMLLinkElement: [class HTMLLinkElement extends HTMLElement],
      HTMLIFrameElement: [class HTMLIFrameElement extends HTMLElement],
      HTMLFormElement: [class HTMLFormElement extends HTMLElement],
      HTMLUListElement: [class HTMLUListElement extends HTMLElement],
      HTMLTrackElement: [class HTMLTrackElement extends HTMLElement],
      HTMLTableRowElement: [class HTMLTableRowElement extends HTMLElement],
      HTMLTitleElement: [class HTMLTitleElement extends HTMLElement],
      HTMLTimeElement: [class HTMLTimeElement extends HTMLElement],
      HTMLTableSectionElement: [class HTMLTableSectionElement extends HTMLElement],
      HTMLTableCellElement: [class HTMLTableCellElement extends HTMLElement],
      HTMLTableElement: [class HTMLTableElement extends HTMLElement],
      HTMLSpanElement: [class HTMLSpanElement extends HTMLElement],
      HTMLSourceElement: [class HTMLSourceElement extends HTMLElement],
      HTMLQuoteElement: [class HTMLQuoteElement extends HTMLElement],
      HTMLProgressElement: [class HTMLProgressElement extends HTMLElement],
      HTMLPreElement: [class HTMLPreElement extends HTMLElement],
      HTMLPictureElement: [class HTMLPictureElement extends HTMLElement],
      HTMLParamElement: [class HTMLParamElement extends HTMLElement],
      HTMLParagraphElement: [class HTMLParagraphElement extends HTMLElement],
      HTMLOutputElement: [class HTMLOutputElement extends HTMLElement],
      HTMLOListElement: [class HTMLOListElement extends HTMLElement],
      HTMLObjectElement: [class HTMLObjectElement extends HTMLElement],
      HTMLMeterElement: [class HTMLMeterElement extends HTMLElement],
      HTMLMenuElement: [class HTMLMenuElement extends HTMLElement],
      HTMLMapElement: [class HTMLMapElement extends HTMLElement],
      HTMLLIElement: [class HTMLLIElement extends HTMLElement],
      HTMLLegendElement: [class HTMLLegendElement extends HTMLElement],
      HTMLModElement: [class HTMLModElement extends HTMLElement],
      HTMLHtmlElement: [class HTMLHtmlElement extends HTMLElement],
      HTMLHRElement: [class HTMLHRElement extends HTMLElement],
      HTMLHeadElement: [class HTMLHeadElement extends HTMLElement],
      HTMLHeadingElement: [class HTMLHeadingElement extends HTMLElement],
      HTMLFieldSetElement: [class HTMLFieldSetElement extends HTMLElement],
      HTMLEmbedElement: [class HTMLEmbedElement extends HTMLElement],
      HTMLDListElement: [class HTMLDListElement extends HTMLElement],
      HTMLDivElement: [class HTMLDivElement extends HTMLElement],
      HTMLDetailsElement: [class HTMLDetailsElement extends HTMLElement],
      HTMLDataListElement: [class HTMLDataListElement extends HTMLElement],
      HTMLDataElement: [class HTMLDataElement extends HTMLElement],
      HTMLTableColElement: [class HTMLTableColElement extends HTMLElement],
      HTMLTableCaptionElement: [class HTMLTableCaptionElement extends HTMLElement],
      HTMLCanvasElement: [class HTMLCanvasElement extends HTMLElement],
      HTMLBRElement: [class HTMLBRElement extends HTMLElement],
      HTMLBodyElement: [class HTMLBodyElement extends HTMLElement],
      HTMLAreaElement: [class HTMLAreaElement extends HTMLElement],
      SVGSVGElement: [class SVGSVGElement extends SVGGraphicsElement],
      SVGAnimateElement: [class SVGAnimateElement extends SVGAnimationElement],
      SVGAnimateMotionElement: [class SVGAnimateMotionElement extends SVGAnimationElement],
      SVGAnimateTransformElement: [class SVGAnimateTransformElement extends SVGAnimationElement],
      SVGCircleElement: [class SVGCircleElement extends SVGGeometryElement],
      SVGClipPathElement: [class SVGClipPathElement extends SVGElement],
      SVGDefsElement: [class SVGDefsElement extends SVGGraphicsElement],
      SVGDescElement: [class SVGDescElement extends SVGElement],
      SVGEllipseElement: [class SVGEllipseElement extends SVGGeometryElement],
      SVGFEBlendElement: [Function],
      SVGFEColorMatrixElement: [Function],
      SVGFEComponentTransferElement: [class SVGFEComponentTransferElement extends SVGElement],
      SVGFECompositeElement: [Function],
      SVGFEConvolveMatrixElement: [Function],
      SVGFEDiffuseLightingElement: [class SVGFEDiffuseLightingElement extends SVGElement],
      SVGFEDisplacementMapElement: [Function],
      SVGFEDistantLightElement: [class SVGFEDistantLightElement extends SVGElement],
      SVGFEDropShadowElement: [class SVGFEDropShadowElement extends SVGElement],
      SVGFEFloodElement: [class SVGFEFloodElement extends SVGElement],
      SVGFEFuncAElement: [class SVGFEFuncAElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncBElement: [class SVGFEFuncBElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncGElement: [class SVGFEFuncGElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncRElement: [class SVGFEFuncRElement extends SVGComponentTransferFunctionElement],
      SVGFEGaussianBlurElement: [Function],
      SVGFEImageElement: [class SVGFEImageElement extends SVGElement],
      SVGFEMergeElement: [class SVGFEMergeElement extends SVGElement],
      SVGFEMergeNodeElement: [class SVGFEMergeNodeElement extends SVGElement],
      SVGFEMorphologyElement: [Function],
      SVGFEOffsetElement: [class SVGFEOffsetElement extends SVGElement],
      SVGFEPointLightElement: [class SVGFEPointLightElement extends SVGElement],
      SVGFESpecularLightingElement: [class SVGFESpecularLightingElement extends SVGElement],
      SVGFESpotLightElement: [class SVGFESpotLightElement extends SVGElement],
      SVGFETileElement: [class SVGFETileElement extends SVGElement],
      SVGFETurbulenceElement: [Function],
      SVGFilterElement: [class SVGFilterElement extends SVGElement],
      SVGForeignObjectElement: [class SVGForeignObjectElement extends SVGGraphicsElement],
      SVGGElement: [class SVGGElement extends SVGGraphicsElement],
      SVGImageElement: [class SVGImageElement extends SVGGraphicsElement],
      SVGLineElement: [class SVGLineElement extends SVGGeometryElement],
      SVGLinearGradientElement: [class SVGLinearGradientElement extends SVGGradientElement],
      SVGMarkerElement: [Function],
      SVGMaskElement: [class SVGMaskElement extends SVGElement],
      SVGMetadataElement: [class SVGMetadataElement extends SVGElement],
      SVGMPathElement: [class SVGMPathElement extends SVGElement],
      SVGPathElement: [class SVGPathElement extends SVGGeometryElement],
      SVGPatternElement: [class SVGPatternElement extends SVGElement],
      SVGPolygonElement: [class SVGPolygonElement extends SVGGeometryElement],
      SVGPolylineElement: [class SVGPolylineElement extends SVGGeometryElement],
      SVGRadialGradientElement: [class SVGRadialGradientElement extends SVGGradientElement],
      SVGRectElement: [class SVGRectElement extends SVGGeometryElement],
      SVGScriptElement: [class SVGScriptElement extends SVGGraphicsElement],
      SVGSetElement: [class SVGSetElement extends SVGAnimationElement],
      SVGStopElement: [class SVGStopElement extends SVGElement],
      SVGStyleElement: [class SVGStyleElement extends SVGElement],
      SVGSwitchElement: [class SVGSwitchElement extends SVGGraphicsElement],
      SVGSymbolElement: [class SVGSymbolElement extends SVGGraphicsElement],
      SVGTextElement: [class SVGTextElement extends SVGTextPositioningElement],
      SVGTextPathElement: [Function],
      SVGTitleElement: [class SVGTitleElement extends SVGElement],
      SVGTSpanElement: [class SVGTSpanElement extends SVGTextPositioningElement],
      SVGUseElement: [class SVGUseElement extends SVGGraphicsElement],
      SVGViewElement: [class SVGViewElement extends SVGElement],
      SVGElement: [class SVGElement extends Element],
      SVGAnimationElement: [class SVGAnimationElement extends SVGElement],
      SVGComponentTransferFunctionElement: [Function],
      SVGGeometryElement: [class SVGGeometryElement extends SVGGraphicsElement],
      SVGGradientElement: [Function],
      SVGTextPositioningElement: [class SVGTextPositioningElement extends SVGTextContentElement],
      SVGGraphicsElement: [class SVGGraphicsElement extends SVGElement],
      Event: [Function],
      UIEvent: [Function],
      CustomEvent: [class CustomEvent extends Event],
      CloseEvent: [class CloseEvent extends Event],
      AnimationEvent: [class AnimationEvent extends Event],
      KeyboardEvent: [Function],
      MessageEvent: [class MessageEvent extends Event],
      MouseEvent: [class MouseEvent extends UIEvent],
      PointerEvent: [class PointerEvent extends MouseEvent],
      FocusEvent: [class FocusEvent extends UIEvent],
      WheelEvent: [Function],
      InputEvent: [class InputEvent extends UIEvent],
      ErrorEvent: [class ErrorEvent extends UIEvent],
      StorageEvent: [class StorageEvent extends Event],
      SubmitEvent: [class SubmitEvent extends Event],
      ProgressEvent: [class ProgressEvent extends Event],
      MediaQueryListEvent: [class MediaQueryListEvent extends Event],
      HashChangeEvent: [class HashChangeEvent extends Event],
      ClipboardEvent: [class ClipboardEvent extends Event],
      CookieChangeEvent: [class CookieChangeEvent extends Event],
      TouchEvent: [class TouchEvent extends UIEvent],
      PopStateEvent: [class PopStateEvent extends Event],
      Touch: [class Touch],
      AudioProcessingEvent: [Function],
      BeforeInputEvent: [Function],
      BeforeUnloadEvent: [Function],
      BlobEvent: [Function],
      CompositionEvent: [Function],
      CSSFontFaceLoadEvent: [Function],
      DeviceLightEvent: [Function],
      DeviceMotionEvent: [Function],
      DeviceOrientationEvent: [Function],
      DeviceProximityEvent: [Function],
      DOMTransactionEvent: [Function],
      DragEvent: [Function],
      EditingBeforeInputEvent: [Function],
      FetchEvent: [Function],
      GamepadEvent: [Function],
      IDBVersionChangeEvent: [Function],
      MediaStreamEvent: [Function],
      MutationEvent: [Function],
      OfflineAudioCompletionEvent: [Function],
      OverconstrainedError: [Function],
      PageTransitionEvent: [Function],
      PaymentRequestUpdateEvent: [Function],
      RelatedEvent: [Function],
      RTCDataChannelEvent: [Function],
      RTCIdentityErrorEvent: [Function],
      RTCIdentityEvent: [Function],
      RTCPeerConnectionIceEvent: [Function],
      SensorEvent: [Function],
      SVGEvent: [Function],
      SVGZoomEvent: [Function],
      TimeEvent: [Function],
      TrackEvent: [Function],
      TransitionEvent: [Function],
      UserProximityEvent: [Function],
      WebGLContextEvent: [Function],
      TextEvent: [Function],
      Permissions: [class Permissions],
      History: [class History],
      Navigator: [class Navigator],
      Clipboard: [class Clipboard],
      TimeRanges: [class TimeRanges],
      TextTrackCueList: [class TextTrackCueList extends Array],
      ValidityState: [class ValidityState],
      MutationRecord: [class MutationRecord],
      IntersectionObserver: [class IntersectionObserver],
      IntersectionObserverEntry: [class IntersectionObserverEntry],
      CSSStyleDeclaration: [class CSSStyleDeclaration],
      CSSRule: [Function],
      CSSContainerRule: [class CSSContainerRule extends CSSConditionRule],
      CSSFontFaceRule: [class CSSFontFaceRule extends CSSRule],
      CSSKeyframeRule: [class CSSKeyframeRule extends CSSRule],
      CSSKeyframesRule: [class CSSKeyframesRule extends CSSRule],
      CSSMediaRule: [class CSSMediaRule extends CSSConditionRule],
      CSSStyleRule: [class CSSStyleRule extends CSSGroupingRule],
      CSSSupportsRule: [class CSSSupportsRule extends CSSConditionRule],
      CSSConditionRule: [class CSSConditionRule extends CSSGroupingRule],
      CSSGroupingRule: [class CSSGroupingRule extends CSSRule],
      CSSScopeRule: [class CSSScopeRule extends CSSGroupingRule],
      DOMRect: [class DOMRect extends DOMRectReadOnly],
      DOMRectReadOnly: [class DOMRectReadOnly],
      Plugin: [class Plugin],
      PluginArray: [class PluginArray],
      Location: [class Location],
      CustomElementRegistry: [class CustomElementRegistry],
      ResizeObserver: [class ResizeObserver],
      Blob: [class Blob],
      File: [class File extends Blob],
      Storage: [class Storage],
      CookieStore: [class CookieStore extends EventTarget],
      MimeType: [class MimeType],
      MimeTypeArray: [class MimeTypeArray],
      NodeFilter: [Object],
      ImageBitmap: [class ImageBitmap],
      HTMLCollection: [class HTMLCollection],
      HTMLFormControlCollection: [class HTMLFormControlsCollection extends HTMLCollection],
      HTMLOptionsCollection: [class HTMLOptionsCollection extends HTMLCollection],
      NodeList: [class NodeList],
      RadioNodeList: [class RadioNodeList extends NodeList],
      FileList: [class FileList extends Array],
      Screen: [class Screen extends EventTarget],
      ScreenDetails: [class ScreenDetails extends EventTarget],
      ScreenDetailed: [class ScreenDetailed extends Screen],
      NamedNodeMap: [class NamedNodeMap],
      TreeWalker: [class TreeWalker],
      NodeIterator: [class NodeIterator],
      DataTransfer: [class DataTransfer],
      DataTransferItem: [class DataTransferItem],
      DataTransferItemList: [class DataTransferItemList extends Array],
      XMLSerializer: [class XMLSerializer],
      ClipboardItem: [class ClipboardItem],
      Selection: [class Selection],
      CSSUnitValue: [class CSSUnitValue],
      SVGAngle: [Function],
      SVGAnimatedAngle: [class SVGAnimatedAngle],
      SVGAnimatedBoolean: [class SVGAnimatedBoolean],
      SVGAnimatedEnumeration: [class SVGAnimatedEnumeration],
      SVGAnimatedInteger: [class SVGAnimatedInteger],
      SVGAnimatedLength: [class SVGAnimatedLength],
      SVGAnimatedNumber: [class SVGAnimatedNumber],
      SVGAnimatedNumberList: [class SVGAnimatedNumberList],
      SVGAnimatedPreserveAspectRatio: [class SVGAnimatedPreserveAspectRatio],
      SVGAnimatedRect: [class SVGAnimatedRect],
      SVGAnimatedString: [class SVGAnimatedString],
      SVGAnimatedTransformList: [class SVGAnimatedTransformList],
      SVGLength: [Function],
      SVGLengthList: [class SVGLengthList],
      SVGMatrix: [class SVGMatrix],
      SVGNumber: [class SVGNumber],
      SVGNumberList: [class SVGNumberList],
      SVGPoint: [class SVGPoint],
      SVGPointList: [class SVGPointList],
      SVGPreserveAspectRatio: [Function],
      SVGRect: [class SVGRect],
      SVGStringList: [class SVGStringList],
      SVGTransform: [Function],
      SVGTransformList: [class SVGTransformList],
      SVGAnimatedLengthList: [class SVGAnimatedLengthList],
      SVGUnitTypes: [Function],
      StylePropertyMap: [class StylePropertyMap extends StylePropertyMapReadOnly],
      StylePropertyMapReadOnly: [class StylePropertyMapReadOnly],
      MediaList: [class MediaList],
      CSSKeywordValue: [class CSSKeywordValue],
      CSSStyleValue: [class CSSStyleValue],
      Window: [class GlobalWindow extends Window],
      URLSearchParams: [class URLSearchParams],
      WritableStream: [Function],
      ReadableStream: [Function],
      TransformStream: [Function: Transform],
      PerformanceObserver: [class PerformanceObserver],
      PerformanceEntry: [class PerformanceEntry],
      PerformanceObserverEntryList: [class PerformanceObserverEntryList],
      document: [HTMLDocument],
      customElements: [CustomElementRegistry],
      window: [Circular *3],
      globalThis: [Circular *3],
      performance: [Performance],
      screenLeft: 0,
      screenTop: 0,
      screenX: 0,
      screenY: 0,
      crypto: Crypto {},
      TextEncoder: [class TextEncoder],
      TextDecoder: [class TextDecoder],
      closed: false,
      console: [Object [console]],
      name: '',
      Buffer: [Function],
      Document: [class Document extends Document],
      HTMLDocument: [class HTMLDocument extends HTMLDocument],
      XMLDocument: [class XMLDocument extends XMLDocument],
      DocumentFragment: [class DocumentFragment extends DocumentFragment],
      Text: [class Text extends Text],
      Comment: [class Comment extends Comment],
      Image: [class Image extends Image],
      Audio: [class Audio extends Audio],
      MutationObserver: [class MutationObserver extends MutationObserver],
      MessagePort: [class MessagePort extends MessagePort],
      CSSStyleSheet: [class CSSStyleSheet extends CSSStyleSheet],
      DOMException: [class DOMException extends DOMException],
      Headers: [class Headers extends Headers],
      Request: [class Request extends Request],
      Response: [Function],
      EventTarget: [class EventTarget extends EventTarget],
      XMLHttpRequestUpload: [class XMLHttpRequestUpload extends XMLHttpRequestUpload],
      XMLHttpRequestEventTarget: [class XMLHttpRequestEventTarget extends XMLHttpRequestEventTarget],
      AbortController: [class AbortController extends AbortController],
      AbortSignal: [Function],
      FormData: [class FormData extends FormData],
      PermissionStatus: [class PermissionStatus extends PermissionStatus],
      XMLHttpRequest: [class XMLHttpRequest extends XMLHttpRequest],
      DOMParser: [class DOMParser extends DOMParser],
      Range: [class Range extends Range],
      VTTCue: [class VTTCue extends VTTCue],
      TextTrack: [class TextTrack extends TextTrack],
      TextTrackList: [class TextTrackList extends TextTrackList],
      TextTrackCue: [class TextTrackCue extends TextTrackCue],
      RemotePlayback: [class RemotePlayback extends RemotePlayback],
      FileReader: [class FileReader extends FileReader],
      MediaStream: [class MediaStream extends MediaStream],
      MediaStreamTrack: [class MediaStreamTrack extends MediaStreamTrack],
      CanvasCaptureMediaStreamTrack: [class CanvasCaptureMediaStreamTrack extends CanvasCaptureMediaStreamTrack],
      URL: [class URL extends URL],
      WebSocket: [class WebSocket extends WebSocket],
      ImageData: [class ImageData extends ImageData],
      OffscreenCanvas: [class OffscreenCanvas extends OffscreenCanvas],
      DOMMatrixReadOnly: [class DOMMatrixReadOnly extends DOMMatrixReadOnly],
      DOMMatrix: [class DOMMatrix extends DOMMatrix],
      DOMPointReadOnly: [class DOMPointReadOnly extends DOMPointReadOnly],
      DOMPoint: [class DOMPoint extends DOMPoint],
      Animation: [class Animation extends Animation],
      KeyframeEffect: [class KeyframeEffect extends KeyframeEffect],
      happyDOM: DetachedWindowAPI {},
      Array: [Function: Array],
      ArrayBuffer: [Function: ArrayBuffer],
      Boolean: [Function: Boolean],
      DataView: [Function: DataView],
      Date: [Function: Date],
      Error: [Function],
      EvalError: [Function: EvalError],
      Float32Array: [Function: Float32Array],
      Float64Array: [Function: Float64Array],
      Function: [Function: Function],
      Infinity: Infinity,
      Int16Array: [Function: Int16Array],
      Int32Array: [Function: Int32Array],
      Int8Array: [Function: Int8Array],
      Intl: Object [Intl] {},
      JSON: Object [JSON] {},
      Map: [Function: Map],
      Math: Object [Math] {},
      NaN: NaN,
      Number: [Function: Number],
      Object: [Function: Object],
      Promise: [Function: Promise],
      RangeError: [Function: RangeError],
      ReferenceError: [Function: ReferenceError],
      RegExp: [Function: RegExp],
      Set: [Function: Set],
      String: [Function: String],
      Symbol: [Function: Symbol],
      SyntaxError: [Function: SyntaxError],
      TypeError: [Function: TypeError],
      URIError: [Function: URIError],
      Uint16Array: [Function: Uint16Array],
      Uint32Array: [Function: Uint32Array],
      Uint8Array: [Function: Uint8Array],
      Uint8ClampedArray: [Function: Uint8ClampedArray],
      WeakMap: [Function: WeakMap],
      WeakSet: [Function: WeakSet],
      decodeURI: [Function: decodeURI],
      decodeURIComponent: [Function: decodeURIComponent],
      encodeURI: [Function: encodeURI],
      encodeURIComponent: [Function: encodeURIComponent],
      eval: [Function: eval],
      escape: [Function: escape],
      global: [Object [global]],
      isFinite: [Function: isFinite],
      isNaN: [Function: isNaN],
      parseFloat: [Function: parseFloat],
      parseInt: [Function: parseInt],
      undefined: undefined,
      unescape: [Function: unescape],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(mutationObservers)]: [Array],
      [Symbol(readyStateManager)]: DocumentReadyStateManager {},
      [Symbol(location)]: [Location],
      [Symbol(history)]: History {},
      [Symbol(navigator)]: Navigator {},
      [Symbol(screen)]: [Screen [EventTarget]],
      [Symbol(sessionStorage)]: [Storage],
      [Symbol(localStorage)]: [Storage],
      [Symbol(cookieStore)]: [CookieStore [EventTarget]],
      [Symbol(self)]: [Circular *3],
      [Symbol(top)]: [Circular *3],
      [Symbol(parent)]: [Circular *3],
      [Symbol(window)]: [Circular *3],
      [Symbol(frames)]: [Circular *3],
      [Symbol(internalId)]: 0,
      [Symbol(customElementReactionStack)]: [CustomElementReactionStack],
      [Symbol(modules)]: [Object],
      [Symbol(moduleImportMap)]: null,
      [Symbol(openWebSockets)]: [],
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(querySelectorCache)]: [Map]
    },
    [Symbol(classList)]: null,
    [Symbol(isValue)]: null,
    [Symbol(nodeType)]: 1,
    [Symbol(prefix)]: null,
    [Symbol(shadowRoot)]: null,
    [Symbol(scrollHeight)]: 0,
    [Symbol(scrollWidth)]: 0,
    [Symbol(scrollTop)]: 0,
    [Symbol(scrollLeft)]: 0,
    [Symbol(attributes)]: NamedNodeMap {
      [Symbol(itemsByNamespaceURI)]: [Map],
      [Symbol(itemsByName)]: [Map],
      [Symbol(items)]: [Map],
      [Symbol(ownerElement)]: [Circular *2]
    },
    [Symbol(attributesProxy)]: null,
    [Symbol(children)]: null,
    [Symbol(computedStyle)]: null,
    [Symbol(pointerCaptures)]: Set(0) {},
    [Symbol(propertyEventListeners)]: Map(0) {},
    [Symbol(animations)]: [],
    [Symbol(tagName)]: 'STYLE',
    [Symbol(localName)]: 'style',
    [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
    [Symbol(accessKey)]: '',
    [Symbol(offsetHeight)]: 0,
    [Symbol(offsetWidth)]: 0,
    [Symbol(offsetLeft)]: 0,
    [Symbol(offsetTop)]: 0,
    [Symbol(clientHeight)]: 0,
    [Symbol(clientWidth)]: 0,
    [Symbol(clientLeft)]: 0,
    [Symbol(clientTop)]: 0,
    [Symbol(style)]: null,
    [Symbol(dataset)]: null,
    [Symbol(customElementDefineCallback)]: null,
    [Symbol(sheet)]: null,
    [Symbol(disabled)]: false
  },
  className: 'tiptap',
  editorView: <ref *5> EditorView {
    _root: null,
    focused: false,
    trackWrites: null,
    mounted: false,
    markCursor: null,
    cursorWrapper: null,
    lastSelectedViewDesc: undefined,
    input: InputState {
      shiftKey: false,
      mouseDown: null,
      lastKeyCode: null,
      lastKeyCodeTime: 0,
      lastClick: [Object],
      lastSelectionOrigin: null,
      lastSelectionTime: 0,
      lastIOSEnter: 0,
      lastIOSEnterFallbackTimeout: -1,
      lastFocus: 0,
      lastTouch: 0,
      lastChromeDelete: 0,
      composing: false,
      compositionNode: null,
      composingTimeout: -1,
      compositionNodes: [],
      compositionEndedAt: -200000000,
      compositionID: 1,
      badSafariComposition: false,
      compositionPendingChanges: 0,
      domChangeCount: 0,
      eventHandlers: [Object: null prototype],
      hideSelectionGuard: null
    },
    prevDirectPlugins: [],
    pluginViews: [
      [Object],
      [Object],
      [Object],
      [Object],
      [DropCursorView],
      [Object],
      [Object],
      [Object]
    ],
    requiresGeckoHackNode: false,
    dragging: null,
    _props: {
      attributes: [Object],
      dispatchTransaction: [Function: bound dispatchTransaction],
      transformPastedHTML: [Function (anonymous)],
      state: [EditorState],
      markViews: {},
      nodeViews: [Object]
    },
    state: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    directPlugins: [],
    dispatch: [Function: bound ],
    dom: HTMLDivElement {
      pmViewDesc: [NodeViewDesc],
      editor: [Circular *4],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: [HTMLDivElement],
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [Array],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: [DOMTokenList],
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: [CSSStyleDeclaration],
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    editable: true,
    nodeViews: [Object: null prototype] { taskItem: [Function: nodeview] },
    docView: NodeViewDesc {
      parent: undefined,
      children: [Array],
      dom: [HTMLDivElement],
      contentDOM: [HTMLDivElement],
      dirty: 0,
      node: [Node],
      outerDeco: [Array],
      innerDeco: [DecorationSet],
      nodeDOM: [HTMLDivElement]
    },
    domObserver: DOMObserver {
      view: [Circular *5],
      handleDOMChange: [Function (anonymous)],
      queue: [],
      flushingSoon: -1,
      observer: MutationObserver {},
      currentSelection: [SelectionState],
      onCharData: null,
      suppressingSelectionUpdates: false,
      lastChangedTextNode: null,
      onSelectionChange: [Function: bound onSelectionChange]
    }
  },
  isFocused: false,
  destroyed: false,
  isInitialized: false,
  extensionStorage: {
    link: {},
    paragraph: {},
    listItemBranchingDeleteKeymap: {},
    taskItemBranchingDeleteKeymap: {},
    textStyle: {},
    editable: {},
    clipboardTextSerializer: {},
    commands: {},
    focusEvents: {},
    keymap: {},
    tabindex: {},
    drop: {},
    paste: {},
    delete: {},
    textDirection: {},
    starterKit: {},
    bold: {},
    blockquote: {},
    bulletList: {},
    code: {},
    doc: {},
    dropCursor: {},
    gapCursor: {},
    hardBreak: {},
    undoRedo: {},
    italic: {},
    listItem: {},
    listKeymap: {},
    orderedList: {},
    strike: {},
    text: {},
    underline: {},
    trailingNode: {},
    taskList: {},
    taskItem: {},
    color: {},
    fontSize: {},
    textAlign: {},
    typography: {},
    placeholder: {}
  },
  instanceId: 'ghfenf8',
  hasWarnedStaleDecorationRead: false,
  options: {
    element: HTMLDivElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    content: { type: 'doc', content: [Array] },
    injectCSS: true,
    injectNonce: undefined,
    extensions: [
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    autofocus: false,
    editable: true,
    textDirection: undefined,
    editorProps: { attributes: [Object] },
    parseOptions: {},
    coreExtensionOptions: {},
    enableInputRules: true,
    enablePasteRules: true,
    enableCoreExtensions: true,
    enableContentCheck: false,
    emitContentError: false,
    onBeforeCreate: [Function: onBeforeCreate],
    onCreate: [Function: onCreate],
    onMount: [Function: onMount],
    onUnmount: [Function: onUnmount],
    onUpdate: [Function: onUpdate],
    onSelectionUpdate: [Function: onSelectionUpdate],
    onTransaction: [Function: onTransaction],
    onFocus: [Function: onFocus],
    onBlur: [Function: onBlur],
    onDestroy: [Function: onDestroy],
    onContentError: [Function: onContentError],
    onPaste: [Function: onPaste],
    onDrop: [Function: onDrop],
    onDelete: [Function: onDelete],
    enableExtensionDispatchTransaction: true
  },
  isCapturingTransaction: false,
  capturedTransaction: null,
  utils: {
    getUpdatedPosition: [Function: getUpdatedPosition],
    createMappablePosition: [Function: createMappablePosition]
  },
  extensionManager: ExtensionManager {
    splittableMarks: [ 'textStyle', 'bold', 'code', 'italic', 'strike', 'underline' ],
    nonClearableMarks: [],
    decorationManager: DecorationManager {
      warnedWidgetKeys: Set(0) {},
      warnedOutOfRangeExtensions: Set(0) {},
      handleBeforeTransaction: [Function (anonymous)],
      editor: [Circular *4],
      entries: [],
      plugin: null
    },
    editor: [Circular *4],
    baseExtensions: [
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    extensions: [
      [Mark],      [Node],      [Extension],
      [Extension], [Mark],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Mark],      [Node],
      [Node],      [Mark],      [Node],
      [Extension], [Extension], [Node],
      [Extension], [Mark],      [Node],
      [Extension], [Node],      [Mark],
      [Node],      [Mark],      [Extension],
      [Node],      [Node],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension]
    ],
    schema: Schema {
      linebreakReplacement: [NodeType],
      cached: [Object: null prototype],
      spec: [Object],
      nodes: [Object: null prototype],
      marks: [Object: null prototype],
      nodeFromJSON: [Function (anonymous)],
      markFromJSON: [Function (anonymous)],
      topNodeType: [NodeType]
    }
  },
  commandManager: CommandManager {
    editor: [Circular *4],
    rawCommands: {
      setLink: [Function: setLink],
      toggleLink: [Function: toggleLink],
      unsetLink: [Function: unsetLink],
      setParagraph: [Function: setParagraph],
      toggleTextStyle: [Function: toggleTextStyle],
      removeEmptyTextStyle: [Function: removeEmptyTextStyle],
      blur: [Function: blur],
      clearContent: [Function: clearContent],
      clearNodes: [Function: clearNodes],
      command: [Function: command],
      createParagraphNear: [Function: createParagraphNear$1],
      cut: [Function: cut],
      deleteCurrentNode: [Function: deleteCurrentNode],
      deleteNode: [Function: deleteNode],
      deleteRange: [Function: deleteRange],
      deleteSelection: [Function: deleteSelection],
      enter: [Function: enter],
      exitCode: [Function: exitCode$1],
      extendMarkRange: [Function: extendMarkRange],
      first: [Function: first],
      focus: [Function: focus],
      forEach: [Function: forEach],
      insertContent: [Function: insertContent],
      insertContentAt: [Function: insertContentAt],
      insertDefaultBlock: [Function: insertDefaultBlock],
      joinBackward: [Function: joinBackward$1],
      joinDown: [Function: joinDown$1],
      joinForward: [Function: joinForward$1],
      joinItemBackward: [Function: joinItemBackward],
      joinItemForward: [Function: joinItemForward],
      joinTextblockBackward: [Function: joinTextblockBackward$1],
      joinTextblockForward: [Function: joinTextblockForward$1],
      joinUp: [Function: joinUp$1],
      keyboardShortcut: [Function: keyboardShortcut],
      lift: [Function: lift$1],
      liftEmptyBlock: [Function: liftEmptyBlock$1],
      liftListItem: [Function: liftListItem$1],
      newlineInCode: [Function: newlineInCode$1],
      resetAttributes: [Function: resetAttributes],
      scrollIntoView: [Function: scrollIntoView],
      selectAll: [Function: selectAll],
      selectNodeBackward: [Function: selectNodeBackward$1],
      selectNodeForward: [Function: selectNodeForward$1],
      selectParentNode: [Function: selectParentNode$1],
      selectTextblockEnd: [Function: selectTextblockEnd$1],
      selectTextblockStart: [Function: selectTextblockStart$1],
      setContent: [Function: setContent],
      setMark: [Function: setMark],
      setMeta: [Function: setMeta],
      setNode: [Function: setNode],
      setNodeSelection: [Function: setNodeSelection],
      setTextDirection: [Function: setTextDirection],
      setTextSelection: [Function: setTextSelection],
      sinkListItem: [Function: sinkListItem$1],
      splitBlock: [Function: splitBlock],
      splitListItem: [Function: splitListItem],
      toggleList: [Function: toggleList],
      toggleMark: [Function: toggleMark],
      toggleNode: [Function: toggleNode],
      toggleWrap: [Function: toggleWrap],
      undoInputRule: [Function: undoInputRule],
      unsetAllMarks: [Function: unsetAllMarks],
      unsetMark: [Function: unsetMark],
      unsetTextDirection: [Function: unsetTextDirection],
      updateAttributes: [Function: updateAttributes],
      updateDecorations: [Function: updateDecorations],
      wrapIn: [Function: wrapIn$1],
      wrapInList: [Function: wrapInList$1],
      setBold: [Function: setBold],
      toggleBold: [Function: toggleBold],
      unsetBold: [Function: unsetBold],
      setBlockquote: [Function: setBlockquote],
      toggleBlockquote: [Function: toggleBlockquote],
      unsetBlockquote: [Function: unsetBlockquote],
      toggleBulletList: [Function: toggleBulletList],
      setCode: [Function: setCode],
      toggleCode: [Function: toggleCode],
      unsetCode: [Function: unsetCode],
      setHardBreak: [Function: setHardBreak],
      undo: [Function: undo],
      redo: [Function: redo],
      setItalic: [Function: setItalic],
      toggleItalic: [Function: toggleItalic],
      unsetItalic: [Function: unsetItalic],
      toggleOrderedList: [Function: toggleOrderedList],
      setStrike: [Function: setStrike],
      toggleStrike: [Function: toggleStrike],
      unsetStrike: [Function: unsetStrike],
      setUnderline: [Function: setUnderline],
      toggleUnderline: [Function: toggleUnderline],
      unsetUnderline: [Function: unsetUnderline],
      toggleTaskList: [Function: toggleTaskList],
      setColor: [Function: setColor],
      unsetColor: [Function: unsetColor],
      setFontSize: [Function: setFontSize],
      unsetFontSize: [Function: unsetFontSize],
      setTextAlign: [Function: setTextAlign],
      unsetTextAlign: [Function: unsetTextAlign],
      toggleTextAlign: [Function: toggleTextAlign]
    },
    customState: undefined
  },
  schema: <ref *6> Schema {
    linebreakReplacement: NodeType {
      name: 'hardBreak',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [Array],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: false,
      isText: false
    },
    cached: [Object: null prototype] { wrappings: [Object: null prototype] {} },
    spec: { topNode: 'doc', nodes: [OrderedMap], marks: [OrderedMap] },
    nodes: [Object: null prototype] {
      paragraph: [NodeType],
      blockquote: [NodeType],
      bulletList: [NodeType],
      doc: [NodeType],
      hardBreak: [NodeType],
      listItem: [NodeType],
      orderedList: [NodeType],
      text: [NodeType],
      taskList: [NodeType],
      taskItem: [NodeType]
    },
    marks: [Object: null prototype] {
      link: [MarkType],
      textStyle: [MarkType],
      bold: [MarkType],
      code: [MarkType],
      italic: [MarkType],
      strike: [MarkType],
      underline: [MarkType]
    },
    nodeFromJSON: [Function (anonymous)],
    markFromJSON: [Function (anonymous)],
    topNodeType: NodeType {
      name: 'doc',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: true,
      isText: false
    }
  },
  editorState: EditorState {
    config: Configuration {
      schema: [Schema],
      plugins: [],
      pluginsByKey: [Object: null prototype] {},
      fields: [Array]
    },
    doc: Node {
      type: [NodeType],
      attrs: [Object: null prototype] {},
      marks: [],
      content: [Fragment]
    },
    selection: TextSelection {
      '$anchor': [ResolvedPos],
      '$head': [ResolvedPos],
      ranges: [Array]
    },
    storedMarks: null,
    scrollToSelection: 0
  },
  contentComponent: null,
  appContext: null,
  reactiveState: CustomRefImpl {
    __v_isRef: true,
    _value: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: [Link],
      map: undefined,
      key: undefined,
      sc: 2,
      __v_skip: true,
      subsHead: [Link]
    },
    _get: [Function: get],
    _set: [Function: set]
  },
  reactiveExtensionStorage: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  }
} selected=undefined > 
  at <BubbleMenu editor= <ref *4> Editor {
  callbacks: {
    create: [ [Function: bound onCreate], [Function: onCreate] ],
    destroy: [ [Function: bound onDestroy], [Function: onDestroy] ],
    update: [ [Function: bound onUpdate], [Function: onUpdate] ],
    beforeCreate: [ [Function: onBeforeCreate] ],
    mount: [ [Function: onMount] ],
    unmount: [ [Function: onUnmount] ],
    contentError: [ [Function: onContentError] ],
    selectionUpdate: [ [Function: onSelectionUpdate] ],
    transaction: [ [Function: onTransaction] ],
    focus: [ [Function: onFocus] ],
    blur: [ [Function: onBlur] ],
    drop: [ [Function (anonymous)] ],
    paste: [ [Function (anonymous)] ],
    delete: [ [Function: onDelete] ],
    beforeTransaction: [ [Function (anonymous)], [Function (anonymous)] ]
  },
  css: <ref *2> HTMLStyleElement {
    [Symbol(listeners)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(listenerOptions)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(isConnected)]: true,
    [Symbol(parentNode)]: HTMLHeadElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: [HTMLHtmlElement],
      [Symbol(rootNode)]: [HTMLDocument],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'HEAD',
      [Symbol(localName)]: 'head',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    [Symbol(rootNode)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(styleNode)]: [Circular *2],
    [Symbol(textAreaNode)]: null,
    [Symbol(formNode)]: null,
    [Symbol(selectNode)]: null,
    [Symbol(mutationListeners)]: [],
    [Symbol(nodeArray)]: [ [Text] ],
    [Symbol(elementArray)]: [],
    [Symbol(childNodes)]: null,
    [Symbol(assignedToSlot)]: null,
    [Symbol(cache)]: {
      querySelector: Map(0) {},
      querySelectorAll: Map(0) {},
      matches: Map(0) {},
      elementsByTagName: Map(0) {},
      elementsByTagNameNS: Map(0) {},
      elementByTagName: Map(0) {},
      elementById: Map(0) {},
      computedStyle: null
    },
    [Symbol(affectsCache)]: [],
    [Symbol(ownerDocument)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(window)]: <ref *3> GlobalWindow [EventTarget] {
      Node: [Function],
      Attr: [class Attr extends Node],
      ShadowRoot: [class ShadowRoot extends DocumentFragment],
      ProcessingInstruction: [class ProcessingInstruction extends CharacterData],
      Element: [Function],
      CharacterData: [class CharacterData extends Node],
      DocumentType: [class DocumentType extends Node],
      AnimationTimeline: [class AnimationTimeline],
      DocumentTimeline: [class DocumentTimeline extends AnimationTimeline],
      HTMLAnchorElement: [class HTMLAnchorElement extends HTMLElement],
      HTMLButtonElement: [class HTMLButtonElement extends HTMLElement],
      HTMLOptGroupElement: [class HTMLOptGroupElement extends HTMLElement],
      HTMLOptionElement: [class HTMLOptionElement extends HTMLElement],
      HTMLElement: [Function],
      HTMLUnknownElement: [class HTMLUnknownElement extends HTMLElement],
      HTMLTemplateElement: [class HTMLTemplateElement extends HTMLElement],
      HTMLInputElement: [class HTMLInputElement extends HTMLElement],
      HTMLSelectElement: [class HTMLSelectElement extends HTMLElement],
      HTMLTextAreaElement: [class HTMLTextAreaElement extends HTMLElement],
      HTMLImageElement: [class HTMLImageElement extends HTMLElement],
      HTMLStyleElement: [class HTMLStyleElement extends HTMLElement],
      HTMLLabelElement: [class HTMLLabelElement extends HTMLElement],
      HTMLSlotElement: [class HTMLSlotElement extends HTMLElement],
      HTMLMetaElement: [class HTMLMetaElement extends HTMLElement],
      HTMLMediaElement: [class HTMLMediaElement extends HTMLElement],
      HTMLAudioElement: [class HTMLAudioElement extends HTMLMediaElement],
      HTMLVideoElement: [class HTMLVideoElement extends HTMLMediaElement],
      HTMLBaseElement: [class HTMLBaseElement extends HTMLElement],
      HTMLDialogElement: [class HTMLDialogElement extends HTMLElement],
      HTMLScriptElement: [class HTMLScriptElement extends HTMLElement],
      HTMLLinkElement: [class HTMLLinkElement extends HTMLElement],
      HTMLIFrameElement: [class HTMLIFrameElement extends HTMLElement],
      HTMLFormElement: [class HTMLFormElement extends HTMLElement],
      HTMLUListElement: [class HTMLUListElement extends HTMLElement],
      HTMLTrackElement: [class HTMLTrackElement extends HTMLElement],
      HTMLTableRowElement: [class HTMLTableRowElement extends HTMLElement],
      HTMLTitleElement: [class HTMLTitleElement extends HTMLElement],
      HTMLTimeElement: [class HTMLTimeElement extends HTMLElement],
      HTMLTableSectionElement: [class HTMLTableSectionElement extends HTMLElement],
      HTMLTableCellElement: [class HTMLTableCellElement extends HTMLElement],
      HTMLTableElement: [class HTMLTableElement extends HTMLElement],
      HTMLSpanElement: [class HTMLSpanElement extends HTMLElement],
      HTMLSourceElement: [class HTMLSourceElement extends HTMLElement],
      HTMLQuoteElement: [class HTMLQuoteElement extends HTMLElement],
      HTMLProgressElement: [class HTMLProgressElement extends HTMLElement],
      HTMLPreElement: [class HTMLPreElement extends HTMLElement],
      HTMLPictureElement: [class HTMLPictureElement extends HTMLElement],
      HTMLParamElement: [class HTMLParamElement extends HTMLElement],
      HTMLParagraphElement: [class HTMLParagraphElement extends HTMLElement],
      HTMLOutputElement: [class HTMLOutputElement extends HTMLElement],
      HTMLOListElement: [class HTMLOListElement extends HTMLElement],
      HTMLObjectElement: [class HTMLObjectElement extends HTMLElement],
      HTMLMeterElement: [class HTMLMeterElement extends HTMLElement],
      HTMLMenuElement: [class HTMLMenuElement extends HTMLElement],
      HTMLMapElement: [class HTMLMapElement extends HTMLElement],
      HTMLLIElement: [class HTMLLIElement extends HTMLElement],
      HTMLLegendElement: [class HTMLLegendElement extends HTMLElement],
      HTMLModElement: [class HTMLModElement extends HTMLElement],
      HTMLHtmlElement: [class HTMLHtmlElement extends HTMLElement],
      HTMLHRElement: [class HTMLHRElement extends HTMLElement],
      HTMLHeadElement: [class HTMLHeadElement extends HTMLElement],
      HTMLHeadingElement: [class HTMLHeadingElement extends HTMLElement],
      HTMLFieldSetElement: [class HTMLFieldSetElement extends HTMLElement],
      HTMLEmbedElement: [class HTMLEmbedElement extends HTMLElement],
      HTMLDListElement: [class HTMLDListElement extends HTMLElement],
      HTMLDivElement: [class HTMLDivElement extends HTMLElement],
      HTMLDetailsElement: [class HTMLDetailsElement extends HTMLElement],
      HTMLDataListElement: [class HTMLDataListElement extends HTMLElement],
      HTMLDataElement: [class HTMLDataElement extends HTMLElement],
      HTMLTableColElement: [class HTMLTableColElement extends HTMLElement],
      HTMLTableCaptionElement: [class HTMLTableCaptionElement extends HTMLElement],
      HTMLCanvasElement: [class HTMLCanvasElement extends HTMLElement],
      HTMLBRElement: [class HTMLBRElement extends HTMLElement],
      HTMLBodyElement: [class HTMLBodyElement extends HTMLElement],
      HTMLAreaElement: [class HTMLAreaElement extends HTMLElement],
      SVGSVGElement: [class SVGSVGElement extends SVGGraphicsElement],
      SVGAnimateElement: [class SVGAnimateElement extends SVGAnimationElement],
      SVGAnimateMotionElement: [class SVGAnimateMotionElement extends SVGAnimationElement],
      SVGAnimateTransformElement: [class SVGAnimateTransformElement extends SVGAnimationElement],
      SVGCircleElement: [class SVGCircleElement extends SVGGeometryElement],
      SVGClipPathElement: [class SVGClipPathElement extends SVGElement],
      SVGDefsElement: [class SVGDefsElement extends SVGGraphicsElement],
      SVGDescElement: [class SVGDescElement extends SVGElement],
      SVGEllipseElement: [class SVGEllipseElement extends SVGGeometryElement],
      SVGFEBlendElement: [Function],
      SVGFEColorMatrixElement: [Function],
      SVGFEComponentTransferElement: [class SVGFEComponentTransferElement extends SVGElement],
      SVGFECompositeElement: [Function],
      SVGFEConvolveMatrixElement: [Function],
      SVGFEDiffuseLightingElement: [class SVGFEDiffuseLightingElement extends SVGElement],
      SVGFEDisplacementMapElement: [Function],
      SVGFEDistantLightElement: [class SVGFEDistantLightElement extends SVGElement],
      SVGFEDropShadowElement: [class SVGFEDropShadowElement extends SVGElement],
      SVGFEFloodElement: [class SVGFEFloodElement extends SVGElement],
      SVGFEFuncAElement: [class SVGFEFuncAElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncBElement: [class SVGFEFuncBElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncGElement: [class SVGFEFuncGElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncRElement: [class SVGFEFuncRElement extends SVGComponentTransferFunctionElement],
      SVGFEGaussianBlurElement: [Function],
      SVGFEImageElement: [class SVGFEImageElement extends SVGElement],
      SVGFEMergeElement: [class SVGFEMergeElement extends SVGElement],
      SVGFEMergeNodeElement: [class SVGFEMergeNodeElement extends SVGElement],
      SVGFEMorphologyElement: [Function],
      SVGFEOffsetElement: [class SVGFEOffsetElement extends SVGElement],
      SVGFEPointLightElement: [class SVGFEPointLightElement extends SVGElement],
      SVGFESpecularLightingElement: [class SVGFESpecularLightingElement extends SVGElement],
      SVGFESpotLightElement: [class SVGFESpotLightElement extends SVGElement],
      SVGFETileElement: [class SVGFETileElement extends SVGElement],
      SVGFETurbulenceElement: [Function],
      SVGFilterElement: [class SVGFilterElement extends SVGElement],
      SVGForeignObjectElement: [class SVGForeignObjectElement extends SVGGraphicsElement],
      SVGGElement: [class SVGGElement extends SVGGraphicsElement],
      SVGImageElement: [class SVGImageElement extends SVGGraphicsElement],
      SVGLineElement: [class SVGLineElement extends SVGGeometryElement],
      SVGLinearGradientElement: [class SVGLinearGradientElement extends SVGGradientElement],
      SVGMarkerElement: [Function],
      SVGMaskElement: [class SVGMaskElement extends SVGElement],
      SVGMetadataElement: [class SVGMetadataElement extends SVGElement],
      SVGMPathElement: [class SVGMPathElement extends SVGElement],
      SVGPathElement: [class SVGPathElement extends SVGGeometryElement],
      SVGPatternElement: [class SVGPatternElement extends SVGElement],
      SVGPolygonElement: [class SVGPolygonElement extends SVGGeometryElement],
      SVGPolylineElement: [class SVGPolylineElement extends SVGGeometryElement],
      SVGRadialGradientElement: [class SVGRadialGradientElement extends SVGGradientElement],
      SVGRectElement: [class SVGRectElement extends SVGGeometryElement],
      SVGScriptElement: [class SVGScriptElement extends SVGGraphicsElement],
      SVGSetElement: [class SVGSetElement extends SVGAnimationElement],
      SVGStopElement: [class SVGStopElement extends SVGElement],
      SVGStyleElement: [class SVGStyleElement extends SVGElement],
      SVGSwitchElement: [class SVGSwitchElement extends SVGGraphicsElement],
      SVGSymbolElement: [class SVGSymbolElement extends SVGGraphicsElement],
      SVGTextElement: [class SVGTextElement extends SVGTextPositioningElement],
      SVGTextPathElement: [Function],
      SVGTitleElement: [class SVGTitleElement extends SVGElement],
      SVGTSpanElement: [class SVGTSpanElement extends SVGTextPositioningElement],
      SVGUseElement: [class SVGUseElement extends SVGGraphicsElement],
      SVGViewElement: [class SVGViewElement extends SVGElement],
      SVGElement: [class SVGElement extends Element],
      SVGAnimationElement: [class SVGAnimationElement extends SVGElement],
      SVGComponentTransferFunctionElement: [Function],
      SVGGeometryElement: [class SVGGeometryElement extends SVGGraphicsElement],
      SVGGradientElement: [Function],
      SVGTextPositioningElement: [class SVGTextPositioningElement extends SVGTextContentElement],
      SVGGraphicsElement: [class SVGGraphicsElement extends SVGElement],
      Event: [Function],
      UIEvent: [Function],
      CustomEvent: [class CustomEvent extends Event],
      CloseEvent: [class CloseEvent extends Event],
      AnimationEvent: [class AnimationEvent extends Event],
      KeyboardEvent: [Function],
      MessageEvent: [class MessageEvent extends Event],
      MouseEvent: [class MouseEvent extends UIEvent],
      PointerEvent: [class PointerEvent extends MouseEvent],
      FocusEvent: [class FocusEvent extends UIEvent],
      WheelEvent: [Function],
      InputEvent: [class InputEvent extends UIEvent],
      ErrorEvent: [class ErrorEvent extends UIEvent],
      StorageEvent: [class StorageEvent extends Event],
      SubmitEvent: [class SubmitEvent extends Event],
      ProgressEvent: [class ProgressEvent extends Event],
      MediaQueryListEvent: [class MediaQueryListEvent extends Event],
      HashChangeEvent: [class HashChangeEvent extends Event],
      ClipboardEvent: [class ClipboardEvent extends Event],
      CookieChangeEvent: [class CookieChangeEvent extends Event],
      TouchEvent: [class TouchEvent extends UIEvent],
      PopStateEvent: [class PopStateEvent extends Event],
      Touch: [class Touch],
      AudioProcessingEvent: [Function],
      BeforeInputEvent: [Function],
      BeforeUnloadEvent: [Function],
      BlobEvent: [Function],
      CompositionEvent: [Function],
      CSSFontFaceLoadEvent: [Function],
      DeviceLightEvent: [Function],
      DeviceMotionEvent: [Function],
      DeviceOrientationEvent: [Function],
      DeviceProximityEvent: [Function],
      DOMTransactionEvent: [Function],
      DragEvent: [Function],
      EditingBeforeInputEvent: [Function],
      FetchEvent: [Function],
      GamepadEvent: [Function],
      IDBVersionChangeEvent: [Function],
      MediaStreamEvent: [Function],
      MutationEvent: [Function],
      OfflineAudioCompletionEvent: [Function],
      OverconstrainedError: [Function],
      PageTransitionEvent: [Function],
      PaymentRequestUpdateEvent: [Function],
      RelatedEvent: [Function],
      RTCDataChannelEvent: [Function],
      RTCIdentityErrorEvent: [Function],
      RTCIdentityEvent: [Function],
      RTCPeerConnectionIceEvent: [Function],
      SensorEvent: [Function],
      SVGEvent: [Function],
      SVGZoomEvent: [Function],
      TimeEvent: [Function],
      TrackEvent: [Function],
      TransitionEvent: [Function],
      UserProximityEvent: [Function],
      WebGLContextEvent: [Function],
      TextEvent: [Function],
      Permissions: [class Permissions],
      History: [class History],
      Navigator: [class Navigator],
      Clipboard: [class Clipboard],
      TimeRanges: [class TimeRanges],
      TextTrackCueList: [class TextTrackCueList extends Array],
      ValidityState: [class ValidityState],
      MutationRecord: [class MutationRecord],
      IntersectionObserver: [class IntersectionObserver],
      IntersectionObserverEntry: [class IntersectionObserverEntry],
      CSSStyleDeclaration: [class CSSStyleDeclaration],
      CSSRule: [Function],
      CSSContainerRule: [class CSSContainerRule extends CSSConditionRule],
      CSSFontFaceRule: [class CSSFontFaceRule extends CSSRule],
      CSSKeyframeRule: [class CSSKeyframeRule extends CSSRule],
      CSSKeyframesRule: [class CSSKeyframesRule extends CSSRule],
      CSSMediaRule: [class CSSMediaRule extends CSSConditionRule],
      CSSStyleRule: [class CSSStyleRule extends CSSGroupingRule],
      CSSSupportsRule: [class CSSSupportsRule extends CSSConditionRule],
      CSSConditionRule: [class CSSConditionRule extends CSSGroupingRule],
      CSSGroupingRule: [class CSSGroupingRule extends CSSRule],
      CSSScopeRule: [class CSSScopeRule extends CSSGroupingRule],
      DOMRect: [class DOMRect extends DOMRectReadOnly],
      DOMRectReadOnly: [class DOMRectReadOnly],
      Plugin: [class Plugin],
      PluginArray: [class PluginArray],
      Location: [class Location],
      CustomElementRegistry: [class CustomElementRegistry],
      ResizeObserver: [class ResizeObserver],
      Blob: [class Blob],
      File: [class File extends Blob],
      Storage: [class Storage],
      CookieStore: [class CookieStore extends EventTarget],
      MimeType: [class MimeType],
      MimeTypeArray: [class MimeTypeArray],
      NodeFilter: [Object],
      ImageBitmap: [class ImageBitmap],
      HTMLCollection: [class HTMLCollection],
      HTMLFormControlCollection: [class HTMLFormControlsCollection extends HTMLCollection],
      HTMLOptionsCollection: [class HTMLOptionsCollection extends HTMLCollection],
      NodeList: [class NodeList],
      RadioNodeList: [class RadioNodeList extends NodeList],
      FileList: [class FileList extends Array],
      Screen: [class Screen extends EventTarget],
      ScreenDetails: [class ScreenDetails extends EventTarget],
      ScreenDetailed: [class ScreenDetailed extends Screen],
      NamedNodeMap: [class NamedNodeMap],
      TreeWalker: [class TreeWalker],
      NodeIterator: [class NodeIterator],
      DataTransfer: [class DataTransfer],
      DataTransferItem: [class DataTransferItem],
      DataTransferItemList: [class DataTransferItemList extends Array],
      XMLSerializer: [class XMLSerializer],
      ClipboardItem: [class ClipboardItem],
      Selection: [class Selection],
      CSSUnitValue: [class CSSUnitValue],
      SVGAngle: [Function],
      SVGAnimatedAngle: [class SVGAnimatedAngle],
      SVGAnimatedBoolean: [class SVGAnimatedBoolean],
      SVGAnimatedEnumeration: [class SVGAnimatedEnumeration],
      SVGAnimatedInteger: [class SVGAnimatedInteger],
      SVGAnimatedLength: [class SVGAnimatedLength],
      SVGAnimatedNumber: [class SVGAnimatedNumber],
      SVGAnimatedNumberList: [class SVGAnimatedNumberList],
      SVGAnimatedPreserveAspectRatio: [class SVGAnimatedPreserveAspectRatio],
      SVGAnimatedRect: [class SVGAnimatedRect],
      SVGAnimatedString: [class SVGAnimatedString],
      SVGAnimatedTransformList: [class SVGAnimatedTransformList],
      SVGLength: [Function],
      SVGLengthList: [class SVGLengthList],
      SVGMatrix: [class SVGMatrix],
      SVGNumber: [class SVGNumber],
      SVGNumberList: [class SVGNumberList],
      SVGPoint: [class SVGPoint],
      SVGPointList: [class SVGPointList],
      SVGPreserveAspectRatio: [Function],
      SVGRect: [class SVGRect],
      SVGStringList: [class SVGStringList],
      SVGTransform: [Function],
      SVGTransformList: [class SVGTransformList],
      SVGAnimatedLengthList: [class SVGAnimatedLengthList],
      SVGUnitTypes: [Function],
      StylePropertyMap: [class StylePropertyMap extends StylePropertyMapReadOnly],
      StylePropertyMapReadOnly: [class StylePropertyMapReadOnly],
      MediaList: [class MediaList],
      CSSKeywordValue: [class CSSKeywordValue],
      CSSStyleValue: [class CSSStyleValue],
      Window: [class GlobalWindow extends Window],
      URLSearchParams: [class URLSearchParams],
      WritableStream: [Function],
      ReadableStream: [Function],
      TransformStream: [Function: Transform],
      PerformanceObserver: [class PerformanceObserver],
      PerformanceEntry: [class PerformanceEntry],
      PerformanceObserverEntryList: [class PerformanceObserverEntryList],
      document: [HTMLDocument],
      customElements: [CustomElementRegistry],
      window: [Circular *3],
      globalThis: [Circular *3],
      performance: [Performance],
      screenLeft: 0,
      screenTop: 0,
      screenX: 0,
      screenY: 0,
      crypto: Crypto {},
      TextEncoder: [class TextEncoder],
      TextDecoder: [class TextDecoder],
      closed: false,
      console: [Object [console]],
      name: '',
      Buffer: [Function],
      Document: [class Document extends Document],
      HTMLDocument: [class HTMLDocument extends HTMLDocument],
      XMLDocument: [class XMLDocument extends XMLDocument],
      DocumentFragment: [class DocumentFragment extends DocumentFragment],
      Text: [class Text extends Text],
      Comment: [class Comment extends Comment],
      Image: [class Image extends Image],
      Audio: [class Audio extends Audio],
      MutationObserver: [class MutationObserver extends MutationObserver],
      MessagePort: [class MessagePort extends MessagePort],
      CSSStyleSheet: [class CSSStyleSheet extends CSSStyleSheet],
      DOMException: [class DOMException extends DOMException],
      Headers: [class Headers extends Headers],
      Request: [class Request extends Request],
      Response: [Function],
      EventTarget: [class EventTarget extends EventTarget],
      XMLHttpRequestUpload: [class XMLHttpRequestUpload extends XMLHttpRequestUpload],
      XMLHttpRequestEventTarget: [class XMLHttpRequestEventTarget extends XMLHttpRequestEventTarget],
      AbortController: [class AbortController extends AbortController],
      AbortSignal: [Function],
      FormData: [class FormData extends FormData],
      PermissionStatus: [class PermissionStatus extends PermissionStatus],
      XMLHttpRequest: [class XMLHttpRequest extends XMLHttpRequest],
      DOMParser: [class DOMParser extends DOMParser],
      Range: [class Range extends Range],
      VTTCue: [class VTTCue extends VTTCue],
      TextTrack: [class TextTrack extends TextTrack],
      TextTrackList: [class TextTrackList extends TextTrackList],
      TextTrackCue: [class TextTrackCue extends TextTrackCue],
      RemotePlayback: [class RemotePlayback extends RemotePlayback],
      FileReader: [class FileReader extends FileReader],
      MediaStream: [class MediaStream extends MediaStream],
      MediaStreamTrack: [class MediaStreamTrack extends MediaStreamTrack],
      CanvasCaptureMediaStreamTrack: [class CanvasCaptureMediaStreamTrack extends CanvasCaptureMediaStreamTrack],
      URL: [class URL extends URL],
      WebSocket: [class WebSocket extends WebSocket],
      ImageData: [class ImageData extends ImageData],
      OffscreenCanvas: [class OffscreenCanvas extends OffscreenCanvas],
      DOMMatrixReadOnly: [class DOMMatrixReadOnly extends DOMMatrixReadOnly],
      DOMMatrix: [class DOMMatrix extends DOMMatrix],
      DOMPointReadOnly: [class DOMPointReadOnly extends DOMPointReadOnly],
      DOMPoint: [class DOMPoint extends DOMPoint],
      Animation: [class Animation extends Animation],
      KeyframeEffect: [class KeyframeEffect extends KeyframeEffect],
      happyDOM: DetachedWindowAPI {},
      Array: [Function: Array],
      ArrayBuffer: [Function: ArrayBuffer],
      Boolean: [Function: Boolean],
      DataView: [Function: DataView],
      Date: [Function: Date],
      Error: [Function],
      EvalError: [Function: EvalError],
      Float32Array: [Function: Float32Array],
      Float64Array: [Function: Float64Array],
      Function: [Function: Function],
      Infinity: Infinity,
      Int16Array: [Function: Int16Array],
      Int32Array: [Function: Int32Array],
      Int8Array: [Function: Int8Array],
      Intl: Object [Intl] {},
      JSON: Object [JSON] {},
      Map: [Function: Map],
      Math: Object [Math] {},
      NaN: NaN,
      Number: [Function: Number],
      Object: [Function: Object],
      Promise: [Function: Promise],
      RangeError: [Function: RangeError],
      ReferenceError: [Function: ReferenceError],
      RegExp: [Function: RegExp],
      Set: [Function: Set],
      String: [Function: String],
      Symbol: [Function: Symbol],
      SyntaxError: [Function: SyntaxError],
      TypeError: [Function: TypeError],
      URIError: [Function: URIError],
      Uint16Array: [Function: Uint16Array],
      Uint32Array: [Function: Uint32Array],
      Uint8Array: [Function: Uint8Array],
      Uint8ClampedArray: [Function: Uint8ClampedArray],
      WeakMap: [Function: WeakMap],
      WeakSet: [Function: WeakSet],
      decodeURI: [Function: decodeURI],
      decodeURIComponent: [Function: decodeURIComponent],
      encodeURI: [Function: encodeURI],
      encodeURIComponent: [Function: encodeURIComponent],
      eval: [Function: eval],
      escape: [Function: escape],
      global: [Object [global]],
      isFinite: [Function: isFinite],
      isNaN: [Function: isNaN],
      parseFloat: [Function: parseFloat],
      parseInt: [Function: parseInt],
      undefined: undefined,
      unescape: [Function: unescape],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(mutationObservers)]: [Array],
      [Symbol(readyStateManager)]: DocumentReadyStateManager {},
      [Symbol(location)]: [Location],
      [Symbol(history)]: History {},
      [Symbol(navigator)]: Navigator {},
      [Symbol(screen)]: [Screen [EventTarget]],
      [Symbol(sessionStorage)]: [Storage],
      [Symbol(localStorage)]: [Storage],
      [Symbol(cookieStore)]: [CookieStore [EventTarget]],
      [Symbol(self)]: [Circular *3],
      [Symbol(top)]: [Circular *3],
      [Symbol(parent)]: [Circular *3],
      [Symbol(window)]: [Circular *3],
      [Symbol(frames)]: [Circular *3],
      [Symbol(internalId)]: 0,
      [Symbol(customElementReactionStack)]: [CustomElementReactionStack],
      [Symbol(modules)]: [Object],
      [Symbol(moduleImportMap)]: null,
      [Symbol(openWebSockets)]: [],
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(querySelectorCache)]: [Map]
    },
    [Symbol(classList)]: null,
    [Symbol(isValue)]: null,
    [Symbol(nodeType)]: 1,
    [Symbol(prefix)]: null,
    [Symbol(shadowRoot)]: null,
    [Symbol(scrollHeight)]: 0,
    [Symbol(scrollWidth)]: 0,
    [Symbol(scrollTop)]: 0,
    [Symbol(scrollLeft)]: 0,
    [Symbol(attributes)]: NamedNodeMap {
      [Symbol(itemsByNamespaceURI)]: [Map],
      [Symbol(itemsByName)]: [Map],
      [Symbol(items)]: [Map],
      [Symbol(ownerElement)]: [Circular *2]
    },
    [Symbol(attributesProxy)]: null,
    [Symbol(children)]: null,
    [Symbol(computedStyle)]: null,
    [Symbol(pointerCaptures)]: Set(0) {},
    [Symbol(propertyEventListeners)]: Map(0) {},
    [Symbol(animations)]: [],
    [Symbol(tagName)]: 'STYLE',
    [Symbol(localName)]: 'style',
    [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
    [Symbol(accessKey)]: '',
    [Symbol(offsetHeight)]: 0,
    [Symbol(offsetWidth)]: 0,
    [Symbol(offsetLeft)]: 0,
    [Symbol(offsetTop)]: 0,
    [Symbol(clientHeight)]: 0,
    [Symbol(clientWidth)]: 0,
    [Symbol(clientLeft)]: 0,
    [Symbol(clientTop)]: 0,
    [Symbol(style)]: null,
    [Symbol(dataset)]: null,
    [Symbol(customElementDefineCallback)]: null,
    [Symbol(sheet)]: null,
    [Symbol(disabled)]: false
  },
  className: 'tiptap',
  editorView: <ref *5> EditorView {
    _root: null,
    focused: false,
    trackWrites: null,
    mounted: false,
    markCursor: null,
    cursorWrapper: null,
    lastSelectedViewDesc: undefined,
    input: InputState {
      shiftKey: false,
      mouseDown: null,
      lastKeyCode: null,
      lastKeyCodeTime: 0,
      lastClick: [Object],
      lastSelectionOrigin: null,
      lastSelectionTime: 0,
      lastIOSEnter: 0,
      lastIOSEnterFallbackTimeout: -1,
      lastFocus: 0,
      lastTouch: 0,
      lastChromeDelete: 0,
      composing: false,
      compositionNode: null,
      composingTimeout: -1,
      compositionNodes: [],
      compositionEndedAt: -200000000,
      compositionID: 1,
      badSafariComposition: false,
      compositionPendingChanges: 0,
      domChangeCount: 0,
      eventHandlers: [Object: null prototype],
      hideSelectionGuard: null
    },
    prevDirectPlugins: [],
    pluginViews: [
      [Object],
      [Object],
      [Object],
      [Object],
      [DropCursorView],
      [Object],
      [Object],
      [Object]
    ],
    requiresGeckoHackNode: false,
    dragging: null,
    _props: {
      attributes: [Object],
      dispatchTransaction: [Function: bound dispatchTransaction],
      transformPastedHTML: [Function (anonymous)],
      state: [EditorState],
      markViews: {},
      nodeViews: [Object]
    },
    state: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    directPlugins: [],
    dispatch: [Function: bound ],
    dom: HTMLDivElement {
      pmViewDesc: [NodeViewDesc],
      editor: [Circular *4],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: [HTMLDivElement],
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [Array],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: [DOMTokenList],
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: [CSSStyleDeclaration],
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    editable: true,
    nodeViews: [Object: null prototype] { taskItem: [Function: nodeview] },
    docView: NodeViewDesc {
      parent: undefined,
      children: [Array],
      dom: [HTMLDivElement],
      contentDOM: [HTMLDivElement],
      dirty: 0,
      node: [Node],
      outerDeco: [Array],
      innerDeco: [DecorationSet],
      nodeDOM: [HTMLDivElement]
    },
    domObserver: DOMObserver {
      view: [Circular *5],
      handleDOMChange: [Function (anonymous)],
      queue: [],
      flushingSoon: -1,
      observer: MutationObserver {},
      currentSelection: [SelectionState],
      onCharData: null,
      suppressingSelectionUpdates: false,
      lastChangedTextNode: null,
      onSelectionChange: [Function: bound onSelectionChange]
    }
  },
  isFocused: false,
  destroyed: false,
  isInitialized: false,
  extensionStorage: {
    link: {},
    paragraph: {},
    listItemBranchingDeleteKeymap: {},
    taskItemBranchingDeleteKeymap: {},
    textStyle: {},
    editable: {},
    clipboardTextSerializer: {},
    commands: {},
    focusEvents: {},
    keymap: {},
    tabindex: {},
    drop: {},
    paste: {},
    delete: {},
    textDirection: {},
    starterKit: {},
    bold: {},
    blockquote: {},
    bulletList: {},
    code: {},
    doc: {},
    dropCursor: {},
    gapCursor: {},
    hardBreak: {},
    undoRedo: {},
    italic: {},
    listItem: {},
    listKeymap: {},
    orderedList: {},
    strike: {},
    text: {},
    underline: {},
    trailingNode: {},
    taskList: {},
    taskItem: {},
    color: {},
    fontSize: {},
    textAlign: {},
    typography: {},
    placeholder: {}
  },
  instanceId: 'ghfenf8',
  hasWarnedStaleDecorationRead: false,
  options: {
    element: HTMLDivElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    content: { type: 'doc', content: [Array] },
    injectCSS: true,
    injectNonce: undefined,
    extensions: [
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    autofocus: false,
    editable: true,
    textDirection: undefined,
    editorProps: { attributes: [Object] },
    parseOptions: {},
    coreExtensionOptions: {},
    enableInputRules: true,
    enablePasteRules: true,
    enableCoreExtensions: true,
    enableContentCheck: false,
    emitContentError: false,
    onBeforeCreate: [Function: onBeforeCreate],
    onCreate: [Function: onCreate],
    onMount: [Function: onMount],
    onUnmount: [Function: onUnmount],
    onUpdate: [Function: onUpdate],
    onSelectionUpdate: [Function: onSelectionUpdate],
    onTransaction: [Function: onTransaction],
    onFocus: [Function: onFocus],
    onBlur: [Function: onBlur],
    onDestroy: [Function: onDestroy],
    onContentError: [Function: onContentError],
    onPaste: [Function: onPaste],
    onDrop: [Function: onDrop],
    onDelete: [Function: onDelete],
    enableExtensionDispatchTransaction: true
  },
  isCapturingTransaction: false,
  capturedTransaction: null,
  utils: {
    getUpdatedPosition: [Function: getUpdatedPosition],
    createMappablePosition: [Function: createMappablePosition]
  },
  extensionManager: ExtensionManager {
    splittableMarks: [ 'textStyle', 'bold', 'code', 'italic', 'strike', 'underline' ],
    nonClearableMarks: [],
    decorationManager: DecorationManager {
      warnedWidgetKeys: Set(0) {},
      warnedOutOfRangeExtensions: Set(0) {},
      handleBeforeTransaction: [Function (anonymous)],
      editor: [Circular *4],
      entries: [],
      plugin: null
    },
    editor: [Circular *4],
    baseExtensions: [
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    extensions: [
      [Mark],      [Node],      [Extension],
      [Extension], [Mark],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Mark],      [Node],
      [Node],      [Mark],      [Node],
      [Extension], [Extension], [Node],
      [Extension], [Mark],      [Node],
      [Extension], [Node],      [Mark],
      [Node],      [Mark],      [Extension],
      [Node],      [Node],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension]
    ],
    schema: Schema {
      linebreakReplacement: [NodeType],
      cached: [Object: null prototype],
      spec: [Object],
      nodes: [Object: null prototype],
      marks: [Object: null prototype],
      nodeFromJSON: [Function (anonymous)],
      markFromJSON: [Function (anonymous)],
      topNodeType: [NodeType]
    }
  },
  commandManager: CommandManager {
    editor: [Circular *4],
    rawCommands: {
      setLink: [Function: setLink],
      toggleLink: [Function: toggleLink],
      unsetLink: [Function: unsetLink],
      setParagraph: [Function: setParagraph],
      toggleTextStyle: [Function: toggleTextStyle],
      removeEmptyTextStyle: [Function: removeEmptyTextStyle],
      blur: [Function: blur],
      clearContent: [Function: clearContent],
      clearNodes: [Function: clearNodes],
      command: [Function: command],
      createParagraphNear: [Function: createParagraphNear$1],
      cut: [Function: cut],
      deleteCurrentNode: [Function: deleteCurrentNode],
      deleteNode: [Function: deleteNode],
      deleteRange: [Function: deleteRange],
      deleteSelection: [Function: deleteSelection],
      enter: [Function: enter],
      exitCode: [Function: exitCode$1],
      extendMarkRange: [Function: extendMarkRange],
      first: [Function: first],
      focus: [Function: focus],
      forEach: [Function: forEach],
      insertContent: [Function: insertContent],
      insertContentAt: [Function: insertContentAt],
      insertDefaultBlock: [Function: insertDefaultBlock],
      joinBackward: [Function: joinBackward$1],
      joinDown: [Function: joinDown$1],
      joinForward: [Function: joinForward$1],
      joinItemBackward: [Function: joinItemBackward],
      joinItemForward: [Function: joinItemForward],
      joinTextblockBackward: [Function: joinTextblockBackward$1],
      joinTextblockForward: [Function: joinTextblockForward$1],
      joinUp: [Function: joinUp$1],
      keyboardShortcut: [Function: keyboardShortcut],
      lift: [Function: lift$1],
      liftEmptyBlock: [Function: liftEmptyBlock$1],
      liftListItem: [Function: liftListItem$1],
      newlineInCode: [Function: newlineInCode$1],
      resetAttributes: [Function: resetAttributes],
      scrollIntoView: [Function: scrollIntoView],
      selectAll: [Function: selectAll],
      selectNodeBackward: [Function: selectNodeBackward$1],
      selectNodeForward: [Function: selectNodeForward$1],
      selectParentNode: [Function: selectParentNode$1],
      selectTextblockEnd: [Function: selectTextblockEnd$1],
      selectTextblockStart: [Function: selectTextblockStart$1],
      setContent: [Function: setContent],
      setMark: [Function: setMark],
      setMeta: [Function: setMeta],
      setNode: [Function: setNode],
      setNodeSelection: [Function: setNodeSelection],
      setTextDirection: [Function: setTextDirection],
      setTextSelection: [Function: setTextSelection],
      sinkListItem: [Function: sinkListItem$1],
      splitBlock: [Function: splitBlock],
      splitListItem: [Function: splitListItem],
      toggleList: [Function: toggleList],
      toggleMark: [Function: toggleMark],
      toggleNode: [Function: toggleNode],
      toggleWrap: [Function: toggleWrap],
      undoInputRule: [Function: undoInputRule],
      unsetAllMarks: [Function: unsetAllMarks],
      unsetMark: [Function: unsetMark],
      unsetTextDirection: [Function: unsetTextDirection],
      updateAttributes: [Function: updateAttributes],
      updateDecorations: [Function: updateDecorations],
      wrapIn: [Function: wrapIn$1],
      wrapInList: [Function: wrapInList$1],
      setBold: [Function: setBold],
      toggleBold: [Function: toggleBold],
      unsetBold: [Function: unsetBold],
      setBlockquote: [Function: setBlockquote],
      toggleBlockquote: [Function: toggleBlockquote],
      unsetBlockquote: [Function: unsetBlockquote],
      toggleBulletList: [Function: toggleBulletList],
      setCode: [Function: setCode],
      toggleCode: [Function: toggleCode],
      unsetCode: [Function: unsetCode],
      setHardBreak: [Function: setHardBreak],
      undo: [Function: undo],
      redo: [Function: redo],
      setItalic: [Function: setItalic],
      toggleItalic: [Function: toggleItalic],
      unsetItalic: [Function: unsetItalic],
      toggleOrderedList: [Function: toggleOrderedList],
      setStrike: [Function: setStrike],
      toggleStrike: [Function: toggleStrike],
      unsetStrike: [Function: unsetStrike],
      setUnderline: [Function: setUnderline],
      toggleUnderline: [Function: toggleUnderline],
      unsetUnderline: [Function: unsetUnderline],
      toggleTaskList: [Function: toggleTaskList],
      setColor: [Function: setColor],
      unsetColor: [Function: unsetColor],
      setFontSize: [Function: setFontSize],
      unsetFontSize: [Function: unsetFontSize],
      setTextAlign: [Function: setTextAlign],
      unsetTextAlign: [Function: unsetTextAlign],
      toggleTextAlign: [Function: toggleTextAlign]
    },
    customState: undefined
  },
  schema: <ref *6> Schema {
    linebreakReplacement: NodeType {
      name: 'hardBreak',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [Array],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: false,
      isText: false
    },
    cached: [Object: null prototype] { wrappings: [Object: null prototype] {} },
    spec: { topNode: 'doc', nodes: [OrderedMap], marks: [OrderedMap] },
    nodes: [Object: null prototype] {
      paragraph: [NodeType],
      blockquote: [NodeType],
      bulletList: [NodeType],
      doc: [NodeType],
      hardBreak: [NodeType],
      listItem: [NodeType],
      orderedList: [NodeType],
      text: [NodeType],
      taskList: [NodeType],
      taskItem: [NodeType]
    },
    marks: [Object: null prototype] {
      link: [MarkType],
      textStyle: [MarkType],
      bold: [MarkType],
      code: [MarkType],
      italic: [MarkType],
      strike: [MarkType],
      underline: [MarkType]
    },
    nodeFromJSON: [Function (anonymous)],
    markFromJSON: [Function (anonymous)],
    topNodeType: NodeType {
      name: 'doc',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: true,
      isText: false
    }
  },
  editorState: EditorState {
    config: Configuration {
      schema: [Schema],
      plugins: [],
      pluginsByKey: [Object: null prototype] {},
      fields: [Array]
    },
    doc: Node {
      type: [NodeType],
      attrs: [Object: null prototype] {},
      marks: [],
      content: [Fragment]
    },
    selection: TextSelection {
      '$anchor': [ResolvedPos],
      '$head': [ResolvedPos],
      ranges: [Array]
    },
    storedMarks: null,
    scrollToSelection: 0
  },
  contentComponent: null,
  appContext: null,
  reactiveState: CustomRefImpl {
    __v_isRef: true,
    _value: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: [Link],
      map: undefined,
      key: undefined,
      sc: 2,
      __v_skip: true,
      subsHead: [Link]
    },
    _get: [Function: get],
    _set: [Function: set]
  },
  reactiveExtensionStorage: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  }
} append-to=fn<appendToCanvasOverlay> should-show=fn<shouldShow>  ... > 
  at <SpaceCanvasTextSelectionBar key=0 editor= <ref *4> Editor {
  callbacks: {
    create: [ [Function: bound onCreate], [Function: onCreate] ],
    destroy: [ [Function: bound onDestroy], [Function: onDestroy] ],
    update: [ [Function: bound onUpdate], [Function: onUpdate] ],
    beforeCreate: [ [Function: onBeforeCreate] ],
    mount: [ [Function: onMount] ],
    unmount: [ [Function: onUnmount] ],
    contentError: [ [Function: onContentError] ],
    selectionUpdate: [ [Function: onSelectionUpdate] ],
    transaction: [ [Function: onTransaction] ],
    focus: [ [Function: onFocus] ],
    blur: [ [Function: onBlur] ],
    drop: [ [Function (anonymous)] ],
    paste: [ [Function (anonymous)] ],
    delete: [ [Function: onDelete] ],
    beforeTransaction: [ [Function (anonymous)], [Function (anonymous)] ]
  },
  css: <ref *2> HTMLStyleElement {
    [Symbol(listeners)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(listenerOptions)]: { bubbling: Map(0) {}, capturing: Map(0) {} },
    [Symbol(isConnected)]: true,
    [Symbol(parentNode)]: HTMLHeadElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: [HTMLHtmlElement],
      [Symbol(rootNode)]: [HTMLDocument],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'HEAD',
      [Symbol(localName)]: 'head',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    [Symbol(rootNode)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(styleNode)]: [Circular *2],
    [Symbol(textAreaNode)]: null,
    [Symbol(formNode)]: null,
    [Symbol(selectNode)]: null,
    [Symbol(mutationListeners)]: [],
    [Symbol(nodeArray)]: [ [Text] ],
    [Symbol(elementArray)]: [],
    [Symbol(childNodes)]: null,
    [Symbol(assignedToSlot)]: null,
    [Symbol(cache)]: {
      querySelector: Map(0) {},
      querySelectorAll: Map(0) {},
      matches: Map(0) {},
      elementsByTagName: Map(0) {},
      elementsByTagNameNS: Map(0) {},
      elementByTagName: Map(0) {},
      elementById: Map(0) {},
      computedStyle: null
    },
    [Symbol(affectsCache)]: [],
    [Symbol(ownerDocument)]: <ref *1> HTMLDocument {
      defaultView: [Getter],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: true,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: [Circular *1],
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [Array],
      [Symbol(ownerDocument)]: null,
      [Symbol(children)]: null,
      [Symbol(activeElement)]: null,
      [Symbol(nextActiveElement)]: null,
      [Symbol(currentScript)]: null,
      [Symbol(isFirstWrite)]: true,
      [Symbol(isFirstWriteAfterOpen)]: false,
      [Symbol(nodeType)]: 9,
      [Symbol(adoptedStyleSheets)]: [],
      [Symbol(implementation)]: DOMImplementation {},
      [Symbol(readyState)]: 'complete',
      [Symbol(referrer)]: '',
      [Symbol(defaultView)]: [GlobalWindow [EventTarget]],
      [Symbol(forms)]: null,
      [Symbol(links)]: null,
      [Symbol(affectsComputedStyleCache)]: [],
      [Symbol(elementIdMap)]: Map(0) {},
      [Symbol(contentType)]: 'text/html',
      [Symbol(xmlProcessingInstruction)]: null,
      [Symbol(preloads)]: Map(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(selection)]: null,
      [Symbol(timeline)]: DocumentTimeline {}
    },
    [Symbol(window)]: <ref *3> GlobalWindow [EventTarget] {
      Node: [Function],
      Attr: [class Attr extends Node],
      ShadowRoot: [class ShadowRoot extends DocumentFragment],
      ProcessingInstruction: [class ProcessingInstruction extends CharacterData],
      Element: [Function],
      CharacterData: [class CharacterData extends Node],
      DocumentType: [class DocumentType extends Node],
      AnimationTimeline: [class AnimationTimeline],
      DocumentTimeline: [class DocumentTimeline extends AnimationTimeline],
      HTMLAnchorElement: [class HTMLAnchorElement extends HTMLElement],
      HTMLButtonElement: [class HTMLButtonElement extends HTMLElement],
      HTMLOptGroupElement: [class HTMLOptGroupElement extends HTMLElement],
      HTMLOptionElement: [class HTMLOptionElement extends HTMLElement],
      HTMLElement: [Function],
      HTMLUnknownElement: [class HTMLUnknownElement extends HTMLElement],
      HTMLTemplateElement: [class HTMLTemplateElement extends HTMLElement],
      HTMLInputElement: [class HTMLInputElement extends HTMLElement],
      HTMLSelectElement: [class HTMLSelectElement extends HTMLElement],
      HTMLTextAreaElement: [class HTMLTextAreaElement extends HTMLElement],
      HTMLImageElement: [class HTMLImageElement extends HTMLElement],
      HTMLStyleElement: [class HTMLStyleElement extends HTMLElement],
      HTMLLabelElement: [class HTMLLabelElement extends HTMLElement],
      HTMLSlotElement: [class HTMLSlotElement extends HTMLElement],
      HTMLMetaElement: [class HTMLMetaElement extends HTMLElement],
      HTMLMediaElement: [class HTMLMediaElement extends HTMLElement],
      HTMLAudioElement: [class HTMLAudioElement extends HTMLMediaElement],
      HTMLVideoElement: [class HTMLVideoElement extends HTMLMediaElement],
      HTMLBaseElement: [class HTMLBaseElement extends HTMLElement],
      HTMLDialogElement: [class HTMLDialogElement extends HTMLElement],
      HTMLScriptElement: [class HTMLScriptElement extends HTMLElement],
      HTMLLinkElement: [class HTMLLinkElement extends HTMLElement],
      HTMLIFrameElement: [class HTMLIFrameElement extends HTMLElement],
      HTMLFormElement: [class HTMLFormElement extends HTMLElement],
      HTMLUListElement: [class HTMLUListElement extends HTMLElement],
      HTMLTrackElement: [class HTMLTrackElement extends HTMLElement],
      HTMLTableRowElement: [class HTMLTableRowElement extends HTMLElement],
      HTMLTitleElement: [class HTMLTitleElement extends HTMLElement],
      HTMLTimeElement: [class HTMLTimeElement extends HTMLElement],
      HTMLTableSectionElement: [class HTMLTableSectionElement extends HTMLElement],
      HTMLTableCellElement: [class HTMLTableCellElement extends HTMLElement],
      HTMLTableElement: [class HTMLTableElement extends HTMLElement],
      HTMLSpanElement: [class HTMLSpanElement extends HTMLElement],
      HTMLSourceElement: [class HTMLSourceElement extends HTMLElement],
      HTMLQuoteElement: [class HTMLQuoteElement extends HTMLElement],
      HTMLProgressElement: [class HTMLProgressElement extends HTMLElement],
      HTMLPreElement: [class HTMLPreElement extends HTMLElement],
      HTMLPictureElement: [class HTMLPictureElement extends HTMLElement],
      HTMLParamElement: [class HTMLParamElement extends HTMLElement],
      HTMLParagraphElement: [class HTMLParagraphElement extends HTMLElement],
      HTMLOutputElement: [class HTMLOutputElement extends HTMLElement],
      HTMLOListElement: [class HTMLOListElement extends HTMLElement],
      HTMLObjectElement: [class HTMLObjectElement extends HTMLElement],
      HTMLMeterElement: [class HTMLMeterElement extends HTMLElement],
      HTMLMenuElement: [class HTMLMenuElement extends HTMLElement],
      HTMLMapElement: [class HTMLMapElement extends HTMLElement],
      HTMLLIElement: [class HTMLLIElement extends HTMLElement],
      HTMLLegendElement: [class HTMLLegendElement extends HTMLElement],
      HTMLModElement: [class HTMLModElement extends HTMLElement],
      HTMLHtmlElement: [class HTMLHtmlElement extends HTMLElement],
      HTMLHRElement: [class HTMLHRElement extends HTMLElement],
      HTMLHeadElement: [class HTMLHeadElement extends HTMLElement],
      HTMLHeadingElement: [class HTMLHeadingElement extends HTMLElement],
      HTMLFieldSetElement: [class HTMLFieldSetElement extends HTMLElement],
      HTMLEmbedElement: [class HTMLEmbedElement extends HTMLElement],
      HTMLDListElement: [class HTMLDListElement extends HTMLElement],
      HTMLDivElement: [class HTMLDivElement extends HTMLElement],
      HTMLDetailsElement: [class HTMLDetailsElement extends HTMLElement],
      HTMLDataListElement: [class HTMLDataListElement extends HTMLElement],
      HTMLDataElement: [class HTMLDataElement extends HTMLElement],
      HTMLTableColElement: [class HTMLTableColElement extends HTMLElement],
      HTMLTableCaptionElement: [class HTMLTableCaptionElement extends HTMLElement],
      HTMLCanvasElement: [class HTMLCanvasElement extends HTMLElement],
      HTMLBRElement: [class HTMLBRElement extends HTMLElement],
      HTMLBodyElement: [class HTMLBodyElement extends HTMLElement],
      HTMLAreaElement: [class HTMLAreaElement extends HTMLElement],
      SVGSVGElement: [class SVGSVGElement extends SVGGraphicsElement],
      SVGAnimateElement: [class SVGAnimateElement extends SVGAnimationElement],
      SVGAnimateMotionElement: [class SVGAnimateMotionElement extends SVGAnimationElement],
      SVGAnimateTransformElement: [class SVGAnimateTransformElement extends SVGAnimationElement],
      SVGCircleElement: [class SVGCircleElement extends SVGGeometryElement],
      SVGClipPathElement: [class SVGClipPathElement extends SVGElement],
      SVGDefsElement: [class SVGDefsElement extends SVGGraphicsElement],
      SVGDescElement: [class SVGDescElement extends SVGElement],
      SVGEllipseElement: [class SVGEllipseElement extends SVGGeometryElement],
      SVGFEBlendElement: [Function],
      SVGFEColorMatrixElement: [Function],
      SVGFEComponentTransferElement: [class SVGFEComponentTransferElement extends SVGElement],
      SVGFECompositeElement: [Function],
      SVGFEConvolveMatrixElement: [Function],
      SVGFEDiffuseLightingElement: [class SVGFEDiffuseLightingElement extends SVGElement],
      SVGFEDisplacementMapElement: [Function],
      SVGFEDistantLightElement: [class SVGFEDistantLightElement extends SVGElement],
      SVGFEDropShadowElement: [class SVGFEDropShadowElement extends SVGElement],
      SVGFEFloodElement: [class SVGFEFloodElement extends SVGElement],
      SVGFEFuncAElement: [class SVGFEFuncAElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncBElement: [class SVGFEFuncBElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncGElement: [class SVGFEFuncGElement extends SVGComponentTransferFunctionElement],
      SVGFEFuncRElement: [class SVGFEFuncRElement extends SVGComponentTransferFunctionElement],
      SVGFEGaussianBlurElement: [Function],
      SVGFEImageElement: [class SVGFEImageElement extends SVGElement],
      SVGFEMergeElement: [class SVGFEMergeElement extends SVGElement],
      SVGFEMergeNodeElement: [class SVGFEMergeNodeElement extends SVGElement],
      SVGFEMorphologyElement: [Function],
      SVGFEOffsetElement: [class SVGFEOffsetElement extends SVGElement],
      SVGFEPointLightElement: [class SVGFEPointLightElement extends SVGElement],
      SVGFESpecularLightingElement: [class SVGFESpecularLightingElement extends SVGElement],
      SVGFESpotLightElement: [class SVGFESpotLightElement extends SVGElement],
      SVGFETileElement: [class SVGFETileElement extends SVGElement],
      SVGFETurbulenceElement: [Function],
      SVGFilterElement: [class SVGFilterElement extends SVGElement],
      SVGForeignObjectElement: [class SVGForeignObjectElement extends SVGGraphicsElement],
      SVGGElement: [class SVGGElement extends SVGGraphicsElement],
      SVGImageElement: [class SVGImageElement extends SVGGraphicsElement],
      SVGLineElement: [class SVGLineElement extends SVGGeometryElement],
      SVGLinearGradientElement: [class SVGLinearGradientElement extends SVGGradientElement],
      SVGMarkerElement: [Function],
      SVGMaskElement: [class SVGMaskElement extends SVGElement],
      SVGMetadataElement: [class SVGMetadataElement extends SVGElement],
      SVGMPathElement: [class SVGMPathElement extends SVGElement],
      SVGPathElement: [class SVGPathElement extends SVGGeometryElement],
      SVGPatternElement: [class SVGPatternElement extends SVGElement],
      SVGPolygonElement: [class SVGPolygonElement extends SVGGeometryElement],
      SVGPolylineElement: [class SVGPolylineElement extends SVGGeometryElement],
      SVGRadialGradientElement: [class SVGRadialGradientElement extends SVGGradientElement],
      SVGRectElement: [class SVGRectElement extends SVGGeometryElement],
      SVGScriptElement: [class SVGScriptElement extends SVGGraphicsElement],
      SVGSetElement: [class SVGSetElement extends SVGAnimationElement],
      SVGStopElement: [class SVGStopElement extends SVGElement],
      SVGStyleElement: [class SVGStyleElement extends SVGElement],
      SVGSwitchElement: [class SVGSwitchElement extends SVGGraphicsElement],
      SVGSymbolElement: [class SVGSymbolElement extends SVGGraphicsElement],
      SVGTextElement: [class SVGTextElement extends SVGTextPositioningElement],
      SVGTextPathElement: [Function],
      SVGTitleElement: [class SVGTitleElement extends SVGElement],
      SVGTSpanElement: [class SVGTSpanElement extends SVGTextPositioningElement],
      SVGUseElement: [class SVGUseElement extends SVGGraphicsElement],
      SVGViewElement: [class SVGViewElement extends SVGElement],
      SVGElement: [class SVGElement extends Element],
      SVGAnimationElement: [class SVGAnimationElement extends SVGElement],
      SVGComponentTransferFunctionElement: [Function],
      SVGGeometryElement: [class SVGGeometryElement extends SVGGraphicsElement],
      SVGGradientElement: [Function],
      SVGTextPositioningElement: [class SVGTextPositioningElement extends SVGTextContentElement],
      SVGGraphicsElement: [class SVGGraphicsElement extends SVGElement],
      Event: [Function],
      UIEvent: [Function],
      CustomEvent: [class CustomEvent extends Event],
      CloseEvent: [class CloseEvent extends Event],
      AnimationEvent: [class AnimationEvent extends Event],
      KeyboardEvent: [Function],
      MessageEvent: [class MessageEvent extends Event],
      MouseEvent: [class MouseEvent extends UIEvent],
      PointerEvent: [class PointerEvent extends MouseEvent],
      FocusEvent: [class FocusEvent extends UIEvent],
      WheelEvent: [Function],
      InputEvent: [class InputEvent extends UIEvent],
      ErrorEvent: [class ErrorEvent extends UIEvent],
      StorageEvent: [class StorageEvent extends Event],
      SubmitEvent: [class SubmitEvent extends Event],
      ProgressEvent: [class ProgressEvent extends Event],
      MediaQueryListEvent: [class MediaQueryListEvent extends Event],
      HashChangeEvent: [class HashChangeEvent extends Event],
      ClipboardEvent: [class ClipboardEvent extends Event],
      CookieChangeEvent: [class CookieChangeEvent extends Event],
      TouchEvent: [class TouchEvent extends UIEvent],
      PopStateEvent: [class PopStateEvent extends Event],
      Touch: [class Touch],
      AudioProcessingEvent: [Function],
      BeforeInputEvent: [Function],
      BeforeUnloadEvent: [Function],
      BlobEvent: [Function],
      CompositionEvent: [Function],
      CSSFontFaceLoadEvent: [Function],
      DeviceLightEvent: [Function],
      DeviceMotionEvent: [Function],
      DeviceOrientationEvent: [Function],
      DeviceProximityEvent: [Function],
      DOMTransactionEvent: [Function],
      DragEvent: [Function],
      EditingBeforeInputEvent: [Function],
      FetchEvent: [Function],
      GamepadEvent: [Function],
      IDBVersionChangeEvent: [Function],
      MediaStreamEvent: [Function],
      MutationEvent: [Function],
      OfflineAudioCompletionEvent: [Function],
      OverconstrainedError: [Function],
      PageTransitionEvent: [Function],
      PaymentRequestUpdateEvent: [Function],
      RelatedEvent: [Function],
      RTCDataChannelEvent: [Function],
      RTCIdentityErrorEvent: [Function],
      RTCIdentityEvent: [Function],
      RTCPeerConnectionIceEvent: [Function],
      SensorEvent: [Function],
      SVGEvent: [Function],
      SVGZoomEvent: [Function],
      TimeEvent: [Function],
      TrackEvent: [Function],
      TransitionEvent: [Function],
      UserProximityEvent: [Function],
      WebGLContextEvent: [Function],
      TextEvent: [Function],
      Permissions: [class Permissions],
      History: [class History],
      Navigator: [class Navigator],
      Clipboard: [class Clipboard],
      TimeRanges: [class TimeRanges],
      TextTrackCueList: [class TextTrackCueList extends Array],
      ValidityState: [class ValidityState],
      MutationRecord: [class MutationRecord],
      IntersectionObserver: [class IntersectionObserver],
      IntersectionObserverEntry: [class IntersectionObserverEntry],
      CSSStyleDeclaration: [class CSSStyleDeclaration],
      CSSRule: [Function],
      CSSContainerRule: [class CSSContainerRule extends CSSConditionRule],
      CSSFontFaceRule: [class CSSFontFaceRule extends CSSRule],
      CSSKeyframeRule: [class CSSKeyframeRule extends CSSRule],
      CSSKeyframesRule: [class CSSKeyframesRule extends CSSRule],
      CSSMediaRule: [class CSSMediaRule extends CSSConditionRule],
      CSSStyleRule: [class CSSStyleRule extends CSSGroupingRule],
      CSSSupportsRule: [class CSSSupportsRule extends CSSConditionRule],
      CSSConditionRule: [class CSSConditionRule extends CSSGroupingRule],
      CSSGroupingRule: [class CSSGroupingRule extends CSSRule],
      CSSScopeRule: [class CSSScopeRule extends CSSGroupingRule],
      DOMRect: [class DOMRect extends DOMRectReadOnly],
      DOMRectReadOnly: [class DOMRectReadOnly],
      Plugin: [class Plugin],
      PluginArray: [class PluginArray],
      Location: [class Location],
      CustomElementRegistry: [class CustomElementRegistry],
      ResizeObserver: [class ResizeObserver],
      Blob: [class Blob],
      File: [class File extends Blob],
      Storage: [class Storage],
      CookieStore: [class CookieStore extends EventTarget],
      MimeType: [class MimeType],
      MimeTypeArray: [class MimeTypeArray],
      NodeFilter: [Object],
      ImageBitmap: [class ImageBitmap],
      HTMLCollection: [class HTMLCollection],
      HTMLFormControlCollection: [class HTMLFormControlsCollection extends HTMLCollection],
      HTMLOptionsCollection: [class HTMLOptionsCollection extends HTMLCollection],
      NodeList: [class NodeList],
      RadioNodeList: [class RadioNodeList extends NodeList],
      FileList: [class FileList extends Array],
      Screen: [class Screen extends EventTarget],
      ScreenDetails: [class ScreenDetails extends EventTarget],
      ScreenDetailed: [class ScreenDetailed extends Screen],
      NamedNodeMap: [class NamedNodeMap],
      TreeWalker: [class TreeWalker],
      NodeIterator: [class NodeIterator],
      DataTransfer: [class DataTransfer],
      DataTransferItem: [class DataTransferItem],
      DataTransferItemList: [class DataTransferItemList extends Array],
      XMLSerializer: [class XMLSerializer],
      ClipboardItem: [class ClipboardItem],
      Selection: [class Selection],
      CSSUnitValue: [class CSSUnitValue],
      SVGAngle: [Function],
      SVGAnimatedAngle: [class SVGAnimatedAngle],
      SVGAnimatedBoolean: [class SVGAnimatedBoolean],
      SVGAnimatedEnumeration: [class SVGAnimatedEnumeration],
      SVGAnimatedInteger: [class SVGAnimatedInteger],
      SVGAnimatedLength: [class SVGAnimatedLength],
      SVGAnimatedNumber: [class SVGAnimatedNumber],
      SVGAnimatedNumberList: [class SVGAnimatedNumberList],
      SVGAnimatedPreserveAspectRatio: [class SVGAnimatedPreserveAspectRatio],
      SVGAnimatedRect: [class SVGAnimatedRect],
      SVGAnimatedString: [class SVGAnimatedString],
      SVGAnimatedTransformList: [class SVGAnimatedTransformList],
      SVGLength: [Function],
      SVGLengthList: [class SVGLengthList],
      SVGMatrix: [class SVGMatrix],
      SVGNumber: [class SVGNumber],
      SVGNumberList: [class SVGNumberList],
      SVGPoint: [class SVGPoint],
      SVGPointList: [class SVGPointList],
      SVGPreserveAspectRatio: [Function],
      SVGRect: [class SVGRect],
      SVGStringList: [class SVGStringList],
      SVGTransform: [Function],
      SVGTransformList: [class SVGTransformList],
      SVGAnimatedLengthList: [class SVGAnimatedLengthList],
      SVGUnitTypes: [Function],
      StylePropertyMap: [class StylePropertyMap extends StylePropertyMapReadOnly],
      StylePropertyMapReadOnly: [class StylePropertyMapReadOnly],
      MediaList: [class MediaList],
      CSSKeywordValue: [class CSSKeywordValue],
      CSSStyleValue: [class CSSStyleValue],
      Window: [class GlobalWindow extends Window],
      URLSearchParams: [class URLSearchParams],
      WritableStream: [Function],
      ReadableStream: [Function],
      TransformStream: [Function: Transform],
      PerformanceObserver: [class PerformanceObserver],
      PerformanceEntry: [class PerformanceEntry],
      PerformanceObserverEntryList: [class PerformanceObserverEntryList],
      document: [HTMLDocument],
      customElements: [CustomElementRegistry],
      window: [Circular *3],
      globalThis: [Circular *3],
      performance: [Performance],
      screenLeft: 0,
      screenTop: 0,
      screenX: 0,
      screenY: 0,
      crypto: Crypto {},
      TextEncoder: [class TextEncoder],
      TextDecoder: [class TextDecoder],
      closed: false,
      console: [Object [console]],
      name: '',
      Buffer: [Function],
      Document: [class Document extends Document],
      HTMLDocument: [class HTMLDocument extends HTMLDocument],
      XMLDocument: [class XMLDocument extends XMLDocument],
      DocumentFragment: [class DocumentFragment extends DocumentFragment],
      Text: [class Text extends Text],
      Comment: [class Comment extends Comment],
      Image: [class Image extends Image],
      Audio: [class Audio extends Audio],
      MutationObserver: [class MutationObserver extends MutationObserver],
      MessagePort: [class MessagePort extends MessagePort],
      CSSStyleSheet: [class CSSStyleSheet extends CSSStyleSheet],
      DOMException: [class DOMException extends DOMException],
      Headers: [class Headers extends Headers],
      Request: [class Request extends Request],
      Response: [Function],
      EventTarget: [class EventTarget extends EventTarget],
      XMLHttpRequestUpload: [class XMLHttpRequestUpload extends XMLHttpRequestUpload],
      XMLHttpRequestEventTarget: [class XMLHttpRequestEventTarget extends XMLHttpRequestEventTarget],
      AbortController: [class AbortController extends AbortController],
      AbortSignal: [Function],
      FormData: [class FormData extends FormData],
      PermissionStatus: [class PermissionStatus extends PermissionStatus],
      XMLHttpRequest: [class XMLHttpRequest extends XMLHttpRequest],
      DOMParser: [class DOMParser extends DOMParser],
      Range: [class Range extends Range],
      VTTCue: [class VTTCue extends VTTCue],
      TextTrack: [class TextTrack extends TextTrack],
      TextTrackList: [class TextTrackList extends TextTrackList],
      TextTrackCue: [class TextTrackCue extends TextTrackCue],
      RemotePlayback: [class RemotePlayback extends RemotePlayback],
      FileReader: [class FileReader extends FileReader],
      MediaStream: [class MediaStream extends MediaStream],
      MediaStreamTrack: [class MediaStreamTrack extends MediaStreamTrack],
      CanvasCaptureMediaStreamTrack: [class CanvasCaptureMediaStreamTrack extends CanvasCaptureMediaStreamTrack],
      URL: [class URL extends URL],
      WebSocket: [class WebSocket extends WebSocket],
      ImageData: [class ImageData extends ImageData],
      OffscreenCanvas: [class OffscreenCanvas extends OffscreenCanvas],
      DOMMatrixReadOnly: [class DOMMatrixReadOnly extends DOMMatrixReadOnly],
      DOMMatrix: [class DOMMatrix extends DOMMatrix],
      DOMPointReadOnly: [class DOMPointReadOnly extends DOMPointReadOnly],
      DOMPoint: [class DOMPoint extends DOMPoint],
      Animation: [class Animation extends Animation],
      KeyframeEffect: [class KeyframeEffect extends KeyframeEffect],
      happyDOM: DetachedWindowAPI {},
      Array: [Function: Array],
      ArrayBuffer: [Function: ArrayBuffer],
      Boolean: [Function: Boolean],
      DataView: [Function: DataView],
      Date: [Function: Date],
      Error: [Function],
      EvalError: [Function: EvalError],
      Float32Array: [Function: Float32Array],
      Float64Array: [Function: Float64Array],
      Function: [Function: Function],
      Infinity: Infinity,
      Int16Array: [Function: Int16Array],
      Int32Array: [Function: Int32Array],
      Int8Array: [Function: Int8Array],
      Intl: Object [Intl] {},
      JSON: Object [JSON] {},
      Map: [Function: Map],
      Math: Object [Math] {},
      NaN: NaN,
      Number: [Function: Number],
      Object: [Function: Object],
      Promise: [Function: Promise],
      RangeError: [Function: RangeError],
      ReferenceError: [Function: ReferenceError],
      RegExp: [Function: RegExp],
      Set: [Function: Set],
      String: [Function: String],
      Symbol: [Function: Symbol],
      SyntaxError: [Function: SyntaxError],
      TypeError: [Function: TypeError],
      URIError: [Function: URIError],
      Uint16Array: [Function: Uint16Array],
      Uint32Array: [Function: Uint32Array],
      Uint8Array: [Function: Uint8Array],
      Uint8ClampedArray: [Function: Uint8ClampedArray],
      WeakMap: [Function: WeakMap],
      WeakSet: [Function: WeakSet],
      decodeURI: [Function: decodeURI],
      decodeURIComponent: [Function: decodeURIComponent],
      encodeURI: [Function: encodeURI],
      encodeURIComponent: [Function: encodeURIComponent],
      eval: [Function: eval],
      escape: [Function: escape],
      global: [Object [global]],
      isFinite: [Function: isFinite],
      isNaN: [Function: isNaN],
      parseFloat: [Function: parseFloat],
      parseInt: [Function: parseInt],
      undefined: undefined,
      unescape: [Function: unescape],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(mutationObservers)]: [Array],
      [Symbol(readyStateManager)]: DocumentReadyStateManager {},
      [Symbol(location)]: [Location],
      [Symbol(history)]: History {},
      [Symbol(navigator)]: Navigator {},
      [Symbol(screen)]: [Screen [EventTarget]],
      [Symbol(sessionStorage)]: [Storage],
      [Symbol(localStorage)]: [Storage],
      [Symbol(cookieStore)]: [CookieStore [EventTarget]],
      [Symbol(self)]: [Circular *3],
      [Symbol(top)]: [Circular *3],
      [Symbol(parent)]: [Circular *3],
      [Symbol(window)]: [Circular *3],
      [Symbol(frames)]: [Circular *3],
      [Symbol(internalId)]: 0,
      [Symbol(customElementReactionStack)]: [CustomElementReactionStack],
      [Symbol(modules)]: [Object],
      [Symbol(moduleImportMap)]: null,
      [Symbol(openWebSockets)]: [],
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(querySelectorCache)]: [Map]
    },
    [Symbol(classList)]: null,
    [Symbol(isValue)]: null,
    [Symbol(nodeType)]: 1,
    [Symbol(prefix)]: null,
    [Symbol(shadowRoot)]: null,
    [Symbol(scrollHeight)]: 0,
    [Symbol(scrollWidth)]: 0,
    [Symbol(scrollTop)]: 0,
    [Symbol(scrollLeft)]: 0,
    [Symbol(attributes)]: NamedNodeMap {
      [Symbol(itemsByNamespaceURI)]: [Map],
      [Symbol(itemsByName)]: [Map],
      [Symbol(items)]: [Map],
      [Symbol(ownerElement)]: [Circular *2]
    },
    [Symbol(attributesProxy)]: null,
    [Symbol(children)]: null,
    [Symbol(computedStyle)]: null,
    [Symbol(pointerCaptures)]: Set(0) {},
    [Symbol(propertyEventListeners)]: Map(0) {},
    [Symbol(animations)]: [],
    [Symbol(tagName)]: 'STYLE',
    [Symbol(localName)]: 'style',
    [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
    [Symbol(accessKey)]: '',
    [Symbol(offsetHeight)]: 0,
    [Symbol(offsetWidth)]: 0,
    [Symbol(offsetLeft)]: 0,
    [Symbol(offsetTop)]: 0,
    [Symbol(clientHeight)]: 0,
    [Symbol(clientWidth)]: 0,
    [Symbol(clientLeft)]: 0,
    [Symbol(clientTop)]: 0,
    [Symbol(style)]: null,
    [Symbol(dataset)]: null,
    [Symbol(customElementDefineCallback)]: null,
    [Symbol(sheet)]: null,
    [Symbol(disabled)]: false
  },
  className: 'tiptap',
  editorView: <ref *5> EditorView {
    _root: null,
    focused: false,
    trackWrites: null,
    mounted: false,
    markCursor: null,
    cursorWrapper: null,
    lastSelectedViewDesc: undefined,
    input: InputState {
      shiftKey: false,
      mouseDown: null,
      lastKeyCode: null,
      lastKeyCodeTime: 0,
      lastClick: [Object],
      lastSelectionOrigin: null,
      lastSelectionTime: 0,
      lastIOSEnter: 0,
      lastIOSEnterFallbackTimeout: -1,
      lastFocus: 0,
      lastTouch: 0,
      lastChromeDelete: 0,
      composing: false,
      compositionNode: null,
      composingTimeout: -1,
      compositionNodes: [],
      compositionEndedAt: -200000000,
      compositionID: 1,
      badSafariComposition: false,
      compositionPendingChanges: 0,
      domChangeCount: 0,
      eventHandlers: [Object: null prototype],
      hideSelectionGuard: null
    },
    prevDirectPlugins: [],
    pluginViews: [
      [Object],
      [Object],
      [Object],
      [Object],
      [DropCursorView],
      [Object],
      [Object],
      [Object]
    ],
    requiresGeckoHackNode: false,
    dragging: null,
    _props: {
      attributes: [Object],
      dispatchTransaction: [Function: bound dispatchTransaction],
      transformPastedHTML: [Function (anonymous)],
      state: [EditorState],
      markViews: {},
      nodeViews: [Object]
    },
    state: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    directPlugins: [],
    dispatch: [Function: bound ],
    dom: HTMLDivElement {
      pmViewDesc: [NodeViewDesc],
      editor: [Circular *4],
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: [HTMLDivElement],
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [Array],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: [DOMTokenList],
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: [CSSStyleDeclaration],
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    editable: true,
    nodeViews: [Object: null prototype] { taskItem: [Function: nodeview] },
    docView: NodeViewDesc {
      parent: undefined,
      children: [Array],
      dom: [HTMLDivElement],
      contentDOM: [HTMLDivElement],
      dirty: 0,
      node: [Node],
      outerDeco: [Array],
      innerDeco: [DecorationSet],
      nodeDOM: [HTMLDivElement]
    },
    domObserver: DOMObserver {
      view: [Circular *5],
      handleDOMChange: [Function (anonymous)],
      queue: [],
      flushingSoon: -1,
      observer: MutationObserver {},
      currentSelection: [SelectionState],
      onCharData: null,
      suppressingSelectionUpdates: false,
      lastChangedTextNode: null,
      onSelectionChange: [Function: bound onSelectionChange]
    }
  },
  isFocused: false,
  destroyed: false,
  isInitialized: false,
  extensionStorage: {
    link: {},
    paragraph: {},
    listItemBranchingDeleteKeymap: {},
    taskItemBranchingDeleteKeymap: {},
    textStyle: {},
    editable: {},
    clipboardTextSerializer: {},
    commands: {},
    focusEvents: {},
    keymap: {},
    tabindex: {},
    drop: {},
    paste: {},
    delete: {},
    textDirection: {},
    starterKit: {},
    bold: {},
    blockquote: {},
    bulletList: {},
    code: {},
    doc: {},
    dropCursor: {},
    gapCursor: {},
    hardBreak: {},
    undoRedo: {},
    italic: {},
    listItem: {},
    listKeymap: {},
    orderedList: {},
    strike: {},
    text: {},
    underline: {},
    trailingNode: {},
    taskList: {},
    taskItem: {},
    color: {},
    fontSize: {},
    textAlign: {},
    typography: {},
    placeholder: {}
  },
  instanceId: 'ghfenf8',
  hasWarnedStaleDecorationRead: false,
  options: {
    element: HTMLDivElement {
      [Symbol(listeners)]: [Object],
      [Symbol(listenerOptions)]: [Object],
      [Symbol(isConnected)]: false,
      [Symbol(parentNode)]: null,
      [Symbol(rootNode)]: null,
      [Symbol(styleNode)]: null,
      [Symbol(textAreaNode)]: null,
      [Symbol(formNode)]: null,
      [Symbol(selectNode)]: null,
      [Symbol(mutationListeners)]: [],
      [Symbol(nodeArray)]: [Array],
      [Symbol(elementArray)]: [Array],
      [Symbol(childNodes)]: null,
      [Symbol(assignedToSlot)]: null,
      [Symbol(cache)]: [Object],
      [Symbol(affectsCache)]: [],
      [Symbol(ownerDocument)]: [HTMLDocument],
      [Symbol(window)]: [GlobalWindow [EventTarget]],
      [Symbol(classList)]: null,
      [Symbol(isValue)]: null,
      [Symbol(nodeType)]: 1,
      [Symbol(prefix)]: null,
      [Symbol(shadowRoot)]: null,
      [Symbol(scrollHeight)]: 0,
      [Symbol(scrollWidth)]: 0,
      [Symbol(scrollTop)]: 0,
      [Symbol(scrollLeft)]: 0,
      [Symbol(attributes)]: [NamedNodeMap],
      [Symbol(attributesProxy)]: null,
      [Symbol(children)]: null,
      [Symbol(computedStyle)]: null,
      [Symbol(pointerCaptures)]: Set(0) {},
      [Symbol(propertyEventListeners)]: Map(0) {},
      [Symbol(animations)]: [],
      [Symbol(tagName)]: 'DIV',
      [Symbol(localName)]: 'div',
      [Symbol(namespaceURI)]: 'http://www.w3.org/1999/xhtml',
      [Symbol(accessKey)]: '',
      [Symbol(offsetHeight)]: 0,
      [Symbol(offsetWidth)]: 0,
      [Symbol(offsetLeft)]: 0,
      [Symbol(offsetTop)]: 0,
      [Symbol(clientHeight)]: 0,
      [Symbol(clientWidth)]: 0,
      [Symbol(clientLeft)]: 0,
      [Symbol(clientTop)]: 0,
      [Symbol(style)]: null,
      [Symbol(dataset)]: null,
      [Symbol(customElementDefineCallback)]: null
    },
    content: { type: 'doc', content: [Array] },
    injectCSS: true,
    injectNonce: undefined,
    extensions: [
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    autofocus: false,
    editable: true,
    textDirection: undefined,
    editorProps: { attributes: [Object] },
    parseOptions: {},
    coreExtensionOptions: {},
    enableInputRules: true,
    enablePasteRules: true,
    enableCoreExtensions: true,
    enableContentCheck: false,
    emitContentError: false,
    onBeforeCreate: [Function: onBeforeCreate],
    onCreate: [Function: onCreate],
    onMount: [Function: onMount],
    onUnmount: [Function: onUnmount],
    onUpdate: [Function: onUpdate],
    onSelectionUpdate: [Function: onSelectionUpdate],
    onTransaction: [Function: onTransaction],
    onFocus: [Function: onFocus],
    onBlur: [Function: onBlur],
    onDestroy: [Function: onDestroy],
    onContentError: [Function: onContentError],
    onPaste: [Function: onPaste],
    onDrop: [Function: onDrop],
    onDelete: [Function: onDelete],
    enableExtensionDispatchTransaction: true
  },
  isCapturingTransaction: false,
  capturedTransaction: null,
  utils: {
    getUpdatedPosition: [Function: getUpdatedPosition],
    createMappablePosition: [Function: createMappablePosition]
  },
  extensionManager: ExtensionManager {
    splittableMarks: [ 'textStyle', 'bold', 'code', 'italic', 'strike', 'underline' ],
    nonClearableMarks: [],
    decorationManager: DecorationManager {
      warnedWidgetKeys: Set(0) {},
      warnedOutOfRangeExtensions: Set(0) {},
      handleBeforeTransaction: [Function (anonymous)],
      editor: [Circular *4],
      entries: [],
      plugin: null
    },
    editor: [Circular *4],
    baseExtensions: [
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension], [Node],
      [Node],      [Mark],
      [Extension], [Extension],
      [Extension], [Extension],
      [Extension]
    ],
    extensions: [
      [Mark],      [Node],      [Extension],
      [Extension], [Mark],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Extension], [Extension],
      [Extension], [Mark],      [Node],
      [Node],      [Mark],      [Node],
      [Extension], [Extension], [Node],
      [Extension], [Mark],      [Node],
      [Extension], [Node],      [Mark],
      [Node],      [Mark],      [Extension],
      [Node],      [Node],      [Extension],
      [Extension], [Extension], [Extension],
      [Extension]
    ],
    schema: Schema {
      linebreakReplacement: [NodeType],
      cached: [Object: null prototype],
      spec: [Object],
      nodes: [Object: null prototype],
      marks: [Object: null prototype],
      nodeFromJSON: [Function (anonymous)],
      markFromJSON: [Function (anonymous)],
      topNodeType: [NodeType]
    }
  },
  commandManager: CommandManager {
    editor: [Circular *4],
    rawCommands: {
      setLink: [Function: setLink],
      toggleLink: [Function: toggleLink],
      unsetLink: [Function: unsetLink],
      setParagraph: [Function: setParagraph],
      toggleTextStyle: [Function: toggleTextStyle],
      removeEmptyTextStyle: [Function: removeEmptyTextStyle],
      blur: [Function: blur],
      clearContent: [Function: clearContent],
      clearNodes: [Function: clearNodes],
      command: [Function: command],
      createParagraphNear: [Function: createParagraphNear$1],
      cut: [Function: cut],
      deleteCurrentNode: [Function: deleteCurrentNode],
      deleteNode: [Function: deleteNode],
      deleteRange: [Function: deleteRange],
      deleteSelection: [Function: deleteSelection],
      enter: [Function: enter],
      exitCode: [Function: exitCode$1],
      extendMarkRange: [Function: extendMarkRange],
      first: [Function: first],
      focus: [Function: focus],
      forEach: [Function: forEach],
      insertContent: [Function: insertContent],
      insertContentAt: [Function: insertContentAt],
      insertDefaultBlock: [Function: insertDefaultBlock],
      joinBackward: [Function: joinBackward$1],
      joinDown: [Function: joinDown$1],
      joinForward: [Function: joinForward$1],
      joinItemBackward: [Function: joinItemBackward],
      joinItemForward: [Function: joinItemForward],
      joinTextblockBackward: [Function: joinTextblockBackward$1],
      joinTextblockForward: [Function: joinTextblockForward$1],
      joinUp: [Function: joinUp$1],
      keyboardShortcut: [Function: keyboardShortcut],
      lift: [Function: lift$1],
      liftEmptyBlock: [Function: liftEmptyBlock$1],
      liftListItem: [Function: liftListItem$1],
      newlineInCode: [Function: newlineInCode$1],
      resetAttributes: [Function: resetAttributes],
      scrollIntoView: [Function: scrollIntoView],
      selectAll: [Function: selectAll],
      selectNodeBackward: [Function: selectNodeBackward$1],
      selectNodeForward: [Function: selectNodeForward$1],
      selectParentNode: [Function: selectParentNode$1],
      selectTextblockEnd: [Function: selectTextblockEnd$1],
      selectTextblockStart: [Function: selectTextblockStart$1],
      setContent: [Function: setContent],
      setMark: [Function: setMark],
      setMeta: [Function: setMeta],
      setNode: [Function: setNode],
      setNodeSelection: [Function: setNodeSelection],
      setTextDirection: [Function: setTextDirection],
      setTextSelection: [Function: setTextSelection],
      sinkListItem: [Function: sinkListItem$1],
      splitBlock: [Function: splitBlock],
      splitListItem: [Function: splitListItem],
      toggleList: [Function: toggleList],
      toggleMark: [Function: toggleMark],
      toggleNode: [Function: toggleNode],
      toggleWrap: [Function: toggleWrap],
      undoInputRule: [Function: undoInputRule],
      unsetAllMarks: [Function: unsetAllMarks],
      unsetMark: [Function: unsetMark],
      unsetTextDirection: [Function: unsetTextDirection],
      updateAttributes: [Function: updateAttributes],
      updateDecorations: [Function: updateDecorations],
      wrapIn: [Function: wrapIn$1],
      wrapInList: [Function: wrapInList$1],
      setBold: [Function: setBold],
      toggleBold: [Function: toggleBold],
      unsetBold: [Function: unsetBold],
      setBlockquote: [Function: setBlockquote],
      toggleBlockquote: [Function: toggleBlockquote],
      unsetBlockquote: [Function: unsetBlockquote],
      toggleBulletList: [Function: toggleBulletList],
      setCode: [Function: setCode],
      toggleCode: [Function: toggleCode],
      unsetCode: [Function: unsetCode],
      setHardBreak: [Function: setHardBreak],
      undo: [Function: undo],
      redo: [Function: redo],
      setItalic: [Function: setItalic],
      toggleItalic: [Function: toggleItalic],
      unsetItalic: [Function: unsetItalic],
      toggleOrderedList: [Function: toggleOrderedList],
      setStrike: [Function: setStrike],
      toggleStrike: [Function: toggleStrike],
      unsetStrike: [Function: unsetStrike],
      setUnderline: [Function: setUnderline],
      toggleUnderline: [Function: toggleUnderline],
      unsetUnderline: [Function: unsetUnderline],
      toggleTaskList: [Function: toggleTaskList],
      setColor: [Function: setColor],
      unsetColor: [Function: unsetColor],
      setFontSize: [Function: setFontSize],
      unsetFontSize: [Function: unsetFontSize],
      setTextAlign: [Function: setTextAlign],
      unsetTextAlign: [Function: unsetTextAlign],
      toggleTextAlign: [Function: toggleTextAlign]
    },
    customState: undefined
  },
  schema: <ref *6> Schema {
    linebreakReplacement: NodeType {
      name: 'hardBreak',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [Array],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: false,
      isText: false
    },
    cached: [Object: null prototype] { wrappings: [Object: null prototype] {} },
    spec: { topNode: 'doc', nodes: [OrderedMap], marks: [OrderedMap] },
    nodes: [Object: null prototype] {
      paragraph: [NodeType],
      blockquote: [NodeType],
      bulletList: [NodeType],
      doc: [NodeType],
      hardBreak: [NodeType],
      listItem: [NodeType],
      orderedList: [NodeType],
      text: [NodeType],
      taskList: [NodeType],
      taskItem: [NodeType]
    },
    marks: [Object: null prototype] {
      link: [MarkType],
      textStyle: [MarkType],
      bold: [MarkType],
      code: [MarkType],
      italic: [MarkType],
      strike: [MarkType],
      underline: [MarkType]
    },
    nodeFromJSON: [Function (anonymous)],
    markFromJSON: [Function (anonymous)],
    topNodeType: NodeType {
      name: 'doc',
      schema: [Circular *6],
      spec: [Object],
      markSet: [],
      groups: [],
      attrs: [Object: null prototype] {},
      defaultAttrs: [Object: null prototype] {},
      contentMatch: [ContentMatch],
      inlineContent: false,
      isBlock: true,
      isText: false
    }
  },
  editorState: EditorState {
    config: Configuration {
      schema: [Schema],
      plugins: [],
      pluginsByKey: [Object: null prototype] {},
      fields: [Array]
    },
    doc: Node {
      type: [NodeType],
      attrs: [Object: null prototype] {},
      marks: [],
      content: [Fragment]
    },
    selection: TextSelection {
      '$anchor': [ResolvedPos],
      '$head': [ResolvedPos],
      ranges: [Array]
    },
    storedMarks: null,
    scrollToSelection: 0
  },
  contentComponent: null,
  appContext: null,
  reactiveState: CustomRefImpl {
    __v_isRef: true,
    _value: EditorState {
      config: [Configuration],
      doc: [Node],
      selection: [TextSelection],
      storedMarks: null,
      scrollToSelection: 0,
      'plugin$8': null,
      'plugin$13': null,
      'trailingNode$': false,
      'plugin$19': null,
      'plugin$22': null,
      'plugin$27': null,
      'history$': [HistoryState],
      'plugin$38': null,
      'plugin$41': null,
      'plugin$43': null,
      'plugin$45': null
    },
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: [Link],
      map: undefined,
      key: undefined,
      sc: 2,
      __v_skip: true,
      subsHead: [Link]
    },
    _get: [Function: get],
    _set: [Function: set]
  },
  reactiveExtensionStorage: CustomRefImpl {
    __v_isRef: true,
    _value: undefined,
    dep: Dep {
      computed: undefined,
      version: 0,
      activeLink: undefined,
      subs: undefined,
      map: undefined,
      key: undefined,
      sc: 0,
      __v_skip: true,
      subsHead: undefined
    },
    _get: [Function: get],
    _set: [Function: set]
  }
} default-font-size=undefined  ... > 
  at <SpaceCanvasNote id="note" data= {
  kind: 'note',
  title: 'First',
  tiptapContent: { type: 'doc', content: [ [Object], [Object] ] },
  plainText: 'First\nBody'
} detailLevel="full"  ... > 
  at <VTUROOT>
[Vue warn]: Unhandled error during execution of component update 
  at <VTUROOT>

 ❯ src/features/spaces/__tests__/konva-rich-node-interactions.test.ts (3 tests | 1 failed) 650ms
     × persists direct note editing and refuses editing for locked notes 574ms
     ✓ consumes media requests and synchronizes playback state 55ms
     ✓ surfaces background media analysis on the canvas node 16ms
 ❯ src/features/spaces/__tests__/selection-actions.test.ts (3 tests | 3 failed) 45ms
     × allows a Section itself to be locked 32ms
     × removes selected relations before nodes and undoes both in one step 1ms
     × deletes a selected relation on its own 5ms
 ✓ src/features/spaces/__tests__/json-canvas-service.test.ts (1 test) 41ms
 ❯ src/features/spaces/__tests__/manifest-parser.test.ts (5 tests | 4 failed) 152ms
     × accepts regenerated canonical citations and detached historical snapshots 81ms
     × accepts All-Spaces threads with detached external citation snapshots 27ms
     × rejects a canonical citation whose snapshot revision is stale 17ms
     × rejects derivations that reference unavailable nodes 14ms
     ✓ rejects canvas state that copies domain content into the layout document 6ms
 ✓ src/foundation/ai/__tests__/integration-settings-schema.test.ts (8 tests) 101ms
 ✓ src/features/spaces/__tests__/package-structure-remap.test.ts (3 tests) 85ms
 ❯ src/features/spaces/__tests__/structure-contract.test.ts (8 tests | 1 failed) 97ms
     ✓ keeps structure and local layout separate and resolves operation roots 42ms
     ✓ derives tight local Group bounds without rotating world AABBs again 8ms
     ✓ rejects a missing active content placement 5ms
     ✓ rejects nested Sections 1ms
     ✓ rejects duplicate sibling orders 2ms
     × rejects parent cycles without recursive traversal 25ms
     ✓ rejects kind-mismatched layouts 5ms
     ✓ rejects composed scales outside the contract even when locals are legal 1ms
 ✓ src/features/spaces/__tests__/imported-asset-payload.test.ts (8 tests) 13ms
 ❯ src/features/spaces/__tests__/canvas-movement.test.ts (8 tests | 8 failed) 44ms
     × captures all selected Section members when s1 starts the drag 23ms
     × captures all selected Section members when s2 starts the drag 3ms
     × copies every captured frame and retains locked descendants 1ms
     × rejects capture rather than skipping a locked selected root 5ms
     × applies a different offset to each whole root for alignment/distribution 3ms
     × moves a selected child only once when its Section is also selected 2ms
     × does not interpret resizing as a movement 2ms
     × rejects missing descendant geometry instead of silently leaving it behind 2ms
 ❯ src/features/spaces/__tests__/canvas-gesture-camera.test.ts (4 tests | 4 failed) 56ms
     × keeps camera autosave free of uncommitted geometry (rejected=false) 41ms
     × keeps camera autosave free of uncommitted geometry (rejected=true) 1ms
     × preserves camera changes while undo is committing 7ms
     × preserves camera changes while redo is committing 1ms
 ❯ src/foundation/i18n/__tests__/locale-completeness.test.ts (5 tests | 1 failed) 114ms
     ✓ keeps English and Simplified Chinese keys aligned 12ms
     ✓ defines every statically referenced translation key 24ms
     ✓ keeps placeholders aligned between languages 19ms
     ✓ keeps locale module paths aligned 1ms
     × uses semantic lowerCamelCase message keys 53ms
 ✓ src/features/spaces/__tests__/media-import.test.ts (9 tests) 54ms
 ❯ src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts (5 tests | 5 failed) 61ms
     × commits multiple dragged Sections without separately reparenting their members 39ms
     × still permits an explicit content drag to leave its Section 4ms
     × does not change membership when a resize moves a content center outside its Section 3ms
     × does not record an empty selection or a zero-offset movement 6ms
     × rejects a missing descendant layout before any persisted keyboard movement 6ms
 ❯ src/features/spaces/__tests__/node-content-height-batch.test.ts (2 tests | 2 failed) 51ms
     × fits text height in both directions without creating history 40ms
     × defers content layout while a direct resize owns the frame 7ms
 ❯ src/features/spaces/__tests__/canvas-arrangement-operation.test.ts (4 tests | 4 failed) 42ms
     × distributes three Section roots and carries every member by its parent's delta 24ms
     × gives ELK only operation roots and expands its resulting movement to members 5ms
     × does not persist a partial alignment when any selected root is locked 3ms
     × leaves the document unchanged if automatic layout computation fails 2ms
 ✓ src/features/spaces/__tests__/tiptap-outline.test.ts (3 tests) 24ms
 ✓ src/features/spaces/__tests__/konva-transform-geometry.test.ts (6 tests) 19ms
 ❯ src/features/spaces/__tests__/konva-canvas-presentation-document.test.ts (0 test)
 ✓ src/features/spaces/__tests__/website-refresh.test.ts (2 tests) 28ms
 ✓ src/features/spaces/__tests__/json-canvas-resource-preparation.test.ts (1 test) 41ms
 ❯ src/features/spaces/__tests__/canvas-section-gesture.test.ts (1 test | 1 failed) 21ms
     × moves section members and preserves a newer viewport on undo 18ms
 ✓ src/features/spaces/__tests__/knowledge-structure-scope.test.ts (8 tests) 46ms
 ✓ src/foundation/ai/__tests__/ai-execution.test.ts (6 tests) 33ms
 ✓ src/features/spaces/__tests__/konva-preview-scene.test.ts (1 test) 22ms
 ✓ src/features/spaces/__tests__/media-schema.test.ts (5 tests) 108ms
 ✓ src/features/spaces/__tests__/node-knowledge.test.ts (4 tests) 34ms
 ✓ src/features/spaces/__tests__/media-editing.test.ts (5 tests) 37ms
 ✓ src/features/spaces/__tests__/space-resource-content.test.ts (4 tests) 279ms
 ❯ src/features/spaces/__tests__/node.test.ts (4 tests | 1 failed) 277ms
     ✓ keeps renderer state out of the domain node contract 8ms
     × rejects current section geometry in domain data 46ms
     ✓ deep-clones media knowledge for independent duplicated nodes 19ms
     ✓ clones reactive node data into a plain persistable value 191ms
 ✓ src/features/spaces/__tests__/inspector-actions.test.ts (4 tests) 17ms
 ✓ src/features/spaces/__tests__/resource-refresh.test.ts (3 tests) 26ms
 ✓ src/features/spaces/__tests__/canvas-connector-gesture.test.ts (4 tests) 32ms
 ✓ src/features/spaces/__tests__/movement-scope.test.ts (7 tests) 35ms
 ✓ src/features/spaces/__tests__/creation-workflow-executor.test.ts (2 tests) 40ms
 ✓ src/features/spaces/__tests__/canvas-editing-commands.test.ts (2 tests) 13ms
 ❯ src/features/spaces/__tests__/canvas-gesture-update.test.ts (4 tests | 4 failed) 46ms
     × commits frame and typography scale as one layout mutation 28ms
     × captures absence explicitly so undo removes gesture-created appearance 7ms
     × preserves unchanged records without inventing absent appearance fields 1ms
     × restores absence as the exact original layout shape without mutating the source 1ms
 ✓ src/foundation/tasks/__tests__/task-store.test.ts (5 tests) 119ms
 ❯ src/features/spaces/__tests__/selection-product-state.test.ts (4 tests | 4 failed) 30ms
     × exposes semantic product actions without duplicating engine arrangement actions 18ms
     × uses section context and preserves the selected lock state 6ms
     × connects only an exact two-node semantic selection 1ms
     × identifies managed assets as system-preview targets 3ms
 ✓ src/features/spaces/__tests__/markdown-document.test.ts (8 tests) 26ms
 ✓ src/features/spaces/enrichment/__tests__/job-scheduler.test.ts (2 tests) 70ms
 ✓ src/features/spaces/__tests__/credential.test.ts (3 tests) 48ms
 ✓ src/features/spaces/__tests__/canvas-website-preview.test.ts (3 tests) 180ms
 ❯ src/features/spaces/__tests__/clipboard-transfer.test.ts (3 tests | 3 failed) 207ms
     × round-trips the structured HTML marker while keeping useful plain text 161ms
     × copies reactive node data without structured-clone failures 8ms
     × rejects any clipboard payload that is not version 1 34ms
 ✓ src/features/spaces/__tests__/space-history-pending.test.ts (3 tests) 46ms
 ✓ src/features/spaces/__tests__/canvas-snapping.test.ts (4 tests) 34ms
 ✓ src/features/spaces/__tests__/konva-selection-chrome.test.ts (3 tests) 29ms
 ✓ src/features/spaces/__tests__/media-chapters.test.ts (4 tests) 30ms
 ❯ src/features/spaces/__tests__/derivation-schema.test.ts (4 tests | 2 failed) 69ms
     × accepts complete generated video metadata 55ms
     ✓ requires metadata for a completed knowledge video 1ms
     ✓ rejects obsolete review metadata 5ms
     × accepts a completed Markdown PDF export without provider metadata 6ms
 ✓ src/shell/__tests__/commands.test.ts (2 tests) 26ms
 ✓ src/features/spaces/__tests__/canvas-drop.test.ts (2 tests) 70ms
 ✓ src/features/spaces/__tests__/derivation-output-application.test.ts (2 tests) 27ms
 ❯ src/features/spaces/__tests__/konva-selection.test.ts (2 tests | 2 failed) 38ms
     × preserves a multi-selection when an already-selected node begins a drag 28ms
     × removes selected nodes that leave the projected scene 6ms
 ✓ src/features/spaces/__tests__/caption.test.ts (4 tests) 25ms
 ✓ src/features/spaces/__tests__/space-canvas-node-presentation.test.ts (2 tests) 14ms
 ✓ src/features/settings/components/__tests__/SecretSettingsRow.test.ts (3 tests) 238ms
 ❯ src/features/spaces/__tests__/knowledge-location.test.ts (3 tests | 1 failed) 51ms
     ✓ formats precise source positions 14ms
     × checks source and editor revisions against the correct authority 25ms
     ✓ preserves explicit FTS match segments for rendering 5ms
 ❯ src/features/spaces/__tests__/konva-node-gesture-callbacks.test.ts (1 test | 1 failed) 43ms
     × keeps event callbacks bound to their gesture controller 41ms
 ❯ src/features/spaces/__tests__/canvas-overview-geometry.test.ts (3 tests | 3 failed) 38ms
     × fits the whole document independently of editor viewport 30ms
     × includes rotated node corners 1ms
     × does not depend on a mounted viewport when empty 3ms
 ✓ src/features/spaces/__tests__/json-canvas-node-preparation.test.ts (1 test) 27ms
 ✓ src/features/spaces/__tests__/website-metadata.test.ts (1 test) 148ms
 ✓ src/features/spaces/__tests__/relation-graph.test.ts (3 tests) 24ms
 ✓ src/shared/ui/__tests__/UiButton.test.ts (4 tests) 241ms
 ✓ src/foundation/ai/__tests__/structured-output.test.ts (5 tests) 38ms
 ✓ src/features/spaces/__tests__/clipboard-image-import.test.ts (2 tests) 22ms
 ✓ src/features/spaces/__tests__/test-session-context.test.ts (1 test) 23ms
 ✓ src/features/spaces/workspace/__tests__/navigation-history.test.ts (3 tests) 38ms
 ✓ src/features/spaces/__tests__/knowledge-video-inputs.test.ts (2 tests) 20ms
 ✓ src/features/spaces/__tests__/space-transition.test.ts (1 test) 19ms
 ✓ src/features/spaces/__tests__/canvas-text-transform.test.ts (4 tests) 18ms
 ✓ src/features/spaces/__tests__/canvas-arrangement.test.ts (3 tests) 15ms
 ✓ src/features/spaces/__tests__/canvas-text-commit-session.test.ts (2 tests) 20ms
 ✓ src/features/spaces/__tests__/knowledge-index-scheduler.test.ts (1 test) 37ms
 ✓ src/foundation/context-menu/__tests__/context-menu.test.ts (4 tests) 17ms
 ✓ src/app/__tests__/workspace-session.test.ts (2 tests) 41ms
 ✓ src/features/spaces/__tests__/canvas-editor-navigation.test.ts (2 tests) 24ms
 ✓ src/features/spaces/__tests__/canvas-thumbnail-file.test.ts (2 tests) 36ms
 ✓ src/features/spaces/__tests__/media-platform-url.test.ts (21 tests) 28ms
 ❯ src/features/spaces/__tests__/canvas-section-membership.test.ts (0 test)
 ✓ src/foundation/preferences/__tests__/preferences-schema.test.ts (4 tests) 66ms
 ✓ src/foundation/async/__tests__/keyed-serial-queue.test.ts (2 tests) 29ms
 ✓ src/foundation/dom/__tests__/scrollable-target.test.ts (4 tests) 52ms
 ✓ src/features/settings/__tests__/ai-connection-draft.test.ts (2 tests) 38ms
 ✓ src/features/spaces/__tests__/content-remap.test.ts (2 tests) 20ms
 ✓ src/features/spaces/__tests__/file-presentation.test.ts (2 tests) 20ms
 ✓ src/features/spaces/__tests__/canvas-video-caption.test.ts (1 test) 103ms
 ✓ src/shared/ui/__tests__/UiDropdownMenu.test.ts (2 tests) 435ms
 ✓ src/shared/ui/__tests__/UiSearchInput.test.ts (3 tests) 189ms
 ✓ src/shared/ui/__tests__/UiProgress.test.ts (3 tests) 214ms
 ✓ src/features/spaces/__tests__/canvas-content-layout-coordinator.test.ts (1 test) 22ms
 ✓ src/features/spaces/__tests__/json-canvas-group-background.test.ts (3 tests) 19ms
 ✓ src/foundation/ai/__tests__/agent-models.test.ts (3 tests) 49ms
 ❯ src/features/spaces/__tests__/canvas-gesture-repository.test.ts (1 test | 1 failed) 47ms
     × constructs the strict persistence DTO without leaking gesture-only fields 38ms
 ❯ src/features/spaces/__tests__/canvas-document-rebase.test.ts (1 test | 1 failed) 35ms
     × merges independent relation presentation changes by relation identity 23ms
 ❯ src/features/spaces/__tests__/canvas-package-codec.test.ts (1 test | 1 failed) 29ms
     × remaps node references in the neutral document 25ms
 ✓ src/shared/ui/__tests__/UiKbd.test.ts (3 tests) 152ms
 ✓ src/features/spaces/__tests__/canvas-selection-projection.test.ts (1 test) 35ms
 ✓ src/shared/ui/__tests__/UiBadge.test.ts (3 tests) 193ms
 ✓ src/features/spaces/__tests__/preview-policy.test.ts (3 tests) 12ms
 ✓ src/shared/ui/__tests__/UiIconButton.test.ts (2 tests) 256ms
 ✓ src/features/spaces/__tests__/assistant-state.test.ts (2 tests) 26ms
 ✓ src/features/spaces/__tests__/url.test.ts (4 tests) 40ms
 ✓ src/shared/ui/__tests__/use-kbd.test.ts (2 tests) 28ms
 ✓ src/features/spaces/__tests__/knowledge-passage.test.ts (2 tests) 64ms
 ✓ src/features/spaces/__tests__/space-repository.test.ts (1 test) 24ms
 ✓ src/features/spaces/__tests__/canvas-text-selection-bar.test.ts (1 test) 69ms
 ✓ src/shared/ui/__tests__/UiNumberField.test.ts (2 tests) 213ms
 ✓ src/features/spaces/__tests__/json-canvas-relations.test.ts (1 test) 33ms
 ✓ src/shared/ui/__tests__/UiSwitch.test.ts (2 tests) 120ms
 ✓ src/shared/ui/__tests__/UiDropdownMenuItem.test.ts (1 test) 270ms
 ✓ src/features/spaces/__tests__/canvas-edge-auto-pan.test.ts (3 tests) 23ms
 ✓ src/features/spaces/__tests__/knowledge-object-presentation.test.ts (2 tests) 26ms
 ✓ src/features/spaces/__tests__/text.test.ts (3 tests) 13ms
 ✓ src/shared/ui/__tests__/UiCheckbox.test.ts (2 tests) 181ms
 ✓ src/shared/ui/__tests__/UiDropdownMenuCheckboxItem.test.ts (1 test) 416ms
     ✓ renders checkbox item with slot content  405ms
 ✓ src/features/spaces/__tests__/deferred-editor-mount.test.ts (1 test) 15ms
 ✓ src/features/spaces/__tests__/konva-viewport.test.ts (2 tests) 22ms
 ✓ src/features/spaces/__tests__/search-snippet.test.ts (2 tests) 17ms
 ✓ src/shared/ui/__tests__/UiTagsInput.test.ts (2 tests) 219ms
 ✓ src/features/spaces/__tests__/konva-viewport-culling.test.ts (1 test) 35ms
 ✓ src/features/spaces/__tests__/import-placement.test.ts (1 test) 10ms
 ✓ src/shared/ui/__tests__/UiSeparator.test.ts (2 tests) 128ms
 ✓ src/app/__tests__/runtime-diagnostics.test.ts (1 test) 21ms
 ✓ src/features/spaces/__tests__/url-source-kind.test.ts (4 tests) 24ms
 ✓ src/foundation/file-drop/__tests__/file-drop.test.ts (2 tests) 18ms
 ✓ src/features/spaces/__tests__/direct-media-resource.test.ts (1 test) 12ms
 ✓ src/shared/ui/__tests__/cn.test.ts (2 tests) 53ms
 ✓ src/shared/ui/__tests__/UiPopover.test.ts (1 test) 223ms
 ✓ src/features/spaces/__tests__/space-command-shortcuts.test.ts (1 test) 4ms

⎯⎯⎯⎯⎯⎯ Failed Suites 2 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  src/features/spaces/__tests__/canvas-section-membership.test.ts [ src/features/spaces/__tests__/canvas-section-membership.test.ts ]
Error: Failed to resolve import "../canvas/konva/scene/section-membership" from "src/features/spaces/__tests__/canvas-section-membership.test.ts". Does the file exist?
  Plugin: vite:import-analysis
  File: /home/runner/work/knovum/knovum/src/features/spaces/__tests__/canvas-section-membership.test.ts:4:35
  1  |  import { describe, expect, it } from "vitest";
  2  |  import { sectionAssignments } from "../canvas/konva/scene/section-membership";
     |                                      ^
  3  |  import { createSpaceNotePayload } from "../editor/tiptap-content";
  4  |  describe("canvas section membership", () => {
 ❯ TransformPluginContext._formatLog node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:31147:39
 ❯ TransformPluginContext.error node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:31144:14
 ❯ normalizeUrl node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:28083:18
 ❯ node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:28153:30
 ❯ TransformPluginContext.transform node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:28119:4
 ❯ EnvironmentPluginContainer.transform node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:30932:14
 ❯ loadAndTransform node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:20671:26

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/119]⎯

 FAIL  src/features/spaces/__tests__/konva-canvas-presentation-document.test.ts [ src/features/spaces/__tests__/konva-canvas-presentation-document.test.ts ]
Error: Failed to resolve import "../canvas/konva/document/canvas-layer-order" from "src/features/spaces/__tests__/konva-canvas-presentation-document.test.ts". Does the file exist?
  Plugin: vite:import-analysis
  File: /home/runner/work/knovum/knovum/src/features/spaces/__tests__/konva-canvas-presentation-document.test.ts:6:35
  3  |  import { createCanvasSelectionSnapshot } from "../canvas/document/canvas-selection-snapshot";
  4  |  import { CanvasDocumentModel } from "../canvas/konva/document/canvas-document-model";
  5  |  import { reorderCanvasNodes } from "../canvas/konva/document/canvas-layer-order";
     |                                      ^
  6  |  import { createSpaceNotePayload } from "../editor/tiptap-content";
  7  |  describe("Konva canvas presentation document", () => {
 ❯ TransformPluginContext._formatLog node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:31147:39
 ❯ TransformPluginContext.error node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:31144:14
 ❯ normalizeUrl node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:28083:18
 ❯ node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:28153:30
 ❯ TransformPluginContext.transform node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:28119:4
 ❯ EnvironmentPluginContainer.transform node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:30932:14
 ❯ loadAndTransform node_modules/.pnpm/vite@8.2.2_@types+node@26.4.1_esbuild@0.28.2_jiti@2.7.0_tsx@4.23.12_yaml@2.9.0/node_modules/vite/dist/node/chunks/node.js:20671:26

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[2/119]⎯


⎯⎯⎯⎯⎯⎯ Failed Tests 110 ⎯⎯⎯⎯⎯⎯

 FAIL  src/features/spaces/__tests__/canvas-arrangement-operation.test.ts > whole-operation arrangement through the canvas port > distributes three Section roots and carries every member by its parent's delta
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ Module.movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:27:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[3/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-arrangement-operation.test.ts > whole-operation arrangement through the canvas port > gives ELK only operation roots and expands its resulting movement to members
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ Module.movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:39:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[4/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-arrangement-operation.test.ts > whole-operation arrangement through the canvas port > does not persist a partial alignment when any selected root is locked
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ Module.movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:60:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[5/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-arrangement-operation.test.ts > whole-operation arrangement through the canvas port > leaves the document unchanged if automatic layout computation fails
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ Module.movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-arrangement-operation.test.ts:68:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[6/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-document-rebase.test.ts > canvas document rebase > merges independent relation presentation changes by relation identity
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ rebaseCanvasDocument src/features/spaces/canvas/host/canvas-document-rebase.ts:16:24
     14|       version: 1,
     15|       items: merge(
     16|         base.structure.items,
       |                        ^
     17|         committed.structure.items,
     18|         pending.structure.items,
 ❯ src/features/spaces/__tests__/canvas-document-rebase.test.ts:33:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[7/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-document-workspace.test.ts > Space-owned document workspace > persists the captured camera at the committed revision even after detachment
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ rebaseCanvasDocument src/features/spaces/canvas/host/canvas-document-rebase.ts:16:24
     14|       version: 1,
     15|       items: merge(
     16|         base.structure.items,
       |                        ^
     17|         committed.structure.items,
     18|         pending.structure.items,
 ❯ src/features/spaces/canvas/host/canvas-document-workspace.ts:61:13

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[8/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-document.test.ts > CanvasDocument v1 > accepts engine-neutral layout state without domain content
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
 ❯ parseSpaceEditorDocument src/features/spaces/canvas/contract/canvas-document.ts:79:36
     77| }
     78| export function parseSpaceEditorDocument(value: unknown): SpaceEditorD…
     79|   return spaceEditorDocumentSchema.parse(value);
       |                                    ^
     80| }
     81| export function emptySpaceEditorDocument(): SpaceEditorDocument {
 ❯ src/features/spaces/__tests__/canvas-document.test.ts:30:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[9/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-document.test.ts > CanvasDocument v1 > rejects duplicate nodes
AssertionError: expected [Function] to throw error matching /unique/u but got '[\n  {\n    "expected": "object",\n  …'

- Expected:
/unique/u

+ Received:
"[
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"structure\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  },
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"layout\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  },
  {
    \"code\": \"unrecognized_keys\",
    \"keys\": [
      \"nodes\"
    ],
    \"path\": [],
    \"message\": \"Unrecognized key: \\\"nodes\\\"\"
  }
]"

 ❯ src/features/spaces/__tests__/canvas-document.test.ts:45:84
     43|   it("rejects duplicate nodes", () => {
     44|     const node = document().nodes[0];
     45|     expect(() => parseSpaceEditorDocument({ ...document(), nodes: [nod…
       |                                                                                    ^
     46|   });
     47|

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[10/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-document.test.ts > CanvasDocument v1 > uses the same unsigned 32-bit layer range as the Rust contract
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
 ❯ parseSpaceEditorDocument src/features/spaces/canvas/contract/canvas-document.ts:79:36
     77| }
     78| export function parseSpaceEditorDocument(value: unknown): SpaceEditorD…
     79|   return spaceEditorDocumentSchema.parse(value);
       |                                    ^
     80| }
     81| export function emptySpaceEditorDocument(): SpaceEditorDocument {
 ❯ src/features/spaces/__tests__/canvas-document.test.ts:55:7

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[11/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-document.test.ts > CanvasDocument v1 > accepts only the current semantic appearance contract
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
 ❯ parseSpaceEditorDocument src/features/spaces/canvas/contract/canvas-document.ts:79:36
     77| }
     78| export function parseSpaceEditorDocument(value: unknown): SpaceEditorD…
     79|   return spaceEditorDocumentSchema.parse(value);
       |                                    ^
     80| }
     81| export function emptySpaceEditorDocument(): SpaceEditorDocument {
 ❯ src/features/spaces/__tests__/canvas-document.test.ts:79:7

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[12/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-document.test.ts > CanvasDocument v1 > stores only non-default relation presentation overrides
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
 ❯ parseSpaceEditorDocument src/features/spaces/canvas/contract/canvas-document.ts:79:36
     77| }
     78| export function parseSpaceEditorDocument(value: unknown): SpaceEditorD…
     79|   return spaceEditorDocumentSchema.parse(value);
       |                                    ^
     80| }
     81| export function emptySpaceEditorDocument(): SpaceEditorDocument {
 ❯ src/features/spaces/__tests__/canvas-document.test.ts:144:7

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[13/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-camera.test.ts > committing geometry while the camera remains live > keeps camera autosave free of uncommitted geometry (rejected=false)
 FAIL  src/features/spaces/__tests__/canvas-gesture-camera.test.ts > committing geometry while the camera remains live > keeps camera autosave free of uncommitted geometry (rejected=true)
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-gesture-camera.test.ts:11:23

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[14/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-camera.test.ts > committing geometry while the camera remains live > preserves camera changes while undo is committing
 FAIL  src/features/spaces/__tests__/canvas-gesture-camera.test.ts > committing geometry while the camera remains live > preserves camera changes while redo is committing
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-gesture-camera.test.ts:51:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[15/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-repository.test.ts > canvas gesture repository > constructs the strict persistence DTO without leaking gesture-only fields
AssertionError: expected "vi.fn()" to be called with arguments: [ …(2) ]

Received:

  1st vi.fn() call:

@@ -1,11 +1,10 @@
  [
    "plugin:spaces|commit_canvas_gesture",
    {
      "request": {
        "baseRevision": 3,
-       "changes": [],
        "document": {
          "nodes": [],
          "version": 1,
          "viewport": {
            "x": 0,


Number of calls: 1

 ❯ src/features/spaces/__tests__/canvas-gesture-repository.test.ts:27:26
     25|     await commitCanvasGesture(mutation, "space", []);
     26|
     27|     expect(tauri.invoke).toHaveBeenCalledWith("plugin:spaces|commit_ca…
       |                          ^
     28|       request: { spaceId: "space", baseRevision: 3, document, changes:…
     29|     });

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[16/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-session.test.ts > canvas gesture session > commits keyboard movement without implicitly changing Section membership
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ harness src/features/spaces/__tests__/canvas-gesture-session.test.ts:141:41
 ❯ src/features/spaces/__tests__/canvas-gesture-session.test.ts:29:31

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[17/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-session.test.ts > canvas gesture session > serializes rapid keyboard nudges from the latest committed document
AssertionError: expected "vi.fn()" to be called 1 times, but got 0 times
 ❯ src/features/spaces/__tests__/canvas-gesture-session.test.ts:62:36
     60|     await Promise.resolve();
     61|     expect(session.busy.value).toBe(true);
     62|     expect(events.onCommitGesture).toHaveBeenCalledTimes(1);
       |                                    ^
     63|     const firstDocument = vi.mocked(events.onCommitGesture).mock.calls…
     64|     resolveFirst?.({

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[18/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-session.test.ts > canvas gesture session > calculates queued frame mutations from the latest committed document
TypeError: Cannot read properties of undefined (reading '0')
 ❯ movedFrame src/features/spaces/__tests__/canvas-gesture-session.test.ts:136:33
    134|
    135| function movedFrame(model: CanvasDocumentModel, offsetX: number) {
    136|   const layout = model.document.nodes[0]!;
       |                                 ^
    137|   return { nodeId: layout.nodeId, frame: { ...layout.frame, x: layout.…
    138| }
 ❯ src/features/spaces/__tests__/canvas-gesture-session.test.ts:82:51
 ❯ Object.runFrameMutation src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts:117:27
 ❯ src/features/spaces/__tests__/canvas-gesture-session.test.ts:82:27

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[19/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-session.test.ts > canvas gesture session > keeps geometry interaction busy while an asynchronous frame calculation is pending
AssertionError: expected false to be true // Object.is equality

- Expected
+ Received

- true
+ false

 ❯ src/features/spaces/__tests__/canvas-gesture-session.test.ts:100:32
     98|     const mutation = session.runFrameMutation(() => calculation);
     99|     await Promise.resolve();
    100|     expect(session.busy.value).toBe(true);
       |                                ^
    101|     expect(events.onCommitGesture).not.toHaveBeenCalled();
    102|     resolveCalculation?.([movedFrame(model, 10)]);

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[20/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-session.test.ts > canvas gesture session > keeps a queued nudge bound to the selection that received the command
 FAIL  src/features/spaces/__tests__/konva-editor-port.test.ts > Konva editor port > routes clipboard operations through product ports and uses the last canvas pointer
TypeError: this.events.executeCommand is not a function
 ❯ src/features/spaces/canvas/konva/document/canvas-document-model.ts:65:40
     63|   executeNative(command: SpaceEditorCommand, baseDocument?: SpaceEdito…
     64|     return this.enqueue(async () => {
     65|       const result = await this.events.executeCommand({
       |                                        ^
     66|         command,
     67|         mutationId: crypto.randomUUID(),

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[21/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-update.test.ts > canvas gesture layout updates > commits frame and typography scale as one layout mutation
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ applyEditorFrames src/features/spaces/canvas/document/editor-frame-edit.ts:23:19
 ❯ applyCanvasGestureUpdates src/features/spaces/canvas/konva/document/canvas-document-mutations.ts:16:18
 ❯ src/features/spaces/__tests__/canvas-gesture-update.test.ts:24:18

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[22/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-update.test.ts > canvas gesture layout updates > captures absence explicitly so undo removes gesture-created appearance
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectEditorLayouts src/features/spaces/canvas/document/editor-layout-projection.ts:9:19
 ❯ captureCanvasGestureLayouts src/features/spaces/canvas/konva/document/canvas-document-mutations.ts:36:10
 ❯ src/features/spaces/__tests__/canvas-gesture-update.test.ts:38:22

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[23/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-update.test.ts > canvas gesture layout updates > preserves unchanged records without inventing absent appearance fields
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ applyEditorFrames src/features/spaces/canvas/document/editor-frame-edit.ts:23:19
 ❯ applyCanvasGestureUpdates src/features/spaces/canvas/konva/document/canvas-document-mutations.ts:16:18
 ❯ src/features/spaces/__tests__/canvas-gesture-update.test.ts:48:18

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[24/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-gesture-update.test.ts > canvas gesture layout updates > restores absence as the exact original layout shape without mutating the source
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectEditorLayouts src/features/spaces/canvas/document/editor-layout-projection.ts:9:19
 ❯ captureCanvasGestureLayouts src/features/spaces/canvas/konva/document/canvas-document-mutations.ts:36:10
 ❯ src/features/spaces/__tests__/canvas-gesture-update.test.ts:57:22

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[25/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-integration.test.ts > wired movement and persistence semantics > aligns two Sections and both member sets through the real editor port
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts:13:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[26/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-integration.test.ts > wired movement and persistence semantics > does not realign a selected member separately from its Section
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts:25:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[27/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-integration.test.ts > wired movement and persistence semantics > moves multiple Sections with keyboard input and does not rewrite memberships
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts:35:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[28/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-integration.test.ts > wired movement and persistence semantics > keeps locked descendants attached to an unlocked Section for keyboard input
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts:48:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[29/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-integration.test.ts > wired movement and persistence semantics > rejects an entire nudge against a locked operation root
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts:60:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[30/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-integration.test.ts > wired movement and persistence semantics > keeps fit separate from moving a Section and does not adopt overlapping content
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts:68:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[31/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-integration.test.ts > wired movement and persistence semantics > never sends an optimistic gesture or a rejected gesture to the independent autosave path
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts:75:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[32/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-integration.test.ts > wired movement and persistence semantics > records and undoes the entire aligned operation while preserving a newer camera
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-integration.test.ts:85:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[33/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts > movement gesture completion > commits multiple dragged Sections without separately reparenting their members
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:15:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[34/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts > movement gesture completion > still permits an explicit content drag to leave its Section
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:27:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[35/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts > movement gesture completion > does not change membership when a resize moves a content center outside its Section
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:40:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[36/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts > movement gesture completion > does not record an empty selection or a zero-offset movement
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:50:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[37/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts > movement gesture completion > rejects a missing descendant layout before any persisted keyboard movement
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ movementHarness src/features/spaces/canvas/konva/testing/canvas-movement-harness.ts:14:41
 ❯ src/features/spaces/__tests__/canvas-movement-lifecycle.test.ts:60:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[38/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement.test.ts > container movement plans > captures all selected Section members when s1 starts the drag
 FAIL  src/features/spaces/__tests__/canvas-movement.test.ts > container movement plans > captures all selected Section members when s2 starts the drag
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/canvas-movement.test.ts:11:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[39/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement.test.ts > container movement plans > copies every captured frame and retains locked descendants
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/canvas-movement.test.ts:25:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[40/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement.test.ts > container movement plans > rejects capture rather than skipping a locked selected root
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/canvas-movement.test.ts:42:18

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[41/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement.test.ts > container movement plans > applies a different offset to each whole root for alignment/distribution
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/canvas-movement.test.ts:51:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[42/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement.test.ts > container movement plans > moves a selected child only once when its Section is also selected
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/canvas-movement.test.ts:62:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[43/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement.test.ts > container movement plans > does not interpret resizing as a movement
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/canvas-movement.test.ts:71:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[44/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-movement.test.ts > container movement plans > rejects missing descendant geometry instead of silently leaving it behind
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/canvas-movement.test.ts:81:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[45/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-overview-geometry.test.ts > canvas overview geometry > fits the whole document independently of editor viewport
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ sceneAt src/features/spaces/__tests__/canvas-overview-geometry.test.ts:21:10
 ❯ src/features/spaces/__tests__/canvas-overview-geometry.test.ts:37:19

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[46/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-overview-geometry.test.ts > canvas overview geometry > includes rotated node corners
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ sceneAt src/features/spaces/__tests__/canvas-overview-geometry.test.ts:21:10
 ❯ src/features/spaces/__tests__/canvas-overview-geometry.test.ts:42:45

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[47/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-overview-geometry.test.ts > canvas overview geometry > does not depend on a mounted viewport when empty
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ src/features/spaces/__tests__/canvas-overview-geometry.test.ts:49:19

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[48/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-package-codec.test.ts > Canvas package codec > remaps node references in the neutral document
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [],
    "message": "Unrecognized key: \"nodes\""
  }
]
 ❯ parseSpaceEditorDocument src/features/spaces/canvas/contract/canvas-document.ts:79:36
     77| }
     78| export function parseSpaceEditorDocument(value: unknown): SpaceEditorD…
     79|   return spaceEditorDocumentSchema.parse(value);
       |                                    ^
     80| }
     81| export function emptySpaceEditorDocument(): SpaceEditorDocument {
 ❯ Object.remap src/features/spaces/canvas/canvas-package-codec.ts:10:22
 ❯ src/features/spaces/__tests__/canvas-package-codec.test.ts:7:38

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[49/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-persistence-history.test.ts > canvas persistence history > writes the clipboard before one atomic cut command
ReferenceError: placements is not defined
 ❯ nodesAtPlacements src/features/spaces/session/actions/canvas-selection-actions.ts:23:31
     21| export function createCanvasSelectionActions(store: SpaceStore) {
     22|   function nodesAtPlacements(transfer: CanvasSelectionTransfer): Posit…
     23|     const positions = new Map(placements.map(({ id, x, y }) => [id, { …
       |                               ^
     24|     return store.nodes.flatMap((node) => {
     25|       const position = positions.get(node.id);
 ❯ Object.cutSelection src/features/spaces/session/actions/canvas-selection-actions.ts:121:19
 ❯ src/features/spaces/__tests__/canvas-persistence-history.test.ts:46:47

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[50/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-section-gesture.test.ts > canvas section gestures > moves section members and preserves a newer viewport on undo
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/canvas-section-gesture.test.ts:19:43

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[51/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > does not install a stale load after the active space changes
TypeError: Cannot read properties of null (reading 'shapeFlag')
 ❯ getRootNodes node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7497:15
 ❯ getRootNodes node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7502:16
 ❯ getRootNodes node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7502:16
 ❯ VueWrapper.getRootNodes node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7732:16
 ❯ VueWrapper.findAllDOMElements node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7209:39
 ❯ VueWrapper.findAll node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7744:21
 ❯ VueWrapper.find node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7238:31
 ❯ VueWrapper.get node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7320:29
 ❯ src/features/spaces/__tests__/canvas-surface-host.test.ts:62:20
     60|     await flushPromises();
     61|
     62|     expect(wrapper.get('[data-testid="surface"]').text()).toBe("b");
       |                    ^
     63|   });
     64|

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[52/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > does not install a stale load after the active space changes
 FAIL  src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > rejects an in-flight commit after the active space changes
 FAIL  src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > rejects an in-flight commit after unmount
 FAIL  src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > preserves an in-flight viewport change without replacing committed node frames
 FAIL  src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > persists reconciliation instead of leaving it as an in-memory repair
 FAIL  src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > waits for the previous A flush across a keyed A to B to A remount
 FAIL  src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > waits for the previous A flush across a keyed A to B to A remount
 FAIL  src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > waits for an in-flight formal commit before remounting the same space
TypeError: Cannot destructure property 'type' of 'vnode' as it is null.
 ❯ unmount node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6703:7
 ❯ unmountComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7
 ❯ unmount node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7
 ❯ unmountComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7
 ❯ unmount node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7
 ❯ unmountComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6845:7
 ❯ unmount node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6736:7
 ❯ render node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6878:9
 ❯ Object.unmount node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4367:11
 ❯ VueWrapper.unmount node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7857:20

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[53/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > rejects an in-flight commit after the active space changes
 FAIL  src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > waits for an in-flight formal commit before remounting the same space
TypeError: Cannot read properties of null (reading 'shapeFlag')
 ❯ getRootNodes node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7497:15
 ❯ getRootNodes node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7502:16
 ❯ VueWrapper.getRootNodes node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7732:16
 ❯ VueWrapper.attachNativeEventListener node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7759:35
 ❯ new VueWrapper node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7703:14
 ❯ Object.<anonymous> node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7860:64
 ❯ createVueWrapper node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:1553:82
 ❯ node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7278:15
 ❯ VueWrapper.findAllComponents node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:7277:24

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[54/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > rejects an in-flight commit after unmount
Error: Cannot call props on an empty VueWrapper.
 ❯ Object.get node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:1479:27
 ❯ deleteEmptySelection src/features/spaces/__tests__/canvas-surface-host.test.ts:198:51
    196| function deleteEmptySelection(wrapper: ReturnType<typeof mountHost>) {
    197|   const surfaceEvents = wrapper
    198|     .findComponent({ name: "KonvaCanvasSurface" })
       |                                                   ^
    199|     .props("events") as CanvasSurfaceEvents;
    200|   return surfaceEvents.onDeleteSelection({
 ❯ src/features/spaces/__tests__/canvas-surface-host.test.ts:96:25

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[55/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-surface-host.test.ts > CanvasSurfaceHost > preserves an in-flight viewport change without replacing committed node frames
Error: Cannot call props on an empty VueWrapper.
 ❯ Object.get node_modules/.pnpm/@vue+test-utils@2.5.0_@vue+compiler-dom@3.5.42_@vue+server-renderer@3.5.42_vue@3.5.42_typescript@6.0.3_/node_modules/@vue/test-utils/dist/vue-test-utils.cjs.js:1479:27
 ❯ deleteEmptySelection src/features/spaces/__tests__/canvas-surface-host.test.ts:198:51
    196| function deleteEmptySelection(wrapper: ReturnType<typeof mountHost>) {
    197|   const surfaceEvents = wrapper
    198|     .findComponent({ name: "KonvaCanvasSurface" })
       |                                                   ^
    199|     .props("events") as CanvasSurfaceEvents;
    200|   return surfaceEvents.onDeleteSelection({
 ❯ src/features/spaces/__tests__/canvas-surface-host.test.ts:115:25

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[56/119]⎯

 FAIL  src/features/spaces/__tests__/canvas-surface-remount.test.ts > CanvasSurfaceHost remount persistence > persists reconciliation instead of leaving it as an in-memory repair
AssertionError: expected "vi.fn()" to be called with arguments: [ { spaceId: 'space', …(2) } ]

Number of calls: 0

 ❯ src/features/spaces/__tests__/canvas-surface-remount.test.ts:46:29
     44|     await vi.advanceTimersByTimeAsync(301);
     45|
     46|     expect(repository.save).toHaveBeenCalledWith({
       |                             ^
     47|       spaceId: "space",
     48|       baseRevision: 4,

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[57/119]⎯

 FAIL  src/features/spaces/__tests__/clipboard-transfer.test.ts > spaces clipboard > round-trips the structured HTML marker while keeping useful plain text
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [],
    "message": "Invalid input: expected object, received array"
  }
]
 ❯ parseSpaceClipboardPayload src/features/spaces/clipboard/payload-schema.ts:53:47
     51|
     52| export function parseSpaceClipboardPayload(value: unknown) {
     53|   const payload = spaceClipboardPayloadSchema.parse(value);
       |                                               ^
     54|   if (!payload.structure.items.length || payload.structure.items.lengt…
     55|     throw new Error("A clipboard selection must contain 1 to 10,000 st…
 ❯ clipboardHtml src/features/spaces/clipboard/transfer.ts:31:33
 ❯ writeSpaceClipboard src/features/spaces/clipboard/transfer.ts:13:44
 ❯ src/features/spaces/__tests__/clipboard-transfer.test.ts:27:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[58/119]⎯

 FAIL  src/features/spaces/__tests__/clipboard-transfer.test.ts > spaces clipboard > copies reactive node data without structured-clone failures
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [],
    "message": "Invalid input: expected object, received array"
  }
]
 ❯ parseSpaceClipboardPayload src/features/spaces/clipboard/payload-schema.ts:53:47
     51|
     52| export function parseSpaceClipboardPayload(value: unknown) {
     53|   const payload = spaceClipboardPayloadSchema.parse(value);
       |                                               ^
     54|   if (!payload.structure.items.length || payload.structure.items.lengt…
     55|     throw new Error("A clipboard selection must contain 1 to 10,000 st…
 ❯ clipboardHtml src/features/spaces/clipboard/transfer.ts:31:33
 ❯ writeSpaceClipboard src/features/spaces/clipboard/transfer.ts:13:44
 ❯ src/features/spaces/__tests__/clipboard-transfer.test.ts:44:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[59/119]⎯

 FAIL  src/features/spaces/__tests__/clipboard-transfer.test.ts > spaces clipboard > rejects any clipboard payload that is not version 1
ZodError: [
  {
    "code": "invalid_value",
    "values": [
      1
    ],
    "path": [
      "version"
    ],
    "message": "Invalid input: expected 1"
  },
  {
    "code": "invalid_value",
    "values": [
      "space-selection"
    ],
    "path": [
      "kind"
    ],
    "message": "Invalid input: expected \"space-selection\""
  },
  {
    "expected": "string",
    "code": "invalid_type",
    "path": [
      "sourceSpaceId"
    ],
    "message": "Invalid input: expected string, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "position"
    ],
    "path": [
      "nodes",
      0
    ],
    "message": "Unrecognized key: \"position\""
  },
  {
    "expected": "array",
    "code": "invalid_type",
    "path": [
      "objectTypes"
    ],
    "message": "Invalid input: expected array, received undefined"
  },
  {
    "expected": "array",
    "code": "invalid_type",
    "path": [
      "relations"
    ],
    "message": "Invalid input: expected array, received undefined"
  }
]
 ❯ parseSpaceClipboardPayload src/features/spaces/clipboard/payload-schema.ts:53:47
     51|
     52| export function parseSpaceClipboardPayload(value: unknown) {
     53|   const payload = spaceClipboardPayloadSchema.parse(value);
       |                                               ^
     54|   if (!payload.structure.items.length || payload.structure.items.lengt…
     55|     throw new Error("A clipboard selection must contain 1 to 10,000 st…
 ❯ readSpaceClipboardHtml src/features/spaces/clipboard/transfer.ts:28:10
 ❯ readSpaceClipboard src/features/spaces/clipboard/transfer.ts:18:10
 ❯ src/features/spaces/__tests__/clipboard-transfer.test.ts:61:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[60/119]⎯

 FAIL  src/features/spaces/__tests__/derivation-schema.test.ts > knowledge video package metadata > accepts complete generated video metadata
AssertionError: expected false to be true // Object.is equality

- Expected
+ Received

- true
+ false

 ❯ src/features/spaces/__tests__/derivation-schema.test.ts:32:76
     30| describe("knowledge video package metadata", () => {
     31|   it("accepts complete generated video metadata", () => {
     32|     expect(derivationSchema.safeParse(generatedVideoDerivation()).succ…
       |                                                                            ^
     33|   });
     34|

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[61/119]⎯

 FAIL  src/features/spaces/__tests__/derivation-schema.test.ts > document export package metadata > accepts a completed Markdown PDF export without provider metadata
AssertionError: expected false to be true // Object.is equality

- Expected
+ Received

- true
+ false

 ❯ src/features/spaces/__tests__/derivation-schema.test.ts:64:7
     62|         completedAt: 2,
     63|       }).success,
     64|     ).toBe(true);
       |       ^
     65|   });
     66| });

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[62/119]⎯

 FAIL  src/features/spaces/__tests__/knowledge-location.test.ts > Space knowledge locations > checks source and editor revisions against the correct authority
AssertionError: expected false to be true // Object.is equality

- Expected
+ Received

- true
+ false

 ❯ src/features/spaces/__tests__/knowledge-location.test.ts:32:78
     30|     } as const;
     31|     expect(isLocatorCurrent(locator({ kind: "pdf", page: 1 }), data)).…
     32|     expect(isLocatorCurrent(locator({ kind: "node" }, "node:editor"), …
       |                                                                              ^
     33|     expect(isLocatorCurrent(locator({ kind: "pdf", page: 1 }, "sha256:…
     34|   });

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[63/119]⎯

 FAIL  src/features/spaces/__tests__/konva-canvas-document.test.ts > Konva canvas document synchronization > creates canonical frames from domain nodes
TypeError: Cannot destructure property 'width' of 'spaceNodeSizePolicy(...)' as it is undefined.
 ❯ src/features/spaces/canvas/document/canvas-document-sync.ts:36:19
     34|         ...document.layout.items,
     35|         ...nodes.map((node) => {
     36|           const { width, height } = spaceNodeSizePolicy(node.data);
       |                   ^
     37|           return {
     38|             kind: "content",
 ❯ appendEditorContent src/features/spaces/canvas/document/canvas-document-sync.ts:35:18
 ❯ createCanvasDocument src/features/spaces/canvas/document/canvas-document-sync.ts:50:10
 ❯ src/features/spaces/__tests__/konva-canvas-document.test.ts:32:22

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[64/119]⎯

 FAIL  src/features/spaces/__tests__/konva-canvas-document.test.ts > Konva canvas document synchronization > adopts intrinsic media geometry until the persisted frame becomes authoritative
TypeError: reconcileCanvasDocument is not a function
 ❯ src/features/spaces/__tests__/konva-canvas-document.test.ts:50:23
     48|     const enriched = { ...pending, data: imagePayload(1600, 900) };
     49|
     50|     const intrinsic = reconcileCanvasDocument(document, [enriched]);
       |                       ^
     51|     expect(intrinsic.nodes[0]).toMatchObject({
     52|       sizeAuthority: "content",

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[65/119]⎯

 FAIL  src/features/spaces/__tests__/konva-canvas-document.test.ts > Konva canvas document synchronization > preserves authored frames while adding and removing domain nodes
TypeError: Cannot read properties of undefined (reading '0')
 ❯ src/features/spaces/__tests__/konva-canvas-document.test.ts:71:14
     69|   it("preserves authored frames while adding and removing domain nodes…
     70|     const document = createCanvasDocument([node("a", 0, 0), node("stal…
     71|     document.nodes[0]!.frame.x = 720;
       |              ^
     72|
     73|     const next = reconcileCanvasDocument(document, [node("a", 0, 0), n…

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[66/119]⎯

 FAIL  src/features/spaces/__tests__/konva-canvas-document.test.ts > Konva canvas document synchronization > rejects content authority on nodes without content-sized geometry
TypeError: Cannot read properties of undefined (reading '0')
 ❯ src/features/spaces/__tests__/konva-canvas-document.test.ts:83:14
     81|     const note = node("note", 0, 0);
     82|     const document = createCanvasDocument([note]);
     83|     document.nodes[0]!.sizeAuthority = "content";
       |              ^
     84|
     85|     expect(() => reconcileCanvasDocument(document, [note])).toThrow(

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[67/119]⎯

 FAIL  src/features/spaces/__tests__/konva-canvas-document.test.ts > Konva canvas document synchronization > preserves document and layout identities when domain geometry is unchanged
TypeError: reconcileCanvasDocument is not a function
 ❯ src/features/spaces/__tests__/konva-canvas-document.test.ts:94:12
     92|     const document = createCanvasDocument(nodes);
     93|
     94|     expect(reconcileCanvasDocument(document, nodes)).toBe(document);
       |            ^
     95|   });
     96|

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[68/119]⎯

 FAIL  src/features/spaces/__tests__/konva-canvas-document.test.ts > Konva canvas document synchronization > keeps viewport changes made after an import when undoing imported content
TypeError: model.document.nodes is not iterable
 ❯ src/features/spaces/__tests__/konva-canvas-document.test.ts:107:27
    105|       ...model.document,
    106|       nodes: [
    107|         ...model.document.nodes,
       |                           ^
    108|         {
    109|           nodeId: "b",

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[69/119]⎯

 FAIL  src/features/spaces/__tests__/konva-editor-port.test.ts > Konva editor port > commits node and semantic relation deletion through one host transaction
TypeError: options.scopeGroupId is not a function
 ❯ Object.snapshot [as getSelectionSnapshot] src/features/spaces/canvas/konva/create-canvas-editor-port.ts:37:27
     35|       options.relationBounds(),
     36|     ),
     37|     scopeGroupId: options.scopeGroupId(),
       |                           ^
     38|   });
     39|   return {
 ❯ src/features/spaces/__tests__/konva-editor-port.test.ts:135:17

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[70/119]⎯

 FAIL  src/features/spaces/__tests__/konva-node-gesture-callbacks.test.ts > Konva node gesture callbacks > keeps event callbacks bound to their gesture controller
TypeError: this.options.document is not a function
 ❯ beginDrag src/features/spaces/canvas/konva/interaction/node-gesture-controller.ts:41:20
     39|     const captured = captureCanvasNodeDragState(
     40|       this.options.layouts(),
     41|       this.options.document(),
       |                    ^
     42|       this.options.selectedNodeIds(),
     43|       primaryId,
 ❯ src/features/spaces/__tests__/konva-node-gesture-callbacks.test.ts:37:5

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[71/119]⎯

 FAIL  src/features/spaces/__tests__/konva-node-gesture.test.ts > Konva node gestures > moves a section and its domain members in one frame commit
TypeError: this.options.document is not a function
 ❯ NodeGestureController.beginDrag src/features/spaces/canvas/konva/interaction/node-gesture-controller.ts:41:20
     39|     const captured = captureCanvasNodeDragState(
     40|       this.options.layouts(),
     41|       this.options.document(),
       |                    ^
     42|       this.options.selectedNodeIds(),
     43|       primaryId,
 ❯ src/features/spaces/__tests__/konva-node-gesture.test.ts:42:14

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[72/119]⎯

 FAIL  src/features/spaces/__tests__/konva-node-gesture.test.ts > Konva node gestures > rejects the complete drag when a selected operation root is locked
TypeError: this.options.document is not a function
 ❯ NodeGestureController.beginDrag src/features/spaces/canvas/konva/interaction/node-gesture-controller.ts:41:20
     39|     const captured = captureCanvasNodeDragState(
     40|       this.options.layouts(),
     41|       this.options.document(),
       |                    ^
     42|       this.options.selectedNodeIds(),
     43|       primaryId,
 ❯ src/features/spaces/__tests__/konva-node-gesture.test.ts:75:21

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[73/119]⎯

 FAIL  src/features/spaces/__tests__/konva-node-gesture.test.ts > Konva node gestures > normalizes the native Transformer scale when the gesture ends
AssertionError: expected 1.5 to be 1 // Object.is equality

- Expected
+ Received

- 1
+ 1.5

 ❯ src/features/spaces/__tests__/konva-node-gesture.test.ts:101:34
     99|     expect(updates[0]?.frame).toEqual({ x: 20, y: 30, width: 450, heig…
    100|     expect(updates[0]?.sizeAuthority).toBe("frame");
    101|     expect(transformed.scaleX()).toBe(1);
       |                                  ^
    102|     expect(transformed.scaleY()).toBe(1);
    103|   });

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[74/119]⎯

 FAIL  src/features/spaces/__tests__/konva-rich-node-interactions.test.ts > Konva rich node interactions > persists direct note editing and refuses editing for locked notes
TypeError: Cannot read properties of undefined (reading 'icon')
 ❯ src/features/spaces/presentation/canvas/nodes/SpaceCanvasTextAlignmentMenu.vue:28:38
     26|     <template #trigger>
     27|       <UiIconButton :label="t(`spaces.canvas.textAlignments.${selected…
     28|         <component :is="selectedItem.icon" class="size-(--knovum-icon-…
       |                                      ^
     29|       </UiIconButton>
     30|     </template>
 ❯ renderFnWithContext node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13
 ❯ renderSlot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55
 ❯ src/shared/ui/UiIconButton.vue:49:7
 ❯ renderFnWithContext node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13
 ❯ renderSlot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55
 ❯ src/shared/ui/UiTooltip.vue:34:32
 ❯ renderFnWithContext node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13
 ❯ renderSlot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55
 ❯ node_modules/.pnpm/reka-ui@2.10.4_vue@3.5.42_typescript@6.0.3_/node_modules/reka-ui/src/Tooltip/TooltipTrigger.vue:119:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[75/119]⎯

 FAIL  src/features/spaces/__tests__/konva-scene-projection.test.ts > Konva scene projection > always paints section backgrounds behind their members
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ src/features/spaces/__tests__/konva-scene-projection.test.ts:39:19

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[76/119]⎯

 FAIL  src/features/spaces/__tests__/konva-scene-projection.test.ts > Konva scene projection > projects collapsed sections without mutating stored geometry or rendering members
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ src/features/spaces/__tests__/konva-scene-projection.test.ts:72:19

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[77/119]⎯

 FAIL  src/features/spaces/__tests__/konva-scene-projection.test.ts > Konva scene projection > projects one identity for each semantic relation
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ src/features/spaces/__tests__/konva-scene-projection.test.ts:114:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[78/119]⎯

 FAIL  src/features/spaces/__tests__/konva-scene-projection.test.ts > Konva scene projection > projects parallel relations onto stable distinct curve lanes
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ src/features/spaces/__tests__/konva-scene-projection.test.ts:150:23

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[79/119]⎯

 FAIL  src/features/spaces/__tests__/konva-scene-projection.test.ts > Konva scene projection > projects external relations from collapsed members onto their section boundary
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ src/features/spaces/__tests__/konva-scene-projection.test.ts:205:25

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[80/119]⎯

 FAIL  src/features/spaces/__tests__/konva-scene-projection.test.ts > Konva scene projection > keeps hidden layout data out of the rendered scene
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectCanvasScene src/features/spaces/canvas/konva/scene/scene-projection.ts:36:19
 ❯ src/features/spaces/__tests__/konva-scene-projection.test.ts:223:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[81/119]⎯

 FAIL  src/features/spaces/__tests__/konva-selection.test.ts > Konva canvas selection > preserves a multi-selection when an already-selected node begins a drag
TypeError: options.document is not a function
 ❯ setNodeIds src/features/spaces/canvas/konva/selection/use-canvas-selection.ts:90:41
     88|   }
     89|   function setNodeIds(ids: readonly string[]) {
     90|     const tree = indexStructure(options.document().structure);
       |                                         ^
     91|     nodeIds.value = new Set(
     92|       structureOperationRoots(
 ❯ Object.selectNode src/features/spaces/canvas/konva/selection/use-canvas-selection.ts:40:5
 ❯ src/features/spaces/__tests__/konva-selection.test.ts:32:15

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[82/119]⎯

 FAIL  src/features/spaces/__tests__/konva-selection.test.ts > Konva canvas selection > removes selected nodes that leave the projected scene
TypeError: options.document is not a function
 ❯ Object.setNodeIds src/features/spaces/canvas/konva/selection/use-canvas-selection.ts:90:41
     88|   }
     89|   function setNodeIds(ids: readonly string[]) {
     90|     const tree = indexStructure(options.document().structure);
       |                                         ^
     91|     nodeIds.value = new Set(
     92|       structureOperationRoots(
 ❯ src/features/spaces/__tests__/konva-selection.test.ts:47:15

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[83/119]⎯

 FAIL  src/features/spaces/__tests__/manifest-parser.test.ts > Space package manifest parser > accepts regenerated canonical citations and detached historical snapshots
ZodError: [
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "canvasState",
      "document",
      "structure"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "expected": "object",
    "code": "invalid_type",
    "path": [
      "canvasState",
      "document",
      "layout"
    ],
    "message": "Invalid input: expected object, received undefined"
  },
  {
    "code": "unrecognized_keys",
    "keys": [
      "nodes"
    ],
    "path": [
      "canvasState",
      "document"
    ],
    "message": "Unrecognized key: \"nodes\""
  }
]
 ❯ parseSpacePackageManifest src/features/spaces/package/manifest-parser.ts:25:35
     23|     throw new Error(`Unsupported Space package version: ${String(sourc…
     24|   }
     25|   const manifest = manifestSchema.parse(source);
       |                                   ^
     26|   canvasCodec.validate(manifest.canvasState);
     27|   validateManifestOwnership(manifest);
 ❯ src/features/spaces/__tests__/manifest-parser.test.ts:89:20

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[84/119]⎯

 FAIL  src/features/spaces/__tests__/manifest-parser.test.ts > Space package manifest parser > accepts All-Spaces threads with detached external citation snapshots
AssertionError: expected [Function] to not throw an error but '[\n  {\n    "expected": "object",\n  …' was thrown

- Expected:
undefined

+ Received:
"[
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"canvasState\",
      \"document\",
      \"structure\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  },
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"canvasState\",
      \"document\",
      \"layout\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  },
  {
    \"code\": \"unrecognized_keys\",
    \"keys\": [
      \"nodes\"
    ],
    \"path\": [
      \"canvasState\",
      \"document\"
    ],
    \"message\": \"Unrecognized key: \\\"nodes\\\"\"
  }
]"

 ❯ src/features/spaces/__tests__/manifest-parser.test.ts:100:92
     98|     const value = manifest();
     99|     value.aiThreads[0]!.scope = "all-spaces";
    100|     expect(() => parseSpacePackageManifest(JSON.stringify(value), canv…
       |                                                                                            ^
    101|   });
    102|

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[85/119]⎯

 FAIL  src/features/spaces/__tests__/manifest-parser.test.ts > Space package manifest parser > rejects a canonical citation whose snapshot revision is stale
AssertionError: expected [Function] to throw error matching /canonical citation revision/u but got '[\n  {\n    "expected": "object",\n  …'

- Expected:
/canonical citation revision/u

+ Received:
"[
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"canvasState\",
      \"document\",
      \"structure\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  },
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"canvasState\",
      \"document\",
      \"layout\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  },
  {
    \"code\": \"unrecognized_keys\",
    \"keys\": [
      \"nodes\"
    ],
    \"path\": [
      \"canvasState\",
      \"document\"
    ],
    \"message\": \"Unrecognized key: \\\"nodes\\\"\"
  }
]"

 ❯ src/features/spaces/__tests__/manifest-parser.test.ts:107:88
    105|     value.aiMessages[0]!.citations[0]!.sourceRevision = "node1:stale";
    106|     value.aiMessages[0]!.citations[0]!.locator.sourceRevision = "node1…
    107|     expect(() => parseSpacePackageManifest(JSON.stringify(value), canv…
       |                                                                                        ^
    108|       /canonical citation revision/u,
    109|     );

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[86/119]⎯

 FAIL  src/features/spaces/__tests__/manifest-parser.test.ts > Space package manifest parser > rejects derivations that reference unavailable nodes
AssertionError: expected [Function] to throw error matching /unavailable node/u but got '[\n  {\n    "expected": "object",\n  …'

- Expected:
/unavailable node/u

+ Received:
"[
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"canvasState\",
      \"document\",
      \"structure\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  },
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"canvasState\",
      \"document\",
      \"layout\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  },
  {
    \"code\": \"unrecognized_keys\",
    \"keys\": [
      \"nodes\"
    ],
    \"path\": [
      \"canvasState\",
      \"document\"
    ],
    \"message\": \"Unrecognized key: \\\"nodes\\\"\"
  },
  {
    \"expected\": \"string\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"derivations\",
      0,
      \"inputs\",
      0,
      \"sourceIdentity\"
    ],
    \"message\": \"Invalid input: expected string, received undefined\"
  },
  {
    \"expected\": \"object\",
    \"code\": \"invalid_type\",
    \"path\": [
      \"derivations\",
      0,
      \"inputs\",
      0,
      \"sourceSnapshot\"
    ],
    \"message\": \"Invalid input: expected object, received undefined\"
  }
]"

 ❯ src/features/spaces/__tests__/manifest-parser.test.ts:128:88
    126|       completedAt: 2,
    127|     });
    128|     expect(() => parseSpacePackageManifest(JSON.stringify(value), canv…
       |                                                                                        ^
    129|       /unavailable node/u,
    130|     );

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[87/119]⎯

 FAIL  src/features/spaces/__tests__/node-content-height-batch.test.ts > node content height batch > fits text height in both directions without creating history
AssertionError: expected 220 to be 80 // Object.is equality

- Expected
+ Received

- 80
+ 220

 ❯ src/features/spaces/__tests__/node-content-height-batch.test.ts:31:51
     29|     contentHeight.fit("text", measurement(80));
     30|     await Promise.resolve();
     31|     expect(model.document.nodes[0]?.frame.height).toBe(80);
       |                                                   ^
     32|     contentHeight.fit("text", measurement(48));
     33|     contentHeight.fit("text", measurement(32));

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[88/119]⎯

 FAIL  src/features/spaces/__tests__/node-content-height-batch.test.ts > node content height batch > defers content layout while a direct resize owns the frame
AssertionError: expected 220 to be 96 // Object.is equality

- Expected
+ Received

- 96
+ 220

 ❯ src/features/spaces/__tests__/node-content-height-batch.test.ts:79:51
     77|     contentHeight.setPaused(false);
     78|     await Promise.resolve();
     79|     expect(model.document.nodes[0]?.frame.height).toBe(96);
       |                                                   ^
     80|   });
     81| });

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[89/119]⎯

 FAIL  src/features/spaces/__tests__/node-size.test.ts > spaces node size policy > gives section a semantic desktop default
TypeError: Cannot read properties of undefined (reading 'width')
 ❯ src/features/spaces/__tests__/node-size.test.ts:44:28
     42|   ] as const)("gives %s a semantic desktop default", (kind, width, hei…
     43|     const policy = spaceNodeSizePolicy({ kind, title: kind } as SpaceN…
     44|     expect({ width: policy.width, height: policy.height }).toEqual({ w…
       |                            ^
     45|   });
     46|

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[90/119]⎯

 FAIL  src/features/spaces/__tests__/node-size.test.ts > spaces node size policy > uses the rendered compact geometry for collapsed sections
TypeError: Cannot read properties of undefined (reading 'resizeMode')
 ❯ spaceNodeDimensions src/features/spaces/canvas/node-size.ts:53:12
     51| export function spaceNodeDimensions(data: SpaceNodePayload) {
     52|   const size = spaceNodeSizePolicy(data);
     53|   if (size.resizeMode === "fixed") return { width: size.width, height:…
       |            ^
     54|   return { width: size.width, height: size.height };
     55| }
 ❯ src/features/spaces/__tests__/node-size.test.ts:77:7

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[91/119]⎯

 FAIL  src/features/spaces/__tests__/node-size.test.ts > spaces node size policy > keeps document and section geometry independently resizable
TypeError: Cannot convert undefined or null to object
 ❯ pick node_modules/.pnpm/es-toolkit@1.52.0/node_modules/es-toolkit/dist/object/pick.mjs:23:14
 ❯ spaceNodeResizePolicy src/features/spaces/canvas/node-size.ts:71:10
     69|
     70| export function spaceNodeResizePolicy(data: SpaceNodePayload) {
     71|   return pick(spaceNodeSizePolicy(data), [
       |          ^
     72|     "minWidth",
     73|     "minHeight",
 ❯ src/features/spaces/__tests__/node-size.test.ts:131:14

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[92/119]⎯

 FAIL  src/features/spaces/__tests__/node.test.ts > createSpaceNode > rejects current section geometry in domain data
AssertionError: expected [Function] to throw error including 'Unrecognized keys' but got '[\n  {\n    "code": "invalid_union",\…'

- Expected
+ Received

- Unrecognized keys
+ [
+   {
+     "code": "invalid_union",
+     "errors": [],
+     "note": "No matching discriminator",
+     "discriminator": "kind",
+     "options": [
+       "note",
+       "text",
+       "markdown",
+       "subtitle",
+       "website",
+       "image",
+       "video",
+       "audio",
+       "file",
+       "code",
+       "credential",
+       "object",
+       "portal"
+     ],
+     "path": [
+       "kind"
+     ],
+     "message": "Invalid discriminator value. Expected 'note' | 'text' | 'markdown' | 'subtitle' | 'website' | 'image' | 'video' | 'audio' | 'file' | 'code' | 'credential' | 'object' | 'portal'"
+   }
+ ]

 ❯ src/features/spaces/__tests__/node.test.ts:23:7
     21|     expect(() =>
     22|       validateSpaceNodePayload({ kind: "section", title: "Reference", …
     23|     ).toThrow("Unrecognized keys");
       |       ^
     24|   });
     25|

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[93/119]⎯

 FAIL  src/features/spaces/__tests__/selection-actions.test.ts > spaces organization > allows a Section itself to be locked
Error: The Space editor is not ready for an organizational command
 ❯ editor src/features/spaces/organization/selection-actions.ts:8:23
      6|   function editor() {
      7|     const value = currentCanvasEditor();
      8|     if (!value) throw new Error("The Space editor is not ready for an …
       |                       ^
      9|     return value;
     10|   }
 ❯ Object.setLocked src/features/spaces/organization/selection-actions.ts:26:7
 ❯ src/features/spaces/__tests__/selection-actions.test.ts:52:24

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[94/119]⎯

 FAIL  src/features/spaces/__tests__/selection-actions.test.ts > deleteSelection > removes selected relations before nodes and undoes both in one step
Error: The Space editor is not ready for an organizational command
 ❯ editor src/features/spaces/organization/selection-actions.ts:8:23
      6|   function editor() {
      7|     const value = currentCanvasEditor();
      8|     if (!value) throw new Error("The Space editor is not ready for an …
       |                       ^
      9|     return value;
     10|   }
 ❯ Object.deleteSelection src/features/spaces/organization/selection-actions.ts:28:7
 ❯ src/features/spaces/__tests__/selection-actions.test.ts:112:24

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[95/119]⎯

 FAIL  src/features/spaces/__tests__/selection-actions.test.ts > deleteSelection > deletes a selected relation on its own
Error: The Space editor is not ready for an organizational command
 ❯ editor src/features/spaces/organization/selection-actions.ts:8:23
      6|   function editor() {
      7|     const value = currentCanvasEditor();
      8|     if (!value) throw new Error("The Space editor is not ready for an …
       |                       ^
      9|     return value;
     10|   }
 ❯ Object.deleteSelection src/features/spaces/organization/selection-actions.ts:28:7
 ❯ src/features/spaces/__tests__/selection-actions.test.ts:123:24

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[96/119]⎯

 FAIL  src/features/spaces/__tests__/selection-product-state.test.ts > canvasSelectionProductState > exposes semantic product actions without duplicating engine arrangement actions
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ structureSelectionEligibility src/features/spaces/organization/structure/structure-eligibility.ts:6:16
 ❯ canvasSelectionProductState src/features/spaces/presentation/canvas/selection-product-state.ts:15:17
 ❯ src/features/spaces/__tests__/selection-product-state.test.ts:21:19

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[97/119]⎯

 FAIL  src/features/spaces/__tests__/selection-product-state.test.ts > canvasSelectionProductState > uses section context and preserves the selected lock state
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ structureSelectionEligibility src/features/spaces/organization/structure/structure-eligibility.ts:6:16
 ❯ canvasSelectionProductState src/features/spaces/presentation/canvas/selection-product-state.ts:15:17
 ❯ src/features/spaces/__tests__/selection-product-state.test.ts:39:19

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[98/119]⎯

 FAIL  src/features/spaces/__tests__/selection-product-state.test.ts > canvasSelectionProductState > connects only an exact two-node semantic selection
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ structureSelectionEligibility src/features/spaces/organization/structure/structure-eligibility.ts:6:16
 ❯ canvasSelectionProductState src/features/spaces/presentation/canvas/selection-product-state.ts:15:17
 ❯ src/features/spaces/__tests__/selection-product-state.test.ts:49:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[99/119]⎯

 FAIL  src/features/spaces/__tests__/selection-product-state.test.ts > canvasSelectionProductState > identifies managed assets as system-preview targets
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ structureSelectionEligibility src/features/spaces/organization/structure/structure-eligibility.ts:6:16
 ❯ canvasSelectionProductState src/features/spaces/presentation/canvas/selection-product-state.ts:15:17
 ❯ src/features/spaces/__tests__/selection-product-state.test.ts:65:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[100/119]⎯

 FAIL  src/features/spaces/__tests__/space-session-actions.test.ts > openSpace > coalesces repeated clicks while the same space is loading
AssertionError: expected "vi.fn()" to be called 1 times, but got 0 times
 ❯ src/features/spaces/__tests__/space-session-actions.test.ts:78:38
     76|     const first = openSpace(state, "inbox");
     77|     const second = openSpace(state, "inbox");
     78|     expect(nodeRepository.listNodes).toHaveBeenCalledTimes(1);
       |                                      ^
     79|
     80|     release([]);

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[101/119]⎯

 FAIL  src/features/spaces/__tests__/space-session-actions.test.ts > openSpace > keeps the current canvas snapshot visible until the new Space is ready
Error: Test timed out in 5000ms.
If this is a long-running test, pass a timeout value as the last argument or configure it globally with "testTimeout".
 ❯ src/features/spaces/__tests__/space-session-actions.test.ts:85:3
     83|     expect(state.loading.value).toBe(false);
     84|   });
     85|   it("keeps the current canvas snapshot visible until the new Space is…
       |   ^
     86|     let release!: (nodes: SpaceCanvasNode[]) => void;
     87|     nodeRepository.listNodes.mockReturnValue(new Promise((resolve) => …

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[102/119]⎯

 FAIL  src/features/spaces/__tests__/space-session-actions.test.ts > openSpace > cancels a pending switch when the user returns to the active Space
AssertionError: expected false to be true // Object.is equality

- Expected
+ Received

- true
+ false

 ❯ src/features/spaces/__tests__/space-session-actions.test.ts:122:33
    120|
    121|     const pending = openSpace(state, "research");
    122|     expect(state.loading.value).toBe(true);
       |                                 ^
    123|     await openSpace(state, "inbox");
    124|     expect(state.loading.value).toBe(false);

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[103/119]⎯

 FAIL  src/features/spaces/__tests__/space-session-actions.test.ts > draft Space creation > reuses one session draft without persisting repeated new-Space actions
Error: Test timed out in 5000ms.
If this is a long-running test, pass a timeout value as the last argument or configure it globally with "testTimeout".
 ❯ src/features/spaces/__tests__/space-session-actions.test.ts:136:3
    134|   beforeEach(() => lifecycleRepository.createSpace.mockReset().mockRes…
    135|
    136|   it("reuses one session draft without persisting repeated new-Space a…
       |   ^
    137|     const state = context();
    138|

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[104/119]⎯

 FAIL  src/features/spaces/__tests__/space-session-actions.test.ts > draft Space creation > persists the draft exactly once before its first content write
Error: Test timed out in 5000ms.
If this is a long-running test, pass a timeout value as the last argument or configure it globally with "testTimeout".
 ❯ src/features/spaces/__tests__/space-session-actions.test.ts:150:3
    148|   });
    149|
    150|   it("persists the draft exactly once before its first content write",…
       |   ^
    151|     const state = context();
    152|     await createSpace(state, "Untitled Space");

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[105/119]⎯

 FAIL  src/features/spaces/__tests__/structure-contract.test.ts > Space structure contract > rejects parent cycles without recursive traversal
AssertionError: expected [Function] to throw error matching /cycle/u but got 'Sibling order must be unique and cont…'

- Expected:
/cycle/u

+ Received:
"Sibling order must be unique and contiguous"

 ❯ src/features/spaces/__tests__/structure-contract.test.ts:81:33
     79|     const source = fixture();
     80|     source.structure.items[1]!.parentId = "g";
     81|     expect(() => parse(source)).toThrow(/cycle/u);
       |                                 ^
     82|   });
     83|   it("rejects kind-mismatched layouts", () => {

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[106/119]⎯

 FAIL  src/foundation/i18n/__tests__/locale-completeness.test.ts > locale completeness > uses semantic lowerCamelCase message keys
AssertionError: expected [ …(9) ] to deeply equal []

- Expected
+ Received

- []
+ [
+   "settings.ai.agentConnections.native_cli",
+   "settings.ai.agentConnections.node_adapter",
+   "settings.ai.agentConnections.standalone_server",
+   "spaces.processing.documentWarnings.office_vector_drawings_unsupported",
+   "spaces.processing.documentWarnings.spreadsheet_image_cell_anchors_unavailable",
+   "spaces.processing.documentWarnings.spreadsheet_chart_rendering_unsupported",
+   "spaces.processing.documentWarnings.legacy_office_visual_occurrence_coverage_limited",
+   "spaces.processing.documentWarnings.legacy_spreadsheet_formulas_unavailable",
+   "spaces.processing.documentWarnings.pdf_visual_coverage_limited",
+ ]

 ❯ src/foundation/i18n/__tests__/locale-completeness.test.ts:112:72
    110|
    111|   it("uses semantic lowerCamelCase message keys", () => {
    112|     expect(nonCamelCaseKeys(en as unknown as Record<string, unknown>))…
       |                                                                        ^
    113|     expect(nonCamelCaseKeys(zhCN as unknown as Record<string, unknown>…
    114|   });

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[107/119]⎯

⎯⎯⎯⎯⎯⎯ Unhandled Errors ⎯⎯⎯⎯⎯⎯

Vitest caught 10 unhandled errors during the test run.
This might cause false positive tests. Resolve unhandled errors to make sure your tests are not affected.

⎯⎯⎯⎯ Unhandled Rejection ⎯⎯⎯⎯⎯
TypeError: Cannot read properties of undefined (reading '0')
 ❯ Proxy.render node_modules/.pnpm/vue@3.5.42_typescript@6.0.3/node_modules/vue/dist/vue.cjs.js:71:18
 ❯ renderComponentRoot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4661:16
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6194:46
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19
 ❯ setupRenderEffect node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6329:5
 ❯ mountComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6101:7
 ❯ processComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6053:9
 ❯ patch node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:5558:11
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6281:9
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19

This error originated in "src/features/spaces/__tests__/canvas-surface-host.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.
The latest test that might've caused the error is "does not install a stale load after the active space changes". It might mean one of the following:
- The error was thrown, while Vitest was running this test.
- If the error occurred after the test had been completed, this was the last documented test before it was thrown.

⎯⎯⎯⎯ Unhandled Rejection ⎯⎯⎯⎯⎯
TypeError: Cannot read properties of undefined (reading '0')
 ❯ Proxy.render node_modules/.pnpm/vue@3.5.42_typescript@6.0.3/node_modules/vue/dist/vue.cjs.js:71:18
 ❯ renderComponentRoot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4661:16
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6194:46
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19
 ❯ setupRenderEffect node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6329:5
 ❯ mountComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6101:7
 ❯ processComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6053:9
 ❯ patch node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:5558:11
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6281:9
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19

This error originated in "src/features/spaces/__tests__/canvas-surface-host.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.
The latest test that might've caused the error is "rejects an in-flight commit after the active space changes". It might mean one of the following:
- The error was thrown, while Vitest was running this test.
- If the error occurred after the test had been completed, this was the last documented test before it was thrown.

⎯⎯⎯⎯ Unhandled Rejection ⎯⎯⎯⎯⎯
TypeError: this.events.executeCommand is not a function
 ❯ src/features/spaces/canvas/konva/document/canvas-document-model.ts:65:40
     63|   executeNative(command: SpaceEditorCommand, baseDocument?: SpaceEdito…
     64|     return this.enqueue(async () => {
     65|       const result = await this.events.executeCommand({
       |                                        ^
     66|         command,
     67|         mutationId: crypto.randomUUID(),
 ❯ processTicksAndRejections node:internal/process/task_queues:103:5

This error originated in "src/features/spaces/__tests__/canvas-gesture-session.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.

⎯⎯⎯⎯ Unhandled Rejection ⎯⎯⎯⎯⎯
TypeError: this.events.executeCommand is not a function
 ❯ src/features/spaces/canvas/konva/document/canvas-document-model.ts:65:40
     63|   executeNative(command: SpaceEditorCommand, baseDocument?: SpaceEdito…
     64|     return this.enqueue(async () => {
     65|       const result = await this.events.executeCommand({
       |                                        ^
     66|         command,
     67|         mutationId: crypto.randomUUID(),
 ❯ processTicksAndRejections node:internal/process/task_queues:103:5

This error originated in "src/features/spaces/__tests__/canvas-gesture-session.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.

⎯⎯⎯⎯ Unhandled Rejection ⎯⎯⎯⎯⎯
TypeError: Cannot read properties of undefined (reading '0')
 ❯ Proxy.render node_modules/.pnpm/vue@3.5.42_typescript@6.0.3/node_modules/vue/dist/vue.cjs.js:71:18
 ❯ renderComponentRoot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4661:16
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6194:46
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19
 ❯ setupRenderEffect node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6329:5
 ❯ mountComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6101:7
 ❯ processComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6053:9
 ❯ patch node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:5558:11
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6281:9
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19

This error originated in "src/features/spaces/__tests__/canvas-surface-remount.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.
The latest test that might've caused the error is "persists reconciliation instead of leaving it as an in-memory repair". It might mean one of the following:
- The error was thrown, while Vitest was running this test.
- If the error occurred after the test had been completed, this was the last documented test before it was thrown.

⎯⎯⎯⎯ Unhandled Rejection ⎯⎯⎯⎯⎯
TypeError: Cannot read properties of undefined (reading '0')
 ❯ Proxy.render node_modules/.pnpm/vue@3.5.42_typescript@6.0.3/node_modules/vue/dist/vue.cjs.js:71:18
 ❯ renderComponentRoot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4661:16
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6194:46
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19
 ❯ setupRenderEffect node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6329:5
 ❯ mountComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6101:7
 ❯ processComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6053:9
 ❯ patch node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:5558:11
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6281:9
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19

This error originated in "src/features/spaces/__tests__/canvas-surface-remount.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.
The latest test that might've caused the error is "waits for the previous A flush across a keyed A to B to A remount". It might mean one of the following:
- The error was thrown, while Vitest was running this test.
- If the error occurred after the test had been completed, this was the last documented test before it was thrown.

⎯⎯⎯⎯ Unhandled Rejection ⎯⎯⎯⎯⎯
TypeError: Cannot read properties of undefined (reading '0')
 ❯ Proxy.render node_modules/.pnpm/vue@3.5.42_typescript@6.0.3/node_modules/vue/dist/vue.cjs.js:71:18
 ❯ renderComponentRoot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:4661:16
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6194:46
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19
 ❯ setupRenderEffect node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6329:5
 ❯ mountComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6101:7
 ❯ processComponent node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6053:9
 ❯ patch node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:5558:11
 ❯ ReactiveEffect.componentUpdateFn [as fn] node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:6281:9
 ❯ ReactiveEffect.run node_modules/.pnpm/@vue+reactivity@3.5.42/node_modules/@vue/reactivity/dist/reactivity.cjs.js:268:19

This error originated in "src/features/spaces/__tests__/canvas-surface-remount.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.
The latest test that might've caused the error is "waits for an in-flight formal commit before remounting the same space". It might mean one of the following:
- The error was thrown, while Vitest was running this test.
- If the error occurred after the test had been completed, this was the last documented test before it was thrown.

⎯⎯⎯⎯ Unhandled Rejection ⎯⎯⎯⎯⎯
TypeError: Cannot read properties of undefined (reading 'icon')
 ❯ src/features/spaces/presentation/canvas/nodes/SpaceCanvasTextAlignmentMenu.vue:28:38
     26|     <template #trigger>
     27|       <UiIconButton :label="t(`spaces.canvas.textAlignments.${selected…
     28|         <component :is="selectedItem.icon" class="size-(--knovum-icon-…
       |                                      ^
     29|       </UiIconButton>
     30|     </template>
 ❯ renderFnWithContext node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13
 ❯ renderSlot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55
 ❯ src/shared/ui/UiIconButton.vue:49:7
 ❯ renderFnWithContext node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13
 ❯ renderSlot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55
 ❯ src/shared/ui/UiTooltip.vue:34:32
 ❯ renderFnWithContext node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:704:13
 ❯ renderSlot node_modules/.pnpm/@vue+runtime-core@3.5.42/node_modules/@vue/runtime-core/dist/runtime-core.cjs.js:3327:55
 ❯ node_modules/.pnpm/reka-ui@2.10.4_vue@3.5.42_typescript@6.0.3_/node_modules/reka-ui/dist/Tooltip/TooltipTrigger.js:94:30

This error originated in "src/features/spaces/__tests__/konva-rich-node-interactions.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.
The latest test that might've caused the error is "consumes media requests and synchronizes playback state". It might mean one of the following:
- The error was thrown, while Vitest was running this test.
- If the error occurred after the test had been completed, this was the last documented test before it was thrown.

⎯⎯⎯⎯⎯ Uncaught Exception ⎯⎯⎯⎯⎯
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectEditorLayouts src/features/spaces/canvas/document/editor-layout-projection.ts:9:19
 ❯ CanvasDocumentModel.layout src/features/spaces/canvas/konva/document/canvas-document-model.ts:57:12
 ❯ flush src/features/spaces/canvas/konva/node-content-height-batch.ts:38:28
 ❯ node:internal/process/task_queues:149:7
 ❯ AsyncResource.runInAsyncScope node:async_hooks:214:14
 ❯ AsyncResource.runMicrotask node:internal/process/task_queues:146:8
 ❯ processTicksAndRejections node:internal/process/task_queues:103:5

This error originated in "src/features/spaces/__tests__/node-content-height-batch.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.
The latest test that might've caused the error is "fits text height in both directions without creating history". It might mean one of the following:
- The error was thrown, while Vitest was running this test.
- If the error occurred after the test had been completed, this was the last documented test before it was thrown.

⎯⎯⎯⎯⎯ Uncaught Exception ⎯⎯⎯⎯⎯
TypeError: Cannot read properties of undefined (reading 'items')
 ❯ indexStructure src/features/spaces/model/structure/structure-index.ts:11:32
      9|   const items = new Map<string, StructureItem>();
     10|   const children = new Map<string | null, StructureItem[]>();
     11|   for (const item of structure.items) {
       |                                ^
     12|     if (items.has(item.id))
     13|       throw new StructureError("duplicate-identity", "Structure identi…
 ❯ indexStructureSnapshot src/features/spaces/organization/structure/structure-context.ts:11:16
 ❯ projectEditorLayouts src/features/spaces/canvas/document/editor-layout-projection.ts:9:19
 ❯ CanvasDocumentModel.layout src/features/spaces/canvas/konva/document/canvas-document-model.ts:57:12
 ❯ flush src/features/spaces/canvas/konva/node-content-height-batch.ts:38:28
 ❯ node:internal/process/task_queues:149:7
 ❯ AsyncResource.runInAsyncScope node:async_hooks:214:14
 ❯ AsyncResource.runMicrotask node:internal/process/task_queues:146:8
 ❯ processTicksAndRejections node:internal/process/task_queues:103:5

This error originated in "src/features/spaces/__tests__/node-content-height-batch.test.ts" test file. It doesn't mean the error was thrown inside the file itself, but while it was running.
The latest test that might've caused the error is "defers content layout while a direct resize owns the frame". It might mean one of the following:
- The error was thrown, while Vitest was running this test.
- If the error occurred after the test had been completed, this was the last documented test before it was thrown.
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯


 Test Files  38 failed | 115 passed (153)
      Tests  110 failed | 443 passed (553)
     Errors  10 errors
   Start at  09:17:35
   Duration  107.66s (transform 21.14s, setup 0ms, import 136.63s, tests 26.94s, environment 182.05s)

JSON report written to /home/runner/work/_temp/organization-report/tests.json

```
