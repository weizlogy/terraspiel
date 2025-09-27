import useGameStore from '../stores/gameStore';
import { ELEMENTS } from "../types/elements";
import type { ElementName } from "../types/elements";

const StatsPanel: React.FC = () => {
  const stats = useGameStore((state) => state.stats);
  const fps = useGameStore((state) => state.fps);

  // Define which elements to show in the stats panel
  const displayElements: (ElementName | 'ETHER')[] = [
    'SOIL', 'WATER', 'MUD', 'FERTILE_SOIL', 'PEAT', 'CLOUD', 'CLAY', 'ETHER'
  ];

  return (
    <div className="stats-panel bg-gray-900 text-white p-4 rounded-lg w-48 flex-shrink-0">
      <h3 className="text-lg font-bold mb-2">Statistics</h3>
      <div className="flex flex-col gap-1 text-sm">
        <div className="stat-item flex items-center justify-between">
          <span className="font-semibold">FPS:</span>
          <span>{fps.toFixed(1)}</span>
        </div>
        {displayElements.map((element) => {
          const count = stats[element as keyof typeof stats];
          if (!count || count === 0) return null; // Only show elements with count > 0
          
          const elementData = ELEMENTS[element as keyof typeof ELEMENTS];
          const color = element === 'ETHER' ? '#FFFFFF' : elementData?.color || '#000000';

          return (
            <div 
              key={element} 
              className="stat-item flex items-center justify-between gap-2"
            >
              <div className="flex items-center gap-2">
                <div 
                  className="w-4 h-4 rounded-sm flex-shrink-0" 
                  style={{ backgroundColor: color }}
                ></div>
                <span className="font-semibold">{element}:</span>
              </div>
              <span>{count}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default StatsPanel;