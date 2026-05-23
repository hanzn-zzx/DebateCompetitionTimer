<script setup>
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit } from '@tauri-apps/api/event';

const templates = ref([]);
const selectedTemplateId = ref(null);
const currentTemplate = ref(null);
const editingName = ref('');
const timerRecords = ref([]);
const currentView = ref('templates');

const timerSettings = ref({
  debateTopic: '',
  positiveTeamName: '',
  negativeTeamName: '',
  positiveMembers: {
    member1: '',
    member2: '',
    member3: '',
    member4: ''
  },
  negativeMembers: {
    member1: '',
    member2: '',
    member3: '',
    member4: ''
  }
});

async function loadTemplates() {
  try {
    const result = await invoke('load_templates');
    templates.value = result;
  } catch (e) {
    console.error('Failed to load templates:', e);
    templates.value = [];
  }
}

async function loadTeamSettings() {
  try {
    const result = await invoke('load_team_settings');
    timerSettings.value = result;
  } catch (e) {
    console.error('Failed to load team settings:', e);
  }
}

async function loadTimerRecords() {
  try {
    const result = await invoke('load_timer_records');
    timerRecords.value = result;
  } catch (e) {
    console.error('Failed to load timer records:', e);
    timerRecords.value = [];
  }
}

async function saveTeamSettings() {
  try {
    await invoke('save_team_settings', { settings: timerSettings.value });
  } catch (e) {
    console.error('Failed to save team settings:', e);
  }
}

async function deleteTimerRecord(id) {
  try {
    await invoke('delete_timer_record', { id });
    await loadTimerRecords();
  } catch (e) {
    console.error('Failed to delete timer record:', e);
  }
}

async function generateId() {
  try {
    return await invoke('generate_id');
  } catch (e) {
    console.error('Failed to generate id:', e);
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }
}

async function createTemplate() {
  const newTemplate = {
    id: await generateId(),
    name: '新模板',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    segments: []
  };
  templates.value.push(newTemplate);
  await invoke('save_templates', { templates: templates.value });
  selectTemplate(newTemplate.id);
}

function selectTemplate(id) {
  selectedTemplateId.value = id;
  const template = templates.value.find(t => t.id === id);
  if (template) {
    currentTemplate.value = JSON.parse(JSON.stringify(template));
    editingName.value = template.name;
  }
}

async function deleteTemplate(id) {
  const index = templates.value.findIndex(t => t.id === id);
  if (index !== -1) {
    templates.value.splice(index, 1);
    await invoke('save_templates', { templates: templates.value });
    if (selectedTemplateId.value === id) {
      selectedTemplateId.value = null;
      currentTemplate.value = null;
    }
  }
}

async function updateTemplateName(name) {
  if (currentTemplate.value) {
    currentTemplate.value.name = name;
    const template = templates.value.find(t => t.id === currentTemplate.value.id);
    if (template) {
      template.name = name;
      template.updated_at = new Date().toISOString();
    }
    await invoke('save_templates', { templates: templates.value });
  }
}

async function addSegment() {
  if (currentTemplate.value) {
    const newSegment = {
      id: await generateId(),
      name: '新环节',
      speaker: '',
      duration: 180,
      side: 'neutral',
      type: 'normal'
    };
    currentTemplate.value.segments.push(newSegment);
    const template = templates.value.find(t => t.id === currentTemplate.value.id);
    if (template) {
      template.segments.push(newSegment);
      template.updated_at = new Date().toISOString();
    }
    await invoke('save_templates', { templates: templates.value });
  }
}

async function updateSegment(index, segment) {
  if (currentTemplate.value) {
    currentTemplate.value.segments[index] = { ...segment };
    const template = templates.value.find(t => t.id === currentTemplate.value.id);
    if (template) {
      template.segments[index] = { ...segment };
      template.updated_at = new Date().toISOString();
    }
    await invoke('save_templates', { templates: templates.value });
  }
}

async function deleteSegment(index) {
  if (currentTemplate.value) {
    currentTemplate.value.segments.splice(index, 1);
    const template = templates.value.find(t => t.id === currentTemplate.value.id);
    if (template) {
      template.segments.splice(index, 1);
      template.updated_at = new Date().toISOString();
    }
    await invoke('save_templates', { templates: templates.value });
  }
}

async function startTimer() {
  if (!selectedTemplateId.value || !currentTemplate.value || currentTemplate.value.segments.length === 0) {
    alert('请先选择模板并添加环节');
    return;
  }

  const timerData = {
    template: currentTemplate.value,
    settings: timerSettings.value,
    selectedTemplateId: selectedTemplateId.value,
    startTime: new Date().toISOString()
  };

  try {
    const timerWindow = await WebviewWindow.getByLabel('timer');
    if (!timerWindow) {
      alert('计时器窗口不存在');
      return;
    }
    await timerWindow.setSize({ width: window.screen.width, height: window.screen.height });
    await timerWindow.center();
    await timerWindow.setFullscreen(true);
    await timerWindow.show();
    await emit('start-timer', timerData);
  } catch (e) {
    console.error('Failed to open timer window:', e);
    alert('打开计时器失败');
  }
}

async function exportTemplate(id) {
  const template = templates.value.find(t => t.id === id);
  if (!template) return;

  const dataStr = JSON.stringify(template, null, 2);
  const blob = new Blob([dataStr], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${template.name}.json`;
  a.click();
  URL.revokeObjectURL(url);
}

async function importTemplate(event) {
  const file = event.target.files[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = async (e) => {
    try {
      const imported = JSON.parse(e.target.result);
      if (!imported.name || !Array.isArray(imported.segments)) {
        alert('无效的模板文件格式');
        return;
      }
      imported.id = await generateId();
      templates.value.push(imported);
      await invoke('save_templates', { templates: templates.value });
      selectTemplate(imported.id);
    } catch (err) {
      alert('导入失败: ' + err.message);
    }
  };
  reader.readAsText(file);
  event.target.value = '';
}

function formatDate(isoString) {
  const date = new Date(isoString);
  return date.toLocaleString('zh-CN');
}

watch(timerSettings, () => {
  saveTeamSettings();
}, { deep: true });

onMounted(async () => {
  await Promise.all([
    loadTemplates(),
    loadTeamSettings(),
    loadTimerRecords()
  ]);
  if (templates.value.length > 0) {
    selectTemplate(templates.value[0].id);
  }
});
</script>

<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="nav-item" :class="{ active: currentView === 'templates' }" @click="currentView = 'templates'">
        计时模板
      </div>
      <div class="nav-item" :class="{ active: currentView === 'timer' }" @click="currentView = 'timer'">
        计时器
      </div>
      <div class="nav-item" :class="{ active: currentView === 'records' }" @click="currentView = 'records'">
        计时记录
      </div>
      <div class="nav-item" :class="{ active: currentView === 'settings' }" @click="currentView = 'settings'">
        程序设置
      </div>
    </aside>

    <main class="main-content">
      <template v-if="currentView === 'templates'">
        <div class="template-header">
          <h2>计时模板</h2>
        </div>
        <div class="templates-section">
          <div class="template-list">
            <div
              v-for="template in templates"
              :key="template.id"
              class="template-item"
              :class="{ selected: selectedTemplateId === template.id }"
              @click="selectTemplate(template.id)"
            >
              <span class="template-name">{{ template.name }}</span>
              <div class="template-actions">
                <button class="btn-small" @click.stop="exportTemplate(template.id)">导出</button>
                <button class="btn-small danger" @click.stop="deleteTemplate(template.id)">删除</button>
              </div>
            </div>
            <button class="btn-add-template" @click="createTemplate">+ 新建模板</button>
          </div>

          <div class="template-editor" v-if="currentTemplate">
            <div class="editor-header">
              <input
                type="text"
                v-model="editingName"
                @blur="updateTemplateName(editingName)"
                @keyup.enter="$event.target.blur()"
                class="template-name-input"
              />
            </div>

            <div class="segments-section">
              <div class="section-header">
                <h3>环节列表</h3>
              </div>
              <div class="segments-list">
                <div v-for="(segment, index) in currentTemplate.segments" :key="segment.id" class="segment-item">
                  <div class="segment-info">
                    <div class="segment-row">
                      <input
                        type="text"
                        :value="segment.name"
                        @input="updateSegment(index, { ...segment, name: $event.target.value })"
                        placeholder="环节名称"
                        class="segment-name-input"
                      />
                      <input
                        type="number"
                        :value="Math.floor(segment.duration / 60)"
                        @input="updateSegment(index, { ...segment, duration: parseInt($event.target.value) * 60 || 0 })"
                        class="duration-input"
                        min="0"
                      />
                      <span class="duration-label">:</span>
                      <input
                        type="number"
                        :value="segment.duration % 60"
                        @input="updateSegment(index, { ...segment, duration: Math.floor(segment.duration / 60) * 60 + (parseInt($event.target.value) || 0) })"
                        class="duration-input"
                        min="0"
                        max="59"
                      />
                    </div>
                    <div class="segment-row" v-if="segment.type !== 'free_debate'">
                      <input
                        type="text"
                        :value="segment.speaker"
                        @input="updateSegment(index, { ...segment, speaker: $event.target.value })"
                        placeholder="角色/身份"
                        class="speaker-input"
                      />
                    </div>
                    <div class="segment-row">
                      <select
                        :value="segment.type || 'normal'"
                        @change="updateSegment(index, { ...segment, type: $event.target.value })"
                        class="type-select"
                      >
                        <option value="normal">普通环节</option>
                        <option value="free_debate">自由辩论</option>
                      </select>
                      <select
                        :value="segment.side || 'neutral'"
                        @change="updateSegment(index, { ...segment, side: $event.target.value })"
                        class="side-select"
                      >
                        <option value="positive">正方</option>
                        <option value="negative">反方</option>
                        <option value="neutral">中立</option>
                      </select>
                    </div>
                  </div>
                  <button class="btn-delete-segment" @click="deleteSegment(index)">×</button>
                </div>
                <div v-if="currentTemplate.segments.length === 0" class="empty-segments">
                  暂无环节，请添加环节
                </div>
              </div>
              <button class="btn-add-segment" @click="addSegment">+ 添加环节</button>
            </div>

            <div class="import-export">
              <label class="btn btn-secondary">
                导入模板
                <input type="file" accept=".json" @change="importTemplate" hidden />
              </label>
              <button class="btn btn-secondary" @click="exportTemplate(currentTemplate.id)" :disabled="!currentTemplate">
                导出模板
              </button>
            </div>
          </div>

          <div class="template-editor empty" v-else>
            <p>请选择一个模板或创建新模板</p>
          </div>
        </div>
      </template>

      <template v-if="currentView === 'timer'">
        <div class="timer-header">
          <h2>计时器</h2>
        </div>
        <div class="timer-settings">
          <div class="setting-group">
            <label>辩题</label>
            <input
              type="text"
              v-model="timerSettings.debateTopic"
              placeholder="请输入辩题"
              class="topic-input"
            />
          </div>

          <div class="setting-group">
            <label>计时模板</label>
            <select v-model="selectedTemplateId" @change="selectTemplate(selectedTemplateId)" class="template-select">
              <option value="">请选择计时模板</option>
              <option v-for="template in templates" :key="template.id" :value="template.id">
                {{ template.name }}
              </option>
            </select>
          </div>

          <div class="teams-config">
            <div class="team positive">
              <h3>正方</h3>
              <div class="field">
                <label>正方名称</label>
                <input type="text" v-model="timerSettings.positiveTeamName" placeholder="如：冰队" />
              </div>
              <div class="members">
                <div class="field">
                  <label>一辩</label>
                  <input type="text" v-model="timerSettings.positiveMembers.member1" />
                </div>
                <div class="field">
                  <label>二辩</label>
                  <input type="text" v-model="timerSettings.positiveMembers.member2" />
                </div>
                <div class="field">
                  <label>三辩</label>
                  <input type="text" v-model="timerSettings.positiveMembers.member3" />
                </div>
                <div class="field">
                  <label>四辩</label>
                  <input type="text" v-model="timerSettings.positiveMembers.member4" />
                </div>
              </div>
            </div>

            <div class="team negative">
              <h3>反方</h3>
              <div class="field">
                <label>反方名称</label>
                <input type="text" v-model="timerSettings.negativeTeamName" placeholder="如：火队" />
              </div>
              <div class="members">
                <div class="field">
                  <label>一辩</label>
                  <input type="text" v-model="timerSettings.negativeMembers.member1" />
                </div>
                <div class="field">
                  <label>二辩</label>
                  <input type="text" v-model="timerSettings.negativeMembers.member2" />
                </div>
                <div class="field">
                  <label>三辩</label>
                  <input type="text" v-model="timerSettings.negativeMembers.member3" />
                </div>
                <div class="field">
                  <label>四辩</label>
                  <input type="text" v-model="timerSettings.negativeMembers.member4" />
                </div>
              </div>
            </div>
          </div>

          <div class="start-section">
            <button
              class="btn btn-primary btn-start"
              @click="startTimer"
              :disabled="!selectedTemplateId || !currentTemplate || currentTemplate.segments.length === 0"
            >
              开始计时
            </button>
          </div>
        </div>
      </template>

      <template v-if="currentView === 'records'">
        <div class="records-header">
          <h2>计时记录</h2>
        </div>
        <div class="records-list">
          <div v-if="timerRecords.length === 0" class="empty-records">
            暂无计时记录
          </div>
          <div v-for="record in timerRecords" :key="record.id" class="record-item">
            <div class="record-header">
              <span class="record-template">{{ record.template_name }}</span>
              <span class="record-date">{{ formatDate(record.start_time) }}</span>
            </div>
            <div class="record-detail">
              <div v-if="record.debate_topic" class="record-topic">辩题：{{ record.debate_topic }}</div>
              <div class="record-teams">
                <span class="positive">{{ record.positive_team_name || '正方' }}</span>
                <span class="vs">VS</span>
                <span class="negative">{{ record.negative_team_name || '反方' }}</span>
              </div>
            </div>
            <button class="btn-delete-record" @click="deleteTimerRecord(record.id)">删除</button>
          </div>
        </div>
      </template>

      <template v-if="currentView === 'settings'">
        <div class="settings-header">
          <h2>程序设置</h2>
        </div>
        <div class="settings-content">
          <div class="setting-item">
            <label>语言</label>
            <select class="language-select">
              <option value="zh-CN">简体中文</option>
            </select>
          </div>
          <div class="setting-item">
            <button class="btn btn-danger">重置程序</button>
          </div>
        </div>
      </template>
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --color-primary: #1a1a2e;
  --color-secondary: #16213e;
  --color-accent: #e94560;
  --color-success: #4ecca3;
  --color-text: #eaeaea;
  --color-text-secondary: #a0a0a0;
  --shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  --radius: 8px;
  --radius-lg: 12px;
}

html, body {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

body {
  font-family: "Microsoft YaHei", "PingFang SC", sans-serif;
  background-color: var(--color-primary);
  color: var(--color-text);
}

#app {
  height: 100%;
  width: 100%;
}
</style>

<style scoped>
.app-container {
  display: flex;
  height: 100%;
  width: 100%;
}

.sidebar {
  width: 200px;
  background-color: var(--color-secondary);
  padding: 16px 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  padding: 12px 24px;
  cursor: pointer;
  transition: background-color 0.2s;
  border-left: 3px solid transparent;
}

.nav-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.nav-item.active {
  background-color: rgba(78, 204, 163, 0.1);
  border-left-color: var(--color-success);
  color: var(--color-success);
}

.main-content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.template-header,
.timer-header,
.records-header,
.settings-header {
  margin-bottom: 24px;
}

.template-header h2,
.timer-header h2,
.records-header h2,
.settings-header h2 {
  font-size: 24px;
  font-weight: 600;
}

.templates-section {
  display: flex;
  gap: 24px;
}

.template-list {
  width: 250px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.template-item {
  padding: 12px 16px;
  background-color: var(--color-secondary);
  border-radius: var(--radius);
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: background-color 0.2s;
}

.template-item:hover {
  background-color: #1f4068;
}

.template-item.selected {
  background-color: #1f4068;
  border: 1px solid var(--color-success);
}

.template-name {
  font-size: 14px;
}

.template-actions {
  display: flex;
  gap: 4px;
}

.btn-small {
  padding: 4px 8px;
  font-size: 12px;
  background-color: transparent;
  color: var(--color-text-secondary);
  border: 1px solid #333;
  border-radius: 4px;
  cursor: pointer;
}

.btn-small:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.btn-small.danger:hover {
  background-color: rgba(233, 69, 96, 0.2);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.btn-add-template {
  padding: 12px 16px;
  background-color: transparent;
  color: var(--color-success);
  border: 1px dashed var(--color-success);
  border-radius: var(--radius);
  cursor: pointer;
  font-size: 14px;
}

.btn-add-template:hover {
  background-color: rgba(78, 204, 163, 0.1);
}

.template-editor {
  flex: 1;
  background-color: var(--color-secondary);
  border-radius: var(--radius-lg);
  padding: 24px;
}

.template-editor.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
}

.editor-header {
  margin-bottom: 24px;
}

.template-name-input {
  font-size: 20px;
  font-weight: 600;
  width: 100%;
  max-width: 300px;
  padding: 8px 12px;
  background-color: var(--color-primary);
  border: 1px solid #333;
  border-radius: var(--radius);
  color: var(--color-text);
}

.template-name-input:focus {
  border-color: var(--color-success);
  outline: none;
}

.segments-section {
  margin-bottom: 24px;
}

.section-header {
  margin-bottom: 16px;
}

.section-header h3 {
  font-size: 16px;
  font-weight: 500;
}

.segments-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}

.segment-item {
  display: flex;
  gap: 12px;
  padding: 12px;
  background-color: var(--color-primary);
  border-radius: var(--radius);
}

.segment-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.segment-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.segment-name-input {
  flex: 1;
  padding: 8px 12px;
  background-color: var(--color-secondary);
  border: 1px solid #333;
  border-radius: var(--radius);
  color: var(--color-text);
  font-size: 14px;
}

.duration-input {
  width: 50px;
  padding: 8px;
  background-color: var(--color-secondary);
  border: 1px solid #333;
  border-radius: var(--radius);
  color: var(--color-text);
  font-size: 14px;
  text-align: center;
}

.duration-label {
  font-size: 16px;
  color: var(--color-text-secondary);
}

.speaker-input {
  flex: 1;
  padding: 8px 12px;
  background-color: var(--color-secondary);
  border: 1px solid #333;
  border-radius: var(--radius);
  color: var(--color-text);
  font-size: 14px;
}

.type-select,
.side-select {
  padding: 8px 12px;
  background-color: var(--color-secondary);
  border: 1px solid #333;
  border-radius: var(--radius);
  color: var(--color-text);
  font-size: 14px;
}

.btn-delete-segment {
  width: 32px;
  height: 32px;
  background-color: transparent;
  color: var(--color-text-secondary);
  border: 1px solid #333;
  border-radius: var(--radius);
  cursor: pointer;
  font-size: 18px;
}

.btn-delete-segment:hover {
  background-color: rgba(233, 69, 96, 0.2);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.empty-segments {
  padding: 24px;
  text-align: center;
  color: var(--color-text-secondary);
}

.btn-add-segment {
  padding: 12px 16px;
  background-color: transparent;
  color: var(--color-success);
  border: 1px dashed var(--color-success);
  border-radius: var(--radius);
  cursor: pointer;
  font-size: 14px;
  width: 100%;
}

.btn-add-segment:hover {
  background-color: rgba(78, 204, 163, 0.1);
}

.import-export {
  display: flex;
  gap: 12px;
}

.btn {
  padding: 12px 24px;
  border-radius: var(--radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.btn-primary {
  background-color: var(--color-success);
  color: white;
  border: none;
}

.btn-primary:hover:not(:disabled) {
  background-color: #3db892;
}

.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: var(--color-primary);
  color: var(--color-text);
  border: 1px solid #333;
}

.btn-secondary:hover {
  background-color: #1f4068;
}

.timer-settings {
  max-width: 600px;
}

.setting-group {
  margin-bottom: 24px;
}

.setting-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--color-text-secondary);
}

.topic-input,
.template-select {
  width: 100%;
  padding: 12px 16px;
  background-color: var(--color-secondary);
  border: 1px solid #333;
  border-radius: var(--radius);
  color: var(--color-text);
  font-size: 14px;
}

.topic-input:focus,
.template-select:focus {
  border-color: var(--color-success);
  outline: none;
}

.teams-config {
  display: flex;
  gap: 24px;
  margin-bottom: 32px;
}

.team {
  flex: 1;
  padding: 16px;
  background-color: var(--color-secondary);
  border-radius: var(--radius-lg);
}

.team h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #333;
}

.team.positive h3 {
  color: var(--color-success);
}

.team.negative h3 {
  color: var(--color-accent);
}

.team .field {
  margin-bottom: 12px;
}

.team .field label {
  display: block;
  margin-bottom: 4px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.team .field input {
  width: 100%;
  padding: 8px 12px;
  background-color: var(--color-primary);
  border: 1px solid #333;
  border-radius: var(--radius);
  color: var(--color-text);
  font-size: 14px;
}

.team .field input:focus {
  border-color: var(--color-success);
  outline: none;
}

.start-section {
  padding-top: 24px;
  border-top: 1px solid #333;
}

.btn-start {
  width: 100%;
  padding: 16px;
  font-size: 16px;
}

.records-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-records {
  padding: 48px;
  text-align: center;
  color: var(--color-text-secondary);
  background-color: var(--color-secondary);
  border-radius: var(--radius-lg);
}

.record-item {
  padding: 16px;
  background-color: var(--color-secondary);
  border-radius: var(--radius-lg);
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.record-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.record-template {
  font-size: 16px;
  font-weight: 600;
}

.record-date {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.record-detail {
  flex: 1;
  margin: 0 24px;
}

.record-topic {
  font-size: 14px;
  margin-bottom: 8px;
}

.record-teams {
  display: flex;
  gap: 12px;
  font-size: 14px;
}

.record-teams .positive {
  color: var(--color-success);
}

.record-teams .vs {
  color: var(--color-text-secondary);
}

.record-teams .negative {
  color: var(--color-accent);
}

.btn-delete-record {
  padding: 8px 16px;
  background-color: transparent;
  color: var(--color-text-secondary);
  border: 1px solid #333;
  border-radius: var(--radius);
  cursor: pointer;
  font-size: 12px;
}

.btn-delete-record:hover {
  background-color: rgba(233, 69, 96, 0.2);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.settings-content {
  max-width: 400px;
}

.setting-item {
  margin-bottom: 24px;
}

.setting-item label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--color-text-secondary);
}

.language-select {
  width: 100%;
  padding: 12px 16px;
  background-color: var(--color-secondary);
  border: 1px solid #333;
  border-radius: var(--radius);
  color: var(--color-text);
  font-size: 14px;
}

.btn-danger {
  padding: 12px 24px;
  background-color: var(--color-accent);
  color: white;
  border: none;
  border-radius: var(--radius);
  font-size: 14px;
  cursor: pointer;
}

.btn-danger:hover {
  background-color: #d63850;
}
</style>
