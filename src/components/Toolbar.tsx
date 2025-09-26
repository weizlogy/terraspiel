
import useGameStore from '../stores/gameStore';
import { ELEMENTS, type ElementName } from '../types/elements';

const Toolbar: React.FC = () => {
  const selectedElement = useGameStore((state) => state.selectedElement);
  const setSelectedElement = useGameStore((state) => state.setSelectedElement);
  const clearGrid = useGameStore((state) => state.clearGrid);
  const randomizeGrid = useGameStore((state) => state.randomizeGrid);

  // Only show non-empty elements that can be placed by the user
  const placeableElements: ElementName[] = [
    'SOIL', 'WATER'
  ];

  return (
    <div className="toolbar bg-gray-800 text-white p-3 shadow-lg">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="element-selector flex flex-wrap gap-2">
          {placeableElements.map((element) => {
            const elementData = ELEMENTS[element];
            return (
              <button
                key={element}
                className={`element-btn px-3 py-2 rounded-lg flex items-center gap-2 transition-all duration-200 shadow ${
                  selectedElement === element 
                    ? 'bg-blue-600 ring-2 ring-white transform scale-105' 
                    : 'bg-gray-700 hover:bg-gray-600'
                }`}
                onClick={() => setSelectedElement(element)}
                style={{ 
                  borderLeft: `4px solid ${elementData.color}` 
                }}
              >
                <span className="font-medium">{element}</span>
              </button>
            );
          })}
        </div>
        
        <div className="controls flex flex-wrap gap-2">
          <button 
            className="control-btn bg-amber-600 hover:bg-amber-700 px-3 py-2 rounded-lg transition-all duration-200 shadow font-medium"
            onClick={randomizeGrid}
          >
            RANDOM
          </button>
          <button 
            className="control-btn bg-rose-600 hover:bg-rose-700 px-3 py-2 rounded-lg transition-all duration-200 shadow font-medium"
            onClick={clearGrid}
          >
            CLEAR
          </button>
        </div>
      </div>
    </div>
  );
};

export default Toolbar;