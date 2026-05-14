<template>
  <v-container class="text-center fill-height bg-deep-dark" fluid>
    <v-row justify="center" align="center" class="w-100">
      
      <v-col cols="12" class="mb-4 d-flex justify-center align-center">
        <v-btn icon="mdi-arrow-left" variant="text" color="grey" @click="router.push('/')" class="mr-4"></v-btn>
        
        <v-chip
          :color="playerCount < 2 ? 'grey' : (currentPlayer === 1 ? '#00e5ff' : '#ff9800')"
          variant="outlined"
          class="text-subtitle-1 font-weight-bold px-6 py-5 player-chip"
        >
          <span v-if="playerCount < 2">
            <v-progress-circular indeterminate size="20" width="2" class="mr-2"></v-progress-circular>
            Warte auf Gegner ({{ playerCount }}/2)
          </span>
          <span v-else-if="isMyTurn">DU bist am Zug (Spieler {{ currentPlayer }})</span>
          <span v-else>GEGNER ist am Zug (Spieler {{ currentPlayer }})</span>
        </v-chip>
      </v-col>

      <v-col cols="12" class="d-flex justify-center">
        <div class="game-board-wrapper" :class="{ 'disabled-board': !isMyTurn }">
          <div class="game-board">
            
            <div class="empty-corner"></div>
            <div v-for="col in 7" :key="'col-down-'+col" class="shift-btn">
              <v-btn icon="mdi-chevron-double-down" variant="outlined" class="cyber-btn" size="small" @click="shiftCol(col-1, 'down')"></v-btn>
            </div>
            <div class="empty-corner"></div>

            <template v-for="(row, rowIndex) in board" :key="'row-'+rowIndex">
              
              <div class="shift-btn">
                <v-btn icon="mdi-chevron-double-right" variant="outlined" class="cyber-btn" size="small" @click="shiftRow(rowIndex, 'right')"></v-btn>
              </div>

              <div
                v-for="(cell, colIndex) in row"
                :key="'cell-'+rowIndex+'-'+colIndex"
                class="cell"
                :class="{
                  'anim-shift-right': shiftAnim.row === rowIndex && shiftAnim.direction === 'right',
                  'anim-shift-left': shiftAnim.row === rowIndex && shiftAnim.direction === 'left',
                  'anim-shift-down': shiftAnim.col === colIndex && shiftAnim.direction === 'down',
                  'anim-shift-up': shiftAnim.col === colIndex && shiftAnim.direction === 'up',
                }"
                @click="placePiece(rowIndex, colIndex)"
              >
                <div v-if="cell !== 0" class="piece" :class="{'player1': cell === 1, 'player2': cell === 2}"></div>
              </div>

              <div class="shift-btn">
                <v-btn icon="mdi-chevron-double-left" variant="outlined" class="cyber-btn" size="small" @click="shiftRow(rowIndex, 'left')"></v-btn>
              </div>
            </template>

            <div class="empty-corner"></div>
            <div v-for="col in 7" :key="'col-up-'+col" class="shift-btn">
              <v-btn icon="mdi-chevron-double-up" variant="outlined" class="cyber-btn" size="small" @click="shiftCol(col-1, 'up')"></v-btn>
            </div>
            <div class="empty-corner"></div>
            
          </div>
        </div>
      </v-col>
    </v-row>
    <v-dialog v-model="showWinDialog" max-width="500" persistent>
      <v-card class="bg-deep-dark cyber-dialog text-center pa-6">
        <v-card-title 
          class="text-h4 font-weight-bold pt-4 pb-2 title-glow" 
          :class="winner === 1 ? 'text-cyan' : 'text-orange'"
        >
          SYSTEM OVERRIDE
        </v-card-title>
        <v-card-text>
          <div class="mb-6">
            <v-icon size="100" :color="winner === 1 ? '#00e5ff' : '#ff9800'" class="mb-4 trophy-glow">mdi-trophy</v-icon>
          </div>
          <h2 class="text-h4 text-white mb-2">
            SPIELER {{ winner }} GEWINNT!
          </h2>
          <p class="text-grey mt-4">Verbindung zur Lobby wird wiederhergestellt...</p>
        </v-card-text>
        <v-card-actions class="justify-center pb-4 mt-4">
          <v-btn size="x-large" variant="outlined" :color="winner === 1 ? '#00e5ff' : '#ff9800'" @click="returnToLobby">
            Feld verlassen
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';

const route = useRoute();
const router = useRouter();
const roomId = route.params.id as string;

// Das Spielfeld
const board = ref<number[][]>(Array.from({ length: 7 }, () => Array(7).fill(0)));
const currentPlayer = ref<number>(1);
const winner = ref<number | null>(null);

  const playerCount = ref<number>(0);

const showWinDialog = ref<boolean>(false);
const myRole = ref<number>(parseInt(localStorage.getItem(`shift_role_${roomId}`) || '0'));
const isMyTurn = computed(() => myRole.value === currentPlayer.value && playerCount.value >= 2);

interface ShiftAnimation {
  row: number;
  col: number;
  direction: string;
}
const shiftAnim = ref<ShiftAnimation>({ row: -1, col: -1, direction: '' });

// --- WEBSOCKET LOGIK ---
let ws: WebSocket | null = null;

onMounted(() => {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  ws = new WebSocket(`${protocol}//${window.location.host}/ws/games/${roomId}`);

  ws.onopen = () => {
    console.log(`Erfolgreich mit Raum ${roomId} verbunden!`);
  };

  ws.onmessage = (event) => {
    const update = JSON.parse(event.data);
    
    // Server hat Vorrang: Überschreibe lokales Board mit Server-Status
    board.value = update.board;
    currentPlayer.value = update.current_player;
    
    // Prüfen ob jemand gewonnen hat
    if (update.winner) {
      winner.value = update.winner;
      
      // Kurze Verzögerung, damit man den Sieg-Zug noch sieht
      setTimeout(() => {
        alert(`🎉 SPIELER ${update.winner} HAT GEWONNEN! 🎉\nDie Lobby wird nun geschlossen.`);
        router.push('/'); // Zurück zur Startseite
      }, 500);
    }
  };

  ws.onmessage = (event) => {
    const update = JSON.parse(event.data);
    
    board.value = update.board;
    currentPlayer.value = update.current_player;
    playerCount.value = update.player_count;
    
    if (update.winner) {
      winner.value = update.winner;
      
      // ÄNDERUNG: Statt alert() zeigen wir den Dialog an
      setTimeout(() => {
        showWinDialog.value = true;
      }, 500);
    }
  };

  ws.onerror = (error) => {
    console.error("WebSocket Fehler:", error);
  };
});

onUnmounted(() => {
  if (ws) ws.close();
});


// --- SPIELER AKTIONEN ---

const placePiece = (row: number, col: number) => {
  if (!isMyTurn.value) return;
  if (board.value[row][col] === 0 && ws && ws.readyState === WebSocket.OPEN) {
    const payload = { action: 'place', row, col, player: currentPlayer.value };
    ws.send(JSON.stringify(payload));
  }
};

const shiftRow = (rowIndex: number, direction: 'right' | 'left') => {
  if (!isMyTurn.value) return;
  if (ws && ws.readyState === WebSocket.OPEN) {
    triggerAnimation('row', rowIndex, direction);
    const payload = { action: 'shift_row', row: rowIndex, direction, player: currentPlayer.value };
    ws.send(JSON.stringify(payload));
  }
};

const shiftCol = (colIndex: number, direction: 'up' | 'down') => {
  if (!isMyTurn.value) return;
  if (ws && ws.readyState === WebSocket.OPEN) {
    triggerAnimation('col', colIndex, direction);
    const payload = { action: 'shift_col', col: colIndex, direction, player: currentPlayer.value };
    ws.send(JSON.stringify(payload));
  }
};

// Hilfsfunktion für die Animation (wird sofort ausgelöst für besseres lokales Feedback)
const triggerAnimation = (type: 'row' | 'col', index: number, direction: string) => {
  if (type === 'row') shiftAnim.value = { row: index, col: -1, direction };
  if (type === 'col') shiftAnim.value = { row: -1, col: index, direction };
  setTimeout(() => { shiftAnim.value = { row: -1, col: -1, direction: '' }; }, 300);
};

const returnToLobby = () => {
  showWinDialog.value = false;
  router.push('/');
};
</script>

<style scoped>
.bg-deep-dark {
  background-color: #121418;
  background-image: radial-gradient(circle at 50% 0%, #1f2532 0%, #121418 70%);
}

.game-board-wrapper {
  padding: 30px;
  background: linear-gradient(145deg, #252a35, #181a22);
  border-radius: 16px;
  box-shadow: 0 30px 60px rgba(0, 0, 0, 0.6), inset 0 1px 2px rgba(255, 255, 255, 0.1), inset 0 -1px 2px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.game-board {
  display: grid;
  grid-template-columns: 45px repeat(7, 65px) 45px;
  grid-template-rows: 45px repeat(7, 65px) 45px;
  gap: 8px;
  align-items: center;
  justify-items: center;
}

.cell {
  width: 100%;
  height: 100%;
  background-color: #1e222b;
  border-radius: 8px;
  box-shadow: inset 0 3px 6px rgba(0,0,0,0.4), 0 1px 0 rgba(255,255,255,0.05);
  border: 1px solid rgba(0, 229, 255, 0.05);
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: background-color 0.2s ease, border-color 0.2s ease;
}

.cell:hover {
  background-color: #272c38;
  border-color: rgba(0, 229, 255, 0.3);
}

/* --- ANIMATIONEN --- */
.anim-shift-right { animation: slideGlowRight 0.3s ease-out; }
.anim-shift-left { animation: slideGlowLeft 0.3s ease-out; }
.anim-shift-down { animation: slideGlowDown 0.3s ease-out; }
.anim-shift-up { animation: slideGlowUp 0.3s ease-out; }

@keyframes slideGlowRight {
  0% { transform: translateX(-8px); box-shadow: inset 15px 0 15px -10px rgba(0, 229, 255, 0.8); }
  100% { transform: translateX(0); box-shadow: inset 0 0 0 0 rgba(0, 229, 255, 0); }
}
@keyframes slideGlowLeft {
  0% { transform: translateX(8px); box-shadow: inset -15px 0 15px -10px rgba(0, 229, 255, 0.8); }
  100% { transform: translateX(0); box-shadow: inset 0 0 0 0 rgba(0, 229, 255, 0); }
}
@keyframes slideGlowDown {
  0% { transform: translateY(-8px); box-shadow: inset 0 15px 15px -10px rgba(0, 229, 255, 0.8); }
  100% { transform: translateY(0); box-shadow: inset 0 0 0 0 rgba(0, 229, 255, 0); }
}
@keyframes slideGlowUp {
  0% { transform: translateY(8px); box-shadow: inset 0 -15px 15px -10px rgba(0, 229, 255, 0.8); }
  100% { transform: translateY(0); box-shadow: inset 0 0 0 0 rgba(0, 229, 255, 0); }
}

/* SPIELSTEINE */
.piece {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  box-shadow: inset 0 -3px 6px rgba(0,0,0,0.4);
  animation: popIn 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

@keyframes popIn {
  0% { transform: scale(0); opacity: 0; }
  100% { transform: scale(1); opacity: 1; }
}

.player1 {
  background: radial-gradient(circle at 30% 30%, #4dd0e1, #00acc1);
  box-shadow: 0 0 15px rgba(0, 229, 255, 0.6), inset 0 -3px 6px rgba(0,0,0,0.4);
}

.player2 {
  background: radial-gradient(circle at 30% 30%, #ffb74d, #f57c00);
  box-shadow: 0 0 15px rgba(255, 152, 0, 0.6), inset 0 -3px 6px rgba(0,0,0,0.4);
}

/* BUTTONS */
.cyber-btn {
  color: #00e5ff !important;
  border-color: rgba(0, 229, 255, 0.2) !important;
  background-color: rgba(0, 0, 0, 0.2);
  transition: all 0.3s ease;
  border-radius: 8px;
}
.cyber-btn:hover {
  background-color: rgba(0, 229, 255, 0.1);
  border-color: rgba(0, 229, 255, 0.8) !important;
  box-shadow: 0 0 12px rgba(0, 229, 255, 0.4);
  transform: scale(1.05);
}

.player-chip {
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 4px 20px rgba(0,0,0,0.4);
  backdrop-filter: blur(5px);
}

.glow-dot {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  box-shadow: 0 0 10px currentColor;
  animation: pulse 1.5s infinite alternate;
}

@keyframes pulse {
  0% { opacity: 0.6; transform: scale(0.9); }
  100% { opacity: 1; transform: scale(1.1); }
}

.empty-corner { width: 100%; height: 100%; }

.cyber-dialog {
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 0 40px rgba(0,0,0,0.9), inset 0 0 20px rgba(0,0,0,0.5);
  background: linear-gradient(145deg, #1a1e25, #121418) !important;
}
.title-glow { text-shadow: 0 0 15px currentColor; letter-spacing: 2px; }
.text-cyan { color: #00e5ff !important; }
.text-orange { color: #ff9800 !important; }
.trophy-glow { filter: drop-shadow(0 0 15px currentColor); animation: float 2s ease-in-out infinite; }

@keyframes float {
  0% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
  100% { transform: translateY(0px); }
}

.disabled-board {
  opacity: 0.6;
  pointer-events: none; /* Blockiert physisch alle Klicks und Hover-Effekte auf dem CSS Level */
  transition: opacity 0.3s ease;
}
</style>