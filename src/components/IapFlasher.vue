<template>
  <div>
    <n-grid :cols="2" :x-gap="24">
      <n-grid-item>
        <n-form label-placement="top">
          <n-form-item label="DNS 固件 (.bin)">
            <n-input
              :value="firmwarePath"
              placeholder="选择固件文件"
              readonly
            >
              <template #suffix>
                <n-button size="small" @click="selectFirmware">选择</n-button>
              </template>
            </n-input>
          </n-form-item>
          <n-space vertical>
            <n-button
              type="warning"
              :disabled="!firmwarePath || generating"
              :loading="generating"
              @click="generateFirmware"
            >
              生成升级固件 (.bin.bin)
            </n-button>
            <n-button
              type="primary"
              :disabled="!store.connected || !outputPath || sending"
              :loading="sending"
              @click="startUpgrade"
            >
              开始升级
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
            <n-descriptions-item label="升级文件">
              {{ outputName || "-" }}
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
const outputPath = ref("");
const outputName = ref("");
const generating = ref(false);
const sending = ref(false);
const statusText = ref("就绪");

const statusTagType = computed(() => {
  if (statusText.value.includes("成功") || statusText.value.includes("完成"))
    return "success";
  if (statusText.value.includes("失败") || statusText.value.includes("错误"))
    return "error";
  return "info";
});

async function selectFirmware() {
  const selected = await open({
    filters: [{ name: "DNS Firmware", extensions: ["bin"] }],
    multiple: false,
  });
  if (!selected) return;
  firmwarePath.value = selected;
  outputPath.value = "";
  outputName.value = "";
  statusText.value = "已选择固件";
  message.success(`已选择: ${selected}`);
}

async function generateFirmware() {
  if (!firmwarePath.value) return;
  generating.value = true;
  statusText.value = "生成中...";
  try {
    const outPath = firmwarePath.value.replace(/\.bin$/, ".bin.bin");
    const result = await invoke("generate_firmware", {
      firmwarePath: firmwarePath.value,
      outputPath: outPath,
    });
    outputPath.value = outPath;
    outputName.value = outPath.split(/[/\\]/).pop();
    statusText.value = "升级固件已生成";
    message.success(result);
  } catch (e) {
    statusText.value = "生成失败";
    message.error(`生成失败: ${e}`);
  } finally {
    generating.value = false;
  }
}

async function startUpgrade() {
  if (!outputPath.value) {
    message.warning("请先生成升级固件");
    return;
  }
  sending.value = true;
  statusText.value = "正在升级...";
  try {
    const result = await invoke("start_iap_upgrade", {
      firmwarePath: outputPath.value,
    });
    statusText.value = "固件已发送";
    message.success(result);
  } catch (e) {
    statusText.value = "升级失败";
    message.error(`升级失败: ${e}`);
  } finally {
    sending.value = false;
  }
}
</script>
