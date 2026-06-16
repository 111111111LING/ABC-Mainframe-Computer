<template>
  <n-message-provider>
    <n-config-provider :theme="darkTheme">
      <n-layout class="layout">
        <n-layout-header class="header">
          <div class="header-title">
            <n-h2 style="margin: 0">CH32V317 设备配置工具</n-h2>
            <n-text depth="3">v0.2.0</n-text>
          </div>
        </n-layout-header>

        <n-layout-content class="content" content-style="padding: 24px;">
          <n-tabs type="line" animated default-value="device">
            <n-tab-pane name="device" tab="设备信息">
              <DeviceIdentity />
            </n-tab-pane>
            <n-tab-pane name="network" tab="网络配置">
              <NetworkConfig />
            </n-tab-pane>
            <n-tab-pane name="iap" tab="317升级">
              <IapFlasher />
            </n-tab-pane>
            <n-tab-pane name="ota-front" tab="前后板OTA">
              <OtaFront />
            </n-tab-pane>
          </n-tabs>
        </n-layout-content>

        <n-layout-footer class="footer">
          <StatusBar />
        </n-layout-footer>
      </n-layout>
    </n-config-provider>
  </n-message-provider>
</template>

<script setup>
import { onMounted, onBeforeUnmount } from "vue";
import { darkTheme } from "naive-ui";
import { listen } from "@tauri-apps/api/event";
import { useConfigStore } from "./stores/configStore";
import DeviceIdentity from "./components/DeviceIdentity.vue";
import NetworkConfig from "./components/NetworkConfig.vue";
import IapFlasher from "./components/IapFlasher.vue";
import OtaFront from "./components/OtaFront.vue";
import StatusBar from "./components/StatusBar.vue";

const store = useConfigStore();
const unlisteners = [];

onMounted(async () => {
  try {
    unlisteners.push(
      await listen("device-connected", (event) => {
        store.connected = true;
        store.deviceAddr = event.payload;
        store.serverRunning = true;
      })
    );
    unlisteners.push(
      await listen("device-disconnected", () => {
        store.connected = false;
        store.deviceAddr = "";
      })
    );
    unlisteners.push(
      await listen("iap-log", (event) => {
        const now = new Date();
        const ts = `${now.getHours().toString().padStart(2, "0")}:${now.getMinutes().toString().padStart(2, "0")}:${now.getSeconds().toString().padStart(2, "0")}`;
        store.logs.push(`[${ts}] ${event.payload}`);
      })
    );
  } catch (e) {
    console.error("Tauri event listener error:", e);
  }
});

onBeforeUnmount(() => {
  unlisteners.forEach((fn) => fn());
});
</script>

<style>
.layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
.header {
  display: flex;
  align-items: center;
  padding: 12px 24px;
  border-bottom: 1px solid var(--border-color);
}
.header-title {
  display: flex;
  align-items: baseline;
  gap: 12px;
}
.content {
  flex: 1;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
}
.footer {
  display: flex;
  align-items: center;
  padding: 8px 24px;
  border-top: 1px solid var(--border-color);
}
</style>
