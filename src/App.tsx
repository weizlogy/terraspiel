import { useEffect } from 'react';
import './App.css';
import PhaserGame from './components/PhaserGame';
import Toolbar from './components/Toolbar';
import useGameStore from './stores/gameStore';

function App() {
  const initializeGrid = useGameStore((state) => state.initializeGrid);

  useEffect(() => {
    // Calculate grid dimensions based on window size
    const calculateGridDimensions = () => {
      // Use a reasonable cell size (6px) as base, but adjust based on window
      const availableWidth = window.innerWidth;
      const availableHeight = window.innerHeight - 120; // Account for toolbar and footer
      
      // Calculate number of cells that can fit in available space
      const cols = Math.max(50, Math.floor(availableWidth / 6)); // At least 6px per cell
      const rows = Math.max(30, Math.floor(availableHeight / 6)); // At least 6px per cell
      
      return { width: cols, height: rows };
    };
    
    const { width, height } = calculateGridDimensions();
    // Initialize the grid with dimensions based on window size
    initializeGrid(width, height);
  }, [initializeGrid]);

  return (
    <div className="app">
      <Toolbar />
      <main className="app-main">
        <PhaserGame />
      </main>
      <footer className="app-footer text-center py-4">
        <p>Terraspiel - Alchemy Pixel Playground</p>
      </footer>
    </div>
  );
}

export default App;