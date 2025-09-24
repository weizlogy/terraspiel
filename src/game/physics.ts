import { ELEMENTS, type ElementName, type MoveDirection } from "../types/elements";

// Simple physics simulation function
export const simulatePhysics = (
  grid: ElementName[][],
  lastMoveGrid: MoveDirection[][],
  colorGrid: string[][]
): {
  newGrid: ElementName[][];
  newLastMoveGrid: MoveDirection[][];
  newColorGrid: string[][];
} => {
  // Create new grids by copying the current state. This is the double-buffering approach.
  // All modifications will be made to these new grids.
  const newGrid = grid.map(row => [...row]);
  const newColorGrid = colorGrid.map(row => [...row]);
  const newLastMoveGrid: MoveDirection[][] = lastMoveGrid.map(row => [...row]); // Copy lastMoveGrid as well

  const height = grid.length;
  const width = grid[0].length;

  // Create a grid to track which cells have already been moved in this simulation step
  const moved = Array(height).fill(null).map(() => Array(width).fill(false));

  // Process grid from bottom to top for gravity simulation
  const xIndices = Array.from(Array(width).keys());

  for (let y = height - 2; y >= 0; y--) {
    // Shuffle x-indices to randomize processing order for the row (Fisher-Yates shuffle)
    for (let i = xIndices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [xIndices[i], xIndices[j]] = [xIndices[j], xIndices[i]];
    }

    for (const x of xIndices) {
      // Skip if this cell has already been moved
      if (moved[y][x]) {
        continue;
      }

      const element = grid[y][x];
      const color = colorGrid[y][x];

      // Skip empty cells and static elements
      if (element === 'EMPTY') {
        continue;
      }

      // Simple gravity for soil with pseudo-inertia and slipping
      if (element === 'SOIL') {
        let hasMoved = false;

        // 1. Try moving down into empty space
        if (y + 1 < height && grid[y + 1][x] === 'EMPTY' && !moved[y + 1][x]) {
          newGrid[y][x] = 'EMPTY';
          newColorGrid[y][x] = ELEMENTS.EMPTY.color;
          newGrid[y + 1][x] = 'SOIL';
          newColorGrid[y + 1][x] = color;
          moved[y][x] = true;
          moved[y + 1][x] = true;
          newLastMoveGrid[y + 1][x] = 'DOWN';
          hasMoved = true;
        }
        // 2. Try swapping with water below
        else if (y + 1 < height && grid[y + 1][x] === 'WATER' && !moved[y + 1][x]) {
          const waterColor = colorGrid[y + 1][x];
          // Swap elements
          newGrid[y][x] = 'WATER';
          newGrid[y + 1][x] = 'SOIL';
          // Swap colors
          newColorGrid[y][x] = waterColor;
          newColorGrid[y + 1][x] = color;
          
          moved[y][x] = true;
          moved[y + 1][x] = true;
          newLastMoveGrid[y + 1][x] = 'DOWN'; // SOIL moved down
          hasMoved = true;
        }
        // 3. Try moving diagonally down
        else if (y + 1 < height) {
          const lastMove = lastMoveGrid[y][x];
          const inertiaChance = 0.75; // 75% chance to follow inertia

          let directions = [-1, 1]; // Default: left, right
          if (Math.random() > 0.5) directions.reverse();

          // Apply inertia: If last move was diagonal, try that direction first
          if (lastMove === 'DOWN_LEFT' && Math.random() < inertiaChance) {
            directions = [-1, 1];
          } else if (lastMove === 'DOWN_RIGHT' && Math.random() < inertiaChance) {
            directions = [1, -1];
          }

          for (const dx of directions) {
            if (x + dx >= 0 && x + dx < width &&
                grid[y + 1][x + dx] === 'EMPTY' && !moved[y + 1][x + dx]) {
              newGrid[y][x] = 'EMPTY';
              newColorGrid[y][x] = ELEMENTS.EMPTY.color;
              newGrid[y + 1][x + dx] = 'SOIL';
              newColorGrid[y + 1][x + dx] = color;
              moved[y][x] = true;
              moved[y + 1][x + dx] = true;
              newLastMoveGrid[y + 1][x + dx] = dx === -1 ? 'DOWN_LEFT' : 'DOWN_RIGHT';
              hasMoved = true;
              break;
            }
          }
        }

        // 4. Try to slip sideways if on a peak
        if (!hasMoved && Math.random() < 0.3) { // 30% chance to slip
          // Check if the particle is on a peak (empty on both sides)
          if (x > 0 && x < width - 1 && grid[y][x - 1] === 'EMPTY' && grid[y][x + 1] === 'EMPTY') {
            const slipDirection = Math.random() > 0.5 ? 1 : -1;
            
            if (!moved[y][x + slipDirection]) {
              newGrid[y][x] = 'EMPTY';
              newColorGrid[y][x] = ELEMENTS.EMPTY.color;
              newGrid[y][x + slipDirection] = 'SOIL';
              newColorGrid[y][x + slipDirection] = color;
              moved[y][x] = true;
              moved[y][x + slipDirection] = true;
              newLastMoveGrid[y][x + slipDirection] = slipDirection === -1 ? 'LEFT' : 'RIGHT';
              hasMoved = true;
            }
          }
        }
      }

      // Water physics - rewritten for horizontal stability
      else if (element === 'WATER') {
        // Movement priority: down > diagonally down > sideways
        let hasMoved = false;
        
        // 1. Try moving down
        if (!hasMoved && y + 1 < height && grid[y + 1][x] === 'EMPTY' && !moved[y + 1][x]) {
          newGrid[y][x] = 'EMPTY';
          newColorGrid[y][x] = ELEMENTS.EMPTY.color;
          newGrid[y + 1][x] = 'WATER';
          newColorGrid[y + 1][x] = color;
          moved[y][x] = true;
          moved[y + 1][x] = true;
          hasMoved = true;
        }
        
        // 2. Try moving diagonally down (randomize direction to avoid bias)
        if (!hasMoved && y + 1 < height) {
          const diagonalDirections = [-1, 1]; // Left-down and right-down
          if (Math.random() > 0.5) diagonalDirections.reverse();
          
          for (const dx of diagonalDirections) {
            if (x + dx >= 0 && x + dx < width && 
                grid[y + 1][x + dx] === 'EMPTY' && !moved[y + 1][x + dx]) {
              newGrid[y][x] = 'EMPTY';
              newColorGrid[y][x] = ELEMENTS.EMPTY.color;
              newGrid[y + 1][x + dx] = 'WATER';
              newColorGrid[y + 1][x + dx] = color;
              moved[y][x] = true;
              moved[y + 1][x + dx] = true;
              hasMoved = true;
              break;
            }
          }
        }
        
        // 3. Try moving sideways with horizontal stability in mind
        if (!hasMoved) {
          // Instead of random direction, check which side has more space for water to spread evenly
          const leftX = x - 1;
          const rightX = x + 1;
          
          // Check availability of left and right positions
          const canGoLeft = leftX >= 0 && grid[y][leftX] === 'EMPTY' && !moved[y][leftX];
          const canGoRight = rightX < width && grid[y][rightX] === 'EMPTY' && !moved[y][rightX];
          
          if (canGoLeft && canGoRight) {
            // When both sides are available, check which side has more empty space below
            // to spread water more evenly and maintain horizontal level
            let leftOpenSpaces = 0;
            let rightOpenSpaces = 0;
            
            // Count empty spaces below the potential left position
            for (let testY = y; testY < height; testY++) {
              if (grid[testY][leftX] === 'EMPTY') {
                leftOpenSpaces++;
              } else {
                break;
              }
            }
            
            // Count empty spaces below the potential right position
            for (let testY = y; testY < height; testY++) {
              if (grid[testY][rightX] === 'EMPTY') {
                rightOpenSpaces++;
              } else {
                break;
              }
            }
            
            // Move to the side with more open space below, or random if equal
            if (leftOpenSpaces > rightOpenSpaces) {
              newGrid[y][x] = 'EMPTY';
              newColorGrid[y][x] = ELEMENTS.EMPTY.color;
              newGrid[y][leftX] = 'WATER';
              newColorGrid[y][leftX] = color;
              moved[y][x] = true;
              moved[y][leftX] = true;
              hasMoved = true;
            } else if (rightOpenSpaces > leftOpenSpaces) {
              newGrid[y][x] = 'EMPTY';
              newColorGrid[y][x] = ELEMENTS.EMPTY.color;
              newGrid[y][rightX] = 'WATER';
              newColorGrid[y][rightX] = color;
              moved[y][x] = true;
              moved[y][rightX] = true;
              hasMoved = true;
            } else {
              // If equal, move in random direction
              const direction = Math.random() > 0.5 ? leftX : rightX;
              newGrid[y][x] = 'EMPTY';
              newColorGrid[y][x] = ELEMENTS.EMPTY.color;
              newGrid[y][direction] = 'WATER';
              newColorGrid[y][direction] = color;
              moved[y][x] = true;
              moved[y][direction] = true;
              hasMoved = true;
            }
          } else if (canGoLeft) {
            newGrid[y][x] = 'EMPTY';
            newColorGrid[y][x] = ELEMENTS.EMPTY.color;
            newGrid[y][leftX] = 'WATER';
            newColorGrid[y][leftX] = color;
            moved[y][x] = true;
            moved[y][leftX] = true;
            hasMoved = true;
          } else if (canGoRight) {
            newGrid[y][x] = 'EMPTY';
            newColorGrid[y][x] = ELEMENTS.EMPTY.color;
            newGrid[y][rightX] = 'WATER';
            newColorGrid[y][rightX] = color;
            moved[y][x] = true;
            moved[y][rightX] = true;
            hasMoved = true;
          }
        }
      }
    }
  }

  return { newGrid, newLastMoveGrid, newColorGrid };
};
// Calculate statistics for elements in the grid
export const calculateStats = (grid: ElementName[][]): Record<ElementName, number> => {
  const stats: Record<ElementName, number> = {
    EMPTY: 0, // Still need to initialize for type compatibility
    SOIL: 0,
    WATER: 0,
    FIRE: 0,
    WET_SOIL: 0,
    STEAM: 0,
  };
  
  for (let y = 0; y < grid.length; y++) {
    for (let x = 0; x < grid[y].length; x++) {
      const element = grid[y][x];
      if (element in stats && element !== 'EMPTY') {
        stats[element as ElementName]++;
      }
    }
  }
  
  // Don't count EMPTY in stats
  stats.EMPTY = 0;
  
  return stats;
};

// Physics simulation for particles
const GRAVITY = 0.05;
const FRICTION = 0.99;

export const simulateParticles = (particles: import("../types/elements").Particle[], grid: ElementName[][]): import("../types/elements").Particle[] => {
  const height = grid.length;
  const width = grid[0].length;

  const updatedParticles = particles.map(p => {
    // Apply gravity
    p.vy += GRAVITY;

    // Apply friction
    p.vx *= FRICTION;
    p.vy *= FRICTION;

    // Update position
    p.px += p.vx;
    p.py += p.vy;

    // Decrease lifespan
    p.life -= 1;

    // Simple collision with grid boundaries
    if (p.px < 0) {
      p.px = 0;
      p.vx *= -0.5; // Bounce
    }
    if (p.px >= width) {
      p.px = width - 1;
      p.vx *= -0.5; // Bounce
    }
    if (p.py < 0) {
      p.py = 0;
      p.vy *= -0.5; // Bounce
    }
    if (p.py >= height) {
      p.py = height - 1;
      p.vy = 0; // Stop at the bottom
      p.vx = 0;
    }

    // TODO: Add collision with grid cells

    return p;
  });

  // Filter out dead particles
  return updatedParticles.filter(p => p.life > 0);
};