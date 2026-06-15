<template>
  <n-form label-placement="top">
    <n-form-item label="Product ID">
      <n-input v-model:value="store.deviceConfig.productID" placeholder="产品ID" maxlength="32" />
    </n-form-item>
    <n-form-item label="Device Name">
      <n-input v-model:value="store.deviceConfig.deviceName" placeholder="设备名称" maxlength="32" />
    </n-form-item>
    <n-form-item label="SecKey">
      <n-input
        v-model:value="store.deviceConfig.secKey"
        placeholder="设备密钥"
        maxlength="64"
        type="textarea"
        :autosize="{ minRows: 2, maxRows: 3 }"
      />
    </n-form-item>
    <n-form-item label="绑定状态">
      <n-switch v-model:value="store.deviceConfig.bind" :checked-value="1" :unchecked-value="0">
        <template #checked>已绑定</template>
        <template #unchecked>未绑定</template>
      </n-switch>
    </n-form-item>
    <n-space>
      <n-button type="primary" :disabled="!store.connected" :loading="sending" @click="sendConfig">
        发送配置
      </n-button>
      <n-text v-if="!store.connected" depth="3">需等待设备连接</n-text>
    </n-space>
  </n-form>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { useMessage } from "naive-ui";
import { useConfigStore } from "../stores/configStore";

const store = useConfigStore();
const message = useMessage();
const sending = ref(false);

async function sendConfig() {
  sending.value = true;
  try {
    const result = await invoke("send_device_config", {
      config: {
        product_id: store.deviceConfig.productID,
        device_name: store.deviceConfig.deviceName,
        sec_key: store.deviceConfig.secKey,
        bind: store.deviceConfig.bind,
      },
    });
    message.success(result);
  } catch (e) {
    message.error(`发送失败: ${e}`);
  } finally {
    sending.value = false;
  }
}
</script>
