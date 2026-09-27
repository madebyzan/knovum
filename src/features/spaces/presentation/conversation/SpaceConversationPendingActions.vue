<script setup lang="ts">
import type { AssistantTaskUpdate } from "../../contracts/generated/AssistantTaskUpdate";
import type { PendingApproval } from "../../contracts/generated/PendingApproval";
import SpaceConversationApprovalRequest from "./SpaceConversationApprovalRequest.vue";
import SpaceConversationPermissionRequest from "./SpaceConversationPermissionRequest.vue";

defineProps<{
  permission?: Extract<AssistantTaskUpdate, { type: "permission_requested" }> | undefined;
  decidingPermission: boolean;
  approvals: readonly PendingApproval[];
  decidingApproval: boolean;
}>();
const emit = defineEmits<{
  decidePermission: [optionId: string | null];
  decideApproval: [approvalId: string, approved: boolean];
}>();
</script>

<template>
  <SpaceConversationPermissionRequest
    v-if="permission"
    :title="permission.title"
    :options="permission.options"
    :busy="decidingPermission"
    @decide="emit('decidePermission', $event)"
  />
  <SpaceConversationApprovalRequest
    v-for="approval in approvals"
    :key="approval.id"
    :approval="approval"
    :busy="decidingApproval"
    @decide="(id, approved) => emit('decideApproval', id, approved)"
  />
</template>
