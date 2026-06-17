<template>
  <div class="config-manager">
    <div class="toolbar">
      <n-button @click="importExcel" :loading="importing" type="primary">
        导入Excel
      </n-button>
      <n-button @click="exportTemplate" :loading="exporting" type="default">
        导出模板
      </n-button>
      <n-button @click="exportExcel" :loading="exporting" type="primary">
        导出Excel
      </n-button>
      <n-button @click="newRecord" type="success">
        新建
      </n-button>
      <div style="flex:1" />
      <n-button @click="showPasswordDialog" type="warning">
        {{ unlocked ? '已解锁' : (passwordSet ? '解锁' : '设置密码') }}
      </n-button>
    </div>

    <div class="search-bar">
      <n-input
        placeholder="搜索设备名..."
        v-model:value="searchName"
        clearable
        @update:value="onSearch"
        style="flex: 1"
      />
      <n-input
        placeholder="搜索IP地址..."
        v-model:value="searchIp"
        clearable
        @update:value="onSearch"
        style="flex: 1"
      />
    </div>

    <n-data-table
      :columns="columns"
      :data="displayRecords"
      :pagination="pagination"
      :row-props="rowProps"
      :row-class-name="rowClassName"
      :key="dtKey"
      :loading="loading"
    />

    <n-divider />
    <div class="form-area">
      <n-form label-placement="left" :label-width="120">
        <n-grid :cols="2" :x-gap="24">
          <n-gi>
            <n-form-item label="ProductID">
              <n-input v-model:value="formRecord.product_id" :disabled="!unlocked" placeholder="例如: 7KdEHCyUDG" />
            </n-form-item>
            <n-form-item label="DeviceName">
              <n-input v-model:value="formRecord.device_name" :disabled="!unlocked" placeholder="必填, 例如: DVDR260512010901" />
            </n-form-item>
            <n-form-item label="SecKey">
              <n-input v-model:value="formRecord.sec_key" :disabled="!unlocked" />
            </n-form-item>
            <n-form-item label="Bind">
              <n-switch v-model:value="formRecord.bind" :disabled="!unlocked" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="DHCP">
              <n-switch v-model:value="formRecord.lan_dhcp" :disabled="!unlocked" />
            </n-form-item>
            <n-form-item label="LAN IP">
              <n-input v-model:value="formRecord.lan_ip" :disabled="!unlocked" placeholder="例如: 192.168.124.100" />
            </n-form-item>
            <n-form-item label="网关">
              <n-input v-model:value="formRecord.lan_gateway" :disabled="!unlocked" />
            </n-form-item>
            <n-form-item label="掩码">
              <n-input v-model:value="formRecord.lan_mask" :disabled="!unlocked" />
            </n-form-item>
            <n-form-item label="MAC">
              <n-input v-model:value="formRecord.mac_addr" :disabled="!unlocked" placeholder="例如: 04:2B:58:09:D2:F3" />
            </n-form-item>
            <n-form-item label="MQTT Domain">
              <n-input v-model:value="formRecord.mqtt_domain" :disabled="!unlocked" />
            </n-form-item>
            <n-form-item label="MQTT Port">
              <n-input-number v-model:value="formRecord.mqtt_port" :disabled="!unlocked" style="width:100%" />
            </n-form-item>
            <n-form-item label="NTP IP">
              <n-input v-model:value="formRecord.ntp_ip" :disabled="!unlocked" />
            </n-form-item>
            <n-form-item label="NTP Port">
              <n-input-number v-model:value="formRecord.ntp_port" :disabled="!unlocked" style="width:100%" />
            </n-form-item>
          </n-gi>
        </n-grid>
      </n-form>

      <div class="save-area">
        <n-button
          type="primary"
          size="large"
          :loading="saving"
          @click="saveConfig"
        >
          保存配置到设备
        </n-button>
      </div>
    </div>

    <n-modal v-model:show="showPasswordModal" preset="card" title="管理员密码" style="width: 400px">
      <n-input
        type="password"
        v-model:value="passwordInput"
        placeholder="输入管理员密码"
        show-password-on="click"
        @keyup.enter="handlePassword"
      />
      <template #footer>
        <div style="display:flex;justify-content:space-between;align-items:center">
          <n-button text size="small" type="error" @click="handleResetPassword">
            忘记密码？
          </n-button>
          <div>
            <n-button @click="showPasswordModal = false" style="margin-right:8px">取消</n-button>
            <n-button type="primary" @click="handlePassword">
              {{ passwordSet ? '解锁' : '设置密码' }}
            </n-button>
          </div>
        </div>
      </template>
    </n-modal>

    <n-modal v-model:show="showSetPasswordModal" preset="card" title="设置新密码" style="width: 400px">
      <n-input
        type="password"
        v-model:value="newPasswordInput"
        placeholder="输入新密码（至少4位）"
        show-password-on="click"
      />
      <n-input
        type="password"
        v-model:value="confirmPasswordInput"
        placeholder="确认新密码"
        show-password-on="click"
        style="margin-top: 12px"
      />
      <template #footer>
        <n-button @click="showSetPasswordModal = false">取消</n-button>
        <n-button type="primary" @click="handleSetPassword">确认设置</n-button>
      </template>
    </n-modal>

    <n-modal v-model:show="showResetConfirmModal" preset="card" title="重置密码" style="width: 400px">
      <p>确定要重置管理员密码吗？</p>
      <p style="color:var(--warning-color)">重置后任何人都可以访问配置，建议重置后立即设置新密码。</p>
      <template #footer>
        <n-button @click="showResetConfirmModal = false">取消</n-button>
        <n-button type="error" @click="confirmResetPassword">确认重置</n-button>
      </template>
    </n-modal>

    <n-modal v-model:show="showSaveOkModal" preset="card" title="配置已发送" style="width: 400px">
      <p>配置已保存并发送到设备，设备已返回成功。</p>
      <p>该设备已标记为「已配置」。</p>
      <template #footer>
        <n-button type="primary" @click="showSaveOkModal = false">确定</n-button>
      </template>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, reactive, h, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { open, save } from "@tauri-apps/api/dialog";
import { useMessage } from "naive-ui";

const msg = useMessage();

const allRecords = ref([]);
const displayRecords = ref([]);
const formRecord = ref(createEmptyRecord());
const loading = ref(false);
const importing = ref(false);
const exporting = ref(false);
const saving = ref(false);
const unlocked = ref(false);
const passwordSet = ref(false);

const searchName = ref("");
const searchIp = ref("");

const pagination = reactive({
  page: 1,
  pageSize: 15,
  showSizePicker: true,
  pageSizes: [10, 15, 20, 50],
  onChange: (page) => {
    pagination.page = page;
  },
  onUpdatePageSize: (pageSize) => {
    pagination.pageSize = pageSize;
    pagination.page = 1;
  },
});

const selectedName = ref("");
const dtKey = ref(0);

const showPasswordModal = ref(false);
const showSetPasswordModal = ref(false);
const showResetConfirmModal = ref(false);
const showSaveOkModal = ref(false);
const passwordInput = ref("");
const newPasswordInput = ref("");
const confirmPasswordInput = ref("");

function createEmptyRecord() {
  return {
    id: null,
    product_id: "7KdEHCyUDG",
    device_name: "",
    sec_key: "N0Y0NzY2NDYyQzI2MkU0MjAwRjZCNTEwQkRCMkI4MkU=",
    bind: false,
    lan_dhcp: true,
    lan_ip: "",
    lan_gateway: "",
    lan_mask: "255.255.255.0",
    mac_addr: "",
    mqtt_domain: "82.3.18.138",
    mqtt_port: 30980,
    ntp_ip: "10.229.149.9",
    ntp_port: 123,
    configured: false,
    created_at: null,
    configured_at: null,
  };
}

const columns = [
  {
    title: "序号",
    key: "seq",
    width: 60,
    render(row, index) {
      return index + 1 + (pagination.page - 1) * pagination.pageSize;
    },
  },
  { title: "DeviceName", key: "device_name", sortable: true },
  { title: "ProductID", key: "product_id" },
  { title: "LAN IP", key: "lan_ip", sortable: true },
  {
    title: "状态",
    key: "configured",
    width: 80,
    render(row) {
      return h(
        "span",
        {
          style: {
            color: row.configured ? "#e88080" : "#63e2b7",
            fontWeight: "bold",
          },
        },
        row.configured ? "已配置" : "未配置"
      );
    },
  },
];

function rowClassName(row) {
  if (row.device_name === selectedName.value) return "selected-row";
  if (row.configured) return "configured-row";
  return "";
}

function rowProps(row) {
  return {
    style: { cursor: "pointer" },
    onClick: () => selectRow(row),
  };
}

function selectRow(row) {
  formRecord.value = { ...row };
  selectedName.value = row.device_name;
  dtKey.value++;
}

function newRecord() {
  formRecord.value = createEmptyRecord();
  selectedName.value = "";
  dtKey.value++;
}

async function loadRecords() {
  loading.value = true;
  try {
    allRecords.value = await invoke("get_all_devices");
    applySearch();
  } catch (e) {
    msg.error("加载记录失败: " + e);
  } finally {
    loading.value = false;
  }
}

function onSearch() {
  pagination.page = 1;
  applySearch();
}

function applySearch() {
  let filtered = allRecords.value;
  const nameKw = searchName.value.trim().toLowerCase();
  const ipKw = searchIp.value.trim().toLowerCase();

  if (nameKw || ipKw) {
    filtered = allRecords.value.filter((r) => {
      let match = true;
      if (nameKw) {
        match = match && r.device_name.toLowerCase().includes(nameKw);
      }
      if (ipKw) {
        match = match && r.lan_ip.toLowerCase().includes(ipKw);
      }
      return match;
    });
  }

  displayRecords.value = filtered;
}

async function importExcel() {
  const path = await open({
    filters: [{ name: "Excel", extensions: ["xlsx"] }],
  });
  if (!path) return;

  importing.value = true;
  try {
    const records = await invoke("import_excel", { path });

    allRecords.value = records;
    applySearch();
    if (records.length > 0) {
      formRecord.value = { ...records[0] };
    }
    msg.success(`已导入 ${records.length} 条记录`);
  } catch (e) {
    msg.error("导入失败: " + e);
  } finally {
    importing.value = false;
  }
}

async function exportTemplate() {
  exporting.value = true;
  try {
    const path = await save({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      defaultPath: "设备配置模板.xlsx",
    });
    if (!path) {
      exporting.value = false;
      return;
    }
    await invoke("export_template", { path });
    msg.success("模板已导出，请参照表头填写数据", { duration: 5000 });
  } catch (e) {
    msg.error("导出失败: " + e);
  } finally {
    exporting.value = false;
  }
}

async function exportExcel() {
  exporting.value = true;
  try {
    const path = await save({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      defaultPath: "设备配置列表.xlsx",
    });
    if (!path) {
      exporting.value = false;
      return;
    }
    await invoke("export_excel", { path, records: allRecords.value });
    msg.success("导出成功");
  } catch (e) {
    msg.error("导出失败: " + e);
  } finally {
    exporting.value = false;
  }
}

async function saveConfig() {
  if (!formRecord.value.device_name.trim()) {
    msg.warning("DeviceName 不能为空");
    return;
  }
  if (!formRecord.value.product_id.trim()) {
    msg.warning("ProductID 不能为空");
    return;
  }

  saving.value = true;
  try {
    const result = await invoke("save_device_config", {
      record: formRecord.value,
    });
    showSaveOkModal.value = true;

    const saved = { ...formRecord.value, configured: true };
    const idx = allRecords.value.findIndex(
      (r) => r.device_name === saved.device_name
    );
    if (idx >= 0) {
      allRecords.value[idx] = saved;
    } else {
      allRecords.value.push(saved);
    }
    formRecord.value = saved;
    applySearch();
  } catch (e) {
    msg.error("保存失败: " + e);
  } finally {
    saving.value = false;
  }
}

async function showPasswordDialog() {
  passwordInput.value = "";
  newPasswordInput.value = "";
  confirmPasswordInput.value = "";
  try {
    passwordSet.value = await invoke("is_password_set");
  } catch (_) {
    passwordSet.value = false;
  }
  showPasswordModal.value = true;
}

async function handlePassword() {
  if (!passwordInput.value) {
    msg.warning("请输入密码");
    return;
  }
  try {
    const set = await invoke("is_password_set");
    if (!set) {
      showSetPasswordModal.value = true;
      showPasswordModal.value = false;
      return;
    }
    const ok = await invoke("verify_admin_password", {
      password: passwordInput.value,
    });
    if (ok) {
      unlocked.value = true;
      showPasswordModal.value = false;
      msg.success("已解锁");
    } else {
      msg.error("密码错误");
    }
  } catch (e) {
    msg.error("验证失败: " + e);
  }
}

function handleResetPassword() {
  showPasswordModal.value = false;
  showResetConfirmModal.value = true;
}

async function confirmResetPassword() {
  try {
    await invoke("reset_admin_password");
    passwordSet.value = false;
    unlocked.value = false;
    showResetConfirmModal.value = false;
    msg.success("密码已重置，请重新设置新密码");
    showPasswordDialog();
  } catch (e) {
    msg.error("重置失败: " + e);
  }
}

async function handleSetPassword() {
  if (!newPasswordInput.value) {
    msg.warning("请输入新密码");
    return;
  }
  if (newPasswordInput.value.length < 4) {
    msg.warning("密码至少4位");
    return;
  }
  if (newPasswordInput.value !== confirmPasswordInput.value) {
    msg.warning("两次输入的密码不一致");
    return;
  }
  try {
    await invoke("set_admin_password", { password: newPasswordInput.value });
    passwordSet.value = true;
    showSetPasswordModal.value = false;
    unlocked.value = true;
    msg.success("密码已设置");
  } catch (e) {
    msg.error("设置失败: " + e);
  }
}

onMounted(async () => {
  try {
    passwordSet.value = await invoke("is_password_set");
  } catch (_) {
    passwordSet.value = false;
  }
  await loadRecords();
});
</script>

<style scoped>
.config-manager {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.toolbar {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}
.search-bar {
  display: flex;
  gap: 12px;
}
.form-area {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 16px;
}
.save-area {
  display: flex;
  align-items: center;
  margin-top: 16px;
}
</style>

<style>
tr.selected-row {
  background: #3a3a3a !important;
}
tr.selected-row:hover {
  background: #3a3a3a !important;
}
tr.configured-row {
  background: rgba(232, 128, 128, 0.06) !important;
}
</style>
