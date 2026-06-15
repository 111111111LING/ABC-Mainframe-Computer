<template>
  <n-form label-placement="top">
    <n-card title="局域网配置" size="small" :bordered="false">
      <n-form-item label="MAC 地址">
        <n-input
          v-model:value="store.networkConfig.macAddr"
          placeholder="例: 04:2B:58:09:D2:F3"
          :style="{ fontFamily: 'monospace' }"
        />
      </n-form-item>
      <n-form-item label="DHCP">
        <n-switch v-model:value="store.networkConfig.lanDHCP">
          <template #checked>启用</template>
          <template #unchecked>禁用</template>
        </n-switch>
      </n-form-item>
      <template v-if="!store.networkConfig.lanDHCP">
        <n-form-item label="静态 IP 地址">
          <n-input
            v-model:value="store.networkConfig.lanIP"
            placeholder="192.168.124.100"
            :style="{ fontFamily: 'monospace' }"
          />
        </n-form-item>
        <n-form-item label="网关">
          <n-input
            v-model:value="store.networkConfig.lanGateway"
            placeholder="192.168.124.1"
            :style="{ fontFamily: 'monospace' }"
          />
        </n-form-item>
        <n-form-item label="子网掩码">
          <n-input
            v-model:value="store.networkConfig.lanMask"
            placeholder="255.255.255.0"
            :style="{ fontFamily: 'monospace' }"
          />
        </n-form-item>
      </template>
    </n-card>

    <n-card title="MQTT 配置" size="small" :bordered="false" style="margin-top: 12px">
      <n-form-item label="MQTT 服务器 IP">
        <n-input
          v-model:value="store.networkConfig.mqttServerIp"
          placeholder="101.132.160.111"
          :style="{ fontFamily: 'monospace' }"
        />
      </n-form-item>
      <n-form-item label="MQTT 端口">
        <n-input-number
          v-model:value="store.networkConfig.mqttServerPort"
          :min="1"
          :max="65535"
          style="width: 100%"
        />
      </n-form-item>
    </n-card>

    <n-card title="NTP 配置" size="small" :bordered="false" style="margin-top: 12px">
      <n-form-item label="NTP 服务器 IP">
        <n-input
          v-model:value="store.networkConfig.ntpServer"
          placeholder="106.14.18.202"
          :style="{ fontFamily: 'monospace' }"
        />
      </n-form-item>
      <n-form-item label="NTP 端口">
        <n-input-number
          v-model:value="store.networkConfig.ntpPort"
          :min="1"
          :max="65535"
          style="width: 100%"
        />
      </n-form-item>
    </n-card>

    <n-space style="margin-top: 16px">
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
    const result = await invoke("send_network_config", {
      config: {
        lan_dhcp: store.networkConfig.lanDHCP,
        lan_ip: store.networkConfig.lanIP,
        lan_gateway: store.networkConfig.lanGateway,
        lan_mask: store.networkConfig.lanMask,
        mac_addr: store.networkConfig.macAddr,
        mqtt_server_ip: store.networkConfig.mqttServerIp,
        mqtt_server_port: store.networkConfig.mqttServerPort,
        ntp_server: store.networkConfig.ntpServer,
        ntp_port: store.networkConfig.ntpPort,
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
