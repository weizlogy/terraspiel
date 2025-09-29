import { type Cell, type MoveDirection, type ElementName } from "../../types/elements";
import useGameStore from "../../stores/gameStore";

interface BehaviorContext {
  grid: Cell[][];
  lastMoveGrid: MoveDirection[][];
  colorGrid: string[][];
  newGrid: Cell[][];
  newLastMoveGrid: MoveDirection[][];
  newColorGrid: string[][];
  moved: boolean[][];
  x: number;
  y: number;
  width: number;
  height: number;
  scanRight: boolean;
}

export const handleGranular = ({
  grid,
  lastMoveGrid,
  colorGrid,
  newGrid,
  newLastMoveGrid,
  newColorGrid,
  moved,
  x,
  y,
  width,
  height,
  scanRight,
}: BehaviorContext): void => {
  const elements = useGameStore.getState().elements;
  if (Object.keys(elements).length === 0) return;

  const elementType = grid[y][x].type;
  const elementDef = elements[elementType];

  // Exit if the element is not fluid
  if (!elementDef?.fluidity) {
    // If it hasn't moved, copy original state to new grid
    newGrid[y][x] = grid[y][x];
    newColorGrid[y][x] = colorGrid[y][x];
    newLastMoveGrid[y][x] = lastMoveGrid[y][x];
    return;
  }

  const { resistance, spread } = elementDef.fluidity;
  const color = colorGrid[y][x];

  // 1. Try moving down
  if (y + 1 < height && grid[y + 1][x].type === 'EMPTY' && !moved[y + 1][x]) {
    newGrid[y][x] = { type: 'EMPTY' };
    newColorGrid[y][x] = elements.EMPTY.color;
    newGrid[y + 1][x] = { type: elementType };
    newColorGrid[y + 1][x] = color;
    moved[y][x] = true;
    moved[y + 1][x] = true;
    newLastMoveGrid[y + 1][x] = 'NONE';
    return;
  }

  // 2. Try moving diagonally down (with inertia and resistance)
  if (y + 1 < height) {
    const lastMove = lastMoveGrid[y][x];
    const initialDir = scanRight ? 1 : -1;
    const dir = lastMove === 'LEFT' ? -1 : (lastMove === 'RIGHT' ? 1 : initialDir);

    // Check preferred direction
    let dx = dir;
    if (x + dx >= 0 && x + dx < width && grid[y + 1][x + dx].type === 'EMPTY' && !moved[y + 1][x + dx] && Math.random() > resistance) {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[y + 1][x + dx] = { type: elementType };
      newColorGrid[y + 1][x + dx] = color;
      moved[y][x] = true;
      moved[y + 1][x + dx] = true;
      newLastMoveGrid[y + 1][x + dx] = dx === -1 ? 'LEFT' : 'RIGHT';
      return;
    }

    // Check other direction as a fallback
    dx = -dir;
    if (x + dx >= 0 && x + dx < width && grid[y + 1][x + dx].type === 'EMPTY' && !moved[y + 1][x + dx] && Math.random() > resistance) {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[y + 1][x + dx] = { type: elementType };
      newColorGrid[y + 1][x + dx] = color;
      moved[y][x] = true;
      moved[y + 1][x + dx] = true;
      newLastMoveGrid[y + 1][x + dx] = dx === -1 ? 'LEFT' : 'RIGHT';
      return;
    }
  }

  // 3. Try moving sideways (lightweight horizontal balancing and resistance)
  if (Math.random() < spread) {
    const leftX = x - 1;
    const rightX = x + 1;
    const canGoLeft = leftX >= 0 && grid[y][leftX].type === 'EMPTY' && !moved[y][leftX];
    const canGoRight = rightX < width && grid[y][rightX].type === 'EMPTY' && !moved[y][rightX];

    if (canGoLeft && canGoRight) {
      let leftOpenSpaces = 0;
      let rightOpenSpaces = 0;
      for (let i = 1; i <= 3; i++) {
        if (y + i < height && grid[y + i][leftX].type === 'EMPTY') leftOpenSpaces++; else break;
      }
      for (let i = 1; i <= 3; i++) {
        if (y + i < height && grid[y + i][rightX].type === 'EMPTY') rightOpenSpaces++; else break;
      }

      let moveDir = 0;
      if (leftOpenSpaces > rightOpenSpaces) {
        moveDir = -1;
      } else if (rightOpenSpaces > leftOpenSpaces) {
        moveDir = 1;
      } else {
        moveDir = scanRight ? 1 : -1;
      }

      const targetX = x + moveDir;
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[y][targetX] = { type: elementType };
      newColorGrid[y][targetX] = color;
      moved[y][x] = true;
      moved[y][targetX] = true;
      newLastMoveGrid[y][targetX] = moveDir === -1 ? 'LEFT' : 'RIGHT';
      return;
      
    } else if (canGoLeft) {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[y][leftX] = { type: elementType };
      newColorGrid[y][leftX] = color;
      moved[y][x] = true;
      moved[y][leftX] = true;
      newLastMoveGrid[y][leftX] = 'LEFT';
      return;
    } else if (canGoRight) {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[y][rightX] = { type: elementType };
      newColorGrid[y][rightX] = color;
      moved[y][x] = true;
      moved[y][rightX] = true;
      newLastMoveGrid[y][rightX] = 'RIGHT';
      return;
    }
  }

  // If it hasn't moved, copy original state to new grid
  newGrid[y][x] = grid[y][x];
  newColorGrid[y][x] = colorGrid[y][x];
  newLastMoveGrid[y][x] = lastMoveGrid[y][x];
};