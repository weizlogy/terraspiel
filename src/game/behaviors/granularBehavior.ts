import { type Cell, type MoveDirection } from "../../types/elements";
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
  isChained?: boolean;
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
  isChained,
}: BehaviorContext): void => {
  const elements = useGameStore.getState().elements;
  if (Object.keys(elements).length === 0) return;

  const currentCell = grid[y][x];
  const elementType = currentCell.type;
  const elementDef = elements[elementType];

  if (!elementDef?.fluidity) {
    if (!isChained) {
      newGrid[y][x] = currentCell;
      newColorGrid[y][x] = colorGrid[y][x];
      newLastMoveGrid[y][x] = lastMoveGrid[y][x];
    }
    return;
  }

  const { resistance, spread } = elementDef.fluidity;
  const myColor = colorGrid[y][x];

  // Optimization: Check if the particle is likely settled
  const downY = y + 1;
  if (downY < height) {
    const belowCell = grid[downY][x];
    if (belowCell.type !== 'EMPTY') {
      const belowElementDef = elements[belowCell.type];

      // Check if we would swap with the cell below. If not, we are likely settled.
      const shouldSwap = belowElementDef?.fluidity && elementDef.density > belowElementDef.density && belowElementDef.state === 'liquid';

      if (!shouldSwap && elementDef.state !== 'liquid') {
        // If we are not going to swap, there's a high chance we are settled.
        if (Math.random() > 0.1) { // 90% chance to skip further checks
          if (!isChained) {
            newGrid[y][x] = currentCell;
            newColorGrid[y][x] = colorGrid[y][x];
            newLastMoveGrid[y][x] = lastMoveGrid[y][x];
          }
          return;
        }
      }
    }
  }

  // 1. Try moving down (or swapping with a less dense element)
  if (downY < height && !moved[y + 1][x]) {
    const targetCell = grid[downY][x];
    const targetElementDef = elements[targetCell.type];

    if (targetCell.type === 'EMPTY') {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[downY][x] = currentCell; // Preserve properties
      newColorGrid[downY][x] = myColor;
      moved[y][x] = true;
      moved[downY][x] = true;
      newLastMoveGrid[downY][x] = 'NONE';
      return;
    } else if (targetElementDef?.fluidity && elementDef.density > targetElementDef.density && targetElementDef.state === 'liquid') {
      newGrid[y][x] = targetCell; // Preserve properties
      newColorGrid[y][x] = colorGrid[downY][x];
      newGrid[downY][x] = currentCell; // Preserve properties
      newColorGrid[downY][x] = myColor;
      moved[y][x] = true;
      moved[downY][x] = true;
      newLastMoveGrid[y][x] = lastMoveGrid[downY][x]; // Inherit inertia from swapped element
      newLastMoveGrid[downY][x] = 'NONE';
      return;
    }
  }

  // 2. Try moving diagonally down (with inertia, resistance, and swapping)
  if (y + 1 < height) {
    const lastMove = lastMoveGrid[y][x];
    const initialDir = scanRight ? 1 : -1;
    const dir = lastMove === 'LEFT' ? -1 : (lastMove === 'RIGHT' ? 1 : initialDir);

    for (let i = 0; i < 2; i++) {
      const dx = i === 0 ? dir : -dir;
      const targetX = x + dx;

      if (targetX >= 0 && targetX < width && !moved[y + 1][targetX] && Math.random() > resistance) {
        const targetCell = grid[y + 1][targetX];
        const targetElementDef = elements[targetCell.type];

        if (targetCell.type === 'EMPTY') {
          newGrid[y][x] = { type: 'EMPTY' };
          newColorGrid[y][x] = elements.EMPTY.color;
          newGrid[y + 1][targetX] = currentCell; // Preserve properties
          newColorGrid[y + 1][targetX] = myColor;
          moved[y][x] = true;
          moved[y + 1][targetX] = true;
          newLastMoveGrid[y + 1][targetX] = dx === -1 ? 'LEFT' : 'RIGHT';
          return;
        } else if (targetElementDef?.fluidity && elementDef.density > targetElementDef.density && targetElementDef.state === 'liquid') {
          newGrid[y][x] = targetCell; // Preserve properties
          newColorGrid[y][x] = colorGrid[y + 1][targetX];
          newGrid[y + 1][targetX] = currentCell; // Preserve properties
          newColorGrid[y + 1][targetX] = myColor;
          moved[y][x] = true;
          moved[y + 1][targetX] = true;
          newLastMoveGrid[y][x] = lastMoveGrid[y + 1][targetX];
          newLastMoveGrid[y + 1][targetX] = dx === -1 ? 'LEFT' : 'RIGHT';
          return;
        }
      }
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
      newGrid[y][targetX] = currentCell; // Preserve properties
      newColorGrid[y][targetX] = myColor;
      moved[y][x] = true;
      moved[y][targetX] = true;
      newLastMoveGrid[y][targetX] = moveDir === -1 ? 'LEFT' : 'RIGHT';
      return;
    } else if (canGoLeft) {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[y][leftX] = currentCell; // Preserve properties
      newColorGrid[y][leftX] = myColor;
      moved[y][x] = true;
      moved[y][leftX] = true;
      newLastMoveGrid[y][leftX] = 'LEFT';
      return;
    } else if (canGoRight) {
      newGrid[y][x] = { type: 'EMPTY' };
      newColorGrid[y][x] = elements.EMPTY.color;
      newGrid[y][rightX] = currentCell; // Preserve properties
      newColorGrid[y][rightX] = myColor;
      moved[y][x] = true;
      moved[y][rightX] = true;
      newLastMoveGrid[y][rightX] = 'RIGHT';
      return;
    }
  }

  // If it hasn't moved, copy original state to new grid, unless chained
  if (!isChained) {
    newGrid[y][x] = currentCell;
    newColorGrid[y][x] = colorGrid[y][x];
    newLastMoveGrid[y][x] = lastMoveGrid[y][x];
  }
};