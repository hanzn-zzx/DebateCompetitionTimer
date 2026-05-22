<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

const timerData = ref(null);
const currentIndex = ref(0);
const timeRemaining = ref(0);
const positiveTimeRemaining = ref(0);
const negativeTimeRemaining = ref(0);
const isRunning = ref(false);
const isPaused = ref(false);
const isFinished = ref(false);
const isOvertime = ref(false);
const showWarning = ref(false);
const currentSpeakerSide = ref('positive');
const warning30Played = ref(false);
const endPlayed = ref(false);
const segmentStartTime = ref(null);

let timerInterval = null;

const template = computed(() => timerData.value?.template);
const settings = computed(() => timerData.value?.settings);

const currentSegment = computed(() => {
  if (!template.value) return null;
  return template.value.segments[currentIndex.value] || null;
});

const nextSegment = computed(() => {
  if (!template.value) return null;
  return template.value.segments[currentIndex.value + 1] || null;
});

const isFreeDebate = computed(() => {
  return currentSegment.value?.type === 'free_debate';
});

const currentSpeakerLabel = computed(() => {
  if (!currentSegment.value) return '';
  if (isFreeDebate.value) {
    return currentSpeakerSide.value === 'positive' ? '正方发言' : '反方发言';
  }
  const speaker = currentSegment.value.speaker || '';
  const name = currentSegment.value.name || '';
  return speaker ? `${speaker} · ${name}` : name;
});

const formattedTime = computed(() => {
  const absTime = Math.abs(timeRemaining.value);
  const minutes = Math.floor(absTime / 60);
  const seconds = absTime % 60;
  const prefix = timeRemaining.value < 0 ? '-' : '';
  return `${prefix}${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
});

const formattedPositiveTime = computed(() => {
  const minutes = Math.floor(positiveTimeRemaining.value / 60);
  const seconds = positiveTimeRemaining.value % 60;
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
});

const formattedNegativeTime = computed(() => {
  const minutes = Math.floor(negativeTimeRemaining.value / 60);
  const seconds = negativeTimeRemaining.value % 60;
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
});

const sideClass = computed(() => {
  if (!currentSegment.value) return '';
  if (isFreeDebate.value) return currentSpeakerSide.value;
  return currentSegment.value.side;
});

function playBeep(duration = 200, count = 1, interval = 150) {
  const audioContext = new (window.AudioContext || window.webkitAudioContext)();
  const playOnce = (delay) => {
    setTimeout(() => {
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();
      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);
      oscillator.frequency.value = 880;
      oscillator.type = 'sine';
      gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + duration / 1000);
      oscillator.start(audioContext.currentTime);
      oscillator.stop(audioContext.currentTime + duration / 1000);
    }, delay);
  };

  for (let i = 0; i < count; i++) {
    playOnce(i * (duration + interval));
  }
}

function startTimer() {
  if (isFinished.value) return;

  if (timerInterval) clearInterval(timerInterval);

  isRunning.value = true;
  isPaused.value = false;
  segmentStartTime.value = Date.now();

  timerInterval = setInterval(() => {
    if (isFreeDebate.value) {
      if (currentSpeakerSide.value === 'positive') {
        positiveTimeRemaining.value--;
        if (positiveTimeRemaining.value === 30 && !warning30Played.value) {
          playBeep(200, 1);
          warning30Played.value = true;
        }
        if (positiveTimeRemaining.value <= 0 && !endPlayed.value) {
          isOvertime.value = true;
          playBeep(200, 2);
          endPlayed.value = true;
        }
      } else {
        negativeTimeRemaining.value--;
        if (negativeTimeRemaining.value === 30 && !warning30Played.value) {
          playBeep(200, 1);
          warning30Played.value = true;
        }
        if (negativeTimeRemaining.value <= 0 && !endPlayed.value) {
          isOvertime.value = true;
          playBeep(200, 2);
          endPlayed.value = true;
        }
      }
    } else {
      timeRemaining.value--;

      if (timeRemaining.value === 30 && !warning30Played.value) {
        showWarning.value = true;
        playBeep(200, 1);
        warning30Played.value = true;
      }

      if (timeRemaining.value <= 0 && !isOvertime.value) {
        isOvertime.value = true;
        if (!endPlayed.value) {
          playBeep(200, 2);
          endPlayed.value = true;
        }
      }

      if (timeRemaining.value <= -60) {
        goToNext();
      }
    }
  }, 1000);
}

function pauseTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
  isPaused.value = true;
}

function resumeTimer() {
  isPaused.value = false;
  startTimer();
}

function toggleTimer() {
  if (!isRunning.value) {
    if (currentIndex.value === 0 && timeRemaining.value === 0 && !isFreeDebate.value) {
      timeRemaining.value = currentSegment.value.duration;
    }
    startTimer();
  } else if (isPaused.value) {
    resumeTimer();
  } else {
    pauseTimer();
  }
}

function switchSpeaker() {
  if (!isFreeDebate.value) return;

  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }

  if (currentSpeakerSide.value === 'positive') {
    currentSpeakerSide.value = 'negative';
  } else {
    currentSpeakerSide.value = 'positive';
  }

  warning30Played.value = false;
  endPlayed.value = false;

  if (isRunning.value && !isPaused.value) {
    startTimer();
  }
}

function goToPrevious() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }

  if (currentIndex.value > 0) {
    currentIndex.value--;
    resetCurrentSegment();
  }
}

function goToNext() {
  if (currentIndex.value < (template.value?.segments.length || 0) - 1) {
    currentIndex.value++;
    resetCurrentSegment();

    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  } else {
    finishTimer();
  }
}

function resetCurrentSegment() {
  if (!template.value) return;

  warning30Played.value = false;
  endPlayed.value = false;
  segmentStartTime.value = Date.now();

  if (isFreeDebate.value) {
    const totalDuration = currentSegment.value.duration;
    positiveTimeRemaining.value = totalDuration;
    negativeTimeRemaining.value = totalDuration;
    currentSpeakerSide.value = 'positive';
  } else {
    timeRemaining.value = currentSegment.value.duration;
  }
  isOvertime.value = false;
  showWarning.value = false;
  isRunning.value = false;
  isPaused.value = false;
}

async function finishTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
  isFinished.value = true;
  isRunning.value = false;
  await saveTimerRecord();
}

async function saveTimerRecord() {
  if (!timerData.value || !template.value) return;

  const segments = template.value.segments.map((seg, index) => ({
    segment_name: seg.name,
    speaker: seg.speaker || '',
    planned_duration: seg.duration,
    actual_duration: seg.duration
  }));

  const record = {
    id: timerData.value.id || await invoke('generate_id'),
    template_id: timerData.value.selectedTemplateId,
    template_name: template.value.name,
    debate_topic: settings.value?.debateTopic || '',
    positive_team_name: settings.value?.positiveTeamName || '',
    negative_team_name: settings.value?.negativeTeamName || '',
    start_time: timerData.value.startTime,
    end_time: new Date().toISOString(),
    segments: segments
  };

  try {
    await invoke('save_timer_record', { record });
  } catch (e) {
    console.error('Failed to save timer record:', e);
  }
}

async function closeTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
  try {
    const window = await getCurrentWindow();
    await window.hide();
    await window.setFullscreen(false);
  } catch (e) {
    console.error('Failed to close timer:', e);
  }
}

function formatNextTime(segment) {
  if (!segment) return '';
  const minutes = Math.floor(segment.duration / 60);
  const seconds = segment.duration % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

function handleKeydown(e) {
  if (e.key === 'Escape') {
    closeTimer();
  } else if (e.key === ' ') {
    e.preventDefault();
    toggleTimer();
  } else if (e.key === 'ArrowLeft') {
    goToPrevious();
  } else if (e.key === 'ArrowRight') {
    goToNext();
  } else if (e.key === 'Tab' && isFreeDebate.value) {
    e.preventDefault();
    switchSpeaker();
  }
}

async function handleStartTimer(event) {
  timerData.value = event.payload;
  currentIndex.value = 0;
  isFinished.value = false;
  resetCurrentSegment();
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);

  getCurrentWindow().then(window => {
    window.listen('start-timer', handleStartTimer);
  });
});

onUnmounted(() => {
  if (timerInterval) {
    clearInterval(timerInterval);
  }
  window.removeEventListener('keydown', handleKeydown);
});

watch(currentIndex, () => {
  if (currentSegment.value) {
    resetCurrentSegment();
  }
});
</script>

<template>
  <div class="timer-view" :class="sideClass">
    <button class="exit-btn" @click="closeTimer">退出</button>

    <template v-if="template && !isFinished">
      <div class="segment-name">{{ currentSegment?.name }}</div>

      <template v-if="isFreeDebate">
        <div class="free-debate-display">
          <div class="side-time positive">
            <div class="time" :class="{ overtime: positiveTimeRemaining <= 0 }">
              {{ formattedPositiveTime }}
            </div>
            <button
              v-if="currentSpeakerSide === 'positive'"
              class="control-btn"
              @click="toggleTimer"
            >
              {{ !isRunning ? '开始' : (isPaused ? '继续' : '暂停') }}
            </button>
          </div>

          <div class="speaker-badge" :class="currentSpeakerSide">
            {{ currentSpeakerSide === 'positive' ? '正方发言' : '反方发言' }}
          </div>

          <div class="side-time negative">
            <div class="time" :class="{ overtime: negativeTimeRemaining <= 0 }">
              {{ formattedNegativeTime }}
            </div>
            <button
              v-if="currentSpeakerSide === 'negative'"
              class="control-btn"
              @click="toggleTimer"
            >
              {{ !isRunning ? '开始' : (isPaused ? '继续' : '暂停') }}
            </button>
          </div>
        </div>
      </template>

      <template v-else>
        <div class="time-display" :class="{ overtime: isOvertime, warning: showWarning && !isOvertime }">
          {{ formattedTime }}
        </div>
        <button class="control-btn pause-btn" @click="toggleTimer">
          {{ !isRunning ? '开始' : (isPaused ? '继续' : '暂停') }}
        </button>
      </template>

      <div class="speaker-label" v-if="!isFreeDebate">
        {{ currentSpeakerLabel }}
      </div>

      <div class="navigation">
        <button class="nav-btn" @click="goToPrevious" :disabled="currentIndex === 0">
          上一环节
        </button>
        <button class="nav-btn primary" @click="toggleTimer">
          {{ !isRunning ? '开始' : (isPaused ? '继续' : '暂停') }}
        </button>
        <template v-if="isFreeDebate">
          <button class="nav-btn switch" @click="switchSpeaker">
            切换发言方
          </button>
        </template>
        <button class="nav-btn" @click="goToNext" :disabled="currentIndex >= template.segments.length - 1">
          下一环节
        </button>
      </div>

      <div class="divider"></div>

      <div class="next-segment" v-if="nextSegment">
        <span class="next-label">下一个环节:</span>
        <span class="next-name">{{ nextSegment.name }}</span>
        <span class="next-time">{{ formatNextTime(nextSegment) }}</span>
      </div>

      <div class="hint" v-if="isFreeDebate">
        按 Tab 键快速切换发言方
      </div>
    </template>

    <template v-else-if="isFinished">
      <div class="finished-title">计时结束</div>
      <div class="finished-subtitle">{{ template?.name }}</div>
      <button class="control-btn" @click="closeTimer">返回配置界面</button>
    </template>

    <template v-else>
      <div class="waiting-title">等待开始</div>
      <div class="waiting-subtitle">请从配置界面开始计时</div>
    </template>
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
}

html, body, #app {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

body {
  font-family: "Microsoft YaHei", "PingFang SC", sans-serif;
  background-color: var(--color-primary);
  color: var(--color-text);
}
</style>

<style scoped>
.timer-view {
  height: 100vh;
  width: 100vw;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  position: relative;
  transition: background 0.3s;
}

.exit-btn {
  position: absolute;
  top: 24px;
  right: 24px;
  padding: 12px 24px;
  background-color: var(--color-secondary);
  color: var(--color-text);
  border: 1px solid #333;
  border-radius: var(--radius);
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.exit-btn:hover {
  background-color: #1f4068;
}

.timer-view.positive {
  background: linear-gradient(135deg, #1a1a2e 0%, #0f3460 100%);
}

.timer-view.negative {
  background: linear-gradient(135deg, #1a1a2e 0%, #4a1942 100%);
}

.timer-view.neutral {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
}

.segment-name {
  font-size: 32px;
  font-weight: 600;
  margin-bottom: 24px;
}

.time-display {
  font-family: "Roboto Mono", "Consolas", monospace;
  font-size: 120px;
  font-weight: 700;
  color: var(--color-text);
  letter-spacing: 4px;
  margin-bottom: 16px;
}

.time-display.warning {
  color: #f39c12;
  animation: pulse 1s ease-in-out infinite;
}

.time-display.overtime {
  color: var(--color-accent);
  animation: blink 0.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.speaker-label {
  font-size: 18px;
  color: var(--color-text-secondary);
  margin-top: 16px;
  margin-bottom: 32px;
}

.control-btn,
.nav-btn {
  padding: 12px 24px;
  background-color: var(--color-secondary);
  color: var(--color-text);
  border-radius: var(--radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
  border: none;
}

.control-btn:hover,
.nav-btn:hover:not(:disabled) {
  background-color: #1f4068;
}

.control-btn:disabled,
.nav-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.pause-btn {
  margin-bottom: 24px;
}

.nav-btn.primary {
  background-color: var(--color-success);
  color: white;
}

.nav-btn.primary:hover:not(:disabled) {
  background-color: #3db892;
}

.nav-btn.switch {
  background-color: #f39c12;
  color: white;
}

.nav-btn.switch:hover {
  background-color: #e67e22;
}

.navigation {
  display: flex;
  gap: 12px;
  margin-top: 16px;
}

.free-debate-display {
  display: flex;
  align-items: center;
  gap: 48px;
}

.side-time {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.side-time .time {
  font-family: "Roboto Mono", "Consolas", monospace;
  font-size: 80px;
  font-weight: 700;
  color: var(--color-text);
  letter-spacing: 4px;
}

.side-time .time.overtime {
  color: var(--color-accent);
  animation: blink 0.5s ease-in-out infinite;
}

.speaker-badge {
  padding: 8px 24px;
  border-radius: 24px;
  font-size: 16px;
  font-weight: 500;
}

.speaker-badge.positive {
  background-color: rgba(78, 204, 163, 0.2);
  color: var(--color-success);
  border: 1px solid var(--color-success);
}

.speaker-badge.negative {
  background-color: rgba(233, 69, 96, 0.2);
  color: var(--color-accent);
  border: 1px solid var(--color-accent);
}

.divider {
  width: 100%;
  max-width: 600px;
  height: 1px;
  background-color: rgba(255, 255, 255, 0.1);
  margin: 32px 0;
}

.next-segment {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 16px;
}

.next-label {
  color: var(--color-text-secondary);
}

.next-name {
  color: var(--color-text);
}

.next-time {
  font-family: "Roboto Mono", "Consolas", monospace;
  color: var(--color-text-secondary);
}

.hint {
  margin-top: 24px;
  font-size: 14px;
  color: var(--color-text-secondary);
}

.finished-title {
  font-size: 48px;
  font-weight: 700;
  margin-bottom: 16px;
}

.finished-subtitle {
  font-size: 24px;
  color: var(--color-text-secondary);
  margin-bottom: 32px;
}

.waiting-title {
  font-size: 36px;
  font-weight: 700;
  margin-bottom: 16px;
}

.waiting-subtitle {
  font-size: 18px;
  color: var(--color-text-secondary);
}
</style>
