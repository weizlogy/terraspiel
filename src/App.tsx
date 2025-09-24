import { useEffect } from 'react';
import './App.css';
import PhaserGame from './components/PhaserGame';
import Toolbar from './components/Toolbar';
import StatsPanel from './components/StatsPanel';
import useGameStore from './stores/gameStore';

function App() {
  const initializeGrid = useGameStore((state) => state.initializeGrid);

  useEffect(() => {
    initializeGrid();
  }, [initializeGrid]);

  return (
    <div className="app">
      <Toolbar />
      <main className="app-main">
        <div className="game-area-container flex flex-row h-full w-full">
          <PhaserGame />
          <StatsPanel />
        </div>
      </main>
    </div>
  );
}

export default App;