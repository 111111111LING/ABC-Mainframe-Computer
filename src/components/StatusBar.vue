<template>
  <div class="status-bar">
    <n-space align="center" justify="space-between" style="width: 100%">
      <n-space align="center" size="small">
        <n-badge
          :type="store.connected ? 'success' : 'default'"
          :processing="store.connected"
          dot
        />
        <n-text depth="3" style="font-size: 13px">
          {{ store.connected ? `已连接 ${store.deviceAddr}` : "等待设备连接..." }}
        </n-text>
      </n-space>
      <n-space align="center" size="small">
        <n-tag v-if="store.serverRunning" size="tiny" type="success">服务器运行中</n-tag>
        <n-tag v-else size="tiny">服务器未启动</n-tag>
        <n-button
          v-if="!store.serverRunning"
          size="tiny"
          type="primary"
          :loading="starting"
          @click="startServer"
        >
          启动
        </n-button>
        <n-button
          v-else
          size="tiny"
          type="error"
          @click="stopServer"
        >
          停止
        </n-button>
      </n-space>
    </n-space>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { useMessage } from "naive-ui";
import { useConfigStore } from "../stores/configStore";

const store = useConfigStore();
const message = useMessage();
const starting = ref(false);

async function startServer() {
  starting.value = true;
  try {
    store.serverRunning = true;
    await invoke("start_tcp_server");
    message.success("TCP Server 已启动在 :1000");
  } catch (e) {
    store.serverRunning = false;
    message.error(`启动失败: ${e}`);
  } finally {
    starting.value = false;
  }
}

async function stopServer() {
  try {
    await invoke("stop_tcp_server");
    store.serverRunning = false;
    store.connected = false;
    store.deviceAddr = "";
    message.success("服务器已停止");
  } catch (e) {
    message.error(`停止失败: ${e}`);
  }
}
</script>

<style scoped>
.status-bar {
  width: 100%;
}
</style>
