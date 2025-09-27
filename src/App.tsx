import './App.css';
import PhaserGame from './components/PhaserGame';
import Toolbar from './components/Toolbar';
import StatsPanel from './components/StatsPanel';

function App() {
  // initializeGrid is now called within loadElements in gameStore.ts
  // const initializeGrid = useGameStore((state) => state.initializeGrid);

  // useEffect(() => {
  //   initializeGrid();
  // }, [initializeGrid]);

  return (
    <div className="app">
      <Toolbar />
      <main className="app-main">
        <div className="game-area-container flex flex-row h-full w-full p-4 gap-4">
          <PhaserGame />
          <StatsPanel />
        </div>
      </main>
    </div>
  );
}

export default App;