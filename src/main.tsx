import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import useGameStore from './stores/gameStore'

// Initialize the game grid
useGameStore.getState().initializeGrid(80, 60)

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)