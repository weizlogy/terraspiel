import { type Cell } from "../../types/elements";
import useGameStore from "../../stores/gameStore";
import { varyColor } from "../../utils/colors";

interface BehaviorContext {
  grid: Cell[][];
  newGrid: Cell[][];
  newColorGrid: string[][];
  moved: boolean[][];
  x: number;
  y: number;
  width: number;
  height: number;
}

const SPONTANEOUS_COMBUSTION_CHANCE = 0.001; // 自然発火の確率

export const handleOil = ({
  newGrid,
  newColorGrid,
  moved,
  x,
  y,
}: BehaviorContext): void => {
  const elements = useGameStore.getState().elements;
  if (Object.keys(elements).length === 0) return;

  // --- Spontaneous Combustion ---
  if (Math.random() < SPONTANEOUS_COMBUSTION_CHANCE) {
    newGrid[y][x] = { type: 'FIRE' };
    newColorGrid[y][x] = varyColor(elements.FIRE.color);
    moved[y][x] = true;
    return;
  }
};
