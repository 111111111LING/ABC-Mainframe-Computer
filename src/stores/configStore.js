import { defineStore } from "pinia";
import { ref } from "vue";

export const useConfigStore = defineStore("config", () => {
  const deviceConfig = ref({
    productID: "7KdEHCyUDG",
    deviceName: "DVDR260512010901",
    secKey: "N0Y0NzY2NDYyQzI2MkU0MjAwRjZCNTEwQkRCMkI4MkU=",
    bind: 1,
  });

  const networkConfig = ref({
    lanDHCP: true,
    lanIP: "192.168.124.100",
    lanGateway: "192.168.124.1",
    lanMask: "255.255.255.0",
    macAddr: "04:2B:58:09:D2:F3",
    mqttServerIp: "101.132.160.111",
    mqttServerPort: 8883,
    ntpServer: "106.14.18.202",
    ntpPort: 12123,
  });

  const serverRunning = ref(false);
  const connected = ref(false);
  const deviceAddr = ref("");
  const logs = ref([]);

  function loadFromJson(json) {
    const data = typeof json === "string" ? JSON.parse(json) : json;
    if (data.deviceConfig) deviceConfig.value = data.deviceConfig;
    if (data.networkConfig) networkConfig.value = data.networkConfig;
  }

  function toJson() {
    return JSON.stringify(
      {
        deviceConfig: deviceConfig.value,
        networkConfig: networkConfig.value,
      },
      null,
      2
    );
  }

  return {
    deviceConfig,
    networkConfig,
    serverRunning,
    connected,
    deviceAddr,
    logs,
    loadFromJson,
    toJson,
  };
});
