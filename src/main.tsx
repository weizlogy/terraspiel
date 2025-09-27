import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import useGameStore from './stores/gameStore'

// Initialize the game
async function init() {
  await useGameStore.getState().loadElements(); // This will now also initialize and randomize the grid
  await useGameStore.getState().loadTransformationRules();
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)

init();