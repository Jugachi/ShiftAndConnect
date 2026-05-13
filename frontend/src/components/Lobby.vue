<template>
  <v-container class="text-center fill-height bg-deep-dark" fluid>
    <v-row justify="center" align="center" class="w-100">
      <v-col cols="12" sm="10" md="8" lg="6">
        
        <h1 class="text-h3 font-weight-bold text-white mb-8 title-glow">
          SYSTEM UPLINK
        </h1>

        <!-- Der äußere Rahmen im Glass/Cyber-Look -->
        <div class="lobby-wrapper">
          <v-row>
            
            <!-- Linke Seite: Spiel erstellen -->
            <v-col cols="12" md="6" class="border-right-md pa-6">
              <h2 class="text-h5 text-cyan accent-glow mb-6">Spiel erstellen</h2>
              
              <p class="text-grey-lighten-1 mb-4">Wähle deinen Modus:</p>
              
              <!-- Modus Auswahl -->
              <v-btn-toggle
                v-model="gameMode"
                mandatory
                class="cyber-toggle mb-8"
                selected-class="cyber-toggle-active"
              >
                <v-btn value="duo" class="flex-grow-1">
                  <v-icon start>mdi-account-multiple</v-icon> Duo (PvP)
                </v-btn>
                <v-btn value="solo" class="flex-grow-1">
                  <v-icon start>mdi-robot</v-icon> Solo (vs AI)
                </v-btn>
              </v-btn-toggle>

              <v-btn
                block
                size="x-large"
                variant="outlined"
                class="cyber-btn-primary"
                @click="createGame"
              >
                Initialisieren
                <v-icon end>mdi-rocket-launch</v-icon>
              </v-btn>
            </v-col>

            <!-- Rechte Seite: Spiel beitreten -->
            <v-col cols="12" md="6" class="pa-6 d-flex flex-column justify-space-between">
              <div>
                <h2 class="text-h5 text-orange accent-glow mb-6">Spiel beitreten</h2>
                <p class="text-grey-lighten-1 mb-4">Verbinde dich mit einem bestehenden Signal:</p>
                
                <!-- Raumcode Eingabe -->
                <v-text-field
                  v-model="roomCode"
                  label="Raum-Code eingeben"
                  variant="outlined"
                  color="#ff9800"
                  base-color="rgba(255, 255, 255, 0.3)"
                  class="cyber-input"
                  hide-details
                >
                  <template v-slot:prepend-inner>
                    <v-icon color="#ff9800">mdi-key-chain</v-icon>
                  </template>
                </v-text-field>
              </div>

              <v-btn
                block
                size="x-large"
                variant="outlined"
                class="cyber-btn-secondary mt-8"
                @click="joinGame"
                :disabled="!roomCode"
              >
                Verbinden
                <v-icon end>mdi-connection</v-icon>
              </v-btn>
            </v-col>

          </v-row>
        </div>

      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();
const gameMode = ref<string>('duo');
const roomCode = ref<string>('');

const createGame = async () => {
  try {
    // 1. Anfrage an Rust senden, um Spiel in DB zu erstellen
    const response = await fetch('http://localhost:3000/api/games', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      // Wir senden den Modus (Solo/Duo) mit
      body: JSON.stringify({ mode: gameMode.value }), 
    });

    if (response.ok) {
      const data = await response.json();
      console.log(`Neues Spiel vom Server erstellt! ID: ${data.room_code}`);
      // 2. Zum neuen Raum navigieren
      router.push(`/game/${data.room_code}`);
    } else {
      console.error("Fehler beim Erstellen des Spiels");
    }
  } catch (error) {
    console.error("Netzwerkfehler - Läuft das Rust Backend?", error);
  }
};

const joinGame = () => {
  if (roomCode.value) {
    const code = roomCode.value.toUpperCase();
    console.log(`Trete Spiel bei mit Code: ${code}`);
    router.push(`/game/${code}`);
  }
};
</script>

<style scoped>
/* Hintergrund */
.bg-deep-dark {
  background-color: #121418;
  background-image: radial-gradient(circle at 50% 0%, #1f2532 0%, #121418 70%);
}

.title-glow {
  text-shadow: 0 0 20px rgba(255, 255, 255, 0.2);
  letter-spacing: 4px;
}

.accent-glow {
  text-shadow: 0 0 10px currentColor;
}

.text-cyan { color: #00e5ff !important; }
.text-orange { color: #ff9800 !important; }

/* Lobby Panel Style (identisch zum GameBoard Rahmen) */
.lobby-wrapper {
  background: linear-gradient(145deg, #252a35, #181a22);
  border-radius: 16px;
  box-shadow: 
    0 30px 60px rgba(0, 0, 0, 0.6), 
    inset 0 1px 2px rgba(255, 255, 255, 0.1),
    inset 0 -1px 2px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.05);
  overflow: hidden;
}

@media (min-width: 960px) {
  .border-right-md {
    border-right: 1px solid rgba(255, 255, 255, 0.05);
  }
}

/* Toggle Buttons für Solo/Duo */
.cyber-toggle {
  background-color: rgba(0, 0, 0, 0.2) !important;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  width: 100%;
}

.cyber-toggle-active {
  background-color: rgba(0, 229, 255, 0.15) !important;
  color: #00e5ff !important;
  border-color: rgba(0, 229, 255, 0.5) !important;
  box-shadow: inset 0 0 10px rgba(0, 229, 255, 0.2);
}

/* Neon Buttons */
.cyber-btn-primary {
  color: #00e5ff !important;
  border-color: rgba(0, 229, 255, 0.3) !important;
  background-color: rgba(0, 0, 0, 0.2);
  transition: all 0.3s ease;
  border-radius: 8px;
  font-weight: bold;
  letter-spacing: 1px;
}

.cyber-btn-primary:hover {
  background-color: rgba(0, 229, 255, 0.1);
  border-color: rgba(0, 229, 255, 0.8) !important;
  box-shadow: 0 0 15px rgba(0, 229, 255, 0.4);
}

.cyber-btn-secondary {
  color: #ff9800 !important;
  border-color: rgba(255, 152, 0, 0.3) !important;
  background-color: rgba(0, 0, 0, 0.2);
  transition: all 0.3s ease;
  border-radius: 8px;
  font-weight: bold;
  letter-spacing: 1px;
}

.cyber-btn-secondary:hover:not(:disabled) {
  background-color: rgba(255, 152, 0, 0.1);
  border-color: rgba(255, 152, 0, 0.8) !important;
  box-shadow: 0 0 15px rgba(255, 152, 0, 0.4);
}

/* Input Field Styling */
:deep(.cyber-input .v-field__outline__start),
:deep(.cyber-input .v-field__outline__end),
:deep(.cyber-input .v-field__outline__notch::before),
:deep(.cyber-input .v-field__outline__notch::after) {
  border-color: rgba(255, 255, 255, 0.2) !important;
  transition: border-color 0.3s ease;
}

:deep(.cyber-input:hover .v-field__outline__start),
:deep(.cyber-input:hover .v-field__outline__end) {
  border-color: rgba(255, 152, 0, 0.5) !important;
}

:deep(.cyber-input.v-input--is-focused .v-field__outline__start),
:deep(.cyber-input.v-input--is-focused .v-field__outline__end),
:deep(.cyber-input.v-input--is-focused .v-field__outline__notch::before),
:deep(.cyber-input.v-input--is-focused .v-field__outline__notch::after) {
  border-color: #ff9800 !important;
  box-shadow: 0 0 10px rgba(255, 152, 0, 0.2);
}

:deep(.cyber-input input) {
  color: white !important;
  font-weight: bold;
  letter-spacing: 2px;
  text-transform: uppercase;
}
</style>