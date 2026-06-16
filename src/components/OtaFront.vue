<template>
  <div>
    <n-grid :cols="2" :x-gap="24">
      <n-grid-item>
        <n-form label-placement="top">
          <n-form-item label="前板 8018 固件 (.fota)">
            <n-input
              :value="firmwarePath"
              placeholder="选择fota固件文件"
              readonly
            >
              <template #suffix>
                <n-button size="small" @click="selectFirmware">选择</n-button>
              </template>
            </n-input>
          </n-form-item>
          <n-space vertical>
            <n-button
              type="primary"
              :disabled="!store.connected || !firmwarePath || upgrading"
              :loading="upgrading"
              @click="startUpgrade"
            >
              开始OTA升级
            </n-button>
          </n-space>
        </n-form>
      </n-grid-item>
      <n-grid-item>
        <n-card title="升级信息" size="small">
          <n-descriptions label-placement="left" :column="1">
            <n-descriptions-item label="状态">
              <n-tag :type="statusTagType" size="small">{{ statusText }}</n-tag>
            </n-descriptions-item>
            <n-descriptions-item label="已连接设备">
              {{ store.deviceAddr || "等待连接..." }}
            </n-descriptions-item>
          </n-descriptions>
        </n-card>
      </n-grid-item>
    </n-grid>
  </div>
</template>

<script setup>
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { useMessage } from "naive-ui";
import { useConfigStore } from "../stores/configStore";

const store = useConfigStore();
const message = useMessage();

const firmwarePath = ref("");
const upgrading = ref(false);
const statusText = ref("就绪");

const statusTagType = computed(() => {
  if (statusText.value.includes("完成")) return "success";
  if (statusText.value.includes("失败") || statusText.value.includes("错误"))
    return "error";
  return "info";
});

async function selectFirmware() {
  const selected = await open({
    filters: [{ name: "FOTA Firmware", extensions: ["fota"] }],
    multiple: false,
  });
  if (!selected) return;
  firmwarePath.value = selected;
  statusText.value = "已选择固件";
  message.success(`已选择: ${selected}`);
}

async function startUpgrade() {
  if (!firmwarePath.value) return;
  upgrading.value = true;
  statusText.value = "正升级前板...";
  try {
    const result = await invoke("start_ota_upgrade", {
      board: "front",
      firmwarePath: firmwarePath.value,
    });
    statusText.value = "前板OTA完成";
    message.success(result);
  } catch (e) {
    statusText.value = `失败: ${e}`;
    message.error(`前板OTA失败: ${e}`);
  } finally {
    upgrading.value = false;
  }
}
</script>
