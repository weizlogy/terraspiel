import useGameStore from '../stores/gameStore';
import type { ElementName } from '../types/elements';

const Toolbar: React.FC = () => {
  const selectedElement = useGameStore((state) => state.selectedElement);
  const setSelectedElement = useGameStore((state) => state.setSelectedElement);
  const clearGrid = useGameStore((state) => state.clearGrid);
  const randomizeGrid = useGameStore((state) => state.randomizeGrid);
  const elements = useGameStore((state) => state.elements);

  const placeableElements = Object.values(elements)
    .filter(el => el.isPlaceable)
    .map(el => el.name as ElementName);

  if (Object.keys(elements).length === 0) {
    return <div className="toolbar bg-gray-900 text-white p-2 shadow-lg border-t border-gray-700">Loading...</div>;
  }

  return (
    <div className="toolbar bg-gray-900 text-white p-2 shadow-lg border-t border-gray-700">
      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 mx-2">
        <div className="element-selector flex flex-wrap items-center gap-2">
          <span className="text-sm font-medium mr-2">Elements:</span>
          {placeableElements.map((element) => {
            const elementData = elements[element];
            return (
              <button
                key={element}
                className={`px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-150 border ${
                  selectedElement === element
                    ? 'bg-gray-600 border-gray-400'
                    : 'bg-gray-800 border-gray-700 hover:bg-gray-700 hover:border-gray-600'
                }`}
                onClick={() => setSelectedElement(element)}
              >
                <span style={{ color: elementData.color }} className="mr-1.5">●</span>
                {element}
              </button>
            );
          })}
        </div>

        <div className="controls flex flex-wrap items-center gap-2">
          <button
            className="px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-150 border bg-gray-800 border-gray-700 hover:bg-gray-700 hover:border-gray-600"
            onClick={randomizeGrid}
          >
            RANDOM
          </button>
          <button
            className="px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-150 border bg-gray-800 border-gray-700 hover:bg-gray-700 hover:border-gray-600"
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